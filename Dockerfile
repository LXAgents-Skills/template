# syntax=docker/dockerfile:1

# Two stages. The first compiles the server into one static binary on Alpine (musl), with
# content/ embedded in it; the second is a bare Alpine image holding that binary and nothing
# else. The toolchain, the sources and the tests never reach the image.
FROM rust:1.97-alpine AS build

RUN apk add --no-cache musl-dev

WORKDIR /build

# Everything `cargo build` reads: the workspace manifests and lockfile, the build script, the
# sources, the set it embeds, and the xtask manifest the workspace names. `--locked` fails
# rather than resolving something the lockfile does not contain, which is the point of building
# an image from a versioned tree.
COPY Cargo.toml Cargo.lock build.rs ./
COPY .cargo ./.cargo
COPY xtask ./xtask
COPY src ./src
COPY content ./content
RUN cargo build --release --locked --bin lxagents-template

FROM alpine:3.22

# Not root. The process answers JSON-RPC from text compiled into it, and nothing else.
RUN adduser -D -H -u 10001 mcp
COPY --from=build /build/target/release/lxagents-template /usr/local/bin/lxagents-template
USER mcp

# 3000 is the default, and `PORT` overrides it at runtime. The interface is `HOST`, which
# defaults to 0.0.0.0 - every IPv4 interface.
#
# EXPOSE documents the port; it does not publish it. `docker run -p 3000:3000 …` is what makes
# the port reachable from outside the container. See wiki/environments/docker.md.
EXPOSE 3000

# HTTP is the default here, so a container that is run rather than spawned is reachable:
#
#   docker run --rm -p 3000:3000 -e MCP_AUTH_TOKEN lxagents-template
#
# HTTP refuses to start without MCP_AUTH_TOKEN (at least 32 characters): the container exits 1
# with a line saying why, and does not run open. The token is passed at run time and is
# deliberately not an ENV line here, because a value baked into the image ships to everyone who
# pulls it. stdio never reads it.
#
# stdio is one environment variable away:
#
#   docker run --rm -i -e MCP_TRANSPORT=stdio lxagents-template
#
# For stdio, `-i` is not optional. Without it Docker does not attach stdin, the server sees
# closed input, and it exits at once - which reads as a broken image rather than a missing flag.
#
# Set MCP_ALLOWED_HOSTS whenever the container is reachable from anywhere but this machine. The
# Host allow-list is off unless it is set, and the server says so on startup. See
# wiki/environments/env.md.
ENV MCP_TRANSPORT=http
ENTRYPOINT ["/usr/local/bin/lxagents-template"]
