// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Apps Script'ni Node'da taqlid qilish: dist/*.gs fayllari bitta vm kontekstida (Apps Script'dagidek umumiy
// global) yuklanadi. Utilities — base64Decode/newBlob/ungzip ning ishorali baytlar bilan nusxasi.
// TextEncoder/TextDecoder ataylab YO'Q (Apps Script'da ham yo'q) — Engine.gs ning qo'lda UTF-8 yo'li sinaladi.
'use strict';
const fs = require('fs');
const path = require('path');
const vm = require('vm');
const zlib = require('zlib');

const HERE = path.resolve(__dirname, '..');
const DIST = process.env.BITG_GAS_DIST || path.join(HERE, 'dist');
const haveDist = fs.existsSync(path.join(DIST, 'Wasm.gs'));

/** Java byte[] → JS: Apps Script baytlari -128..127. */
function signed(buf) {
  return Array.from(buf, (b) => (b > 127 ? b - 256 : b));
}

function fakeUtilities() {
  const blob = (bytes) => ({ getBytes: () => bytes, bytes });
  return {
    base64Decode: (s) => signed(Buffer.from(s, 'base64')),
    newBlob: (bytes) => blob(bytes),
    ungzip: (b) => blob(signed(zlib.gunzipSync(Buffer.from(b.bytes.map((x) => x & 255)))))
  };
}

/** Kontekst: {Utilities, console, WebAssembly, ...extra} + barcha .gs fayllar. */
function load(extra) {
  const ctx = Object.assign({
    Utilities: fakeUtilities(),
    console,
    WebAssembly,
    Promise,
    JSON,
    Math,
    Date,
    Uint8Array,
    String,
    Object,
    Array
  }, extra || {});
  vm.createContext(ctx);
  const files = ['Wasm.gs', 'Engine.gs', 'Code.gs'];
  for (const f of files) vm.runInContext(fs.readFileSync(path.join(DIST, f), 'utf8'), ctx, { filename: f });
  return ctx;
}

/** Faqat Engine.gs/Code.gs (wasm'siz) — sof funksiyalar uchun. */
function loadPure(extra) {
  const ctx = Object.assign({ console, JSON, Math, String, Uint8Array, Object, Array }, extra || {});
  vm.createContext(ctx);
  for (const f of ['Engine.gs', 'Code.gs']) {
    vm.runInContext(fs.readFileSync(path.join(HERE, 'src', f), 'utf8'), ctx, { filename: f });
  }
  return ctx;
}

module.exports = { load, loadPure, haveDist, DIST, HERE };
