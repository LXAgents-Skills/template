---
name: repository-rules
description: Rules for this repository - where things go, the commands, the TypeScript build, the stdout ban, install by git clone, and what must not be introduced.
---

# Repository Rules

## Where things go

| Thing | Path |
|---|---|
| The served set | `content/` |
| The hub over it | `content/automation.md` |
| Org, repo, server ID, title, summary | `src/identity.ts` — the only file that names the server |
| The ID rule and `OVERRIDE_BASES` | `src/instructions/naming.ts` |
| The surface, derived from `content/` | `src/instructions/from-content.ts` |
| `registerTool` and `listTools()` | `src/server.ts` |
| Every environment variable | `src/config.ts` (except `MCP_AUTH_TOKEN`, read in `src/http/auth.ts`) |
| Entry point and transports | `src/index.ts` → `src/run.ts` → `src/stdio.ts` or `src/http/` |
| The command line | `src/cli.ts` |
| The rename script | `scripts/init.ts`, with its pure parts in `scripts/init/` |
| Tests | `test/{subject}.test.ts` |
| Compiler settings | `tsconfig.json` (build, `src/` to `dist/`); `tsconfig.check.json` (type-checks `src/`, `test/`, `scripts/`) |
| Compiled output | `dist/`, git-ignored, never edited by hand |

## Commands

```bash
npm install          # Node 20+; `prepare` compiles src/ into dist/
npm run build        # tsc: src/ to dist/
npm run typecheck    # tsc over src/, test/ and scripts/, no output
npm test             # node --import tsx --test test/*.test.ts, from source
npm run tools        # every tool ID, its length, and its description
npm start            # stdio
npm run start:http   # HTTP on 127.0.0.1:3000, no token needed
npm run inspect      # MCP Inspector against the stdio server
npm run init -- --org <org> --repo <repo>   # once, in a new repository made from the template
```

## TypeScript, compiled to `dist/`

The source is TypeScript under `strict`. `tsc` compiles `src/` into `dist/`, and everything
that runs the server — `npm start`, the `bin` entries, client registrations, the Docker image —
runs `dist/`. Tests and `npm run init` run from source through `tsx`, so neither needs a build
first.

* Imports between source files end in `.js`, as NodeNext resolution requires, even though the
  file on disk is `.ts`.
* `dist/` is never committed. `prepare` rebuilds it on every `npm install`, which is the install
  step a clone already takes. After changing `src/`, run `npm run build` before `npm start`.
* `scripts/` and `test/` are type-checked by `npm run typecheck` but never compiled.

## Nothing writes to stdout except the CLI

On stdio, stdout is the JSON-RPC channel. Server-side logging goes to stderr; only CLI
commands write to stdout.

## Installed by git clone

People install this server by cloning it, running `npm install` (which builds `dist/`), and
running `node dist/index.js`, not from npm or `npx`. `package.json` is `private`, so it cannot be published by accident. Docs give the clone
path first.

## What must not be introduced

* A second build step, or a committed `dist/`. `tsc` is the only build.
* An `any` that `strict` would otherwise reject, or a `// @ts-ignore`.
* A second list of tools, or a tool registered outside `src/server.ts`.
* A tool that takes an argument, a verb, a credential, or reaches a network.
* A source file other than `src/identity.ts` that spells the org, repo or server ID.
* A copy of the shared instruction set. Read it through its connector.
* A change to the `@modelcontextprotocol/sdk` calls (`registerTool`, `McpServer`, the
  transports) made for style alone.
