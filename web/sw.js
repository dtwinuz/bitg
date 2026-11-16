// bitg konvertor service worker: internet bo'lsa har doim tarmoqdan oladi (yangi deploy darhol ko'rinadi),
// bo'lmasa oxirgi muvaffaqiyatli javobni keshdan beradi. Faqat o'z domenidagi GET so'rovlar.
'use strict';
var CACHE = 'bitg-konvertor-v1';
var SHELL = ['/konvertor/', '/konvertor.webmanifest', '/icon-192.png'];

self.addEventListener('install', function (e) {
  e.waitUntil(caches.open(CACHE).then(function (c) { return c.addAll(SHELL); }).then(function () {
    return self.skipWaiting();
  }));
});

self.addEventListener('activate', function (e) {
  e.waitUntil(caches.keys().then(function (keys) {
    return Promise.all(keys.filter(function (k) { return k !== CACHE; }).map(function (k) { return caches.delete(k); }));
  }).then(function () { return self.clients.claim(); }));
});

self.addEventListener('fetch', function (e) {
  var req = e.request;
  if (req.method !== 'GET') return;
  var url = new URL(req.url);
  if (url.origin !== self.location.origin) return;
  var key = req.mode === 'navigate' ? url.pathname : req;
  e.respondWith(fetch(req).then(function (res) {
    if (res.ok && res.type === 'basic') {
      var copy = res.clone();
      caches.open(CACHE).then(function (c) { c.put(key, copy); });
    }
    return res;
  }).catch(function () {
    return caches.match(key).then(function (hit) { return hit || caches.match('/konvertor/'); });
  }));
});
