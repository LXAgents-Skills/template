---
name: project-wiki-index
description: Index of wiki/ - the human documentation tree, excluding wiki/logs/ which has its own index.
---

# Project Wiki Index

**Scope:** `wiki/`, except `wiki/logs/` · **Parent:** [`root-index.md`](root-index.md)

| File | Purpose |
|---|---|
| [`../../wiki/information/overview.md`](../../wiki/information/overview.md) | What this repository is and what it serves. |
| [`../../wiki/information/architecture.md`](../../wiki/information/architecture.md) | How a request reaches a file: entry point, transports, the derived surface. |
| [`../../wiki/guides/add-a-tool.md`](../../wiki/guides/add-a-tool.md) | Add a served file, give it an ID, list it in the hub, and test it. |
| [`../../wiki/guides/install-by-git-clone.md`](../../wiki/guides/install-by-git-clone.md) | Install the server from a clone and register it with a client. |
| [`../../wiki/environments/env.md`](../../wiki/environments/env.md) | Every environment variable the server reads. |
| [`../../wiki/environments/docker.md`](../../wiki/environments/docker.md) | Building and running the container image. |

<!-- template-only -->
Template only — `npm run init` deletes this page and this table:

| File | Purpose |
|---|---|
| [`../../wiki/guides/create-a-new-repository.md`](../../wiki/guides/create-a-new-repository.md) | Start a new Skill MCP repository from this template. |

<!-- /template-only -->
| Child index | Scope |
|---|---|
| [`logs-index.md`](logs-index.md) | `wiki/logs/` |

Any page added to or removed from `wiki/` is reflected here in the same commit. Pages in this
tree are plain markdown with no frontmatter.
