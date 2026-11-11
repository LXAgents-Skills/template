---
name: root-index
description: Router for this repository - every local index, and where the shared set is read from. Lists indexes only, never leaf content.
---

# Root Index

This file lists **indexes only**. Read exactly one branch per task, plus
[`memory-index.md`](memory-index.md).

| Index | Scope | Load when |
|---|---|---|
| [`agents-index.md`](agents-index.md) | `.agents/rules/` | You need a rule specific to this repository. |
| `lxagents_skills_shared_instruction_automation` | The shared set, through the connector | You need a branching, commit, pull request, planning or writing convention. |
| [`agent-wiki-index.md`](agent-wiki-index.md) | `.agents/wiki/` | You need orientation written for agents. |
| [`project-wiki-index.md`](project-wiki-index.md) | `wiki/` | You need documentation a person will read. |
| [`memory-index.md`](memory-index.md) | `.agents/memory/` | You need prior task state or must record progress. |
| [`logs-index.md`](logs-index.md) | `wiki/logs/` | You need release history or must record a release. |

Adding, removing or renaming an index updates this table in the same commit.
