import { readFileSync } from "node:fs";
import { join } from "node:path";
import { ROOT } from "./paths.js";

/**
 * The package version. `package.json` is the only place it is written; this reads it once.
 * A server that cannot read it still starts, and says `0.0.0`.
 */
function readVersion() {
  try {
    return JSON.parse(readFileSync(join(ROOT, "package.json"), "utf8")).version ?? "0.0.0";
  } catch {
    return "0.0.0";
  }
}

export const version = readVersion();
