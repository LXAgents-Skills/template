import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { createServer } from "../src/server.js";

/**
 * Connect an in-memory client to a fresh server, hand the client to `run`, and close both
 * afterwards whether or not `run` throws.
 */
export async function withClient<T>(run: (client: Client) => Promise<T>): Promise<T> {
  const server: McpServer = createServer({ version: "0.0.0" });
  const client = new Client({ name: "test-client", version: "0.0.0" });
  const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();

  await Promise.all([server.connect(serverTransport), client.connect(clientTransport)]);

  try {
    return await run(client);
  } finally {
    await client.close();
    await server.close();
  }
}
