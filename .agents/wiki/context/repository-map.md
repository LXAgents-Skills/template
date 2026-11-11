---
name: repository-map
description: Every source file in this repository and the one thing it is responsible for, plus how a request flows through them.
---

# Repository Map

## Flow

```
src/index.js ─ loadConfig() ─ run()
                                ├─ stdio  → src/stdio.js ─ createServer()
                                └─ http   → src/http/primary.js ─ fork ─ src/http/worker.js ─ createApp() ─ createServer()

createServer()  (src/server.js) ─ registerTool() for each entry in CONTENT_INSTRUCTIONS
CONTENT_INSTRUCTIONS (src/instructions/from-content.js) ← content/**/*.md, read once at import
```

## Files

| File | Responsible for |
|---|---|
| `src/identity.js` | Naming the server: org, repo, server ID, title, summary; derives `TOOL_PREFIX` and `toolId()`. |
| `src/paths.js` | `ROOT` and `CONTENT_DIR`. |
| `src/version.js` | Reading the version from `package.json`. |
| `src/config.js` | Reading every environment variable into one frozen object. |
| `src/instructions/frontmatter.js` | Parsing single-line frontmatter fields. |
| `src/instructions/naming.js` | The tool ID rule: base derivation, `OVERRIDE_BASES`, the 64-character cap. |
| `src/instructions/from-content.js` | Building the tool list from `content/` and refusing a malformed set. |
| `src/server.js` | Registering the tools on a fresh `McpServer`; `listTools()`. |
| `src/run.js` | Choosing the transport. |
| `src/stdio.js` | The stdio transport. |
| `src/http/app.js` | The express app: `Host` check, token check, `/mcp`, `/healthz`, errors. |
| `src/http/auth.js` | The bearer token: reading it, judging it, checking a request. |
| `src/http/primary.js` | Refusing to start without a required token; forking workers. |
| `src/http/worker.js` | Listening, and draining on shutdown. |
| `src/http/rpc-error.js` | The JSON-RPC error envelope. |
| `src/cli.js` | `serve`, `tools`, `--help`, `--version`. |
| `scripts/init.js` | Renaming a fresh copy of the template. |
| `scripts/init/*.js` | The rename's pure parts: flags, identity defaults, text rewrites. |
