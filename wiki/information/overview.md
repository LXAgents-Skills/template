# Overview

`LXAgents-Skills/template` is a read-only Skill MCP server. Every markdown file in `content/` is
served as its own tool, named `lxagents_skills_template_{base}`, and none of them takes an
argument. A client reads `lxagents_skills_template_automation` first; it lists every other tool
and the condition that activates it.

<!-- template-only -->
It is also the starting point for every new Skill MCP repository in the organization:
create a repository from it on GitHub, run `npm run init`, and replace `content/`. See
[Create a new repository](../guides/create-a-new-repository.md).

The template's own version is always `0.0.0`. It is copied, never released, so each new
repository starts its version history from zero.
<!-- /template-only -->

| Surface | How |
|---|---|
| Skill MCP over stdio | `npm start` |
| Skill MCP over HTTP | `npm run start:http` (loopback, no token) or a container (token required) |
| Command line | `node dist/cli.js tools`, `--help`, `--version` |

The tools are derived from `content/` at boot. Adding a file adds a tool; there is no tool code
to write. Nothing writes, and nothing reaches a network.
