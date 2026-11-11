---
name: memory-tasks-template-repo
description: Building LXAgents-Skills/template, the starting point every new Skill MCP repository is copied from, modeled on shared-instruction and security.
---

# Task: the template repository

The repository started as a licence and a one-line README. The goal is that a new Skill MCP
repository is never written from scratch: it is created from this template, renamed with one
command, and filled with its own `content/`.

References: `LXAgents-Skills/security` for the source layout (config, run, stdio, `http/`,
CLI) and `LXAgents-Skills/shared-instruction` for `registerTool` and the `toolId` helper.
Neither reference repository is changed by this task.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | This record and the ignore rules | `plan/template-repo` | |
| 2 | The Skill MCP server, `content/automation.md` hub, an example tool, tests, Docker | `feat/server` | |
| 3 | `npm run init` renames a fresh copy: org, repo, prefix, package, hub | `feat/init-script` | |
| 4 | `AGENTS.md`, `.agents/` rules and indexes, `wiki/` guides, README | `docs/authoring-guide` | |
| 5 | Version and changelog | `release/0.1.0` | |

## Conventions carried into the template

- Tool IDs are `{org}_{repo}_{base}`, underscores only, at most 64 characters. Only the base is
  ever shortened, through `NAME_OVERRIDES`; the prefix is built from `ORG_NAME` and
  `REPO_NAME` in one file.
- `content/automation.md` lists every served file with its trigger; a test fails on an orphan.
- `MCP_AUTH_TOKEN` is required only for HTTP bound off loopback. Loopback without a token gets
  the localhost `Host` allow-list.
- Tools are registered with `registerTool`; `listTools()` is the one list the CLI and tests read.
- Install is by `git clone`, not npm or npx.

## Decisions taken without the owner

- First version `0.1.0`: there is no earlier version to patch-bump, and it matches security's
  first release.

## Progress

### Task 1 — plan/template-repo

Landed: this record, its row in `memory-index.md`, `.gitignore` (which keeps `.agents/plans/`
out of history) and `.gitattributes`.

### Task 2 — feat/server

Landed. The source layout follows security: `config.js`, `run.js`, `stdio.js`, `http/`
(app, auth, primary, worker, rpc-error) and `cli.js`. What is new for the template:

- `src/identity.js` is the only file that names the server: `ORG_NAME`, `REPO_NAME`,
  `SERVER_ID`, `SERVER_TITLE`, `SERVER_SUMMARY`. `TOOL_PREFIX` and `toolId()` are derived.
- `src/instructions/naming.js` owns the ID rule: `derivedBase`, `OVERRIDE_BASES` (base names
  only), and the 64-character cap, now enforced at boot rather than only in a test.
- `src/paths.js` and `src/version.js` split what security keeps in one file.
- Tools are registered with `registerTool`; `listTools()` feeds the CLI.
- `content/automation.md` and one example tool, `content/example/example-guide.md`.

The tests are written against whatever `content/` holds, so a repository made from the
template keeps them: the bijection, the ID rule, no arguments, byte-for-byte content, the hub
listing every tool once, the naming errors on throwaway sets, the config, and HTTP auth
(token off loopback, localhost `Host` on loopback, the real entry point refusing `0.0.0.0`
without a token). 31 tests pass.

### Task 3 — feat/init-script

Landed. `npm run init -- --org <org> --repo <repo>` (`scripts/init.js`) rewrites
`src/identity.js`, `package.json` and `package-lock.json` (version reset to `0.0.0`), swaps the
old tool prefix, `org/repo` slug and server ID in every markdown file and the Dockerfile,
clears the template's history (`.agents/memory/{tasks,decisions,state}`, `wiki/logs`) with
fresh indexes, and boots the tool list so an ID pushed past 64 characters fails by name.
`--dry-run`, `--keep-history`, `--server-id`, `--title`, `--summary`. It refuses to run when
`origin` is the template itself unless `--force` is passed. The pure parts live in
`scripts/init/`. A test copies the repository, runs init, and runs the copy's own surface
tests. 37 tests pass.
