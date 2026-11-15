# 0.0.0

The template's only version. It is copied, never released, so it stays at `0.0.0` and every
repository made from it starts its own history there. Each change to the template adds a dated
section below, newest first.

## 2026-11-15 — Migrate to Rust

The server, its tests and the rename step are Rust, on the official MCP SDK (`rmcp` 3.10) with
axum 0.8 for HTTP. `cargo build --release` produces one binary, `lxagents-template`, with
`content/` compiled into it. Tool IDs, the ID rule, the boot checks, the HTTP routes and
refusals, `MCP_AUTH_TOKEN` and the environment variables are unchanged. The version stays
`0.0.0`.

### Added

- `Cargo.toml` (package `lxagents-template` 0.0.0, `publish = false`, edition 2024,
  `rust-version` 1.88) and a committed `Cargo.lock`; dependencies `rmcp` 3.10.0, `axum` 0.8.9,
  `tokio` 1.54.1, `tokio-util` 0.7.20, `serde_json` 1.0.154, `sha2` 0.10.9, `subtle` 2.6.1.
- `build.rs`, which embeds every `.md` under `content/` and reruns when it changes.
- `xtask/` and `.cargo/config.toml`: `cargo xtask init -- --org <org> --repo <repo>` replaces
  `npm run init`, with the same flags, and is tested end to end on a temporary copy.
- Integration tests in `tests/` and unit tests beside the code: 58 in all.

### Modified

- The server is a hand-written `rmcp` `ServerHandler`. stdio uses `rmcp`'s stdio transport;
  HTTP mounts `rmcp`'s stateless Streamable HTTP service (JSON responses, no sessions) in an
  axum router with the `Host` allow-list, the bearer check (SHA-256 compared in constant time)
  and a 4 MiB JSON body check in front of it. `rmcp`'s own `Host` check is off, so there is one.
- One binary replaces `dist/index.js` and `dist/cli.js`: with no arguments it serves from the
  environment; `serve`, `tools`, `--help` and `--version` are as before. Client registrations
  name `target/release/lxagents-template` and need no `cwd`.
- `MCP_CLUSTER_WORKERS` sets tokio worker threads instead of forked processes; `1` runs on one
  thread.
- A change to `content/` needs a rebuild, because the set is compiled in.
- The Docker image builds a static musl binary on `rust:1.97-alpine` and runs it on
  `alpine:3.22` as uid 10001.
- Every rule, guide and map describes the cargo build.

### Deleted

- `package.json`, `package-lock.json`, `tsconfig.json`, `tsconfig.check.json`, the TypeScript
  sources in `src/`, `test/` and `scripts/`.

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
