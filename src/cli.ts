#!/usr/bin/env node

import { SERVER_ID } from "./identity.js";
import { version } from "./version.js";

/*
 * The command line: `serve` starts the server, `tools` prints the surface.
 *
 * Only this file writes to stdout. On stdio, stdout is the JSON-RPC channel.
 */

const HELP = `${SERVER_ID} - a Skill MCP instruction set, served read-only

Usage
  node dist/cli.js <command>

Commands
  serve            Run the Skill MCP server
                   [--stdio | --http] [--port N] [--host ADDR]
  tools            List every tool ID, its length, and its description

Options
  -h, --help       Show this help
  -v, --version    Show the version

Environment
  MCP_TRANSPORT        stdio (default) or http
  MCP_AUTH_TOKEN       bearer token, 32+ characters; required for http unless HOST is
                       loopback (127.0.0.1, localhost, ::1); unused by stdio
  PORT                 HTTP port, default 3000
  HOST                 HTTP interface, default 0.0.0.0
  MCP_ALLOWED_HOSTS    comma-separated Host allow-list
  MCP_CLUSTER_WORKERS  HTTP worker count, default one per CPU; 1 forks nothing`;

/** The value after `flag` in `args`, or `undefined`. */
function flagValue(args: readonly string[], flag: string): string | undefined {
  const at = args.indexOf(flag);
  return at === -1 ? undefined : args[at + 1];
}

/** The environment variables a `serve` flag can set. */
type ServeOverrides = Partial<Record<"MCP_TRANSPORT" | "PORT" | "HOST", string>>;

/**
 * Environment overrides from `serve` flags. They are not written back to `process.env`: a
 * forked HTTP worker re-runs this file with the same arguments and reaches the same result.
 */
function serveOverrides(args: readonly string[]): ServeOverrides {
  const overrides: ServeOverrides = {};
  if (args.includes("--http")) overrides.MCP_TRANSPORT = "http";
  if (args.includes("--stdio")) overrides.MCP_TRANSPORT = "stdio";
  const port = flagValue(args, "--port");
  if (port) overrides.PORT = port;
  const host = flagValue(args, "--host");
  if (host) overrides.HOST = host;
  return overrides;
}

async function serve(args: readonly string[]): Promise<void> {
  const [{ loadConfig }, { run }] = await Promise.all([import("./config.js"), import("./run.js")]);
  await run(loadConfig({ ...process.env, ...serveOverrides(args) }));
}

async function printTools(): Promise<void> {
  const { listTools } = await import("./server.js");
  const tools = listTools();
  const width = Math.max(0, ...tools.map((tool) => tool.name.length));
  for (const tool of tools) {
    process.stdout.write(`${tool.name.padEnd(width)}  ${String(tool.name.length).padStart(2)}  ${tool.description}\n`);
  }
}

const [command, ...rest] = process.argv.slice(2);

if (command === "-v" || command === "--version") {
  process.stdout.write(`${version}\n`);
} else if (!command || command === "-h" || command === "--help" || command === "help") {
  process.stdout.write(`${HELP}\n`);
  process.exitCode = command ? 0 : 2;
} else if (command === "serve") {
  await serve(rest);
} else if (command === "tools") {
  await printTools();
} else {
  process.stderr.write(`Unknown command "${command}". Try --help.\n`);
  process.exitCode = 2;
}
