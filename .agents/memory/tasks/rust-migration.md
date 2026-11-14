---
name: memory-tasks-rust-migration
description: Migrating LXAgents-Skills/template from TypeScript to Rust - the official rmcp SDK, axum for HTTP, a cargo xtask init, version still 0.0.0.
---

# Task: migrate the template to Rust

**Status: in progress.**

The owner asked every repository in the organization to move from TypeScript to Rust, using the
official Model Context Protocol Rust SDK (`modelcontextprotocol/rust-sdk`, crate `rmcp`) and
`axum` as the web framework, each repository in its own thread with its own detailed plan. The
owner is away: questions are decided with the recommended option, and the chain may merge
without human review. This record covers the template only; `shared-instruction`, `security`
and `agentic` are migrated separately.

## Goal, objective, detail

- **Goal.** The template every new Skill MCP repository starts from is a Rust server, so every
  repository made from it from now on is Rust too.
- **Objective (done when).** On `master`: no TypeScript, no `package.json`; `cargo build --release`
  produces one binary that serves every file in `content/` as one read-only tool over stdio and
  over Streamable HTTP; `cargo test` passes; `cargo clippy --all-targets -- -D warnings` and
  `cargo fmt --check` are clean; `cargo xtask init --org X --repo Y` turns a copy into a new
  repository and proves its tool list; the version is `0.0.0` everywhere.
- **Detail.** Behaviour is preserved, not redesigned: same tool IDs, same ID rule, same
  64-character cap and boot refusals, same HTTP routes, status codes and JSON-RPC error envelope,
  same `MCP_AUTH_TOKEN` rule, same environment variables, same stdout ban.

## Stack

| Concern | TypeScript today | Rust after |
|---|---|---|
| Toolchain | Node 20+, `tsc` to `dist/` | Rust stable, edition 2024, `rust-version = "1.88"` (rmcp's MSRV); `cargo build` to `target/` |
| MCP SDK | `@modelcontextprotocol/sdk` `McpServer.registerTool` | `rmcp` 3.x, a hand-written `ServerHandler` (`get_info`, `list_tools`, `call_tool`) — tools are data from `content/`, so the `#[tool]` macros do not fit |
| stdio | `StdioServerTransport` | `rmcp::transport::stdio()` (`transport-io`) |
| HTTP | express 5 + `StreamableHTTPServerTransport` | `axum` 0.8 + `rmcp`'s `StreamableHttpService` (`transport-streamable-http-server`), stateless, JSON responses |
| Runtime | Node event loop + `node:cluster` workers | `tokio` multi-thread runtime |
| Token compare | `crypto.timingSafeEqual` over SHA-256 | `sha2` digests compared with `subtle::ConstantTimeEq` |
| Content | read from `content/` at boot | embedded at build time by `build.rs` (`include_str!` per file, `rerun-if-changed=content`) |
| Init | `npm run init` (`tsx scripts/init.ts`) | `cargo xtask init` (workspace member `xtask/`, alias in `.cargo/config.toml`) |
| Tests | `node --test` + `tsx` | `cargo test`: unit tests in modules, integration tests in `tests/` |
| Image | two-stage `node:22-alpine` | two-stage: `rust:1-alpine` builds a static musl binary, `alpine` runs it as a non-root user |

## Target layout

```
Cargo.toml              package lxagents-template 0.0.0, publish = false; workspace with xtask
Cargo.lock              committed (a binary, not a library)
.cargo/config.toml      alias: xtask = "run --package xtask --"
build.rs                walks content/, writes $OUT_DIR/content.rs (path + include_str!)
src/main.rs             entry point: hands argv and env to cli::run
src/lib.rs              the modules, public so tests/ can reach them
src/identity.rs         ORG_NAME, REPO_NAME, SERVER_ID, SERVER_TITLE, SERVER_SUMMARY; id_part, tool_prefix, tool_id — the only file that names the server
src/version.rs          VERSION = env!("CARGO_PKG_VERSION")
src/config.rs           Config::from_env(map), is_loopback, worker count
src/cli.rs              serve | tools | --help | --version; no command = serve from the environment
src/instructions/frontmatter.rs   single-line `key: value` parser
src/instructions/naming.rs        MAX_ID_LENGTH, OVERRIDE_BASES, derived_base, tool_name_for, name_problem
src/instructions/embedded.rs      the build.rs output, as &[(path, text)]
src/instructions/surface.rs       build(files, overrides) -> Result<Surface, SurfaceError>: every boot refusal
src/server.rs           SkillServer: rmcp ServerHandler; list_tools() summary for the CLI
src/stdio.rs            serve over stdio; log to stderr
src/http/mod.rs         start(config): token check, bind, startup lines, graceful shutdown
src/http/app.rs         router(options) -> axum Router: host check, auth, body limit, /healthz, /mcp, 405, 404
src/http/auth.rs        MIN_TOKEN_LENGTH, configured_token, token_problem, bearer_credential, require_bearer
src/http/host.rs        Host allow-list middleware (configured list, or localhost when tokenless)
src/http/rpc_error.rs   the JSON-RPC error envelope
tests/*.rs              naming, config, server, stdio, http, template_version
xtask/src/main.rs       init: flags, identity, rewrites, removals, cargo build, tools check
xtask/src/{args,identity,rewrite}.rs   the pure parts, unit-tested
```

Removed: `package.json`, `package-lock.json`, `tsconfig.json`, `tsconfig.check.json`,
`src/**/*.ts`, `test/`, `scripts/`.

## Behaviour that must survive, and how it is checked

| Behaviour | Test |
|---|---|
| One tool per `.md` in `content/`, ID `{org}_{repo}_{base}` | `tests/server.rs` |
| Tool text returned byte for byte, frontmatter included | `tests/server.rs`, `tests/stdio.rs` |
| Boot refuses: invalid name, >64 chars, collision, stale override, missing description, empty set | unit tests in `surface.rs`, `tests/naming.rs` |
| No tool takes an argument (input schema `{"type":"object"}` with no properties) | `tests/server.rs` |
| `instructions` open with `SERVER_SUMMARY` and name `{prefix}_automation` | `tests/server.rs` |
| stdio: initialize, tools/list, tools/call answered; nothing but JSON-RPC on stdout | `tests/stdio.rs` (spawns the built binary) |
| `GET /healthz` → 200 `{status, server, version}`, no token | `tests/http.rs` |
| Every other route needs `Authorization: Bearer`; 401 + `WWW-Authenticate`, `invalid_token` on a wrong one | `tests/http.rs` |
| Token check before the body is read and before routing (404/405 hidden from a tokenless caller) | `tests/http.rs` |
| `HEAD /healthz` and `/healthz/` are not exempt | `tests/http.rs` |
| Token < 32 chars refused, even on loopback; token required off loopback | unit tests in `auth.rs`, `tests/http.rs` (process exits 1) |
| Tokenless loopback: `Host` limited to localhost unless `MCP_ALLOWED_HOSTS` | `tests/http.rs` |
| `MCP_ALLOWED_HOSTS` guards `/healthz` too | `tests/http.rs` |
| `POST /mcp` stateless; other methods on `/mcp` → 405 `-32000`; unknown path → 404 `-32601` | `tests/http.rs` |
| Malformed or oversized (> 4 MiB) body → 400 `-32700` | `tests/http.rs` |
| No `X-Powered-By` or server banner | `tests/http.rs` |
| Version `0.0.0` in `Cargo.toml`, `Cargo.lock`, `--version`, `/healthz`, the handshake; no tags | `tests/template_version.rs` |
| `init` rewrites identity, strips template-only blocks and files, clears history, builds, lists tools | `xtask` unit tests + an end-to-end test on a temporary copy |

## Plan

| # | Task | Branch | Files / areas | PR |
|---|---|---|---|---|
| 1 | This record | `plan/rust-migration` | `.agents/memory/tasks/rust-migration.md`, `memory-index.md` | |
| 2 | Rust server, tests, xtask init, Docker; TypeScript removed | `refactor/rust` | `Cargo.*`, `build.rs`, `src/`, `tests/`, `xtask/`, `.cargo/`, `Dockerfile`, `.dockerignore`, `.gitignore` | |
| 3 | Rules, README, AGENTS.md, both wikis | `docs/rust` | `AGENTS.md`, `README.md`, `.agents/rules/`, `.agents/wiki/`, `wiki/` | |
| 4 | Changelog, close this record | `release/0.0.0` | `wiki/logs/0/0/0/CHANGELOG.md`, this record | |

Each branch stacks on the one before it. Each pull request is rebased onto `master` and merged
in order, and each branch is deleted as it merges where the platform allows it.

## What does not change

- Version `0.0.0`. The release branch is `release/0.0.0` and carries release notes only.
- `content/`: the served files, and so every tool ID.
- Environment variable names and meanings, except that `MCP_CLUSTER_WORKERS` now sets the
  number of tokio worker threads instead of forked processes.
- Other repositories.

## Rules set aside, by the owner's instruction

- **`tsc` is the only build** (`.agents/rules/repository.md`). The owner asked for Rust; task 3
  rewrites the rule for `cargo`.
- **The TypeScript SDK calls are not changed for style** (`.agents/rules/repository.md`). They
  are replaced, because the language is.
- **Ask before opening and before merging a pull request.** The owner allowed merging without
  human review and asked that questions be decided with the recommended option.

## Decisions taken without the owner

- **D1 — content is embedded at build time.** One self-contained binary: no path to resolve at
  run time, nothing for a caller to traverse, and the image is the binary alone. A content edit
  needs a rebuild; `cargo run` and `cargo build` rebuild on their own because `build.rs` watches
  `content/`. The boot checks still run at boot, and `cargo test` runs them too.
- **D2 — a hand-written `ServerHandler`, not `#[tool_router]`.** The tools are data, not
  functions.
- **D3 — `rmcp`'s own `Host` check is turned off and done in an axum middleware.** One place,
  ahead of `/healthz` and the token check, with the same rules as before.
- **D4 — stateless Streamable HTTP with JSON responses** (`legacy_session_mode = false`,
  `json_response = true`), the same as `sessionIdGenerator: undefined` today. Only `POST /mcp`
  reaches the service.
- **D5 — one binary, `lxagents-template`.** No command serves from the environment (what
  `dist/index.js` did); `serve`, `tools`, `--help` and `--version` are what `dist/cli.js` did.
  Client registrations name `target/release/lxagents-template` with no arguments.
- **D6 — `MCP_CLUSTER_WORKERS` keeps its name** and sets tokio worker threads; `1` is a
  current-thread runtime, unset is one per CPU. A rename would break existing deployments for
  no gain.
- **D7 — `cargo xtask init`** is the Rust idiom for a repository task, and keeps the rename
  script out of the shipped binary. It stays in new repositories, as `scripts/init.ts` did.
- **D8 — hand-rolled argument parsing.** Four flags do not justify `clap`.
- **D9 — `Cargo.lock` is committed**, as Rust does for binaries, and is a version carrier the
  pin test reads.
- **D10 — `rust-version = "1.88"`**, `rmcp`'s own minimum.
- **D11 — branch names follow the shared conventions** (`plan/`, `refactor/`, `docs/`,
  `release/0.0.0`), not the harness's `claude/` branch, as `AGENTS.md` requires.

## Progress

### Task 1 — plan/rust-migration

Landed: this record and its row in `memory-index.md`. No code yet.
