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
