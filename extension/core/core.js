// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// bitg yadrosi (Rust → WASM, crates/bitg-core/src/wasm.rs) — bitg.uz sinov maydoni bilan bir xil fayl va ABI
// (web/app.js `core()`): kirish `bitg_alloc` xotirasiga UTF-8 yoziladi, `bitg_run` JSON uzunligini qaytaradi,
// `bitg_out` — natija manzili. O'girish qoidalari JS'da qayta yozilmagan: hammasi yadroda.
(function (root) {
  'use strict';

  /** `bitg_run` ning `mode`: 0 — yangi alifboga (`{out, toks}`). */
  var MODE_NEW = 0;

  function wrap(x) {
    var enc = new TextEncoder(), dec = new TextDecoder();
    function run(text, mode, flags) {
      var b = enc.encode(text);
      var ptr = x.bitg_alloc(b.length);
      new Uint8Array(x.memory.buffer, ptr, b.length).set(b);
      var n = x.bitg_run(ptr, b.length, mode, flags || 0);
      x.bitg_dealloc(ptr, b.length);
      if (!n) return null;
      // xotira bitg_run ichida o'sgan bo'lishi mumkin: ko'rinish chaqiruvdan keyin olinadi
      return JSON.parse(dec.decode(new Uint8Array(x.memory.buffer, x.bitg_out(), n)));
    }
    return {
      /** `{out, toks}`: toks — `[so'z, sabab, (ikkinchi o'qilish)]`, sabab c/u/p/x/e/r. */
      toNew: function (text, flags) {
        if (!text) return { out: text || '', toks: [] };
        return run(String(text), MODE_NEW, flags) || { out: text, toks: [] };
      },
      convert: function (text, flags) {
        if (!text) return text || '';
        var r = run(String(text), MODE_NEW, flags);
        return r ? r.out : text;
      }
    };
  }

  var ready = null;
  /** `source` — wasm manzili (fetch) yoki baytlari. Bir marta yuklanadi; xato bo'lsa keyingi chaqiruv qayta urinadi. */
  function load(source) {
    if (ready) return ready;
    ready = Promise.resolve(source)
      .then(function (s) {
        if (typeof s !== 'string') return s;
        return fetch(s).then(function (r) {
          if (!r.ok) throw new Error('bitg_core.wasm ' + r.status);
          return r.arrayBuffer();
        });
      })
      .then(function (bytes) { return WebAssembly.instantiate(bytes, {}); })
      .then(function (res) { return wrap(res.instance.exports); });
    ready.catch(function () { ready = null; });
    return ready;
  }

  var api = { load: load, wrap: wrap };
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.bitgCore = api;
})(typeof globalThis !== 'undefined' ? globalThis : this);
