const { test } = require("node:test");
const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { runInNewContext } = require("node:vm");
const { join } = require("node:path");

function worker() {
  const handlers = {};
  const writes = [];
  const context = {
    URL,
    self: {
      location: { origin: "https://xrat.test" },
      addEventListener: (name, handler) => (handlers[name] = handler),
    },
    caches: {
      open: async () => ({ add: async (url) => writes.push(url) }),
      match: async () => "offline document",
    },
    fetch: async () => {
      throw new Error("offline");
    },
  };
  runInNewContext(readFileSync(join(__dirname, "../sw.js"), "utf8"), context);
  return { handlers, writes };
}

test("installation caches only the public offline document", async () => {
  const { handlers, writes } = worker();
  let installed;
  handlers.install({ waitUntil: (promise) => (installed = promise) });
  await installed;
  assert.deepEqual(writes, ["/offline.html"]);
});

test("offline fallback never intercepts credentials, APIs or mutations", async () => {
  const { handlers } = worker();
  for (const [path, mode, method, intercepted] of [
    ["/", "navigate", "GET", true],
    ["/ui/configs", "navigate", "GET", true],
    ["/ui/account?token=secret", "navigate", "GET", false],
    ["/b64?key=secret", "navigate", "GET", false],
    ["/api/v1/me", "navigate", "GET", false],
    ["/api/v1/actions", "cors", "POST", false],
    ["/xrat_web.js", "cors", "GET", false],
  ]) {
    let response;
    handlers.fetch({
      request: { url: `https://xrat.test${path}`, mode, method },
      respondWith: (promise) => (response = promise),
    });
    assert.equal(Boolean(response), intercepted, path);
    if (intercepted) assert.equal(await response, "offline document");
  }
});
