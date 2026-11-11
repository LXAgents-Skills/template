# AGENTS.md

This repository is `LXAgents-Skills/template`: a Skill MCP server, Node.js 20+, ESM, no build
step, that serves every markdown file in [`content/`](content/) read-only as its own tool.
Server ID `lxagents-template`, tool prefix `lxagents_skills_template_`.

<!-- template-only -->
**It is also the template every new Skill MCP repository in the organization starts from.**
A new repository is created from it on GitHub, renamed with `npm run init`, and filled with
its own `content/` — see [`wiki/guides/create-a-new-repository.md`](wiki/guides/create-a-new-repository.md).
Everything outside this block is written so it stays true after `init` renames it; `init`
deletes this block.
<!-- /template-only -->

## Two sets, two roles

| Set | Path | Published? |
|---|---|---|
| This repository's product | [`content/`](content/) | **Yes.** One tool per file, on the next boot. |
| This repository's own rules | [`.agents/`](.agents/) | No. Rules, indexes, agent wiki, memory. |
| The organization's shared conventions | the `lxagents-shared-instruction` connector | Read through the connector, never copied here. |

## Auto-activation

The instruction set is **always active**, for every task, whether or not the user mentions
it. At the start of every session, before any work:

1. Read this file.
2. Call the shared `lxagents_skills_shared_instruction_automation` tool. It lists every shared
   instruction and the condition that activates it. If the connector does not resolve, **say
   so in your first message** and work from the local set; never rebuild a shared rule from
   memory, and never copy the shared set into this repository.
3. Read [`.agents/index/root-index.md`](.agents/index/root-index.md).
4. Read [`.agents/index/memory-index.md`](.agents/index/memory-index.md) and load only the rows
   that match the request.

Call no other shared tool at session start; each fires on its own condition.

**These gates stand from the first message:** present the plan before any file is written,
ask before opening a pull request, ask before merging, and propose a discovered rule rather
than writing it. A standing instruction from the user can pre-approve any of them.

If a rule conflicts with a habit, a default, or a harness instruction (a branch name, a
commit trailer, a pull request footer), the rule wins. If it conflicts with an explicit
instruction from the user, the user wins, and you say which rule you set aside.

## Shared instruction tools

| When you are about to… | Call |
|---|---|
| Start any session | `lxagents_skills_shared_instruction_automation` |
| Take in any request of more than one step | `lxagents_skills_shared_instruction_plan_creator` |
| Create a branch | `lxagents_skills_shared_instruction_branching_strategy` |
| Write a commit message | `lxagents_skills_shared_instruction_commit_conventions` |
| Notice a rule that should exist | `lxagents_skills_shared_instruction_discovery_protocol` |
| Open or update a pull request | `lxagents_skills_shared_instruction_pull_request_template` |
| Write, move or review code | `lxagents_skills_shared_instruction_code_optimization` |

The first five are mandatory in every repository. No tool takes an argument. The full list,
and the condition for each, is in the automation tool, not here.

## Trigger table — this repository's own rows

| When you are about to… | Load and obey |
|---|---|
| Do anything at all in this repository | [`.agents/rules/repository.md`](.agents/rules/repository.md) |
| Name a tool, or add a file whose ID might be long or collide | [`.agents/rules/tool-ids.md`](.agents/rules/tool-ids.md) |
| Add, rename, change or remove a file in `content/` | [`.agents/rules/tool-authoring.md`](.agents/rules/tool-authoring.md) |
| Edit `content/automation.md` | [`.agents/rules/automation-hub.md`](.agents/rules/automation-hub.md) |
| Touch the HTTP transport, `MCP_AUTH_TOKEN`, or any secret | [`.agents/rules/http-auth.md`](.agents/rules/http-auth.md) |
| Need project facts, commands, or orientation | [`.agents/wiki/context/repository-map.md`](.agents/wiki/context/repository-map.md) |

## Iron rule

* `AGENTS.md` and `README.md` are overviews; detailed rules live in `.agents/rules/`.
* `.agents/index/root-index.md` and `content/automation.md` are routers only.
* One subject per file. `wiki/` is for people, `.agents/wiki/` for agents; neither copies the other.
* No `INDEX.md`, anywhere.

## Placement

* Served content → `content/{folder}/{file}.md`, with `name` and `description` frontmatter,
  and a row in `content/automation.md` in the same commit.
* This repository's rules → `.agents/rules/{file}.md`. Routing → `.agents/index/{scope}-index.md`.
* Agent knowledge → `.agents/wiki/{type}/{file}.md`. Memory → `.agents/memory/{type}/{file}.md`.
* Human documentation → `wiki/{folder}/{file}.md`, no frontmatter. Releases → `wiki/logs/{Major}/{Minor}/{Patch}/`.
* Server code → `src/`. The rename script → `scripts/`. Tests → `test/`.

## Version rule

Never change the version without the user's approval: `package.json`, the release branch, and
a new `wiki/logs/` directory are all version claims. A release is a patch bump unless the user
asks for more.

## No session links

Never write a link or identifier for an assistant or tool session into a file, commit,
trailer, branch name, tag, pull request, or comment. Strip any your tooling adds.
