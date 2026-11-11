---
name: tool-ids
description: How a tool ID is built - {org}_{repo}_{base}, lowercase with underscores only, at most 64 characters - and how to shorten the base when it is too long or collides.
---

# Tool IDs

Every file in `content/` is served as a tool, and its ID is derived, never chosen by hand.

```
{org}_{repo}_{base}
```

| Part | Comes from | Example |
|---|---|---|
| `{org}` | `ORG_NAME` in `src/identity.ts`, through `idPart()` | `LXAgents-Skills` → `lxagents_skills` |
| `{repo}` | `REPO_NAME` in `src/identity.ts`, through `idPart()` | `shared-instruction` → `shared_instruction` |
| `{base}` | The file's own name, or its `OVERRIDE_BASES` entry | `git/branching-strategy.md` → `branching_strategy` |

`{org}_{repo}` is `TOOL_PREFIX`. It is the same for every tool on the server, and it exists
so that two servers connected to one client never collide on a bare name like `automation`.

## Deriving the base

1. Take the filename and drop the folder: `references/python-flask.md` → `python-flask.md`.
2. Drop `.md`.
3. Lowercase.
4. Turn every `-` into `_`.

`idPart()` does the same for the org and repo, and also turns any other run of characters
that is not a letter or digit into one `_`.

## The four rules

1. **Lowercase letters, digits and underscores only, starting with a letter.** Nothing else
   survives every client.
2. **At most 64 characters, prefix included.** The protocol allows 128, but several clients
   cut a name at 64. The server refuses to boot with a longer ID, and names the file.
3. **Shorten only the base, never the prefix.** The prefix is what makes an ID unique across
   servers; a shortened prefix is a collision waiting for the next repository. When an ID is
   too long, rename the file, or give it an `OVERRIDE_BASES` entry.
4. **One file, one ID.** The folder is dropped, so `github/api.md` and `gitlab/api.md` would both
   derive `api`. The server refuses to boot on a collision and names both files; give one of
   them (or both) a base that says which it is: `github_api`, `gitlab_api`.

## Budget

The base gets whatever the prefix leaves: `64 - len(TOOL_PREFIX) - 1`.

| Prefix | Length | Room for the base |
|---|---|---|
| `lxagents_skills_template` | 24 | 39 |
| `lxagents_skills_security` | 24 | 39 |
| `lxagents_skills_shared_instruction` | 34 | 29 |

Check before you name a file: `npm run tools` prints every ID and its length.

## Overrides

`OVERRIDE_BASES` in `src/instructions/naming.ts` maps a set-relative path to a base name.
The prefix is added by `toolId()`, so an entry is a base only:

```js
export const OVERRIDE_BASES = Object.freeze({
  "references/javascript-typescript-react-web-frontend-security.md": "js_ts_react_web_frontend_security",
  "skills/github/api.md": "github_api",
});
```

* Prefer renaming the file. An override is for when the filename has to stay — it is
  someone else's name, or a document people already link to.
* Shorten by a consistent abbreviation and say which: `javascript` → `js`,
  `javascript_typescript` → `js_ts`. Do not truncate mid-word.
* A key that matches no file stops the server at boot, so an override cannot outlive the file.
* **Renaming a published ID breaks every caller that used it.** It is a naming decision for
  the owner, and it goes in the changelog with the old and new names.

## Where the rule is enforced

| Check | Where |
|---|---|
| Valid characters, 64-character cap, collisions, stale overrides | At boot, `src/instructions/from-content.ts` |
| Every ID starts with `TOOL_PREFIX_` and matches the derivation | `test/server.test.ts` |
| The derivation itself, overrides, and each refusal | `test/naming.test.ts` |
