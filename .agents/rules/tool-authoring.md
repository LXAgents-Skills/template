---
name: tool-authoring
description: Adding, renaming, changing or removing a file in content/ - each is a change to the published tool surface - and how to write the description a client routes on.
---

# Tool Authoring

**Adding a markdown file to `content/` is adding a tool.** There is no tool module to write and
no list to edit: `src/instructions/from-content.ts` walks `content/` at boot and makes one tool
per `.md` file. A file that is not `.md` is not served.

## Adding a tool

1. Pick the folder and filename. The filename is the ID's base — check it against
   [`tool-ids.md`](tool-ids.md) before writing anything.
2. Write the frontmatter, each field on one line:

   ```markdown
   ---
   name: branching-strategy
   description: One task, one branch — naming as {type}/{primary-noun}, stacked in dependency order.
   ---
   ```

3. Write the body: one subject, complete on its own. A tool never points to another tool; if
   it needs a fact from elsewhere, it states that fact in its own sentence.
4. Add its row to `content/automation.md` in the same commit — see
   [`automation-hub.md`](automation-hub.md).
5. Run `npm test` and `npm run tools`.

## The description

The description is what a client reads in its tool list to decide whether to call the tool,
so it is the most-read sentence in the file.

* Say what the file covers and when it is needed, not what it is called.
* One line. The frontmatter parser reads single-line values only; a folded value is read as
  missing, and the server refuses to boot.
* No file may lack one: a tool without a description cannot be routed on.

## Changing, renaming, removing

| Change | What else changes in the same commit |
|---|---|
| Edit the body | Nothing. The next boot serves the new text. |
| Edit the description | The hub row if the condition changed. |
| Rename or move the file | The ID changes — the hub row, any `OVERRIDE_BASES` key, and the changelog, which names the old and new ID. |
| Remove the file | Its hub row, any `OVERRIDE_BASES` key, and the changelog. |

A renamed or removed ID breaks every caller that used it, so it needs the owner's approval
and is never folded quietly into another change.

## The surface stays read-only

No tool takes an argument, a verb, a credential or a network call, and no tool is registered
anywhere but from `content/`. Tools are registered with `registerTool` in `src/server.ts`, and
`listTools()` is the one list the CLI and the tests read. A hand-written tool beside the
generator would be a second source of truth; do not add one.
