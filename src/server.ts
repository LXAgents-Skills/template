import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { SERVER_ID, SERVER_SUMMARY, SERVER_TITLE, toolId } from "./identity.js";
import { CONTENT_INSTRUCTIONS } from "./instructions/from-content.js";

/*
 * The Skill MCP server.
 *
 * The surface is `CONTENT_INSTRUCTIONS` and nothing else. A tool registered anywhere else is
 * invisible to `listTools()`, to the CLI and to the tests. No tool takes an argument, a verb or
 * a credential, and nothing here writes or reaches a network: the code that would is absent.
 */

const INSTRUCTIONS = [
  SERVER_SUMMARY,
  `Read ${toolId("automation")} once at the start of every session: it lists every tool and the one condition that activates each.`,
  "Call no other tool until its condition is true. No tool takes an argument.",
].join(" ");

/** A registered tool, as the CLI prints it. */
export interface ToolSummary {
  name: string;
  description: string;
}

/**
 * The registered tools, as name and description. The CLI prints this, so the two surfaces
 * cannot drift apart.
 */
export function listTools(): ToolSummary[] {
  return CONTENT_INSTRUCTIONS.map(({ config }) => ({ name: config.name, description: config.description }));
}

/** What `createServer` needs to know that is not fixed at build time. */
export interface ServerOptions {
  version: string;
}

/** A fresh server. One per connection, because `McpServer` holds per-connection state. */
export function createServer({ version }: ServerOptions): McpServer {
  const server = new McpServer(
    { name: SERVER_ID, title: SERVER_TITLE, version },
    { instructions: INSTRUCTIONS },
  );

  for (const { config, handler } of CONTENT_INSTRUCTIONS) {
    server.registerTool(config.name, { description: config.description }, handler);
  }

  return server;
}
