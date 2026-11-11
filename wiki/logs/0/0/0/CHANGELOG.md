# 0.0.0

The template's only version. It is copied, never released, so it stays at `0.0.0` and every
repository made from it starts its own history there. Each change to the template adds a dated
section below, newest first.

## 2026-11-11 — Migrate to TypeScript

The server, its tests and `npm run init` are TypeScript under `strict`. `tsc` compiles `src/`
into `dist/`, and everything that runs the server runs `dist/`. Tool IDs, the tool surface, the
SDK calls and HTTP auth are unchanged. The version stays `0.0.0`.

### Added

- `tsconfig.json` (ES2022, NodeNext, `strict`, `src/` to `dist/`) and `tsconfig.check.json`
  (type-checks `src/`, `test/` and `scripts/` without output).
- Scripts `build` and `prepare` (`tsc`), and `typecheck`.
- Dev dependencies `typescript` 7.0.2, `@types/node` 26.6.6, `tsx` 4.23.15,
  `@types/express` 5.0.6.

### Modified

- Every `.js` in `src/`, `test/` and `scripts/` is now `.ts`, with types for the config, the
  server options, the served tools, the HTTP app options and the init flags and identity.
- Dependencies at their latest: `@modelcontextprotocol/sdk` 1.32.2, `express` 5.3.0, `zod`
  4.6.7. `npm audit` reports 0 vulnerabilities.
- `main`, `bin`, `start`, `start:http`, `cli`, `tools` and `inspect` run `dist/`. `test` is
  `node --import tsx --test test/*.test.ts`; `init` is `tsx scripts/init.ts`.
- Install by git clone is unchanged in shape: `npm install` builds `dist/` through `prepare`.
  Client registrations point at `dist/index.js` instead of `src/index.js`.
- `npm run init` rewrites `src/identity.ts`, writes `dist/` bins, rebuilds `dist/`, and boots the
  tool list from it. Its end-to-end test accepts any new-prefix tool rather than the template's
  example, and checks the repository map, whose version-test note `init` now strips.
- The Docker image builds in two stages; the compiler stays out of the runtime image.
- The repository rule against a build step now describes the one `tsc` build instead.

### Deleted

- The JavaScript sources, replaced by the TypeScript ones above.

## 2026-11-11 — Pin the version at 0.0.0

### Added

- `.agents/rules/template-version.md` (template only): the version is `0.0.0` for good, and a
  template release is always `release/0.0.0` with release notes only.
- `test/template-version.test.js` (template only): fails if `package.json`, the lockfile or
  the version the server reports leaves `0.0.0`.

### Modified

- `package.json`, `package-lock.json`: `0.1.0` to `0.0.0`. `--version`, `/healthz` and the
  MCP handshake report `0.0.0`.
- `npm run init` deletes the version rule and test, so a new repository can release normally.
- `AGENTS.md`, `README.md`, the overview, the agents index, the repository map and the
  create-a-new-repository guide say the template stays at `0.0.0`.

### Deleted

- `wiki/logs/0/1/0/`. Its notes are the next section. No tag or package was ever published
  for `0.1.0`.

## 2026-11-11 — The first template (briefly numbered 0.1.0)

The first version. The template is a working Skill MCP server, and the starting point for every
new Skill MCP repository: create a repository from it, run `npm run init`, replace `content/`.

### Added

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

### For consumers

Install by git clone — see `wiki/guides/install-by-git-clone.md`. `package.json` is `private`;
nothing is published to npm.
