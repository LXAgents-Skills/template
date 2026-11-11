import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { SERVER_ID, TOOL_PREFIX, toolId } from "../src/identity.js";
import { CONTENT_INSTRUCTIONS, INSTRUCTION_FILES, markdownFiles } from "../src/instructions/from-content.js";
import { MAX_ID_LENGTH, toolNameFor } from "../src/instructions/naming.js";
import { CONTENT_DIR } from "../src/paths.js";
import { listTools } from "../src/server.js";
import { withClient } from "./helpers.js";

/*
 * The served surface. Every test here is written against whatever `content/` holds, so a
 * repository made from the template keeps them all when it replaces the example set.
 */

const HUB = toolId("automation");

test("the server names itself SERVER_ID", async () => {
  await withClient(async (client) => {
    assert.equal(client.getServerVersion().name, SERVER_ID);
  });
});

test("the CLI list and the MCP tool list agree", async () => {
  await withClient(async (client) => {
    const { tools } = await client.listTools();
    const listed = new Map(listTools().map((tool) => [tool.name, tool.description]));

    assert.deepEqual(tools.map((tool) => tool.name).sort(), [...listed.keys()].sort());
    for (const tool of tools) assert.equal(tool.description, listed.get(tool.name), tool.name);
  });
});

test("every markdown file is one tool, and every tool is one file", () => {
  const files = markdownFiles();
  const served = [...INSTRUCTION_FILES.values()].sort((a, b) => a.localeCompare(b));

  assert.deepEqual(served, files);
  assert.equal(CONTENT_INSTRUCTIONS.length, files.length);
});

test("every ID is {org}_{repo}_{base}, underscores only, at most 64 characters", () => {
  for (const [name, path] of INSTRUCTION_FILES) {
    assert.ok(name.startsWith(`${TOOL_PREFIX}_`), `${name} must start with ${TOOL_PREFIX}_`);
    assert.match(name, /^[a-z][a-z0-9_]*$/, name);
    assert.ok(name.length <= MAX_ID_LENGTH, `${name} is ${name.length} characters`);
    assert.equal(name, toolNameFor(path), `${path} must derive ${name}`);
  }
});

test("no tool takes an argument", async () => {
  await withClient(async (client) => {
    const { tools } = await client.listTools();
    for (const tool of tools) {
      assert.deepEqual(Object.keys(tool.inputSchema?.properties ?? {}), [], `${tool.name} takes an argument`);
    }
  });
});

test("every tool returns its own file, byte for byte", async () => {
  await withClient(async (client) => {
    for (const [name, path] of INSTRUCTION_FILES) {
      const result = await client.callTool({ name, arguments: {} });
      assert.notEqual(result.isError, true, name);
      assert.equal(result.content.length, 1, name);
      assert.equal(result.content[0].text, readFileSync(join(CONTENT_DIR, path), "utf8"), name);
    }
  });
});

test("the automation hub is served", () => {
  assert.ok(INSTRUCTION_FILES.has(HUB), `content/automation.md must exist and derive ${HUB}`);
});

test("the automation hub lists every other tool exactly once, and nothing else", () => {
  const hub = readFileSync(join(CONTENT_DIR, INSTRUCTION_FILES.get(HUB)), "utf8");
  const pattern = new RegExp(`\`(${TOOL_PREFIX}_[a-z0-9_]+)\``, "g");
  const rows = hub
    .split("\n")
    .filter((line) => line.startsWith("- `"))
    .flatMap((line) => [...line.matchAll(pattern)].slice(0, 1).map((match) => match[1]));

  const expected = [...INSTRUCTION_FILES.keys()].filter((name) => name !== HUB).sort();
  const duplicates = rows.filter((name, at) => rows.indexOf(name) !== at);

  assert.deepEqual(duplicates, [], "a tool is listed twice in automation.md");
  assert.deepEqual(
    expected.filter((name) => !rows.includes(name)),
    [],
    "these tools have no row in content/automation.md",
  );
  assert.deepEqual(
    rows.filter((name) => !INSTRUCTION_FILES.has(name)),
    [],
    "content/automation.md lists tools that do not exist",
  );
  for (const row of rows.filter((name) => INSTRUCTION_FILES.has(name))) {
    const line = hub.split("\n").find((text) => text.startsWith(`- \`${row}\``));
    assert.match(line, /` — \S/, `${row} needs a condition after " — "`);
  }
});
