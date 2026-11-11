import { toolId } from "../identity.js";

/*
 * How a file in `content/` gets its tool ID.
 *
 *   {org}_{repo}_{base}
 *
 * `{org}_{repo}` is `TOOL_PREFIX` from `src/identity.ts`. `{base}` is the filename: folder
 * dropped, `.md` dropped, lowercased, `-` turned into `_`. So `example/example-guide.md` in
 * `LXAgents-Skills/template` is `lxagents_skills_template_example_guide`.
 *
 * Full rules, and when to add an override: `.agents/rules/tool-ids.md`.
 */

/**
 * The longest ID this server publishes. The protocol allows 128, but several clients cap a
 * tool name at 64, so every ID is held to 64 at boot.
 */
export const MAX_ID_LENGTH: number = 64;

/** What an MCP client accepts as a tool name, with the 64-character cap applied. */
export const TOOL_NAME: RegExp = /^[a-z][a-z0-9_]*$/;

/**
 * Base names that do not survive mechanical derivation, keyed by set-relative path.
 *
 * Use one when the derived ID would be longer than `MAX_ID_LENGTH`, when two folders hold the
 * same filename, or when the filename says nothing about the file. **Only the base is ever
 * shortened; the prefix is added below and never edited.** A key that matches no file is a
 * startup error, so a rename cannot leave an override matching nothing.
 *
 * Example: `"references/javascript-typescript-react-web-frontend-security.md": "js_ts_react_web_frontend_security"`.
 */
export const OVERRIDE_BASES: Readonly<Record<string, string>> = Object.freeze({});

/** The base name a path derives on its own. */
export function derivedBase(relativePath: string): string {
  return relativePath
    .split("/")
    .pop()!
    .replace(/\.md$/, "")
    .toLowerCase()
    .replace(/-/g, "_");
}

/** The tool ID for a set-relative path. */
export function toolNameFor(
  relativePath: string,
  overrides: Readonly<Record<string, string>> = OVERRIDE_BASES,
): string {
  return toolId(overrides[relativePath] ?? derivedBase(relativePath));
}

/** Why `name` cannot be published, or `null` when it can. */
export function nameProblem(name: string): string | null {
  if (!TOOL_NAME.test(name)) {
    return `"${name}" is not a valid tool name (lowercase letters, digits and underscores, starting with a letter)`;
  }
  if (name.length > MAX_ID_LENGTH) {
    return `"${name}" is ${name.length} characters, over the ${MAX_ID_LENGTH}-character cap`;
  }
  return null;
}
