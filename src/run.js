import cluster from "node:cluster";

/*
 * Choose the transport and start it.
 *
 * Nothing here may write to stdout: on stdio, stdout is the JSON-RPC channel.
 *
 * The transports are imported only once chosen, so a stdio server never loads express
 * and an HTTP server never loads the stdio transport.
 */

/**
 * @param {ReturnType<typeof import("./config.js").loadConfig>} config
 */
export async function run(config) {
  if (config.transport === "http" || config.transport === "streamable-http") {
    if (cluster.isPrimary) {
      const { startPrimary } = await import("./http/primary.js");
      startPrimary(config);
    } else {
      const { startWorker } = await import("./http/worker.js");
      startWorker(config);
    }
    return;
  }

  const { startStdio } = await import("./stdio.js");
  await startStdio();
}
