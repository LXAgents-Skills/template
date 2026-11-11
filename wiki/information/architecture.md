# Architecture

## From a request to a file

The source is TypeScript in `src/`; `tsc` compiles it to `dist/`, which is what runs. The steps
below name the source files.

1. `src/index.ts` reads the environment once (`src/config.ts`) and calls `run()`.
2. `src/run.ts` picks the transport. stdio starts `src/stdio.ts`. HTTP starts
   `src/http/primary.ts`, which refuses to start without a required token and forks workers;
   each worker (`src/http/worker.ts`) listens with the app from `src/http/app.ts`.
3. Each connection, or each HTTP request, gets a fresh server from `createServer()` in
   `src/server.ts`, which calls `registerTool` once per entry in `CONTENT_INSTRUCTIONS`.
4. `CONTENT_INSTRUCTIONS` is built once, at import, by `src/instructions/from-content.ts`: one
   tool per `.md` file in `content/`, its ID from `src/instructions/naming.ts`, its description
   from the file's frontmatter, and a handler that returns the file byte for byte.

## Why the set is checked at boot

A file without a description, an ID over 64 characters, two files with one ID, or an override
naming a missing file all stop the process with a message naming the file and the fix. A set
that cannot be routed on should not start.

## One name, one place

`src/identity.ts` holds the org, repo, server ID, title and summary. The tool prefix, every ID,
the health check and every log line come from it, which is what lets `npm run init` rename a
copy by rewriting that one file and the documentation.

## HTTP

`POST /mcp` is stateless: a fresh `McpServer` and transport per request. `GET /healthz` is the
only route without a token. The `Host` allow-list, then the token check, run before the body
parser and the routes. See [Environment](../environments/env.md).
