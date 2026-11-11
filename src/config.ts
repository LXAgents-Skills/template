import { availableParallelism } from "node:os";

/*
 * Every environment variable the server reads, read in one place.
 *
 * The one exception is `MCP_AUTH_TOKEN`, which `src/http/auth.ts` reads at call time on
 * purpose: a secret captured once at load is whatever was set then, and the token
 * checks must see it change.
 */

/** A comma-separated list, trimmed, with empty entries dropped. Unset is `[]`. */
function splitList(raw: string | undefined): string[] {
  if (!raw) return [];
  return raw
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

/**
 * How many HTTP workers to run.
 *
 * `MCP_CLUSTER_WORKERS` overrides the count. **A value of 1 means no forking at all** —
 * one process, one listener, the pre-cluster behaviour — which is what makes the
 * cluster change bisectable: the same code answers, with and without workers, and a
 * difference between them is a difference in the fork rather than in the transport.
 *
 * Unset, the count is the number of CPUs the process was actually given, not a
 * constant: a container with two CPUs gets two workers and a laptop does not get
 * eight.
 */
function workerCount(raw: string | undefined): number {
  const configured = Number.parseInt(raw ?? "", 10);
  if (Number.isInteger(configured) && configured >= 0) return configured;
  return availableParallelism();
}

/** The server's configuration, as `loadConfig` returns it. Frozen. */
export interface Config {
  readonly transport: string;
  readonly port: number;
  readonly host: string;
  readonly local: boolean;
  readonly workers: number;
  readonly allowedHosts: readonly string[];
}

/**
 * Whether `host` is a loopback address, which only this machine can reach.
 *
 * `localhost`, any `127.x.x.x`, and `::1` with or without brackets. `0.0.0.0` and `::`
 * mean every interface, so they are not loopback; neither is an empty value.
 */
export function isLoopback(host: string | null | undefined): boolean {
  const value = String(host ?? "").trim().toLowerCase();
  return (
    value === "localhost" ||
    value === "::1" ||
    value === "[::1]" ||
    /^127(?:\.\d{1,3}){3}$/.test(value)
  );
}

/**
 * The server's configuration, from `env`.
 *
 * * `transport` — `MCP_TRANSPORT`, lowercased; `stdio` when unset.
 * * `port` — `PORT`; 3000 when unset.
 * * `host` — `HOST`. The bind address is named rather than implied: listen(port) with
 *   no host binds every interface, which is what a published port needs and what a
 *   container gets, but it is a decision nobody made, so it is made here, visibly.
 *   `HOST=127.0.0.1` is the way to take it back.
 * * `local` — whether `host` is loopback. A local HTTP server needs no token; see
 *   `src/http/auth.ts`.
 * * `workers` — `MCP_CLUSTER_WORKERS`, see `workerCount`.
 * * `allowedHosts` — `MCP_ALLOWED_HOSTS`. Unset, or nothing but commas and spaces,
 *   means no `Host` check is applied rather than one guessed at: a wrong allow-list
 *   silently refusing every request is a worse failure than an absent one, and this
 *   server serves public markdown either way, so the absence is reported on startup
 *   rather than papered over with a default list.
 */
export function loadConfig(env: NodeJS.ProcessEnv = process.env): Config {
  const host = env.HOST || "0.0.0.0";
  return Object.freeze({
    transport: (env.MCP_TRANSPORT ?? "stdio").toLowerCase(),
    port: Number.parseInt(env.PORT ?? "3000", 10),
    host,
    local: isLoopback(host),
    workers: workerCount(env.MCP_CLUSTER_WORKERS),
    allowedHosts: Object.freeze(splitList(env.MCP_ALLOWED_HOSTS)),
  });
}
