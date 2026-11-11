---
name: http-auth
description: When MCP_AUTH_TOKEN is required - only for HTTP bound off loopback - the localhost Host rule for a tokenless loopback server, and how secrets are handled.
---

# HTTP Auth

| Transport and bind | Token | `Host` check |
|---|---|---|
| stdio | Never read | None — whoever can use the pipe runs the process |
| HTTP on loopback (`127.0.0.1`, `localhost`, `::1`) without a token | Optional | `localhost` only, unless `MCP_ALLOWED_HOSTS` is set |
| HTTP on loopback with a token | Enforced | `MCP_ALLOWED_HOSTS` if set |
| HTTP anywhere else (`0.0.0.0`, a container, a public host) | **Required** — the server will not start without it | `MCP_ALLOWED_HOSTS` if set |

* A local install needs no token; a server install always does. A token that is set is always
  enforced, on any bind.
* The token is at least 32 characters (`openssl rand -hex 32`). A shorter one is refused at
  startup, even on loopback.
* Every route except exactly `GET /healthz` needs `Authorization: Bearer <token>`. The check
  runs before the body parser and before routing.
* The loopback `Host` check is what stops a web page that rebinds its own name to `127.0.0.1`
  from reading a tokenless server through the browser. Do not remove it.
* `createApp` refuses to build without a token unless `local` is true, and `local` defaults to
  false, so leaving an option out can never open the server.

## Secrets

* `MCP_AUTH_TOKEN` is read at call time by `src/http/auth.ts`, not captured at import.
* Never log it, echo it, put it in an error, a test fixture, a Dockerfile `ENV` line, or a
  committed file. Error messages name the variable and the fix, never the value.
* No tool reads a credential. A served file is public text.
