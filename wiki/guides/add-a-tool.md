# Add a Tool

Every markdown file in `content/` is a tool. Adding one takes a file and a row in the hub.

## 1. Name the file

The filename becomes the tool ID: `{org}_{repo}_{base}`, where the base is the filename
without its folder or `.md`, lowercased, with `-` turned into `_`.

| File | ID in `LXAgents-Skills/template` |
|---|---|
| `content/example/example-guide.md` | `lxagents_skills_template_example_guide` |
| `content/git/commit-conventions.md` | `lxagents_skills_template_commit_conventions` |

An ID is at most 64 characters. If yours is longer, or another folder already has a file with
the same name, pick a shorter or more specific filename, or give the file a base name in
`OVERRIDE_BASES` (`src/instructions/naming.js`). Only the base is ever shortened. The full rule:
[`.agents/rules/tool-ids.md`](../../.agents/rules/tool-ids.md).

## 2. Write it

```markdown
---
name: commit-conventions
description: Conventional Commits for commit messages — format, scope, body, and what rides in the same commit.
---

# Commit Conventions

…
```

`description` is what a client reads to decide whether to call the tool. Keep it on one line.

## 3. List it in the hub

Add one line to `content/automation.md`, under the group where it fires:

```markdown
- `lxagents_skills_template_commit_conventions` — before you write a commit message.
```

The part after ` — ` is the condition: the moment the tool is needed, not its title.

## 4. Check it

```powershell
npm run tools; npm test
```

`npm run tools` prints every ID with its length. `npm test` fails if the file has no
description, if its ID is too long or collides, or if the hub is missing it.
