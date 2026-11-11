---
name: repository-map
description: Every source file in this repository and the one thing it is responsible for, plus how a request flows through them.
---

# Repository Map

## Flow

Source files are listed. Each compiles to the same path under `dist/` with `.js`, and `dist/` is
what runs.

```
src/index.ts ─ loadConfig() ─ run()
                                ├─ stdio  → src/stdio.ts ─ createServer()
                                └─ http   → src/http/primary.ts ─ fork ─ src/http/worker.ts ─ createApp() ─ createServer()

createServer()  (src/server.ts) ─ registerTool() for each entry in CONTENT_INSTRUCTIONS
CONTENT_INSTRUCTIONS (src/instructions/from-content.ts) ← content/**/*.md, read once at import
```

## Files

| File | Responsible for |
|---|---|
| `src/identity.ts` | Naming the server: org, repo, server ID, title, summary; derives `TOOL_PREFIX` and `toolId()`. |
| `src/paths.ts` | `ROOT` and `CONTENT_DIR`. |
| `src/version.ts` | Reading the version from `package.json`. |
| `src/config.ts` | Reading every environment variable into one frozen object. |
| `src/instructions/frontmatter.ts` | Parsing single-line frontmatter fields. |
| `src/instructions/naming.ts` | The tool ID rule: base derivation, `OVERRIDE_BASES`, the 64-character cap. |
| `src/instructions/from-content.ts` | Building the tool list from `content/` and refusing a malformed set. |
| `src/server.ts` | Registering the tools on a fresh `McpServer`; `listTools()`. |
| `src/run.ts` | Choosing the transport. |
| `src/stdio.ts` | The stdio transport. |
| `src/http/app.ts` | The express app: `Host` check, token check, `/mcp`, `/healthz`, errors. |
| `src/http/auth.ts` | The bearer token: reading it, judging it, checking a request. |
| `src/http/primary.ts` | Refusing to start without a required token; forking workers. |
| `src/http/worker.ts` | Listening, and draining on shutdown. |
| `src/http/rpc-error.ts` | The JSON-RPC error envelope. |
| `src/cli.ts` | `serve`, `tools`, `--help`, `--version`. |
| `scripts/init.ts` | Renaming a fresh copy of the template. |
| `scripts/init/*.ts` | The rename's pure parts: flags, identity defaults, text rewrites. |
| `tsconfig.json` | Compiling `src/` to `dist/`: ES2022, NodeNext, `strict`. |
| `tsconfig.check.json` | Type-checking `src/`, `test/` and `scripts/` without output. |

<!-- template-only -->
Template only, deleted by `init`: `test/template-version.test.ts`, which keeps the template's
version at `0.0.0`.
<!-- /template-only -->
