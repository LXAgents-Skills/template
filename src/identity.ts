/*
 * Who this server is. The one file a new repository edits to become itself.
 *
 * `npm run init -- --org <org> --repo <repo>` rewrites the four constants below. Everything
 * else that names the server — the tool prefix, every tool ID, the health check, the startup
 * lines — is derived from them, so no other source file spells the org or the repo.
 */

/** The GitHub organization, as written there. */
export const ORG_NAME = "LXAgents-Skills";

/** The repository name, as written there. */
export const REPO_NAME = "template";

/** The server's name in `initialize`, `/healthz` and every log line. */
export const SERVER_ID = "lxagents-template";

/** A human-readable title for clients that show one. */
export const SERVER_TITLE = "LXAgents Template";

/** One sentence on what the set is; the server's `instructions` open with it. */
export const SERVER_SUMMARY =
  "An example Skill MCP instruction set, served read-only. Replace content/ with your own set.";

/**
 * A name as it appears inside a tool ID: lowercase, and every run of anything that is not a
 * letter or a digit becomes one underscore, trimmed at both ends.
 *
 * `LXAgents-Skills` is `lxagents_skills`; `shared-instruction` is `shared_instruction`.
 */
export function idPart(name: string): string {
  return String(name)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

/**
 * The prefix every tool ID carries: `{org}_{repo}`.
 *
 * A client that connects several servers sees one flat tool list, so a bare `automation`
 * from two servers would collide. The prefix names the server a tool comes from. It is never
 * shortened: when an ID is too long, the base name is.
 */
export const TOOL_PREFIX = `${idPart(ORG_NAME)}_${idPart(REPO_NAME)}`;

/** A base name with the prefix in front of it. */
export function toolId(base: string): string {
  return `${TOOL_PREFIX}_${base}`;
}
