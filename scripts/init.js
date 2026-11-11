#!/usr/bin/env node

/*
 * Turn a fresh copy of the template into a new repository.
 *
 *   npm run init -- --org LXAgents-Skills --repo my-skills
 *
 * Rewrites the identity in src/identity.js, package.json and package-lock.json, swaps the
 * old tool prefix, org/repo slug and server ID for the new ones in every markdown file and
 * the Dockerfile, clears the template's own history (task records, decisions, release logs),
 * and then boots the tool list to prove every ID is still valid.
 *
 * Run it once, on a clean working tree, and review `git diff` before committing.
 */

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import * as current from "../src/identity.js";
import { parseArgs } from "./init/args.js";
import { identityFor } from "./init/identity.js";
import { replaceAll, rewriteIdentity, rewritePackage } from "./init/rewrite.js";

const ROOT = fileURLToPath(new URL("..", import.meta.url));

const HELP = `Usage: npm run init -- --org <GitHub org> --repo <repository> [options]

Options
  --server-id <id>   server name; default: first word of the org, a dash, the repo
  --title <text>     human-readable title; default: "<Org word> <Repo In Title Case>"
  --summary <text>   one sentence on what the set is
  --keep-history     keep the template's task records, decisions and release logs
  --dry-run          print what would change and write nothing
  --force            run even though origin is the template repository itself
  --help             show this help`;

/** Paths the template's own history lives in. A new repository starts without them. */
const HISTORY = [".agents/memory/tasks", ".agents/memory/decisions", ".agents/memory/state", "wiki/logs"];

/** Fresh indexes for the history that was cleared. */
const FRESH_INDEXES = {
  ".agents/index/memory-index.md": `---
name: memory-index
description: Router over .agents/memory/ - one row per task, decision, or state file, read every session so work continues instead of restarting.
---

# Memory Index

| File | Scope |
|---|---|
`,
  ".agents/index/logs-index.md": `---
name: logs-index
description: Router over wiki/logs/ - one row per release, newest first.
---

# Logs Index

| Version | Changelog |
|---|---|
`,
};

/** Every file under `dir` whose name passes `keep`, skipping node_modules and .git. */
function walk(dir, keep) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    if (entry.name === "node_modules" || entry.name === ".git") return [];
    const full = join(dir, entry.name);
    if (entry.isDirectory()) return walk(full, keep);
    return keep(entry.name) ? [full] : [];
  });
}

/** Read, transform, and write back if changed. Returns the path when it changed. */
function edit(path, transform, dryRun) {
  const before = readFileSync(path, "utf8");
  const after = transform(before);
  if (after === before) return null;
  if (!dryRun) writeFileSync(path, after);
  return relative(ROOT, path);
}

/** Whether this checkout's origin is the template repository itself. */
function isTemplateItself() {
  const origin = spawnSync("git", ["remote", "get-url", "origin"], { cwd: ROOT, encoding: "utf8" });
  const slug = `${current.ORG_NAME}/${current.REPO_NAME}`.toLowerCase();
  return origin.status === 0 && origin.stdout.trim().toLowerCase().replace(/\.git$/, "").endsWith(slug);
}

function main(argv) {
  const flags = parseArgs(argv);
  if (flags.help) {
    process.stdout.write(`${HELP}\n`);
    return 0;
  }
  if (!flags.dryRun && !flags.force && isTemplateItself()) {
    throw new Error(
      "origin is the template repository itself. Run init in the new repository made from it, or pass --force.",
    );
  }

  const next = identityFor(flags);
  const nextPrefix = `${current.idPart(next.org)}_${current.idPart(next.repo)}`;
  const dryRun = Boolean(flags.dryRun);
  const pairs = [
    [`${current.TOOL_PREFIX}_`, `${nextPrefix}_`],
    [`${current.ORG_NAME}/${current.REPO_NAME}`, `${next.org}/${next.repo}`],
    [current.SERVER_ID, next.serverId],
  ];

  const changed = [];
  const removed = [];

  if (!flags.keepHistory) {
    for (const path of HISTORY.map((p) => join(ROOT, p)).filter(existsSync)) {
      if (!dryRun) rmSync(path, { recursive: true, force: true });
      removed.push(relative(ROOT, path));
    }
    for (const [path, text] of Object.entries(FRESH_INDEXES)) {
      if (existsSync(join(ROOT, path))) changed.push(edit(join(ROOT, path), () => text, dryRun));
    }
  }

  changed.push(edit(join(ROOT, "src/identity.js"), (text) => rewriteIdentity(text, next), dryRun));
  changed.push(edit(join(ROOT, "package.json"), (text) => rewritePackage(text, next), dryRun));
  if (existsSync(join(ROOT, "package-lock.json"))) {
    changed.push(
      edit(join(ROOT, "package-lock.json"), (text) => {
        const lock = JSON.parse(text);
        const name = `@${next.org.toLowerCase()}/${next.repo}`;
        lock.name = name;
        lock.version = "0.0.0";
        if (lock.packages?.[""]) Object.assign(lock.packages[""], { name, version: "0.0.0" });
        return `${JSON.stringify(lock, null, 2)}\n`;
      }, dryRun),
    );
  }

  const texts = [
    ...walk(ROOT, (name) => name.endsWith(".md")).filter((path) => !removed.some((gone) => path.startsWith(join(ROOT, gone)))),
    join(ROOT, "Dockerfile"),
  ].filter(existsSync);
  for (const path of texts) changed.push(edit(path, (text) => replaceAll(text, pairs), dryRun));

  const report = [
    `${dryRun ? "Would change" : "Changed"} ${current.ORG_NAME}/${current.REPO_NAME} -> ${next.org}/${next.repo}`,
    `  tool prefix  ${current.TOOL_PREFIX} -> ${nextPrefix}`,
    `  server id    ${current.SERVER_ID} -> ${next.serverId}`,
    ...changed.filter(Boolean).sort().map((path) => `  edited   ${path}`),
    ...removed.map((path) => `  removed  ${path}`),
  ];
  process.stdout.write(`${report.join("\n")}\n`);
  if (dryRun) return 0;

  // Boot the surface: a prefix that pushes an ID past 64 characters fails here, by name.
  const check = spawnSync(process.execPath, [join(ROOT, "src/cli.js"), "tools"], { encoding: "utf8" });
  if (check.status !== 0) {
    process.stderr.write(`\nThe tool list no longer builds:\n${check.stderr}`);
    return 1;
  }
  process.stdout.write(`\nTools:\n${check.stdout}\nNext: edit README.md and content/, then run npm test.\n`);
  return 0;
}

try {
  process.exitCode = main(process.argv.slice(2));
} catch (error) {
  process.stderr.write(`${error.message}\n\n${HELP}\n`);
  process.exitCode = 2;
}
