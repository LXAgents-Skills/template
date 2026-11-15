# Architecture

## From a request to a file

The source is Rust. `cargo build` compiles one binary, `lxagents-template`, with every file in
`content/` inside it.

1. At build time, `build.rs` embeds each `.md` under `content/` as a `(path, text)` pair, which
   `src/instructions/embedded.rs` exposes as `FILES`.
2. `src/main.rs` parses the command line (`src/cli.rs`), reads the environment once
   (`src/config.rs`), and checks the surface. With no command it serves the transport
   `MCP_TRANSPORT` names: stdio through `src/stdio.rs`, or HTTP through `src/http/mod.rs`,
   which refuses to start without a required token, binds, and serves the router from
   `src/http/app.rs` on a tokio runtime with `MCP_CLUSTER_WORKERS` threads.
3. The surface is built once by `src/instructions/surface.rs`: one tool per file, its ID from
   `src/instructions/naming.rs`, its description from the file's frontmatter, and the file's
   text returned byte for byte.
4. `SkillServer` in `src/server.rs` implements `rmcp`'s `ServerHandler` over that surface:
   `get_info` (name, title, version, instructions), `list_tools` and `call_tool`. stdio serves
   one; HTTP builds one per request, which costs nothing because it only points at the surface.

## Why the set is checked at boot

A file without a description, an ID over 64 characters, two files with one ID, or an override
naming a missing file all stop the process with a message naming the file and the fix. A set
that cannot be routed on should not start. The check runs at boot rather than in `build.rs`, so
it fails the same way however the binary was built, and `cargo test` runs it as well.

## One name, one place

`src/identity.rs` holds the org, repo, server ID, title and summary. The tool prefix, every ID,
the health check and every log line come from it, which is what lets `cargo xtask init` rename
a copy by rewriting that one file, the manifests and the documentation.

## HTTP

The router is axum. Requests pass, in order, the `Host` allow-list (`src/http/host.rs`), the
bearer check (`src/http/auth.rs`), and on `POST /mcp` a body check that refuses anything over
4 MiB or not JSON. Then `rmcp`'s Streamable HTTP service answers, stateless and with plain JSON
responses: no sessions, nothing to keep bounded. `GET /healthz` is the only route without a
token. Any other method on `/mcp` is a 405, any other path a 404, all in the JSON-RPC error
envelope. See [Environment](../environments/env.md).
