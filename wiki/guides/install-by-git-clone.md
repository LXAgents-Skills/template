# Install by Git Clone

The server is installed from a clone and run with Node. It is not installed from npm.

## Requirements

Node.js 20 or newer, and git. There is no build step.

## Clone and test

PowerShell:

```powershell
git clone https://github.com/LXAgents-Skills/template.git lxagents-template; cd lxagents-template; npm install; npm test
```

bash:

```bash
git clone https://github.com/LXAgents-Skills/template.git lxagents-template && cd lxagents-template && npm install && npm test
```

## Register it with a client

### stdio (the default for one client)

```json
{
  "mcpServers": {
    "lxagents-template": {
      "command": "node",
      "args": ["src/index.js"],
      "cwd": "C:/path/to/lxagents-template"
    }
  }
}
```

Claude Code, from the clone (PowerShell):

```powershell
claude mcp add lxagents-template -- node "$PWD\src\index.js"
```

stdio needs no token and no environment variable.

### HTTP on this machine

```powershell
npm run start:http
```

This binds `127.0.0.1:3000` and needs no token; only `localhost` `Host` headers are answered.

```json
{ "mcpServers": { "lxagents-template": { "type": "http", "url": "http://localhost:3000/mcp" } } }
```

### HTTP as a server

Any other bind needs `MCP_AUTH_TOKEN`, at least 32 characters, and every client sends it as
`Authorization: Bearer <token>`. See [Environment](../environments/env.md).

## After registering

A client loads its servers when a session starts, so a server added mid-session stays absent
until the session restarts. Check it with `node src/cli.js tools`, which prints the same list
the client should see.
