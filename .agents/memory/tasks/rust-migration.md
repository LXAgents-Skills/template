---
name: memory-tasks-rust-migration
description: Migrating LXAgents-Skills/template from TypeScript to Rust - the official rmcp SDK, axum for HTTP, a cargo xtask init, version still 0.0.0.
---

# Task: migrate the template to Rust

**Status: done.**

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
| 1 | This record | `plan/rust-migration` | `.agents/memory/tasks/rust-migration.md`, `memory-index.md` | #15 |
| 2 | Rust server, tests, xtask init, Docker; TypeScript removed | `refactor/rust` | `Cargo.*`, `build.rs`, `src/`, `tests/`, `xtask/`, `.cargo/`, `Dockerfile`, `.dockerignore`, `.gitignore` | #16 |
| 3 | Rules, README, AGENTS.md, both wikis | `docs/rust` | `AGENTS.md`, `README.md`, `.agents/rules/`, `.agents/wiki/`, `wiki/` | #17 |
| 4 | Changelog, close this record | `release/0.0.0` | `wiki/logs/0/0/0/CHANGELOG.md`, this record | #18 |

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

### Task 2 — refactor/rust

Landed. The TypeScript server, tests and init script are removed (`package.json`, the lockfile,
both `tsconfig` files, `src/**/*.ts`, `test/`, `scripts/`), and the Rust server replaces them in
the layout above.

- `Cargo.toml`: package `lxagents-template` 0.0.0, `publish = false`, library `skill_server`,
  binary `lxagents-template`; a workspace with `xtask`. Dependencies: `rmcp` 3.10 (`server`,
  `transport-io`, `transport-streamable-http-server`), `axum` 0.8, `tokio`, `tokio-util`,
  `serde_json`, `sha2`, `subtle`. `Cargo.lock` committed.
- `build.rs` embeds every `.md` under `content/`; `src/instructions/surface.rs` runs every boot
  check (invalid name, over 64, collision, stale override, missing description, empty set).
- `src/server.rs` is a hand-written `ServerHandler`; every tool's input schema is an object with
  no properties. An unknown tool is `-32602 Tool <name> not found`.
- `src/http/`: axum router with the `Host` allow-list (`host.rs`), the bearer check (`auth.rs`,
  SHA-256 digests compared with `subtle`), a JSON body check (`require_json`: malformed or over
  4 MiB is 400 `-32700`), `GET /healthz`, `POST /mcp` to `rmcp`'s stateless JSON service, 405 for
  other methods on `/mcp`, 404 for the rest. `rmcp`'s own `Host` check is off. Graceful shutdown
  on SIGTERM and SIGINT with a 5-second grace.
- `cargo xtask init` (`xtask/`): flags, identity defaults, constant rewrite in `src/identity.rs`,
  `Cargo.toml` description and keywords, prefix/slug/server-ID swap in markdown, both manifests,
  `tests/*.rs` and the Dockerfile, template-only blocks and files, history cleared, then
  `cargo run -- tools` on the renamed server. `xtask` depends on the server crate for its
  identity, so the prefix rule has one implementation.
- Tests name the server through `SERVER_ID` and `tool_id`, not literals, except the
  `CARGO_BIN_EXE_*` macro, which `init` rewrites.
- Dockerfile: `rust:1.97-alpine` builds `--release --locked`; `alpine:3.22` runs it as uid 10001.

Checks: `cargo test --workspace` 58 passed (48 in the server crate, 10 in `xtask`, including
init end to end on a temporary copy); `cargo clippy --workspace --all-targets -- -D warnings`
and `cargo fmt --check` clean; stdio and HTTP answered by hand (initialize, tools/list,
tools/call; 0.0.0.0 without a token exits 1; SIGTERM drains). **Not verified:** the Docker
image, because this environment has the docker client but no daemon.

### Task 3 — docs/rust

Landed. Every source path in the rules and docs names its `.rs` file, and every command is a
`cargo` command or the built binary. `.agents/rules/repository.md` is rewritten: the layout,
the cargo commands, a "Rust, built by cargo" section (content compiled in, boot checks at
boot, `Cargo.lock` committed, clippy and rustfmt clean), install by clone with `publish = false`,
and the bans (no `unsafe`, no second tool list, no source file but `src/identity.rs` naming the
server). `tool-ids.md` shows `OVERRIDE_BASES` as a Rust slice; `http-auth.md` names `router()`
and the single `Host` check; `template-version.md` lists `Cargo.toml` and `Cargo.lock` as the
carriers. The install guide registers `target/release/lxagents-template` with no `cwd`; the
Docker, environment, architecture, overview, add-a-tool and create-a-new-repository pages,
`README.md`, `AGENTS.md` and the repository map describe the Rust build. `cargo test --workspace`
passes, the init end-to-end test included, so every template-only block still strips.

### Task 4 — release/0.0.0

Landed. A dated "Migrate to Rust" section at the top of `wiki/logs/0/0/0/CHANGELOG.md`.
`Cargo.toml`, `Cargo.lock` and tags untouched. PR column filled. Record done.

Left open: the Docker image is unbuilt here (no daemon); `cargo audit` is not installed, so no
advisory scan was run; GitHub's "Template repository" box and deleting merged branches are the
owner's.
