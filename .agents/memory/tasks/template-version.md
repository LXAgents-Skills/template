---
name: memory-tasks-template-version
description: Pinning LXAgents-Skills/template at version 0.0.0 forever, so every repository made from it starts its own versioning from zero.
---

# Task: pin the template at 0.0.0

**Status: done.**

The template was released as `0.1.0` (`wiki/logs/0/1/0/`). The owner wants it to stay at
`0.0.0` for good: it is never installed as a product, only copied, and every repository made
from it starts its own version history from `0.0.0`.

## What changes

- `package.json` and `package-lock.json` go to `0.0.0`. The server reads its version from
  `package.json`, so the CLI and `/healthz` report it too.
- A template-only rule, `.agents/rules/template-version.md`, and a template-only test,
  `test/template-version.test.js`, which fails if either file leaves `0.0.0`.
- `npm run init` deletes that rule and that test, as it already deletes the
  create-a-new-repository guide. It already resets a new repository to `0.0.0`.
- The release notes become one rolling changelog, `wiki/logs/0/0/0/CHANGELOG.md`, with dated
  sections, newest first. The `0.1.0` notes move into it and `wiki/logs/0/1/0/` is removed.

## What does not change

- The server, its tools, the ID rule, HTTP auth, and everything `init` does for a new
  repository other than removing the two new files.
- `LXAgents-Skills/shared-instruction` and `LXAgents-Skills/security`.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | This record | `plan/template-version` | #6 |
| 2 | Version files, the version test, `init` removes it | `feat/version-pin` | #7 |
| 3 | The rule, `AGENTS.md`, indexes, the guide | `docs/version-pin` | #8 |
| 4 | Rolling changelog, logs index, close this record | `release/0.0.0` | #9 |

## Rules set aside, by the owner's instruction

- **Never delete a version directory** (shared `versioning`). `wiki/logs/0/1/0/` is folded into
  `wiki/logs/0/0/0/`. No tag and no published package exists for `0.1.0`.
- **`release/{version}` names the version it ships** (shared `plan_creator`, project
  `AGENTS.md`). The template's release branch is always `release/0.0.0`; it adds a dated
  changelog entry and never changes `package.json`.

## Decisions taken without the owner

- The version change rides in task 2 with the test that guards it, so every branch is green.
  The release task carries release notes only.

## Progress

### Task 1 — plan/template-version

Landed: this record and its row in `memory-index.md`.

### Task 2 — feat/version-pin

Landed. `package.json`, `package-lock.json` (top level and `packages[""]`) at `0.0.0`.
`test/template-version.test.js` checks all three and the version the server reports; it was
seen to fail with `package.json` at `0.1.0`. `scripts/init.js` lists it and
`.agents/rules/template-version.md` (written in task 3) in `TEMPLATE_ONLY`, and the init
end-to-end test asserts both are gone and the copy is at `0.0.0`. 39 tests pass.

### Task 3 — docs/version-pin

Landed. `.agents/rules/template-version.md` (what carries the version, the test that guards
it, and what the `release/0.0.0` task contains), with a template-only row in
`agents-index.md`. Template-only sentences in `AGENTS.md`, `README.md` and
`wiki/information/overview.md`; the repository map lists the version test; the
create-a-new-repository guide says `init` removes the rule and test and no longer suggests
`release/0.1.0`. The init end-to-end test now also checks `agents-index.md` and that no page
still names `template-version` after `init`. 39 tests pass.

### Task 4 — release/0.0.0

Landed. `wiki/logs/0/0/0/CHANGELOG.md` is the template's one rolling changelog: a dated
section for this change, then the former `0.1.0` notes. `wiki/logs/0/1/0/` removed; the
`logs-index.md` row points at `0/0/0`. `package.json` untouched. PR column filled. Record done.
