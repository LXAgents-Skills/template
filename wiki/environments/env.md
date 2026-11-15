# Environment

Every variable the server reads. None is needed for stdio.

| Variable | Default | Meaning |
|---|---|---|
| `MCP_TRANSPORT` | `stdio` | `stdio` or `http`. The Docker image sets `http`. |
| `MCP_AUTH_TOKEN` | unset | Bearer token, 32+ characters. Required for HTTP unless `HOST` is loopback. Unused by stdio. |
| `HOST` | `0.0.0.0` | HTTP interface. `serve --host 127.0.0.1` sets it for one run. |
| `PORT` | `3000` | HTTP port. Anything that is not a port number stops the server with exit code 2. |
| `MCP_ALLOWED_HOSTS` | unset | Comma-separated `Host` allow-list. Unset on loopback without a token means `localhost` only; unset otherwise means no `Host` check, and the server says so at startup. |
| `MCP_CLUSTER_WORKERS` | one per CPU | HTTP worker threads in the tokio runtime. `1` runs on one thread. The name is kept from when it counted processes, so existing deployments mean the same thing. |

## When the token is required

| Run | Token |
|---|---|
| `lxagents-template` with no arguments (stdio) | Never read |
| `lxagents-template serve --http --host 127.0.0.1` | Optional; enforced if set |
| `HOST=0.0.0.0`, a container, a public host | Required: the server exits 1 without it |

Make one with `openssl rand -hex 32`, or in PowerShell:

```powershell
$env:MCP_AUTH_TOKEN = -join ((1..32) | ForEach-Object { '{0:x2}' -f (Get-Random -Maximum 256) })
```

Every route except `GET /healthz` then needs `Authorization: Bearer <token>`.
