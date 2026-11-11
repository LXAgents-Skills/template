import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

/** The repository root, resolved from this file rather than from the working directory. */
export const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");

/**
 * The published set. Every served file is read from inside this directory and nowhere
 * else, so containment is a property of the constant rather than of what a caller passed.
 */
export const CONTENT_DIR = join(ROOT, "content");
