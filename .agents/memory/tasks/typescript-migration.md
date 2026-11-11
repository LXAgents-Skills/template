---
name: memory-tasks-typescript-migration
description: Migrating LXAgents-Skills/template from JavaScript to TypeScript - latest dependencies, strict tsconfig, dist/ built on install, version still 0.0.0.
---

# Task: migrate the template to TypeScript

**Status: in progress.**

The owner asked every Node.js repository in the organization to move to TypeScript with the
same checklist: latest dependencies and a clean `npm audit`; a strict NodeNext `tsconfig.json`
building `src/` into `dist/`; every `.js` in `src/` and `test/` renamed to `.ts` and typed,
with the logic unchanged; `main`, `bin` and `start` pointing at `dist/`; a `build` script; tests
run through `tsx`. This record covers the template only. Other repositories are migrated
separately.

## What changes

- `package.json`: dependencies at their latest; `typescript`, `@types/node`, `tsx` and
  `@types/express` as dev dependencies; `main`, `bin`, `start`, `start:http`, `cli`, `tools`,
  `inspect` on `dist/`; `build: tsc`; `prepare: tsc`; `test: node --import tsx --test test/*.test.ts`;
  `init: tsx scripts/init.ts`.
- `tsconfig.json` (build: `src/` to `dist/`, `strict`) and `tsconfig.check.json` (type-checks
  `src/`, `test/` and `scripts/` without emitting).
- `src/`, `test/` and `scripts/` become `.ts`. Imports keep their `.js` extensions, as NodeNext
  requires.
- `dist/` is ignored by git and built by `prepare` on `npm install`, so clone + install still
  yields a runnable server.
- `npm run init` rewrites `src/identity.ts`, rebuilds `dist/`, then boots the tool list from it.
- The Dockerfile builds in one stage and runs `dist/` in a second, without dev dependencies.

## What does not change

- Version `0.0.0`, guarded by `test/template-version.test.ts`. The release branch is
  `release/0.0.0` with release notes only.
- Tool IDs, the ID rule, the tool surface, the SDK calls, HTTP auth, the stdout ban.
- Other repositories.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | This record | `plan/typescript-migration` | |
| 2 | Latest dependencies, clean audit | `build/dependencies` | |
| 3 | tsconfig, conversion, scripts, init, Docker | `refactor/typescript` | |
| 4 | Rules, README, wiki, agent wiki | `docs/typescript` | |
| 5 | Changelog, close this record | `release/0.0.0` | |

## Rules set aside, by the owner's instruction

- **No build step** (`.agents/rules/repository.md`). The owner asked for `tsc` to `dist/`.
  The rule is rewritten in task 4 to describe the build instead.

## Decisions taken without the owner

- `dist/` is not committed. `prepare` builds it on `npm install`, which the clone install
  already runs; a committed `dist/` would drift from `src/`.
- `scripts/` is converted too and run through `tsx`, because `init` imports `src/identity`,
  which is now TypeScript. It is type-checked by `tsconfig.check.json`, not built.
- Tests spawn the server from source through `tsx`, so `npm test` does not need a prior build.

## Progress

### Task 1 — plan/typescript-migration

Landed: this record and its row in `memory-index.md`.

### Task 2 — build/dependencies

Landed. `@modelcontextprotocol/sdk` 1.32.2, `express` 5.3.0, `zod` 4.6.7. Dev dependencies
added: `typescript` 7.0.2, `@types/node` 26.6.6, `tsx` 4.23.15, `@types/express` 5.0.6. Nothing
uses them yet; task 3 does. `npm audit`: 0 vulnerabilities. Version still `0.0.0` in
`package.json` and the lockfile. 39 tests pass, still JavaScript.

### Task 3 — refactor/typescript

Landed. Every `.js` in `src/`, `test/` and `scripts/` is now `.ts` (`git mv`, so history
follows). Types added: `Config` (`src/config.ts`), `ServerOptions` and `ToolSummary`
(`src/server.ts`), `ContentInstruction` and `ContentInstructions` (handlers typed as returning
the SDK's `CallToolResult`), `AppOptions` (`src/http/app.ts`), `Flags`, `IdentityFlags` and
`Identity` (`scripts/init/`). Express handlers use express's own types. No logic changed, and
neither did any SDK call or tool ID. `TOOL_PREFIX` keeps no annotation, because the init test
reads `src/identity.ts` as text.

- `tsconfig.json` builds `src/` to `dist/` (ES2022, NodeNext, strict, `types: ["node"]`, source
  maps). `tsconfig.check.json` type-checks `src/`, `test/` and `scripts/` without emitting;
  `npm run typecheck` runs it.
- `package.json`: `main` and `bin` on `dist/`, `files` ships `dist/` instead of `src/`;
  `build` and `prepare` are `tsc`; `start`, `start:http`, `cli`, `tools`, `inspect` run `dist/`;
  `test` is `node --import tsx --test test/*.test.ts`; `init` is `tsx scripts/init.ts`.
- `dist/` is git-ignored and docker-ignored.
- `init` rewrites `src/identity.ts`, writes `dist/` bins into `package.json` and the lockfile,
  rebuilds `dist/` with the local `tsc`, then boots `dist/cli.js tools`. `TEMPLATE_ONLY` names
  `test/template-version.test.ts`.
- The init end-to-end test now runs `init` through `tsx`, checks that the rebuilt `dist/`
  lists tools under the new prefix, and asserts *any* new-prefix ID in the hub instead of
  `example_guide`. That fixes one of the two gaps the agentic bootstrap reported; the other,
  the repository map row, is task 4.
- Dockerfile: a build stage runs `npm ci --ignore-scripts` and `npm run build`, and the runtime
  stage installs with `--omit=dev` and copies `dist/` in. Entry point `node dist/index.js`.
- The HTTP tests spawn `src/index.ts` through `tsx`, so `npm test` needs no prior build.

Checks: `npm run build` and `npm run typecheck` clean; 38 tests pass (the old count of 39
included `test/helpers.js`, which `node --test` ran as a test file); stdio initialize,
`tools/list` and `tools/call` answered from `dist/index.js`; HTTP with two workers on
127.0.0.1 answered `/healthz` and `tools/list`; on 0.0.0.0 without a token it exits 1.
`npm audit`: 0 vulnerabilities.

### Task 4 — docs/typescript

Landed. Every source path in the docs and rules now names its `.ts` file; every command that
runs the server names `dist/`. `.agents/rules/repository.md` gains a "TypeScript, compiled to
`dist/`" section and the new commands, and its "no build step" ban becomes "no second build
step and no committed `dist/`", plus no unchecked `any` or `@ts-ignore`. The install guide,
the Docker page, the architecture page, the create-a-new-repository guide (init rebuilds
`dist/`), `AGENTS.md` and the repository map are updated.

The second gap from the agentic bootstrap is fixed: the repository map's row for
`test/template-version.test.ts` was a table row `init` left behind; it is now a
`<!-- template-only -->` note that `init` strips, and the init end-to-end test checks the
repository map for template-only text too. 38 tests pass; typecheck clean.
