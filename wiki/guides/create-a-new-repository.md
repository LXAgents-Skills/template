# Create a New Repository from the Template

How to start a new Skill MCP repository without writing it from scratch. The result serves its
own `content/` as tools named `{org}_{repo}_{base}`, with the automation hub, the tests, HTTP
auth and the Docker image already in place.

## Once: mark this repository as a template

In the template repository on GitHub: **Settings → General → Template repository**, tick the
box. Without it, step 1 below has no **Use this template** button.

## 1. Create the repository

On the template's GitHub page, **Use this template → Create a new repository**. Pick the
organization and the name. The name becomes the `{repo}` part of every tool ID, so check its
length now: `{org}_{repo}` plus at least 20 characters of base has to fit in 64.

## 2. Clone it and test it

PowerShell:

```powershell
git clone https://github.com/<org>/<repo>.git; cd <repo>; cargo test --workspace
```

bash:

```bash
git clone https://github.com/<org>/<repo>.git && cd <repo> && cargo test --workspace
```

## 3. Rename it

```bash
cargo xtask init -- --org <org> --repo <repo> --dry-run
cargo xtask init -- --org <org> --repo <repo>
```

The dry run prints what will change. The real run:

* rewrites `src/identity.rs` (`ORG_NAME`, `REPO_NAME`, `SERVER_ID`, `SERVER_TITLE`,
  `SERVER_SUMMARY`) and the description and keywords in `Cargo.toml`, with the version reset
  to `0.0.0`;
* replaces the old tool prefix, `org/repo` and server ID in every markdown file, both
  `Cargo.toml` files, `tests/*.rs` and the Dockerfile, so the package and the binary take the
  new server ID;
* removes this guide, the template's version rule and version test, and every
  `<!-- template-only -->` block;
* removes the template's own history: `.agents/memory/` tasks, decisions and state, and
  `wiki/logs/`, and resets their indexes;
* builds the renamed server (which also updates `Cargo.lock`) and prints every new ID with its
  length, so an ID pushed past 64 characters by the new prefix fails here, by name.

Optional flags: `--server-id`, `--title`, `--summary`, `--keep-history`. By default the server
ID is the org's first word and the repo (`LXAgents-Skills` + `security` → `lxagents-security`).

`init` refuses to run in a checkout whose `origin` is the template itself, so a mistaken run
cannot wipe the template's history.

## 4. Make it yours

1. Replace `content/example/` with your own files — see [Add a tool](add-a-tool.md).
2. Rewrite `content/automation.md` so it lists every one of them.
3. Rewrite `README.md` and `wiki/information/overview.md` for what the set is.
4. Run `cargo test --workspace`. Every test is written against whatever `content/` holds, so they keep
   working; the hub test tells you which rows are missing.

## 5. Commit it on the usual branches

The rename is the new repository's first plan: `plan/{slug}` for the record, `feat/…` for the
rename and content, and `release/{version}` for the first release. The template is at `0.0.0`
and never moves, so the new repository's first release is the first version it has; the owner
picks it.
