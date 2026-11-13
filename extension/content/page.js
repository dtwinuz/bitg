// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// "Sahifani yangi alifboda ko'rish": ko'rinadigan matn tugunlari yadro bilan o'giriladi, asli xotirada qoladi.
// Qayta bosilsa — asli tiklanadi (o'shandan beri sahifa o'zi o'zgartirgan tugunlarga tegilmaydi).
// Maydonlar, formalar, kod, skriptlar va contenteditable chetda (bitgLib.skipElement).
(function () {
  'use strict';
  if (window.bitgPage) return;
  var L = window.bitgLib, E = window.bitgEdit;
  var BATCH = 400;
  var state = { active: false, nodes: new Map(), title: null, observer: null, pending: [], timer: 0 };

  function collect(root, out) {
    if (root.nodeType !== 1 && root.nodeType !== 3) return out;
    if (L.insideSkipped(root)) return out;
    L.textNodes(root).forEach(function (n) { if (!state.nodes.has(n)) out.push(n); });
    return out;
  }

  async function convertNodes(nodes) {
    var count = 0;
    for (var i = 0; i < nodes.length; i += BATCH) {
      var chunk = nodes.slice(i, i + BATCH);
      var src = chunk.map(function (n) { return n.data; });
      var outs = await E.convertMany(src);
      if (!Array.isArray(outs) || !state.active) return count;
      chunk.forEach(function (n, j) {
        var out = outs[j];
        if (typeof out !== 'string' || out === src[j] || n.data !== src[j]) return;
        state.nodes.set(n, { orig: src[j], conv: out });
        n.data = out;
        count++;
      });
    }
    return count;
  }

  function flush() {
    state.timer = 0;
    var added = state.pending;
    state.pending = [];
    var nodes = [];
    added.forEach(function (n) { if (n.isConnected) collect(n, nodes); });
    if (nodes.length) convertNodes(nodes);
  }

  function observe() {
    state.observer = new MutationObserver(function (records) {
      records.forEach(function (r) { r.addedNodes.forEach(function (n) { state.pending.push(n); }); });
      if (state.pending.length && !state.timer) state.timer = setTimeout(flush, 250);
    });
    // faqat yangi tugunlar: o'zimizning n.data yozishimiz (characterData) kuzatilmaydi
    state.observer.observe(document.body, { childList: true, subtree: true });
  }

  async function enable() {
    if (!document.body) return 0;
    state.active = true;
    var count = await convertNodes(collect(document.body, []));
    var t = document.title;
    if (L.hasLetters(t)) {
      var nt = await E.convert(t);
      if (typeof nt === 'string' && nt !== t && document.title === t) { state.title = { orig: t, conv: nt }; document.title = nt; }
    }
    observe();
    return count;
  }

  function disable() {
    state.active = false;
    if (state.observer) state.observer.disconnect();
    clearTimeout(state.timer);
    state.observer = null;
    state.pending = [];
    state.timer = 0;
    state.nodes.forEach(function (v, n) { if (n.data === v.conv) n.data = v.orig; });
    state.nodes.clear();
    if (state.title && document.title === state.title.conv) document.title = state.title.orig;
    state.title = null;
  }

  async function toggle(msg) {
    if (state.active) {
      disable();
      if (window.bitgToast) window.bitgToast(msg.toastPageOff);
      return false;
    }
    var n = await enable();
    if (window.bitgToast) window.bitgToast(n ? msg.toastPageOn : msg.toastPageNone);
    if (!n && !state.title) disable();
    return state.active;
  }

  window.bitgPage = { toggle: toggle, active: function () { return state.active; } };
})();
