// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// bitg yadrosi Apps Script V8 ichida: bitg.uz va brauzer kengaytmasi bilan bir xil bitg_core.wasm
// (crates/bitg-core/src/wasm.rs ABI, extension/core/core.js bilan bir xil chaqiruv). wasm baytlari
// build vaqtida gzip + base64 qilib Wasm.gs ga yoziladi (tools/build.mjs) — Apps Script ikkilik fayl saqlamaydi.
// Matn hech qayerga yuborilmaydi: UrlFetchApp ishlatilmaydi, o'girish shu ijro ichida bo'ladi.
//
// Apps Script'da TextEncoder/TextDecoder yo'q — UTF-8 shu yerda qo'lda. Hujjat bilan ishlovchi qism
// (planEdits/mapPos) sof funksiya: Node testlarida (test/*.test.cjs) tekshiriladi.

/** `bitg_run` flags (crates/bitg-core/src/ffi.rs BITG_*). */
var BITG_FLAGS = {
  NO_ENGLISH_GUARD: 1,
  NO_CYRILLIC: 2,
  ASCII_TUTUQ: 4,
  NO_RESTORE: 8
};

/** Sidebar/menyu sozlamalaridan `flags`: {english, cyrillic, asciiTutuq} (yo'q bo'lsa — standart). */
function bitgFlags_(opts) {
  var o = opts || {};
  var f = 0;
  if (o.english === false) f |= BITG_FLAGS.NO_ENGLISH_GUARD;
  if (o.cyrillic === false) f |= BITG_FLAGS.NO_CYRILLIC;
  if (o.asciiTutuq === true) f |= BITG_FLAGS.ASCII_TUTUQ;
  return f;
}

function bitgUtf8Encode_(s) {
  if (typeof TextEncoder === 'function') return new TextEncoder().encode(s);
  var out = new Uint8Array(s.length * 3);
  var n = 0;
  for (var i = 0; i < s.length; i++) {
    var c = s.charCodeAt(i);
    if (c >= 0xd800 && c <= 0xdbff && i + 1 < s.length) {
      var d = s.charCodeAt(i + 1);
      if (d >= 0xdc00 && d <= 0xdfff) {
        c = 0x10000 + ((c - 0xd800) << 10) + (d - 0xdc00);
        i++;
      } else c = 0xfffd;
    } else if (c >= 0xd800 && c <= 0xdfff) c = 0xfffd; // juftsiz surrogat — UTF-8 da yo'q
    if (c < 0x80) out[n++] = c;
    else if (c < 0x800) {
      out[n++] = 0xc0 | (c >> 6);
      out[n++] = 0x80 | (c & 63);
    } else if (c < 0x10000) {
      out[n++] = 0xe0 | (c >> 12);
      out[n++] = 0x80 | ((c >> 6) & 63);
      out[n++] = 0x80 | (c & 63);
    } else {
      out[n++] = 0xf0 | (c >> 18);
      out[n++] = 0x80 | ((c >> 12) & 63);
      out[n++] = 0x80 | ((c >> 6) & 63);
      out[n++] = 0x80 | (c & 63);
    }
  }
  return out.subarray(0, n);
}

/** Yadro chiqishi doim to'g'ri UTF-8 (Rust `String`), shuning uchun tekshiruvsiz dekoder. */
function bitgUtf8Decode_(b) {
  if (typeof TextDecoder === 'function') return new TextDecoder().decode(b);
  var parts = [];
  var chunk = [];
  for (var i = 0; i < b.length; ) {
    var c = b[i++];
    if (c >= 0xf0) c = ((c & 7) << 18) | ((b[i++] & 63) << 12) | ((b[i++] & 63) << 6) | (b[i++] & 63);
    else if (c >= 0xe0) c = ((c & 15) << 12) | ((b[i++] & 63) << 6) | (b[i++] & 63);
    else if (c >= 0xc0) c = ((c & 31) << 6) | (b[i++] & 63);
    if (c >= 0x10000) {
      c -= 0x10000;
      chunk.push(0xd800 + (c >> 10), 0xdc00 + (c & 1023));
    } else chunk.push(c);
    if (chunk.length >= 8192) {
      parts.push(String.fromCharCode.apply(null, chunk));
      chunk = [];
    }
  }
  parts.push(String.fromCharCode.apply(null, chunk));
  return parts.join('');
}

/** wasm eksportlaridan core.js bilan bir xil API: {convert(text, flags), toNew(text, flags)}. */
function bitgWrap_(x) {
  function run(text, flags) {
    var b = bitgUtf8Encode_(text);
    var ptr = x.bitg_alloc(b.length);
    new Uint8Array(x.memory.buffer, ptr, b.length).set(b);
    var n = x.bitg_run(ptr, b.length, 0, flags || 0);
    x.bitg_dealloc(ptr, b.length);
    if (!n) return null;
    // xotira bitg_run ichida o'sgan bo'lishi mumkin: ko'rinish chaqiruvdan keyin olinadi
    return JSON.parse(bitgUtf8Decode_(new Uint8Array(x.memory.buffer, x.bitg_out(), n)));
  }
  return {
    toNew: function (text, flags) {
      if (!text) return { out: text || '', toks: [] };
      return run(String(text), flags) || { out: text, toks: [] };
    },
    convert: function (text, flags) {
      if (!text) return text || '';
      var r = run(String(text), flags);
      return r ? r.out : text;
    }
  };
}

/** Wasm.gs dagi gzip+base64 → wasm baytlari (Uint8Array). */
function bitgWasmBytes_() {
  var b64 = bitgWasmBase64_(); // Wasm.gs (build natijasi)
  var gz = Utilities.base64Decode(b64);
  var raw = Utilities.ungzip(Utilities.newBlob(gz, 'application/x-gzip', 'bitg_core.wasm.gz')).getBytes();
  // Apps Script baytlari ishorali (-128..127): Uint8Array ularni 0..255 ga o'giradi
  return new Uint8Array(raw);
}

var BITG_CORE_ = null;
var BITG_CORE_ERROR_ = null;

/**
 * Sinxron yuklash (`new WebAssembly.Module`) — Apps Script ijrosi sinxron, shuning uchun asosiy yo'l shu.
 * Muhit sinxron kompilyatsiyani rad etsa null qaytaradi va bitgWithCore_ asinxron yo'lga o'tadi.
 */
function bitgCoreSync_() {
  if (BITG_CORE_) return BITG_CORE_;
  try {
    var m = new WebAssembly.Module(bitgWasmBytes_());
    BITG_CORE_ = bitgWrap_(new WebAssembly.Instance(m, {}).exports);
    return BITG_CORE_;
  } catch (e) {
    BITG_CORE_ERROR_ = e;
    return null;
  }
}

/** Google hujjatlaridagi rasmiy naqsh: WebAssembly.instantiate (V8 runtime'dagi yagona asinxron API). */
function bitgCoreAsync_() {
  if (BITG_CORE_) return Promise.resolve(BITG_CORE_);
  return WebAssembly.instantiate(bitgWasmBytes_(), {}).then(function (res) {
    BITG_CORE_ = bitgWrap_(res.instance.exports);
    return BITG_CORE_;
  });
}

/** `fn(core)` ni bajaradi; sinxron yo'l ishlamasa — Promise qaytaradi. */
function bitgWithCore_(fn) {
  var core = bitgCoreSync_();
  if (core) return fn(core);
  return bitgCoreAsync_().then(fn);
}

// ---------------------------------------------------------------------------------------------------------
// Qayta yozish rejasi: asl matn `a` va o'girilgan `b` dan eng kichik almashtirishlar ro'yxati.
// Butun elementni qayta yozish o'rniga faqat o'zgargan bo'laklar almashtiriladi — qalin/kursiv, havola,
// rang va izohlar o'z joyida qoladi.

function bitgIsHigh_(c) { return c >= 0xd800 && c <= 0xdbff; }
function bitgIsLow_(c) { return c >= 0xdc00 && c <= 0xdfff; }

/** Bitta bo'lak: umumiy boshi va oxiri olib tashlanadi, o'rtasi almashtiriladi. */
function bitgDiffPiece_(x, y, base) {
  var p = 0;
  var max = Math.min(x.length, y.length);
  while (p < max && x.charCodeAt(p) === y.charCodeAt(p)) p++;
  if (p > 0 && p < x.length && bitgIsLow_(x.charCodeAt(p))) p--; // surrogat juftini bo'lmaslik
  var s = 0;
  var maxS = Math.min(x.length, y.length) - p;
  while (s < maxS && x.charCodeAt(x.length - 1 - s) === y.charCodeAt(y.length - 1 - s)) s++;
  if (s > 0 && s < x.length - p && bitgIsHigh_(x.charCodeAt(x.length - 1 - s))) s--;
  return { start: base + p, end: base + x.length - s, text: y.slice(p, y.length - s) };
}

/**
 * `a` → `b` uchun almashtirishlar: [{start, end, text}] (a koordinatalarida, o'sish tartibida, kesishmaydi).
 * Bo'sh joylar mos kelsa — so'zma-so'z (formatlash har so'zda saqlanadi); bo'lmasa — butun satr bitta bo'lak.
 */
function planEdits_(a, b) {
  if (a === b) return [];
  var ta = a.split(/(\s+)/);
  var tb = b.split(/(\s+)/);
  var aligned = ta.length === tb.length;
  for (var i = 1; aligned && i < ta.length; i += 2) if (ta[i] !== tb[i]) aligned = false;
  if (!aligned) return [bitgDiffPiece_(a, b, 0)];
  var edits = [];
  var pos = 0;
  for (var j = 0; j < ta.length; j++) {
    if (ta[j] !== tb[j]) edits.push(bitgDiffPiece_(ta[j], tb[j], pos));
    pos += ta[j].length;
  }
  return edits;
}

/** Almashtirishlarni satrga qo'llaydi (test va Sheets uchun). */
function applyEdits_(a, edits) {
  var out = a;
  for (var i = edits.length - 1; i >= 0; i--) {
    var e = edits[i];
    out = out.slice(0, e.start) + e.text + out.slice(e.end);
  }
  return out;
}

/** `a` dagi pozitsiya → natijadagi pozitsiya (formatlash bo'laklarini ko'chirish uchun). */
function mapPos_(edits, p) {
  var shift = 0;
  for (var i = 0; i < edits.length; i++) {
    var e = edits[i];
    if (p < e.start) break;
    if (p >= e.end && !(p === e.start && e.start === e.end)) {
      shift += e.text.length - (e.end - e.start);
      continue;
    }
    return e.start + shift + Math.min(p - e.start, e.text.length);
  }
  return p + shift;
}
