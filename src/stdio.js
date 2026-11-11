import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { SERVER_ID } from "./identity.js";
import { createServer } from "./server.js";
import { version } from "./version.js";

/**
 * Serve over stdio.
 *
 * stdout is the JSON-RPC channel here, so nothing on this path writes to it, and nothing on
 * it reads `MCP_AUTH_TOKEN`: whoever can use the pipe already runs the process.
 */
export async function startStdio() {
  const server = createServer({ version });
  await server.connect(new StdioServerTransport());
  process.stderr.write(`${SERVER_ID} ${version} serving over stdio\n`);
}
