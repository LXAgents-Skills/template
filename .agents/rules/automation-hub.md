---
name: automation-hub
description: The shape of content/automation.md - the one tool a client reads first - and how to write the trigger condition for each row.
---

# The Automation Hub

`content/automation.md` is served as `{prefix}_automation`. A client reads it once, at the start
of a session, and then calls only the tools whose condition is true. It is a router: it lists
tools and conditions, and teaches nothing else.

## Shape

```markdown
---
name: Read this tool every session
description: Read this tool every session, first — it lists every tool and the one condition that activates each, so you load only what the request needs.
---

# Automation

{the short rules for using it}

## {Group}

- `{full tool ID}` — {the condition that activates it}.
```

* **Every other tool has exactly one row**, and every row names a tool that exists.
  `test/server.test.js` fails otherwise, naming the missing or stale IDs.
* A row starts with `- `, then the full ID in backticks, then ` — ` (space, em dash, space),
  then the condition. The test reads the first ID on each such line.
* Group rows under `##` headings by when they fire, in the order work reaches them.
* Keep the frontmatter `name` and `description` as they are: every LXAgents server's hub uses
  the same two lines, so a client recognises it.

## Writing a condition

A condition is the moment the tool becomes needed, written so an agent can test it against the
request in front of it.

| Good | Why |
|---|---|
| `— before you write a commit message.` | A moment the agent can recognise. |
| `— the code in scope is a Flask application.` | A fact about the request. |
| `— you are trying out this template.` | Specific, even if narrow. |

| Bad | Why |
|---|---|
| `— commit conventions.` | A title, not a condition. |
| `— useful for git work.` | Never false, so it is always called. |
| `— read this first.` | Only the hub itself is read first. |

When two tools must be read in order, say so in the later one's condition ("read it after the
general guide"), not in a separate list.
