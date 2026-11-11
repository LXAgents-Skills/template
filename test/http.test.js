import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { get } from "node:http";
import { once } from "node:events";
import { createServer as createNetServer } from "node:net";
import { join } from "node:path";
import { test } from "node:test";
import { createApp } from "../src/http/app.js";
import { SERVER_ID, toolId } from "../src/identity.js";
import { ROOT } from "../src/paths.js";

const TOKEN = "t".repeat(32);
const LIST = { jsonrpc: "2.0", id: 1, method: "tools/list", params: {} };

/** Start `app` on a free loopback port, hand its base URL to `run`, then stop it. */
async function withApp(app, run) {
  const server = app.listen(0, "127.0.0.1");
  await once(server, "listening");
  try {
    return await run(`http://127.0.0.1:${server.address().port}`);
  } finally {
    server.closeAllConnections();
    server.close();
  }
}

/** POST a JSON-RPC body to `/mcp`. */
function rpc(base, body, headers = {}) {
  return fetch(`${base}/mcp`, {
    method: "POST",
    headers: { "content-type": "application/json", accept: "application/json, text/event-stream", ...headers },
    body: JSON.stringify(body),
  });
}

/** GET `path` with a forged `Host` header. `fetch` cannot send one, so this uses node:http. */
async function getWithHost(base, path, host) {
  const request = get(`${base}${path}`, { headers: { host } });
  const [response] = await once(request, "response");
  response.resume();
  return response.statusCode;
}

/** A port nothing is listening on. */
async function freePort() {
  const probe = createNetServer().listen(0, "127.0.0.1");
  await once(probe, "listening");
  const { port } = probe.address();
  probe.close();
  return port;
}

/** Run the real entry point with `env`, and resolve with its exit code and stderr. */
function runServer(env, { until } = {}) {
  const child = spawn(process.execPath, [join(ROOT, "src/index.js")], {
    env: { PATH: process.env.PATH, MCP_TRANSPORT: "http", MCP_CLUSTER_WORKERS: "1", ...env },
    stdio: ["ignore", "ignore", "pipe"],
  });
  let stderr = "";
  return new Promise((resolve) => {
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
      if (until && stderr.includes(until)) resolve({ child, stderr });
    });
    child.on("exit", (code) => resolve({ code, stderr }));
  });
}

test("off loopback, the app cannot be built without a token", () => {
  assert.throws(() => createApp({ token: "", allowedHosts: [], local: false }), /MCP_AUTH_TOKEN is not set/);
});

test("a token shorter than 32 characters is refused even on loopback", () => {
  assert.throws(() => createApp({ token: "short", allowedHosts: [], local: true }), /at least 32/);
});

test("with a token, /mcp needs it and /healthz does not", async () => {
  await withApp(createApp({ token: TOKEN, allowedHosts: [], local: false }), async (base) => {
    const health = await fetch(`${base}/healthz`);
    assert.equal(health.status, 200);
    assert.equal((await health.json()).server, SERVER_ID);

    const missing = await rpc(base, LIST);
    assert.equal(missing.status, 401);
    assert.match(missing.headers.get("www-authenticate"), /^Bearer/);

    const wrong = await rpc(base, LIST, { authorization: `Bearer ${"x".repeat(32)}` });
    assert.equal(wrong.status, 401);
    assert.match(wrong.headers.get("www-authenticate"), /invalid_token/);

    const ok = await rpc(base, LIST, { authorization: `Bearer ${TOKEN}` });
    assert.equal(ok.status, 200);
    assert.match(await ok.text(), new RegExp(toolId("automation")));
  });
});

test("on loopback without a token, only localhost Host headers are answered", async () => {
  await withApp(createApp({ token: "", allowedHosts: [], local: true }), async (base) => {
    assert.equal((await rpc(base, LIST)).status, 200);

    assert.equal(await getWithHost(base, "/healthz", "evil.example"), 403);
    assert.equal(await getWithHost(base, "/healthz", "localhost"), 200);
  });
});

test("unknown routes are 404 and other methods on /mcp are 405", async () => {
  await withApp(createApp({ token: "", allowedHosts: [], local: true }), async (base) => {
    assert.equal((await fetch(`${base}/nope`)).status, 404);
    assert.equal((await fetch(`${base}/mcp`)).status, 405);
  });
});

test("the server refuses to start on 0.0.0.0 without a token", async () => {
  const { code, stderr } = await runServer({ HOST: "0.0.0.0", PORT: String(await freePort()) });
  assert.equal(code, 1);
  assert.match(stderr, /will not start the HTTP transport: MCP_AUTH_TOKEN is not set/);
});

test("the server starts on 127.0.0.1 without a token", async () => {
  const port = await freePort();
  const { child, stderr } = await runServer({ HOST: "127.0.0.1", PORT: String(port) }, { until: "no bearer token is required" });
  try {
    assert.match(stderr, /bound to loopback/);
    assert.equal((await fetch(`http://127.0.0.1:${port}/healthz`)).status, 200);
  } finally {
    child.kill("SIGTERM");
    await once(child, "exit");
  }
});
