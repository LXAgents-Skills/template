# 0.1.0

**Released:** 2026-11-11

The first release. The template is a working Skill MCP server, and the starting point for every
new Skill MCP repository: create a repository from it, run `npm run init`, replace `content/`.

## Added

- A read-only Skill MCP server over stdio and HTTP, with the layout of
  `LXAgents-Skills/security`: `config`, `run`, `stdio`, `http/` and a CLI. Tools are registered
  with `registerTool`, and `listTools()` is the one list the CLI and tests read.
- `src/identity.js`, the only file that names the server. `TOOL_PREFIX` (`{org}_{repo}`) and
  `toolId()` are derived from `ORG_NAME` and `REPO_NAME`.
- Tool IDs `{org}_{repo}_{base}`, underscores only, at most 64 characters, enforced at boot.
  `OVERRIDE_BASES` shortens the base only, never the prefix.
- `content/automation.md`, the hub, and one example tool. The tools at release:
  `lxagents_skills_template_automation` and `lxagents_skills_template_example_guide`.
- HTTP auth: `MCP_AUTH_TOKEN` (32+ characters) is required only when HTTP binds off loopback.
  Loopback without a token answers only `localhost` `Host` headers. `npm run start:http` binds
  `127.0.0.1`.
- `npm run init -- --org <org> --repo <repo>`: renames a fresh copy (identity, package,
  lockfile, prefix, slug and server ID in docs and the Dockerfile), drops template-only text,
  clears the template's history, and boots the tool list to check the new IDs. It refuses to
  run against the template itself without `--force`.
- `npm run tools`: every ID with its length.
- 38 tests, written against whatever `content/` holds so a new repository keeps them: the
  file/tool bijection, the ID rule, no arguments, byte-for-byte content, the hub listing every
  tool once, naming errors, config, HTTP auth, and `init` end to end.
- `AGENTS.md`, `.agents/` rules (`tool-ids`, `tool-authoring`, `automation-hub`, `http-auth`,
  `repository`), indexes and repository map; `wiki/` guides for creating a repository, adding
  a tool, installing by git clone, environment and Docker.
- A single-stage `node:22-alpine` Dockerfile.

## For consumers

Install by git clone — see `wiki/guides/install-by-git-clone.md`. `package.json` is `private`;
nothing is published to npm.
