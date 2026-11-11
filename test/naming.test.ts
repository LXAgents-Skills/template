import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { ORG_NAME, REPO_NAME, TOOL_PREFIX, idPart, toolId } from "../src/identity.js";
import { buildContentInstructions } from "../src/instructions/from-content.js";
import { MAX_ID_LENGTH, derivedBase, nameProblem, toolNameFor } from "../src/instructions/naming.js";

/** A throwaway set: `files` maps a set-relative path to its text. */
function withSet<T>(files: Record<string, string>, run: (dir: string) => T): T {
  const dir = mkdtempSync(join(tmpdir(), "content-"));
  try {
    for (const [path, text] of Object.entries(files)) {
      mkdirSync(join(dir, path, ".."), { recursive: true });
      writeFileSync(join(dir, path), text);
    }
    return run(dir);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

const doc = (description = "A file.") => `---\nname: x\ndescription: ${description}\n---\n\nBody.\n`;

test("idPart lowercases and turns every non-alphanumeric run into one underscore", () => {
  assert.equal(idPart("LXAgents-Skills"), "lxagents_skills");
  assert.equal(idPart("shared-instruction"), "shared_instruction");
  assert.equal(idPart("--My..Repo--"), "my_repo");
});

test("the prefix is built from ORG_NAME and REPO_NAME", () => {
  assert.equal(TOOL_PREFIX, `${idPart(ORG_NAME)}_${idPart(REPO_NAME)}`);
  assert.equal(toolId("automation"), `${TOOL_PREFIX}_automation`);
});

test("the base is the filename: folder and .md dropped, lowercased, - to _", () => {
  assert.equal(derivedBase("git/Branching-Strategy.md"), "branching_strategy");
  assert.equal(toolNameFor("a/b/example-guide.md"), toolId("example_guide"));
});

test("an override replaces the base only, never the prefix", () => {
  const overrides = { "references/very-long-name.md": "short" };
  assert.equal(toolNameFor("references/very-long-name.md", overrides), toolId("short"));
});

test("an ID over 64 characters or with a bad character is refused", () => {
  assert.equal(nameProblem("a".repeat(MAX_ID_LENGTH)), null);
  assert.match(nameProblem("a".repeat(MAX_ID_LENGTH + 1)) ?? "", /65 characters/);
  assert.match(nameProblem("1abc") ?? "", /not a valid tool name/);
  assert.match(nameProblem("ab.c") ?? "", /not a valid tool name/);
});

test("a set with a file and no description does not build", () => {
  withSet({ "a.md": "# no frontmatter\n" }, (dir: string) => {
    assert.throws(() => buildContentInstructions(dir, {}), /no frontmatter `description:`/);
  });
});

test("two files deriving one ID do not build", () => {
  withSet({ "one/same.md": doc(), "two/same.md": doc() }, (dir: string) => {
    assert.throws(() => buildContentInstructions(dir, {}), /both derive/);
  });
});

test("an override that matches no file does not build", () => {
  withSet({ "a.md": doc() }, (dir: string) => {
    assert.throws(() => buildContentInstructions(dir, { "gone.md": "gone" }), /not in the set/);
  });
});

test("a filename too long for 64 characters does not build until it is overridden", () => {
  const path = `${"long-".repeat(12)}name.md`;
  withSet({ [path]: doc() }, (dir: string) => {
    assert.throws(() => buildContentInstructions(dir, {}), /OVERRIDE_BASES/);
    const { tools } = buildContentInstructions(dir, { [path]: "long_name" });
    assert.equal(tools[0].config.name, toolId("long_name"));
  });
});

test("an empty set does not build", () => {
  withSet({ "notes.txt": "not markdown" }, (dir: string) => {
    assert.throws(() => buildContentInstructions(dir, {}), /expose nothing/);
  });
});
