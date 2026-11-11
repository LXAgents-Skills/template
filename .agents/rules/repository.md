---
name: repository-rules
description: Rules for this repository - where things go, the commands, the stdout ban, install by git clone, and what must not be introduced.
---

# Repository Rules

## Where things go

| Thing | Path |
|---|---|
| The served set | `content/` |
| The hub over it | `content/automation.md` |
| Org, repo, server ID, title, summary | `src/identity.js` — the only file that names the server |
| The ID rule and `OVERRIDE_BASES` | `src/instructions/naming.js` |
| The surface, derived from `content/` | `src/instructions/from-content.js` |
| `registerTool` and `listTools()` | `src/server.js` |
| Every environment variable | `src/config.js` (except `MCP_AUTH_TOKEN`, read in `src/http/auth.js`) |
| Entry point and transports | `src/index.js` → `src/run.js` → `src/stdio.js` or `src/http/` |
| The command line | `src/cli.js` |
| The rename script | `scripts/init.js`, with its pure parts in `scripts/init/` |
| Tests | `test/{subject}.test.js` |

## Commands

```bash
npm install          # Node 20+, no build step
npm test             # node --test
npm run tools        # every tool ID, its length, and its description
npm start            # stdio
npm run start:http   # HTTP on 127.0.0.1:3000, no token needed
npm run inspect      # MCP Inspector against the stdio server
npm run init -- --org <org> --repo <repo>   # once, in a new repository made from the template
```

## Nothing writes to stdout except the CLI

On stdio, stdout is the JSON-RPC channel. Server-side logging goes to stderr; only CLI
commands write to stdout.

## Installed by git clone

People install this server by cloning it and running `node src/index.js`, not from npm or
`npx`. `package.json` is `private`, so it cannot be published by accident. Docs give the clone
path first.

## What must not be introduced

* A build step.
* A second list of tools, or a tool registered outside `src/server.js`.
* A tool that takes an argument, a verb, a credential, or reaches a network.
* A source file other than `src/identity.js` that spells the org, repo or server ID.
* A copy of the shared instruction set. Read it through its connector.
* A change to the `@modelcontextprotocol/sdk` calls (`registerTool`, `McpServer`, the
  transports) made for style alone.
