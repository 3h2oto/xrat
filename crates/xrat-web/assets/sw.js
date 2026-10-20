// Only the public offline document is cached. Sessions, API data, subscription
// URLs and application bundles always use the network.
const CACHE = "xrat-offline-v1";
self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE).then((cache) => cache.add("/offline.html")),
  );
});
self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((key) => key.startsWith("xrat-offline-") && key !== CACHE)
            .map((key) => caches.delete(key)),
        ),
      ),
  );
});
self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);
  if (
    event.request.method !== "GET" ||
    event.request.mode !== "navigate" ||
    url.origin !== self.location.origin ||
    url.search ||
    !(url.pathname === "/" || url.pathname.startsWith("/ui/"))
  )
    return;
  event.respondWith(
    fetch(event.request).catch(() => caches.match("/offline.html")),
  );
});
