---
name: repository-map
description: Every source file in this repository and the one thing it is responsible for, plus how a request flows through them.
---

# Repository Map

## Flow

```
build.rs ─ content/**/*.md → $OUT_DIR/content.rs ─ src/instructions/embedded.rs (FILES)

src/main.rs ─ cli::parse() ─ Config::from_lookup() ─ instructions::surface() ─ SkillServer::new()
                                                     ├─ stdio → src/stdio.rs ─ rmcp stdio()
                                                     └─ http  → src/http/mod.rs ─ app::router() ─ axum::serve()

app::router()  (src/http/app.rs) ─ host check ─ bearer check ─ /healthz | POST /mcp → require_json → rmcp StreamableHttpService
SkillServer    (src/server.rs)   ─ ServerHandler: get_info, list_tools, call_tool over the surface
surface()      (src/instructions/mod.rs) ← surface::build(FILES, OVERRIDE_BASES), checked once
```

## Files

| File | Responsible for |
|---|---|
| `build.rs` | Embedding every `.md` under `content/`, and rerunning when it changes. |
| `src/main.rs` | Carrying out the parsed command: the runtime, the transport, exit codes. |
| `src/lib.rs` | Declaring the modules, so `tests/` and `xtask` can reach them. |
| `src/cli.rs` | Parsing `serve`, `tools`, `--help`, `--version`; the help text. |
| `src/identity.rs` | Naming the server: org, repo, server ID, title, summary; derives `TOOL_PREFIX` and `tool_id()`. |
| `src/version.rs` | The version, compiled in from `Cargo.toml`. |
| `src/config.rs` | Reading every environment variable into one `Config`. |
| `src/instructions/embedded.rs` | The set as `build.rs` embedded it. |
| `src/instructions/frontmatter.rs` | Parsing single-line frontmatter fields. |
| `src/instructions/naming.rs` | The tool ID rule: base derivation, `OVERRIDE_BASES`, the 64-character cap. |
| `src/instructions/surface.rs` | Building the tool list and refusing a malformed set. |
| `src/instructions/mod.rs` | The checked surface, built once. |
| `src/server.rs` | The `rmcp` `ServerHandler`; `list_tools()` for the CLI. |
| `src/stdio.rs` | The stdio transport. |
| `src/http/mod.rs` | Refusing to start without a required token; binding; startup lines; draining on a signal. |
| `src/http/app.rs` | The axum router: routes, body check, the stateless `rmcp` service, 404 and 405. |
| `src/http/auth.rs` | The bearer token: reading it, judging it, checking a request in constant time. |
| `src/http/host.rs` | The `Host` allow-list. |
| `src/http/rpc_error.rs` | The JSON-RPC error envelope. |
| `xtask/src/main.rs` | `cargo xtask init`: renaming a fresh copy of the template, then proving its tools. |
| `xtask/src/{args,identity,rewrite}.rs` | The rename's pure parts: flags, identity defaults, text rewrites. |
| `tests/*.rs` | Integration tests: naming, config, server, stdio, http. |
| `xtask/tests/init.rs` | `init` end to end, on a temporary copy. |

<!-- template-only -->
Template only, deleted by `init`: `tests/template_version.rs`, which keeps the template's
version at `0.0.0`.
<!-- /template-only -->
