import { SERVER_ID } from "../identity.js";
import { version } from "../version.js";
import { createApp } from "./app.js";
import { configuredToken } from "./auth.js";

/**
 * The HTTP worker: the process that actually answers.
 *
 * Also the whole server when `MCP_CLUSTER_WORKERS` is 1, since then nothing is forked.
 *
 * Every worker binds the same `port`. The kernel's shared handle and the round-robin
 * scheduler do the distribution — no `SO_REUSEPORT` is set by hand and no sticky
 * session logic is written, because the scheduler already has the information
 * (which connection is next) that a sticky-session scheme would have to reconstruct.
 */
export function startWorker({ port, host, allowedHosts, local }) {
  // A worker whose primary is gone holds the port for whoever starts next. The test
  // harness kills the child process directly, so this is the difference between a
  // suite that passes and a suite that fails on its second run with EADDRINUSE.
  process.on("disconnect", () => process.exit(0));

  const tokenSet = configuredToken() !== "";
  const server = createApp({ allowedHosts, local }).listen(port, host, () => {
    const where = host === "0.0.0.0" ? "all interfaces" : host;
    process.stderr.write(
      `${SERVER_ID} ${version} serving over http on :${port}/mcp (${where})\n`
    );

    // A statement of which control is on. Neither line ever carries the token.
    if (tokenSet) {
      process.stderr.write(
        `${SERVER_ID} ${version} bearer token required on every route except GET /healthz.\n`
      );
    } else {
      process.stderr.write(
        `${SERVER_ID} ${version} bound to loopback, so no bearer token is required; ` +
          `Host is limited to ${allowedHosts.length > 0 ? "MCP_ALLOWED_HOSTS" : "localhost"}.\n`
      );
    }

    // Said out loud, because the default is the unguarded one. Someone reading a
    // container's startup log is the only person who can act on it, and a control
    // that is off silently is worse than no control at all — it reads as present.
    if (allowedHosts.length === 0 && tokenSet) {
      process.stderr.write(
        `${SERVER_ID} ${version} MCP_ALLOWED_HOSTS is unset, so no Host header allow-list is applied.\n`
      );
    }
  });

  /*
   * Shutdown, in three steps, in this order.
   *
   * `close()` first, so nothing new arrives — a request accepted during the drain gets
   * an answer rather than a refused connection. Then idle keep-alive sockets are
   * closed, because `close()` waits on them and a client that opened one and went quiet
   * would hold the process open indefinitely for a request that no longer exists. Then
   * a short grace period, after which whatever is genuinely still in flight is cut off
   * rather than waited on forever.
   *
   * Each request is self-contained — a fresh McpServer, closed when its response
   * closes — so there is no session state to drain. What drains is the requests.
   */
  let shuttingDown = false;

  const shutdown = (signal) => {
    if (shuttingDown) return;
    shuttingDown = true;
    process.stderr.write(`${SERVER_ID} ${version} ${signal}, draining\n`);

    const forced = setTimeout(() => server.closeAllConnections(), 5000);
    forced.unref();

    server.close(() => {
      clearTimeout(forced);
      process.exit(0);
    });
    server.closeIdleConnections();
  };

  process.on("SIGTERM", () => shutdown("SIGTERM"));
  process.on("SIGINT", () => shutdown("SIGINT"));
}
