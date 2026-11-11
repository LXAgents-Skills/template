import { createHash, timingSafeEqual } from "node:crypto";
import { rpcError } from "./rpc-error.js";

/**
 * Bearer-token authentication for the HTTP transport.
 *
 * **The transport decides whether a token is needed.** stdio is a pipe that a client spawns on
 * its own machine, so whoever can use it already runs the process and there is nobody to
 * authenticate. HTTP is a socket that anyone who can reach the port can open, so every request
 * to it must prove the caller holds the token — and the server refuses to run HTTP without one,
 * rather than running open and saying so in a log nobody reads.
 *
 * **Where the socket listens decides whether that is a network.** A server bound to loopback
 * (`127.0.0.1`, `localhost`, `::1`) can only be reached from this machine, which is the same
 * trust a stdio pipe has, so a token is optional there: a local install runs without one, and
 * the localhost `Host` allow-list in `createApp` stops a web page reaching it by DNS
 * rebinding. Any other bind — `0.0.0.0`, a container, a public address — is a server, and it
 * still refuses to start without a token. A token that is set is always enforced.
 *
 * Nothing in this file is reached on the stdio path, and `src/stdio.js` never reads
 * `MCP_AUTH_TOKEN`.
 */

/**
 * The shortest token the server will accept, in characters.
 *
 * The token is the only thing standing between a network and this server, so a value like
 * `test` is refused at startup instead of being discovered by a scanner. 32 characters is what
 * `openssl rand -hex 16` produces at the very least, and the documented command produces 64.
 */
export const MIN_TOKEN_LENGTH = 32;

/**
 * The configured token, or an empty string when there is none.
 *
 * Read at call time and never at import: a value captured when the module loaded is whatever
 * happened to be set then, which breaks a process that sets it later and any test that sets and
 * unsets it. Surrounding whitespace is dropped, because a token copied out of a file or an
 * `env_file` often carries a trailing newline and an HTTP header cannot.
 *
 * @returns {string}
 */
export function configuredToken() {
  return (process.env.MCP_AUTH_TOKEN ?? "").trim();
}

/**
 * Why `token` cannot be used, or `null` when it can.
 *
 * The message names the variable and the fix and never the value, so it is safe to log.
 *
 * `required: false` is the local case: no token at all is then fine, but a token that is set
 * and too short is still a problem, because it was meant to protect something and would not.
 *
 * @param {string} [token]
 * @param {{ required?: boolean }} [options]
 * @returns {string | null}
 */
export function tokenProblem(token = configuredToken(), { required = true } = {}) {
  if (token === "") return required ? "MCP_AUTH_TOKEN is not set" : null;
  if (token.length < MIN_TOKEN_LENGTH) {
    return `MCP_AUTH_TOKEN is ${token.length} characters, and it must be at least ${MIN_TOKEN_LENGTH}`;
  }
  return null;
}

/** SHA-256 of a string, so two values of any length compare in constant time. */
function digest(value) {
  return createHash("sha256").update(value).digest();
}

/**
 * The credential in an `Authorization: Bearer <credential>` header, or `null`.
 *
 * The scheme is case-insensitive (RFC 7235). Anything else — another scheme, no credential, two
 * of them — is not a bearer credential. The pattern is linear: the space run and the credential
 * are disjoint character classes.
 *
 * @param {string | undefined} header
 * @returns {string | null}
 */
function bearerCredential(header) {
  const match = /^Bearer +(\S+)$/i.exec(header ?? "");
  return match ? match[1] : null;
}

/**
 * Express middleware that lets a request through only when it carries `token`.
 *
 * A refusal is a `401` in the JSON-RPC error envelope every other refusal on this server uses,
 * with `WWW-Authenticate: Bearer`. A request that sent a credential which did not match also
 * gets `error="invalid_token"`, so a client can tell a missing header from a wrong one. Neither
 * answer reveals anything about the token itself.
 *
 * The comparison is constant-time over fixed-length digests, so neither the token's length nor
 * how much of it a guess got right shows in how long the answer takes.
 *
 * @param {string} token A token that already passed `tokenProblem`.
 * @returns {import("express").RequestHandler}
 */
export function requireBearerToken(token) {
  const expected = digest(token);

  return (req, res, next) => {
    const credential = bearerCredential(req.get("authorization"));

    if (credential !== null && timingSafeEqual(digest(credential), expected)) {
      next();
      return;
    }

    const challenge =
      credential === null ? 'Bearer realm="mcp"' : 'Bearer realm="mcp", error="invalid_token"';
    res.set("WWW-Authenticate", challenge);
    rpcError(
      res,
      401,
      -32001,
      credential === null
        ? "Unauthorized: send the token as Authorization: Bearer <token>"
        : "Unauthorized: the token is not valid",
    );
  };
}
