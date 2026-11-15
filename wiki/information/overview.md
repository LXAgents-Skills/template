# Overview

`LXAgents-Skills/template` is a read-only Skill MCP server. Every markdown file in `content/` is
served as its own tool, named `lxagents_skills_template_{base}`, and none of them takes an
argument. A client reads `lxagents_skills_template_automation` first; it lists every other tool
and the condition that activates it.

<!-- template-only -->
It is also the starting point for every new Skill MCP repository in the organization:
create a repository from it on GitHub, run `cargo xtask init`, and replace `content/`. See
[Create a new repository](../guides/create-a-new-repository.md).

The template's own version is always `0.0.0`. It is copied, never released, so each new
repository starts its version history from zero.
<!-- /template-only -->

| Surface | How |
|---|---|
| Skill MCP over stdio | `target/release/lxagents-template`, no arguments |
| Skill MCP over HTTP | `lxagents-template serve --http --host 127.0.0.1` (loopback, no token) or a container (token required) |
| Command line | `lxagents-template tools`, `--help`, `--version` |

The tools are derived from `content/`, which is compiled into the binary and checked at boot.
Adding a file and rebuilding adds a tool; there is no tool code to write. It is written in Rust
on the official MCP SDK (`rmcp`), with axum for HTTP. Nothing writes, and nothing reaches a network.
