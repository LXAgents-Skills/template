---
name: repository-rules
description: Rules for this repository - where things go, the commands, the Rust build, the stdout ban, install by git clone, and what must not be introduced.
---

# Repository Rules

## Where things go

| Thing | Path |
|---|---|
| The served set | `content/` |
| The hub over it | `content/automation.md` |
| Embedding the set into the binary | `build.rs` → `src/instructions/embedded.rs` |
| Org, repo, server ID, title, summary | `src/identity.rs` — the only source file that names the server |
| The ID rule and `OVERRIDE_BASES` | `src/instructions/naming.rs` |
| The surface and every boot check | `src/instructions/surface.rs` |
| The `rmcp` `ServerHandler` and `list_tools()` | `src/server.rs` |
| Every environment variable | `src/config.rs` (except `MCP_AUTH_TOKEN`, read in `src/http/auth.rs`) |
| Entry point and transports | `src/main.rs` → `src/stdio.rs` or `src/http/` |
| The command line, parsed | `src/cli.rs` |
| The rename task | `xtask/src/main.rs`, with its pure parts in `xtask/src/{args,identity,rewrite}.rs` |
| Tests | unit tests beside the code; integration tests in `tests/{subject}.rs`; init end to end in `xtask/tests/init.rs` |
| Build output | `target/`, git-ignored |

## Commands

```bash
cargo build --release                 # target/release/lxagents-template
cargo test --workspace                # every test, the init end-to-end test included
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo run -- tools                    # every tool ID, its length, and its description
cargo run                             # stdio
cargo run -- serve --http --host 127.0.0.1   # HTTP on 127.0.0.1:3000, no token needed
npx @modelcontextprotocol/inspector target/release/lxagents-template   # MCP Inspector, optional
cargo xtask init -- --org <org> --repo <repo>   # once, in a new repository made from the template
```

## Rust, built by cargo

The source is Rust, edition 2024, `rust-version = "1.88"` (the minimum `rmcp` supports). One
crate builds the library `skill_server` and the binary `lxagents-template`; `xtask` is a second
workspace member that only `cargo xtask` runs, so the rename task never ships in the binary.

* **`content/` is compiled into the binary.** `build.rs` writes one `include_str!` per `.md` file
  and tells cargo to rerun whenever `content/` changes, so `cargo build`, `cargo run` and
  `cargo test` always serve the set as it is on disk. A running server serves what it was built
  with: after editing `content/`, rebuild and restart.
* The boot checks still run at boot, not in `build.rs`, so a malformed set fails the same way
  however it was built, and `cargo test` runs them too.
* `Cargo.lock` is committed, as it is for any binary. `target/` never is.
* `clippy -D warnings` and `rustfmt` are clean on every commit.

## Nothing writes to stdout except the CLI

On stdio, stdout is the JSON-RPC channel. Server-side logging goes to stderr (`eprintln!`); only
the CLI's own commands (`tools`, `--help`, `--version`) write to stdout.

## Installed by git clone

People install this server by cloning it, running `cargo build --release`, and registering
`target/release/lxagents-template`, not from crates.io. `Cargo.toml` has `publish = false`, so it
cannot be published by accident. Docs give the clone path first.

## What must not be introduced

* A second build step, or committed build output.
* `unsafe`, or a lint silenced with `#[allow]` where the code could be fixed instead.
* A second list of tools, or a tool listed or answered outside `src/server.rs`.
* A tool that takes an argument, a verb, a credential, or reaches a network.
* A source file other than `src/identity.rs` that spells the org, repo or server ID. Tests use
  `SERVER_ID` and `tool_id()`; the one exception is the `CARGO_BIN_EXE_*` macro, which `init`
  rewrites.
* A copy of the shared instruction set. Read it through its connector.
* A change to the `rmcp` calls (`ServerHandler`, `StreamableHttpService`, `stdio()`) made for
  style alone.
