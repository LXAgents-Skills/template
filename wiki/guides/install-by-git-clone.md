# Install by Git Clone

The server is installed from a clone and built with cargo. It is not installed from crates.io.

## Requirements

Rust 1.88 or newer (install it with [rustup](https://rustup.rs)), and git. `cargo build --release`
produces one binary, `target/release/lxagents-template`, with every file in `content/` compiled
into it. Nothing else is needed at run time.

## Clone, test and build

PowerShell:

```powershell
git clone https://github.com/LXAgents-Skills/template.git lxagents-template; cd lxagents-template; cargo test --workspace; cargo build --release
```

bash:

```bash
git clone https://github.com/LXAgents-Skills/template.git lxagents-template && cd lxagents-template && cargo test --workspace && cargo build --release
```

On Windows the binary is `target\release\lxagents-template.exe`.

## Register it with a client

### stdio (the default for one client)

```json
{
  "mcpServers": {
    "lxagents-template": {
      "command": "C:/path/to/lxagents-template/target/release/lxagents-template"
    }
  }
}
```

With no arguments the binary serves over stdio. It needs no `cwd`, because the set is inside it.

Claude Code, from the clone (bash):

```bash
claude mcp add lxagents-template -- "$PWD/target/release/lxagents-template"
```

PowerShell:

```powershell
claude mcp add lxagents-template -- "$PWD\target\release\lxagents-template.exe"
```

stdio needs no token and no environment variable.

### HTTP on this machine

```bash
target/release/lxagents-template serve --http --host 127.0.0.1
```

This binds `127.0.0.1:3000` and needs no token; only `localhost` `Host` headers are answered.

```json
{ "mcpServers": { "lxagents-template": { "type": "http", "url": "http://localhost:3000/mcp" } } }
```

### HTTP as a server

Any other bind needs `MCP_AUTH_TOKEN`, at least 32 characters, and every client sends it as
`Authorization: Bearer <token>`. See [Environment](../environments/env.md).

## After changing `content/` or pulling

The binary serves the set it was built with. Run `cargo build --release` again, then restart the
client's session.

## After registering

A client loads its servers when a session starts, so a server added mid-session stays absent
until the session restarts. Check it with `target/release/lxagents-template tools`, which prints
the same list the client should see.
