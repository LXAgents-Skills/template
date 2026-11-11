import express from "express";
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import {
  hostHeaderValidation,
  localhostHostValidation,
} from "@modelcontextprotocol/sdk/server/middleware/hostHeaderValidation.js";
import { configuredToken, requireBearerToken, tokenProblem } from "./auth.js";
import { loadConfig } from "../config.js";
import { rpcError } from "./rpc-error.js";
import { SERVER_ID } from "../identity.js";
import { createServer } from "../server.js";
import { version } from "../version.js";

/**
 * The HTTP transport, as an application.
 *
 * This module builds an express app and returns it. **It does not listen** —
 * `src/http/worker.js` owns the port. A file that both builds the app and binds a port
 * cannot be reasoned about, or tested, without binding one.
 *
 * The surface is deliberately narrow: `POST /mcp` and `GET /healthz`, a JSON-RPC 404
 * for everything else, and a 405 for any other method on `/mcp`. **Everything except
 * `GET /healthz` needs the bearer token** - see `src/http/auth.js` for why, and `createApp`
 * for where. Nothing here is a
 * second source of truth about the tool list — `createServer()` is the same factory
 * `src/stdio.js` uses, and it returns a fresh `McpServer` per call, so each
 * request gets its own. Sharing one across requests would be a real bug: `McpServer`
 * holds per-connection state.
 */

/**
 * The request body ceiling, in bytes.
 *
 * The previous implementation enforced this by counting chunks as they arrived and
 * throwing past the limit. The same number, declared rather than counted.
 */
export const BODY_LIMIT_BYTES = 4 * 1024 * 1024;

/**
 * Build the HTTP application.
 *
 * `token` is the bearer token every request but `/healthz` must carry. It defaults to
 * `MCP_AUTH_TOKEN`, read now. **There is no way to build this app without one**: a
 * missing or too-short token throws, so an unauthenticated HTTP server cannot come from
 * forgetting an option. `src/http/primary.js` checks first, to report it once
 * instead of once per worker.
 *
 * `allowedHosts` is the `Host` header allow-list, from `MCP_ALLOWED_HOSTS` by default; an
 * empty list applies no check.
 *
 * `local` is set when the server binds loopback. Only then may `token` be empty: the app is
 * built without the token check, and with the localhost `Host` allow-list unless
 * `allowedHosts` names one. It defaults to `false`, so leaving it out can never turn the
 * check off.
 *
 * @param {{ token?: string, allowedHosts?: readonly string[], local?: boolean }} [options]
 * @returns {import("express").Express}
 */
export function createApp({
  token = configuredToken(),
  allowedHosts = loadConfig().allowedHosts,
  local = false,
} = {}) {
  const problem = tokenProblem(token, { required: !local });
  if (problem) {
    throw new Error(`Refusing to build the HTTP app: ${problem}.`);
  }

  const app = express();

  // Express stamps `X-Powered-By: Express` on every response it sends, which hands an
  // unauthenticated caller the framework and the exact version serving the port — a
  // free upgrade suggestion, and a narrowing of what an attacker has to guess. The
  // header is removed here deliberately and this line is a security control, not an
  // omission: do not restore it because a route looks like it is missing a header.
  //
  // `disable` rather than `app.set` because this is a setting of the app itself and it
  // must hold for every response, including the ones no route here produces.
  app.disable("x-powered-by");

  // Ahead of the body parser and ahead of every route, `/healthz` included: an
  // allow-list that guards `/mcp` and not `/healthz` is an allow-list with a hole in
  // it, and a deployment watching a green health check on a service nothing can reach
  // is worse than no check at all.
  //
  // The SDK applies host validation automatically only through its own Express app
  // factory, and only when the host is loopback. This server binds `0.0.0.0` by
  // default, so without an explicit list there is no `Host` filtering in exactly the
  // deployment — a container, a shared host — where it would matter.
  //
  // A local server running without a token gets the localhost list when none is configured:
  // with no token, the `Host` check is what stops a web page that rebinds its own name to
  // 127.0.0.1 from reading this server through the browser.
  const open = token === "";
  if (allowedHosts.length > 0) {
    app.use(hostHeaderValidation([...allowedHosts]));
  } else if (open) {
    app.use(localhostHostValidation());
  }

  // The token check, after the `Host` allow-list and ahead of the body parser and every
  // route. Ahead of the parser, so a caller without the token cannot make this server read
  // and parse up to 4 MB; ahead of the routes, so it cannot find out which exist - the 404
  // and the 405 are answers about routes, and they are only given to a caller who may ask.
  //
  // `GET /healthz` is the one exemption, and it is exact: the method and the path, nothing
  // that merely starts with it. An orchestrator's probe cannot send a token, and the route
  // returns only `{ status, server, version }`. The exemption is never looser than the
  // route: express would also match `/healthz/` and `HEAD`, and those fall through to the
  // check and are refused, which is the safe direction to be wrong in.
  const authenticate = open ? null : requireBearerToken(token);
  app.use((req, res, next) => {
    if (!authenticate) {
      next();
      return;
    }
    if (req.method === "GET" && req.path === "/healthz") {
      next();
      return;
    }
    authenticate(req, res, next);
  });

  app.use(express.json({ limit: BODY_LIMIT_BYTES }));

  /**
   * The health check. Answers without a session, a request, or a tool.
   */
  app.get("/healthz", (_req, res) => {
    res.status(200).json({ status: "ok", server: SERVER_ID, version });
  });

  /**
   * The Skill MCP endpoint.
   *
   * Stateless: a fresh `McpServer` and a fresh transport per request, with
   * `sessionIdGenerator: undefined` telling the transport not to mint one. There is no
   * session store to keep bounded, because there are no sessions.
   */
  app.post("/mcp", async (req, res) => {
    const server = createServer({ version });
    const transport = new StreamableHTTPServerTransport({ sessionIdGenerator: undefined });

    // Fires on disconnect as well as on a clean close, which is the case that leaks.
    // Closing both halves is what keeps a stateless transport stateless: a retained
    // McpServer per request would be a leak per request.
    res.on("close", () => {
      void transport.close();
      void server.close();
    });

    try {
      await server.connect(transport);
      await transport.handleRequest(req, res, req.body);
    } catch (error) {
      if (!res.headersSent) rpcError(res, 500, -32603, String(error));
    }
  });

  // Any other method on `/mcp` is refused rather than served. It is a refusal and not
  // a 404: the path exists, and saying so is more useful to a client than pretending
  // it does not.
  app.all("/mcp", (req, res) => {
    rpcError(res, 405, -32000, `${req.method} is not supported in stateless mode`);
  });

  // Anything else is not a route this server has. A 404 that says so is more useful
  // than a bare one, and it is the only place a request is answered with prose.
  app.use((req, res) => {
    rpcError(res, 404, -32601, `Not found: ${req.originalUrl}`);
  });

  /**
   * Errors the routes above did not answer.
   *
   * `express.json` reports an oversized body as `entity.too.large` and a malformed one
   * as `entity.parse.failed`. **Both become the same 400 / `-32700`**, because the
   * previous hand-rolled reader produced one answer for both: it threw the same
   * failure whichever way the request was wrong. Collapsing them is a deliberate
   * preservation, not an oversight.
   *
   * Registered last, and declared with four arguments, because that is how express
   * recognises an error handler rather than ordinary middleware.
   */
  app.use((error, _req, res, next) => {
    if (res.headersSent) {
      next(error);
      return;
    }

    if (error?.type === "entity.too.large" || error?.type === "entity.parse.failed") {
      rpcError(res, 400, -32700, "Parse error: request body is not valid JSON");
      return;
    }

    rpcError(res, 500, -32603, String(error?.message ?? error));
  });

  return app;
}
