import type { Response } from "express";

/**
 * A JSON-RPC error response.
 *
 * The shape every refusal on this server uses: the same envelope a client parses for a
 * successful response, so a client never has to branch on content type to find out it
 * was refused. The 401 from the token check, the 404, the 405 and the parse errors all
 * come through here, so they cannot drift apart.
 */
export function rpcError(res: Response, status: number, code: number, message: string): void {
  res.status(status).json({ jsonrpc: "2.0", error: { code, message }, id: null });
}
