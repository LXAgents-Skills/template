import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdtempSync, readFileSync, rmSync, symlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT } from "../src/paths.js";
import { parseArgs } from "../scripts/init/args.js";
import { identityFor } from "../scripts/init/identity.js";
import { replaceAll, rewriteIdentity, rewritePackage, stripTemplateOnly } from "../scripts/init/rewrite.js";

test("flags parse to camelCase, and switches take no value", () => {
  assert.deepEqual(parseArgs(["--org", "A", "--server-id", "a-b", "--dry-run", "--keep-history"]), {
    org: "A",
    serverId: "a-b",
    dryRun: true,
    keepHistory: true,
  });
  assert.throws(() => parseArgs(["--org"]), /needs a value/);
  assert.throws(() => parseArgs(["stray"]), /Unexpected argument/);
});

test("the identity defaults follow the existing servers' naming", () => {
  assert.deepEqual(identityFor({ org: "LXAgents-Skills", repo: "security" }), {
    org: "LXAgents-Skills",
    repo: "security",
    serverId: "lxagents-security",
    title: "LXAgents Security",
    summary: "The security instruction set, served read-only.",
  });
  assert.throws(() => identityFor({ org: "LXAgents-Skills" }), /required/);
  assert.throws(() => identityFor({ org: "a b", repo: "c" }), /GitHub names/);
});

test("rewriteIdentity replaces every identity constant, multi-line values included", () => {
  const source = readFileSync(join(ROOT, "src/identity.js"), "utf8");
  const next = identityFor({ org: "Acme", repo: "docs", summary: 'Say "hi".' });
  const out = rewriteIdentity(source, next);

  assert.match(out, /export const ORG_NAME = "Acme";/);
  assert.match(out, /export const REPO_NAME = "docs";/);
  assert.match(out, /export const SERVER_ID = "acme-docs";/);
  assert.match(out, /export const SERVER_SUMMARY = "Say \\"hi\\".";/);
  assert.match(out, /export const TOOL_PREFIX = /, "the derived prefix is left alone");
});

test("rewritePackage renames the package and resets its version", () => {
  const pkg = JSON.parse(rewritePackage(readFileSync(join(ROOT, "package.json"), "utf8"), identityFor({ org: "Acme", repo: "docs" })));
  assert.equal(pkg.name, "@acme/docs");
  assert.equal(pkg.version, "0.0.0");
  assert.deepEqual(pkg.bin, { "acme-docs": "src/cli.js", "acme-docs-server": "src/index.js" });
  assert.equal(pkg.repository.url, "git+https://github.com/Acme/docs.git");
});

test("stripTemplateOnly drops each marked block and its markers", () => {
  const text = "a\n<!-- template-only -->\nonly\n<!-- /template-only -->\nb\n";
  assert.equal(stripTemplateOnly(text), "a\nb\n");
});

test("replaceAll swaps whole strings in order", () => {
  assert.equal(replaceAll("a_x a_x b", [["a_", "c_"], ["b", "d"]]), "c_x c_x d");
});

test("init turns a copy of the template into a new repository that passes its own tests", () => {
  const copy = mkdtempSync(join(tmpdir(), "template-copy-"));
  try {
    cpSync(ROOT, copy, { recursive: true, filter: (src) => !/[\\/](node_modules|\.git)$/.test(src) });
    symlinkSync(join(ROOT, "node_modules"), join(copy, "node_modules"), "junction");

    const init = spawnSync(process.execPath, ["scripts/init.js", "--org", "Acme-Labs", "--repo", "field-notes"], {
      cwd: copy,
      encoding: "utf8",
    });
    assert.equal(init.status, 0, init.stderr);
    assert.match(init.stdout, /acme_labs_field_notes_automation/);

    const hub = readFileSync(join(copy, "content/automation.md"), "utf8");
    assert.match(hub, /`acme_labs_field_notes_example_guide`/);
    assert.doesNotMatch(hub, /lxagents_skills_template_/);
    assert.match(readFileSync(join(copy, "AGENTS.md"), "utf8"), /`Acme-Labs\/field-notes`/);
    assert.equal(existsSync(join(copy, ".agents/memory/tasks")), false, "the template's history is cleared");
    for (const file of ["wiki/guides/create-a-new-repository.md", ".agents/rules/template-version.md", "test/template-version.test.js"]) {
      assert.equal(existsSync(join(copy, file)), false, `${file} is template-only`);
    }
    assert.equal(JSON.parse(readFileSync(join(copy, "package.json"), "utf8")).version, "0.0.0");
    const lock = JSON.parse(readFileSync(join(copy, "package-lock.json"), "utf8"));
    assert.deepEqual(Object.keys(lock.packages[""].bin), ["acme-field-notes", "acme-field-notes-server"]);
    for (const page of ["AGENTS.md", "README.md", ".agents/index/project-wiki-index.md", ".agents/index/agents-index.md"]) {
      const text = readFileSync(join(copy, page), "utf8");
      assert.doesNotMatch(text, /template-only|create-a-new-repository|template-version/, `${page} keeps template-only text`);
    }

    const suite = spawnSync(process.execPath, ["--test", "test/server.test.js", "test/naming.test.js"], {
      cwd: copy,
      encoding: "utf8",
    });
    assert.equal(suite.status, 0, suite.stdout + suite.stderr);
  } finally {
    rmSync(copy, { recursive: true, force: true });
  }
});
