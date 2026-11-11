---
name: agents-index
description: Index of this repository's own rules - repository rules, tool IDs, tool authoring, the automation hub, and HTTP auth.
---

# Agents Index

**Scope:** `.agents/rules/` · **Parent:** [`root-index.md`](root-index.md)

| File | Purpose |
|---|---|
| [`../rules/repository.md`](../rules/repository.md) | Where things go, the commands, the stdout ban, and what must not be introduced. |
| [`../rules/tool-ids.md`](../rules/tool-ids.md) | How a tool ID is built — `{org}_{repo}_{base}`, 64 characters — and when to shorten the base. |
| [`../rules/tool-authoring.md`](../rules/tool-authoring.md) | Adding, renaming, changing or removing a served file, and writing its description. |
| [`../rules/automation-hub.md`](../rules/automation-hub.md) | The shape of `content/automation.md` and how to write a trigger condition. |
| [`../rules/http-auth.md`](../rules/http-auth.md) | When `MCP_AUTH_TOKEN` is required, the localhost `Host` rule, and secrets. |

Any file added to or removed from `.agents/rules/` is reflected here in the same commit.
