// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Brauzer demosi: haqiqiy sozlamalar oynasi (crates/bitg-settings/src/ui.html) + shu fayl. Oyna Rust bilan
// `window.ipc.postMessage` / `bitg.recv` orqali gaplashadi (main.rs `handle`); bu yerda o'sha buyruqlarga xotirada
// javob beriladi, o'girishni esa haqiqiy yadro (bitg_core.wasm) qiladi. Hech narsa diskka yoki tarmoqqa
// yozilmaydi: sahifa qayta yuklansa hammasi boshidan.
(function () {
  "use strict";
  var D = window.__BITG_DEMO__ || {};
  var NOW = Date.now();
  var SEC = Math.floor(NOW / 1000);
  var DAY = 86400;

  // Demo har ochilishda toza holatda boshlansin: oynaning o'z kalitlari (bitg.*) tozalanadi, sahifa manzildan.
  try {
    Object.keys(localStorage).forEach(function (k) { if (k.indexOf("bitg.") === 0) localStorage.removeItem(k); });
    var h = (location.hash || "").replace(/^#\/?/, "");
    if (h) localStorage.setItem("bitg.page", h);
  } catch (e) { /* xotira yopiq: oyna baribir ochiladi */ }

  // ---------- yadro (Rust -> WASM), web/app.js dagi bilan bir xil ABI ----------
  var api = null;
  var ready = WebAssembly.instantiateStreaming
    ? WebAssembly.instantiateStreaming(fetch("bitg_core.wasm"), {}).catch(slow)
    : slow();
  function slow() {
    return fetch("bitg_core.wasm").then(function (r) { return r.arrayBuffer(); })
      .then(function (b) { return WebAssembly.instantiate(b, {}); });
  }
  ready = ready.then(function (res) {
    var x = res.instance.exports, enc = new TextEncoder(), dec = new TextDecoder();
    api = function (text, mode, flags) {
      var b = enc.encode(text);
      var p = x.bitg_alloc(b.length);
      new Uint8Array(x.memory.buffer, p, b.length).set(b);
      var n = x.bitg_run(p, b.length, mode, flags || 0);
      x.bitg_dealloc(p, b.length);
      if (!n) return { out: text, toks: [] };
      return JSON.parse(dec.decode(new Uint8Array(x.memory.buffer, x.bitg_out(), n)));
    };
  });

  // ---------- namuna holat ----------
  var VERSION = D.version || "0.5.0";
  var settings = {
    enabled: true, english_guard: true, cyrillic: true, restore_apostrophes: true, tutuq: "\u02BC",
    fallback: true, undo_backspace: true, undo_remember: true, disabled_apps: (D.default_apps || []).slice(),
    auto_convert: true, keymap: false, layout: "tilmoch", hotkey: true, hotkey_keys: "ctrl+shift+y",
    hotkey_old_keys: "ctrl+shift+o", update_check: true, whats_new: true, last_seen: VERSION, update_check_locked: false,
  };
  settings.disabled_apps.push("termius.exe", "windowsterminal.exe", "putty.exe");
  // Lug'at: ismlar, brendlar, atamalar (o'girilmaydi) va o'z yozuvi bo'lgan so'zlar.
  var exceptions = ["Shohruh", "Shahzoda", "Sharof", "Shirin", "Shoira", "Ashraf", "Shamsiddin", "Shuhrat",
    "Ochilov", "Choriyev", "Bekchanov", "Shopify", "Chatbot", "Chevrolet", "Shell", "Bushido", "Shakespeare",
    "Chaplin", "Shanghai", "Chicago", "Sherlock", "Michelin", "Kashmir", "Chopin"];
  var rules = [
    { from: "shamollash", to: "şamollaş" }, { from: "Ishonch", to: "Işonç" }, { from: "choyxona", to: "çoyxona" },
    { from: "kashshof", to: "kaşşof" }, { from: "Shosh", to: "Şoş" }, { from: "chuchvara", to: "çuçvara" },
    { from: "shaharcha", to: "şaharça" }, { from: "Chashma", to: "Çaşma" }, { from: "shoshilinch", to: "şoşilinç" },
    { from: "bog'cha", to: "bog'ça" }, { from: "mash'al", to: "maşʼal" }, { from: "Shoshqin", to: "Şoşqin" },
  ];
  var layouts = JSON.parse(JSON.stringify(D.layouts || {}));
  var status = { kind: "status", ime: true, keyboard: true, autostart: true,
    hotkey: { on: true, label: "Ctrl+Shift+Y", valid: true, ok: true }, version: VERSION };

  // 14 kunlik faollik va dasturlar (IME log'idagi kabi: faqat son, matn yo'q). Sonlar barqaror: har so'rovda bir xil.
  var DAYS = [212, 388, 451, 0, 96, 534, 610, 487, 402, 0, 173, 566, 702, 0];
  var APPS = [
    ["Telegram.exe", 3184, 0.02], ["chrome.exe", 2471, 0.08], ["WINWORD.EXE", 1906, 0.4], ["OUTLOOK.EXE", 842, 1.2],
    ["Code.exe", 611, 0.6], ["msedge.exe", 538, 2.1], ["EXCEL.EXE", 407, 3.3], ["POWERPNT.EXE", 233, 4.4],
    ["notepad.exe", 156, 6.2], ["WhatsApp.exe", 482, 0.3], ["Slack.exe", 97, 8.5], ["firefox.exe", 64, 12.1],
    ["Discord.exe", 41, 15.4], ["explorer.exe", 18, 2.7], ["SearchHost.exe", 9, 5.1], ["Termius.exe", 0, 1.8],
  ];
  var startedAt = NOW;
  function stats() {
    var days = DAYS.slice();
    // bugungi son: ochilgandan beri sekin o'sadi (oynaning 30 soniyalik yangilanishi ko'rinsin)
    var hour = new Date().getHours();
    days[13] = Math.round(48 + hour * 24 + (Date.now() - startedAt) / 30000);
    var apps = APPS.map(function (a) { return { exe: a[0], n: a[1], last: Math.round(SEC - a[2] * DAY - 600) }; });
    var total = days.reduce(function (s, n) { return s + n; }, 0) + 6482;
    return { kind: "stats", days: days, today: days[13], week: days.slice(7).reduce(function (s, n) { return s + n; }, 0),
      total: total, since: SEC - 47 * DAY, apps: apps };
  }

  // ---------- yordamchilar (logic.rs bilan bir xil ma'no) ----------
  function flags(st) {
    st = st || settings;
    var f = 0;
    if (st.english_guard === false) f |= 1;
    if (st.cyrillic === false) f |= 2;
    if (st.tutuq === "'") f |= 4;
    if (st.restore_apostrophes === false) f |= 8;
    return f;
  }
  function tutuq(out, st) { return st && st.tutuq === "\u2019" ? out.replace(/\u02BC/g, "\u2019") : out; }
  var fold = function (w) { return w.toLowerCase().replace(/[\u2018\u2019\u02BB\u02BC`\u00B4]/g, "'"); };
  var core = function (w) { return w.replace(/^[^\p{L}]+|[^\p{L}'\u2018\u2019\u02BB\u02BC]+$/gu, ""); };
  function chunks(s) { return s.match(/\s+|\S+/g) || []; }
  function caseLike(to, w) {
    if (w === w.toUpperCase() && w.length > 1) return to.toUpperCase();
    if (w[0] === w[0].toUpperCase()) return to[0].toUpperCase() + to.slice(1);
    return to;
  }
  // Foydalanuvchi lug'ati: so'z (yoki uning qo'shimchali shakli, 4+ harf) — o'z yozuvi bilan.
  function applyUser(seg, ex, rl, st) {
    var w = core(seg.s);
    if (!w) return seg;
    var fw = fold(w);
    if ((ex || []).some(function (e) { return fold(e) === fw; })) return { s: seg.s, d: seg.s, r: "exception" };
    for (var i = 0; i < (rl || []).length; i++) {
      var r = rl[i], ff = fold(r.from);
      if (fw === ff || (ff.length >= 4 && fw.indexOf(ff) === 0)) {
        var at = seg.s.indexOf(w), rest = w.slice(r.from.length);
        var restNew = rest ? tutuq(api(rest, 0, flags(st)).out, st) : "";
        var word = caseLike(r.to, w) + restNew;
        return { s: seg.s, d: seg.s.slice(0, at) + word + seg.s.slice(at + w.length), r: "rule" };
      }
    }
    return seg;
  }
  var REASON = { c: "converted", u: "unchanged", p: "protected", x: "exception", e: "english", r: "rule" };
  function preview(m) {
    var text = m.text || "", st = m.settings || settings;
    var res = api(text, 0, flags(st));
    var out = tutuq(res.out, st);
    var a = chunks(text), b = chunks(out), toks = res.toks || [], t = 0, segs = [];
    if (a.length !== b.length) segs.push({ s: text, d: out, r: null });
    else for (var i = 0; i < a.length; i++) {
      var s = a[i];
      if (!s.trim()) { segs.push({ s: s, d: b[i], r: null }); continue; }
      var pos = 0, best = null;
      while (t < toks.length) {
        var k = s.indexOf(toks[t][0], pos);
        if (k < 0) break;
        pos = k + toks[t][0].length;
        var r = REASON[toks[t][1]];
        best = best === "converted" || r === "converted" ? "converted" : (!best || best === "unchanged") ? r : best;
        t++;
      }
      segs.push(applyUser({ s: s, d: b[i], r: best }, m.exceptions, m.rules, st));
    }
    return { kind: "preview", id: m.id, out: segs.map(function (g) { return g.d; }).join(""), segments: segs };
  }
  function convert(m) {
    var text = m.text || "", out;
    if (m.dir === "old") out = api(text, m.style === "ascii" ? 2 : 1, 0).out;
    else {
      var st = Object.assign({}, m.settings || settings, { cyrillic: true });
      out = preview({ text: text, settings: st, exceptions: m.exceptions, rules: m.rules }).out;
    }
    var a = chunks(text), b = chunks(out), changed = 0;
    for (var i = 0; i < Math.min(a.length, b.length); i++) if (a[i] !== b[i]) changed++;
    return { kind: "converted", id: m.id, out: out, changed: changed };
  }

  // ---------- javob berish ----------
  function reply(v, delay) {
    setTimeout(function () { if (window.bitg) window.bitg.recv(v); }, delay || 0);
  }
  function note(text) {
    var el = document.getElementById("demoToast");
    if (!el) return;
    el.textContent = text; el.classList.add("on");
    clearTimeout(note.t); note.t = setTimeout(function () { el.classList.remove("on"); }, 2600);
  }
  function loaded() {
    return { kind: "loaded", settings: JSON.parse(JSON.stringify(settings)), layouts: JSON.parse(JSON.stringify(layouts)),
      exceptions: exceptions.slice(), rules: rules.map(function (r) { return { from: r.from, to: r.to }; }),
      default_apps: (D.default_apps || []).slice(), folder: "C:\\Users\\demo\\AppData\\Roaming\\bitg" };
  }
  function diag() {
    return { os: "Windows 11 26100", enabled: settings.enabled, auto_convert: settings.auto_convert,
      english_guard: settings.english_guard, cyrillic: settings.cyrillic, restore_apostrophes: settings.restore_apostrophes,
      fallback: settings.fallback, keymap: settings.keymap, layout: settings.layout,
      tutuq: "U+" + settings.tutuq.charCodeAt(0).toString(16).toUpperCase().padStart(4, "0"),
      dictionary: { rules: rules.length, words: exceptions.length }, ime: status.ime, keyboard: status.keyboard };
  }
  var LINKS = /^https:\/\/(bitg\.uz|github\.com\/dtwinuz)\//;
  function shot() {
    var c = document.createElement("canvas"); c.width = 960; c.height = 540;
    var g = c.getContext("2d");
    g.fillStyle = "#f3f4f1"; g.fillRect(0, 0, 960, 540);
    g.fillStyle = "#ffffff"; g.fillRect(60, 60, 840, 420);
    g.fillStyle = "#2a4bb0"; g.fillRect(60, 60, 840, 44);
    g.fillStyle = "#ffffff"; g.font = "600 18px Segoe UI, sans-serif"; g.fillText("Telegram \u2014 Ish guruhi", 80, 89);
    g.fillStyle = "#18212d"; g.font = "22px Segoe UI, sans-serif";
    g.fillText("Ertaga soat uçda yiğilamiz, keyin şaharga çiqamiz.", 90, 170);
    g.fillText("Please check the schedule \u2014 Toşkent, Çilonzor.", 90, 214);
    g.fillStyle = "#b3261e"; g.fillText("Kutilgan: \u00abÇilonzor\u00bb, chiqdi: \u00abChilonzor\u00bb?", 90, 280);
    g.strokeStyle = "#b3261e"; g.lineWidth = 3; g.strokeRect(84, 248, 520, 44);
    return c.toDataURL("image/png");
  }

  function handle(m) {
    switch (m.cmd) {
      case "load": reply(loaded()); reply(JSON.parse(JSON.stringify(status)), 5); break;
      case "status": reply(JSON.parse(JSON.stringify(status))); break;
      case "save":
        if (m.settings) { settings = Object.assign({}, settings, m.settings); }
        if (m.exceptions) exceptions = m.exceptions.slice();
        if (m.rules) rules = m.rules.slice();
        if (m.custom) layouts.custom = m.custom;
        status.hotkey.on = settings.enabled && settings.hotkey;
        reply({ kind: "saved" }, 120);
        break;
      case "preview": reply(preview(m)); break;
      case "convert": reply(convert(m)); break;
      case "stats": reply(stats()); break;
      case "news": reply(Object.assign({ kind: "news", current: VERSION, show: false }, { versions: D.news || [] })); break;
      case "news_seen": break;
      case "check_update":
        reply({ kind: "update", state: "checking", current: VERSION });
        reply({ kind: "update", state: "latest", current: VERSION }, 1100);
        break;
      case "install_update": note("Demo: o'rnatish faqat haqiqiy ilovada (bitg.uz/yuklab-olish)"); break;
      case "feedback":
        var ok = m.type === "conversion" ? (m.typed || "").trim() && (m.got || "").trim() : (m.text || "").trim().length >= 3;
        if (!m.type) reply({ kind: "feedback_result", ok: false, error: "Xabar turini tanlang." }, 400);
        else if (!ok) reply({ kind: "feedback_result", ok: false, error: m.type === "conversion"
          ? "Nima yozganingizni va bitg nima chiqarganini kiriting." : "Bir-ikki gap yozing." }, 400);
        else reply({ kind: "feedback_result", ok: true, ticket: "DM-" + Math.random().toString(36).slice(2, 8).toUpperCase() }, 900);
        break;
      case "feedback_diag": reply({ kind: "feedback_diag", diag: diag() }); break;
      case "screenshot": reply({ kind: "screenshot", image: shot() }, 600); break;
      case "autostart": status.autostart = !!m.on; reply(JSON.parse(JSON.stringify(status)), 150); break;
      case "add_keyboard": status.keyboard = true; reply(JSON.parse(JSON.stringify(status)), 300); break;
      case "open_url": if (LINKS.test(m.url || "")) window.open(m.url, "_blank", "noopener"); break;
      case "open_language_settings": note("Demo: Windows til sozlamalari bu yerda ochilmaydi"); break;
      case "open_folder": note("Demo: sozlamalar papkasi (%APPDATA%\\bitg) brauzerda ochilmaydi"); break;
      case "quit": note("Demo: ilovadan chiqish o'rniga sahifani yoping"); break;
    }
  }
  var queue = [];
  window.ipc = {
    postMessage: function (body) {
      var m;
      try { m = JSON.parse(body); } catch (e) { return; }
      if (api) handle(m); else queue.push(m);
    },
  };
  ready.then(function () { queue.splice(0).forEach(handle); }, function () {
    note("Yadro (bitg_core.wasm) yuklanmadi");
  });

  // ---------- hash yo'llar: #/try, #/rules ... (statik hostda to'g'ridan-to'g'ri havola) ----------
  var PAGES = ["home", "try", "text", "rules", "kb", "apps", "ex", "news", "fb", "about"];
  function syncHash() {
    var cur = document.querySelector("nav button[aria-current]");
    if (cur && location.hash !== "#/" + cur.dataset.page) history.replaceState(null, "", "#/" + cur.dataset.page);
  }
  document.addEventListener("click", function () { setTimeout(syncHash, 0); });
  document.addEventListener("keyup", function () { setTimeout(syncHash, 0); });
  window.addEventListener("hashchange", function () {
    var p = location.hash.replace(/^#\/?/, "");
    var b = PAGES.indexOf(p) >= 0 && document.querySelector('nav button[data-page="' + p + '"]');
    if (b && !b.hasAttribute("aria-current")) b.click();
  });

  // ---------- namuna matnlar: Sinash va Matn sahifalari bo'sh turmasin ----------
  var TRY = "Assalomu alaykum! Ertaga soat uchda choyxonada yig'ilamiz, keyin shaharga chiqamiz. " +
    "Please check the report va menga chat orqali yozing: https://bitg.uz, @shohrux_dev. ozbek tili, togri yol. Shohruh ham keladi.";
  var TEXT = "O'zbekiston Respublikasi Prezidentining qarori bilan Toshkent shahrida yangi bog'cha va maktablar ochildi.\n" +
    "Shahar hokimligi ma'lumotiga ko'ra, Chilonzor va Yunusobod tumanlarida 14 ta bino qurilgan.\n" +
    "Тошкент шаҳрида ҳаво ҳарорати +18 даражагача кўтарилади.\n" +
    "Meeting notes: deadline is Friday, keyin hisobotni Shahzoda opaga yuboramiz.";
  function fill() {
    var i = document.getElementById("input"), c = document.getElementById("convIn");
    if (i) { i.value = TRY; i.dispatchEvent(new Event("input")); }
    if (c) { c.value = TEXT; c.dispatchEvent(new Event("input")); }
    syncHash();
  }
  ready.then(function () { setTimeout(fill, 30); });
  window.__bitgDemo = { ready: ready, note: note };
})();
