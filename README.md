# LXAgents-Skills/template

A read-only Skill MCP server: every markdown file in `content/` is served as its own tool.

- **Server ID:** `lxagents-template`
- **Tool prefix:** `lxagents_skills_template_`
- **Start here:** `lxagents_skills_template_automation` lists every tool and when to call it.

<!-- template-only -->
**This is the template for new Skill MCP repositories.** Create a repository from it on GitHub,
then run `cargo xtask init -- --org <org> --repo <repo>` — see
[Create a new repository](wiki/guides/create-a-new-repository.md).
The template itself stays at version `0.0.0`, so every new repository starts from there.
<!-- /template-only -->

## Quick start

Rust 1.88 or newer:

```bash
git clone https://github.com/LXAgents-Skills/template.git lxagents-template && cd lxagents-template
cargo test --workspace
cargo build --release
target/release/lxagents-template tools
target/release/lxagents-template
```

The last line serves over stdio, which needs no token.
`target/release/lxagents-template serve --http --host 127.0.0.1` serves HTTP on
`127.0.0.1:3000`, also without one.
HTTP on any other address needs `MCP_AUTH_TOKEN`.

## Tools

| Tool | Serves |
|---|---|
| `lxagents_skills_template_automation` | `content/automation.md` — the hub |
| `lxagents_skills_template_example_guide` | `content/example/example-guide.md` |

Each ID is `{org}_{repo}_{base}`, at most 64 characters. No tool takes an argument.

## Documentation

- [Install by git clone](wiki/guides/install-by-git-clone.md)
- [Add a tool](wiki/guides/add-a-tool.md)
- [Environment](wiki/environments/env.md) · [Docker](wiki/environments/docker.md)
- [Architecture](wiki/information/architecture.md)
- Agents: [`AGENTS.md`](AGENTS.md)

## Licence

MIT — see [`LICENSE`](LICENSE).
