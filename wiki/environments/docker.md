# Docker

The image has two stages. The first, on `rust:1.97-alpine`, builds the server with
`cargo build --release --locked` into one static musl binary with `content/` compiled into it.
The second, on `alpine:3.22`, holds only that binary and runs it as an unprivileged user
(uid 10001); the toolchain and the sources never reach it. It defaults to HTTP on
`0.0.0.0:3000`, so it needs a token.

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

A change to `content/` needs a new image, because the set is compiled into the binary.
