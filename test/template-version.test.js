/*
 * Template only: `npm run init` deletes this file. The template is copied, never released,
 * so its version stays 0.0.0 and every repository made from it starts its own from there.
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT } from "../src/paths.js";
import { version } from "../src/version.js";

const PINNED = "0.0.0";
const read = (file) => JSON.parse(readFileSync(join(ROOT, file), "utf8"));

test("the template's version is pinned at 0.0.0", () => {
  const lock = read("package-lock.json");
  assert.equal(read("package.json").version, PINNED, "package.json");
  assert.equal(lock.version, PINNED, "package-lock.json");
  assert.equal(lock.packages[""].version, PINNED, "package-lock.json packages[\"\"]");
  assert.equal(version, PINNED, "the version the server reports");
});
