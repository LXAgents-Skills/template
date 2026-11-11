# Docker

The image has two stages on `node:22-alpine`. The first installs every dependency and compiles
`src/` into `dist/`. The second installs production dependencies from the lockfile, copies in
`dist/` and `content/`, and runs `node dist/index.js` as the `node` user; the compiler never
reaches it. It defaults to HTTP on `0.0.0.0:3000`, so it needs a token.

```bash
docker build -t lxagents-template .
docker run --rm -p 3000:3000 -e MCP_AUTH_TOKEN lxagents-template
```

stdio from the same image needs `-i`, or the server sees closed input and exits:

```bash
docker run --rm -i -e MCP_TRANSPORT=stdio lxagents-template
```

Set `MCP_ALLOWED_HOSTS` whenever the container is reachable from anywhere but this machine.
`GET /healthz` answers without a token, for an orchestrator's probe.
