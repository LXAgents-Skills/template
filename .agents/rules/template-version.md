---
name: template-version
description: The template's version is 0.0.0 for good - it is copied, never released - so nothing bumps it, and its release branch is always release/0.0.0.
---

# Template Version

Template only: `npm run init` deletes this file. A repository made from the template versions
itself normally, starting from `0.0.0`.

## The version is `0.0.0`, always

The template is never installed as a product. It is copied with **Use this template**, and
every copy starts its own history from `0.0.0`. A number above that on the template would be
inherited by nobody and would claim a release nothing can consume.

| Carrier | Value |
|---|---|
| `package.json` `version` | `0.0.0` |
| `package-lock.json` `version` and `packages[""].version` | `0.0.0` |
| What the server reports (`--version`, `/healthz`, the MCP handshake) | `0.0.0`, read from `package.json` |
| Git tags and GitHub releases | None |

`test/template-version.test.ts` fails if any of these leaves `0.0.0`. Never change it to make a
bump pass, and never propose a bump for the template, whatever the change.

## The release task

Every chain still ends on a release branch, and it is always `release/0.0.0`. It carries only
release notes:

* a new dated section at the top of `wiki/logs/0/0/0/CHANGELOG.md`, headed
  `## YYYY-MM-DD — {what changed}`, newest first;
* closing the chain's task record.

It never touches `package.json`, the lockfile, or a tag, and it never creates another
`wiki/logs/` directory. `wiki/logs/0/0/0/` is the one log directory, and unlike a released
version's log it grows a section per chain.
