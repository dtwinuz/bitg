// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// bitg.uz — demolar, 3D klaviatura, WASM sinov maydoni, sahifa alifbosi.
// Demo natijalari bitg yadrosi (WASM) chiqishidan olingan: to'qima o'girish yo'q.
(function () {
  'use strict';

  var doc = document, root = doc.documentElement;
  var $ = function (s, r) { return (r || doc).querySelector(s); };
  var $$ = function (s, r) { return Array.prototype.slice.call((r || doc).querySelectorAll(s)); };
  var RM = matchMedia('(prefers-reduced-motion: reduce)');
  var reduced = function () { return RM.matches; };
  var sleep = function (ms) { return new Promise(function (r) { setTimeout(r, ms); }); };
  var EASE = 'cubic-bezier(.22,.8,.24,1)';
  var SOFT = 'cubic-bezier(.33,1,.68,1)';
  function el(tag, cls, text) {
    var e = doc.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  }
  function svg(path) { return '<svg viewBox="0 0 24 24" aria-hidden="true">' + path + '</svg>'; }
  var ICON_LOCK = '<rect x="5" y="10.5" width="14" height="10" rx="2"/><path d="M8 10.5V8a4 4 0 0 1 8 0v2.5"/>';

  // ---------- ko'rinish: animatsiya faqat ekranda va ochiq tabda ----------
  var vis = new WeakMap(), waiters = new WeakMap(), watched = [];
  var io = new IntersectionObserver(function (es) {
    es.forEach(function (e) {
      vis.set(e.target, e.isIntersecting);
      if (e.isIntersecting) flush(e.target);
    });
  }, { rootMargin: '0px 0px -8% 0px' });
  function flush(t) {
    var w = waiters.get(t);
    if (w && !doc.hidden) { waiters.delete(t); w.forEach(function (f) { f(); }); }
  }
  doc.addEventListener('visibilitychange', function () {
    if (!doc.hidden) watched.forEach(function (t) { if (vis.get(t)) flush(t); });
  });
  function watch(t) { watched.push(t); io.observe(t); }
  function seen(t) {
    if (vis.get(t) && !doc.hidden) return Promise.resolve();
    return new Promise(function (r) {
      var w = waiters.get(t);
      if (!w) { w = []; waiters.set(t, w); }
      w.push(r);
    });
  }

  // ---------- bitg yadrosi (Rust → WASM) ----------
  var corePromise = null;
  function core() {
    if (corePromise) return corePromise;
    var v = root.getAttribute('data-v');
    var url = 'bitg_core.wasm' + (v ? '?v=' + v : '');
    function slow() {
      return fetch(url).then(function (r) {
        if (!r.ok) throw new Error('wasm ' + r.status);
        return r.arrayBuffer();
      }).then(function (b) { return WebAssembly.instantiate(b, {}); });
    }
    var p = WebAssembly.instantiateStreaming
      ? WebAssembly.instantiateStreaming(fetch(url), {}).catch(slow)
      : slow();
    corePromise = p.then(function (res) {
      var x = res.instance.exports, enc = new TextEncoder(), dec = new TextDecoder();
      function run(text, mode, flags) {
        var b = enc.encode(text);
        var ptr = x.bitg_alloc(b.length);
        new Uint8Array(x.memory.buffer, ptr, b.length).set(b);
        var n = x.bitg_run(ptr, b.length, mode, flags || 0);
        x.bitg_dealloc(ptr, b.length);
        if (!n) return null;
        return JSON.parse(dec.decode(new Uint8Array(x.memory.buffer, x.bitg_out(), n)));
      }
      return {
        toNew: function (t, flags) { return run(t, 0, flags) || { out: t, toks: [] }; },
        toOld: function (t) { var r = run(t, 1, 0); return r ? r.out : t; }
      };
    });
    corePromise.catch(function () { corePromise = null; });
    return corePromise;
  }
  function idle(f) { (window.requestIdleCallback || function (c) { setTimeout(c, 400); })(f); }

  // ---------- sahifa alifbosi: Yangi / Eski ----------
  var abc = root.getAttribute('data-abc') === 'new' ? 'new' : 'old';
  var origText = new Map(), coreApi = null, origTitle = doc.title;
  function T(s) { return abc === 'new' && coreApi ? coreApi.toNew(s).out : s; }
  function textNodes() {
    var out = [], w = doc.createTreeWalker(doc.body, NodeFilter.SHOW_TEXT, {
      acceptNode: function (n) {
        if (!n.data.trim()) return NodeFilter.FILTER_REJECT;
        var p = n.parentElement;
        var c = p && p.closest('[data-conv],[data-no-conv],script,style,kbd,code,svg');
        if (!p || (c && !c.hasAttribute('data-conv'))) return NodeFilter.FILTER_REJECT;
        return NodeFilter.FILTER_ACCEPT;
      }
    });
    while (w.nextNode()) out.push(w.currentNode);
    return out;
  }
  function applyAbc(animate) {
    textNodes().forEach(function (n) {
      if (!origText.has(n)) origText.set(n, n.data);
      var o = origText.get(n);
      n.data = T(o);
    });
    doc.title = T(origTitle);
    if (hint && hintHtml) hint.innerHTML = T(hintHtml);
    if (animate && !reduced()) {
      var m = $('main');
      m.classList.remove('bg-abc-swap'); void m.offsetWidth; m.classList.add('bg-abc-swap');
    }
  }
  function setAbc(next, animate) {
    abc = next;
    root.setAttribute('data-abc', next);
    try { localStorage.setItem('bitg-abc', next); } catch (e) {}
    $$('button[data-abc]').forEach(function (b) {
      b.setAttribute('aria-pressed', String(b.getAttribute('data-abc') === next));
    });
    if (next === 'old' && !coreApi) return;
    core().then(function (c) { coreApi = c; applyAbc(animate); }, function () {});
  }
  $$('button[data-abc]').forEach(function (b) {
    b.addEventListener('click', function () {
      var n = b.getAttribute('data-abc');
      if (n !== abc) setAbc(n, true);
    });
  });

  // ---------- mavzu ----------
  $('#bg-theme').addEventListener('click', function () {
    var cur = root.getAttribute('data-theme') ||
      (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
    var next = cur === 'dark' ? 'light' : 'dark';
    root.setAttribute('data-theme', next);
    try { localStorage.setItem('bitg-theme', next); } catch (e) {}
  });

  // ---------- sarlavha chizig'i ----------
  var head = $('#bg-head');
  function onScroll() { head.classList.toggle('is-stuck', window.scrollY > 4); }
  window.addEventListener('scroll', onScroll, { passive: true });
  onScroll();

  // ---------- harflar birlashishi ----------
  var APOS = "'‘’ʻ`";
  var PAIRS = { sh: 'ş', Sh: 'Ş', SH: 'Ş', ch: 'ç', Ch: 'Ç', CH: 'Ç', "o'": 'ö', "O'": 'Ö', "g'": 'ğ', "G'": 'Ğ' };
  function norm2(a, b) { return a + (APOS.indexOf(b) >= 0 ? "'" : b); }
  // eski so'z → yangi so'z: harfma-harf moslash (birlashgan juftlik, almashgan tutuq, oddiy harf)
  function align(o, n) {
    var segs = [], i = 0, j = 0;
    while (i < o.length) {
      var pair = i + 1 < o.length ? PAIRS[norm2(o[i], o[i + 1])] : null;
      if (pair && n[j] === pair) { segs.push({ t: 'f', a: o[i], b: o[i + 1], n: pair }); i += 2; j++; continue; }
      if (APOS.indexOf(o[i]) >= 0 && n[j] === 'ʼ') { segs.push({ t: 's', a: o[i], n: 'ʼ' }); i++; j++; continue; }
      if (o[i] === n[j]) { segs.push({ t: 'p', c: o[i] }); i++; j++; continue; }
      return null;
    }
    return j === n.length ? segs : null;
  }
  function ring(host) {
    if (reduced()) return;
    var r = el('span', 'bg-ring');
    host.appendChild(r);
    r.animate([{ transform: 'scale(.35)', opacity: 0.9 }, { transform: 'scale(1.7)', opacity: 0 }],
      { duration: 720, easing: SOFT }).finished.then(function () { r.remove(); }, function () { r.remove(); });
  }
  // .bg-g ichida a+b → n: b a ga singadi, a yangi harfga aylanadi
  function fuse(g, n, dur) {
    var a = g.firstChild, b = a.nextSibling;
    dur = dur || 440;
    if (reduced()) {
      if (b) b.remove();
      a.textContent = n; g.classList.add('is-new');
      return Promise.resolve();
    }
    var p;
    if (b) {
      var bw = b.getBoundingClientRect().width, aw = a.getBoundingClientRect().width;
      g.classList.add('is-fusing');
      g.animate([{ marginRight: bw + 'px' }, { marginRight: '0px' }], { duration: dur, easing: SOFT, fill: 'forwards' });
      a.animate([{ transform: 'none' }, { transform: 'scale(1.06,.94)' }], { duration: dur * 0.8, easing: 'ease-in' });
      p = b.animate([
        { transform: 'none', opacity: 1 },
        { transform: 'translateX(' + (-aw * 0.75) + 'px) scale(.55)', opacity: 0 }
      ], { duration: dur, easing: 'cubic-bezier(.55,0,.7,.4)', fill: 'forwards' }).finished;
    } else {
      p = a.animate([{ transform: 'none', opacity: 1 }, { transform: 'rotateX(90deg)', opacity: 0.1 }],
        { duration: dur * 0.5, easing: 'ease-in', fill: 'forwards' }).finished;
    }
    return p.then(function () {
      if (b && b.parentNode === g) b.remove();
      g.classList.remove('is-fusing');
      g.getAnimations().forEach(function (x) { x.cancel(); });
      a.getAnimations().forEach(function (x) { x.cancel(); });
      a.textContent = n;
      g.classList.add('is-new');
      a.animate([
        { transform: 'translateY(-28%) scale(1.3)', opacity: 0 },
        { transform: 'translateY(4%) scale(.96)', opacity: 1, offset: 0.62 },
        { transform: 'none', opacity: 1 }
      ], { duration: 520, easing: EASE });
      ring(g);
    }, function () {});
  }
  function gEl(a, b) {
    var g = el('span', 'bg-g');
    g.appendChild(el('span', 'bg-g__a', a));
    if (b != null) g.appendChild(el('span', 'bg-g__b', b));
    return g;
  }
  // so'z span'ini yangi shaklga o'tkazadi; moslab bo'lmasa — butun so'z almashadi
  function convertWord(w, nw) {
    var o = w.textContent;
    if (!nw || nw === o) return Promise.resolve(false);
    var segs = align(o, nw);
    w.textContent = '';
    if (!segs) {
      var whole = gEl(nw);
      whole.classList.add('is-new');
      w.appendChild(whole);
      return Promise.resolve(true);
    }
    var jobs = [];
    segs.forEach(function (s, k) {
      if (s.t === 'p') { w.appendChild(doc.createTextNode(s.c)); return; }
      var g = gEl(s.a, s.t === 'f' ? s.b : null);
      w.appendChild(g);
      jobs.push(sleep(k * 30).then(function () { return fuse(g, s.n); }));
    });
    return Promise.all(jobs).then(function () { return true; });
  }
  function revertWord(w, o) {
    w.textContent = o;
    if (!reduced()) {
      w.animate([{ transform: 'translateY(30%)', opacity: 0 }, { transform: 'none', opacity: 1 }], { duration: 360, easing: EASE });
    }
  }

  // ---------- klaviatura ----------
  var ROWS = [
    [['Backquote', '`'], ['Digit1', '1'], ['Digit2', '2'], ['Digit3', '3'], ['Digit4', '4'], ['Digit5', '5'], ['Digit6', '6'], ['Digit7', '7'], ['Digit8', '8'], ['Digit9', '9'], ['Digit0', '0'], ['Minus', '-'], ['Equal', '='], ['Backspace', '⌫', 2]],
    [['Tab', 'Tab', 1.5], ['KeyQ', 'q'], ['KeyW', 'w'], ['KeyE', 'e'], ['KeyR', 'r'], ['KeyT', 't'], ['KeyY', 'y'], ['KeyU', 'u'], ['KeyI', 'i'], ['KeyO', 'o'], ['KeyP', 'p'], ['BracketLeft', '['], ['BracketRight', ']'], ['Backslash', '\\', 1.5]],
    [['CapsLock', 'Caps', 1.8], ['KeyA', 'a'], ['KeyS', 's'], ['KeyD', 'd'], ['KeyF', 'f'], ['KeyG', 'g'], ['KeyH', 'h'], ['KeyJ', 'j'], ['KeyK', 'k'], ['KeyL', 'l'], ['Semicolon', ';'], ['Quote', "'"], ['Enter', 'Enter', 2.2]],
    [['ShiftLeft', 'Shift', 2.35], ['KeyZ', 'z'], ['KeyX', 'x'], ['KeyC', 'c'], ['KeyV', 'v'], ['KeyB', 'b'], ['KeyN', 'n'], ['KeyM', 'm'], ['Comma', ','], ['Period', '.'], ['Slash', '/'], ['ShiftRight', 'Shift', 2.65]],
    [['ControlLeft', 'Ctrl', 1.4], ['MetaLeft', 'Win', 1.2], ['AltLeft', 'Alt', 1.2], ['Space', '', 6.6], ['AltRight', 'AltGr', 1.2], ['ControlRight', 'Ctrl', 1.4]]
  ];
  var LABEL = {};
  ROWS.forEach(function (r) { r.forEach(function (k) { if (k[1].length === 1 && k[0] !== 'Backspace') LABEL[k[0]] = k[1]; }); });
  var CHAR_CODE = { ' ': ['Space'], '!': ['Digit1', 1], '?': ['Slash', 1], ':': ['Semicolon', 1], '"': ['Quote', 1], '&': ['Digit7', 1], '@': ['Digit2', 1], '>': ['Period', 1] };
  Object.keys(LABEL).forEach(function (code) { if (!CHAR_CODE[LABEL[code]]) CHAR_CODE[LABEL[code]] = [code]; });
  function buildKb(host, asButtons) {
    host.textContent = '';
    ROWS.forEach(function (r) {
      var row = el('div', 'bg-kb__row');
      r.forEach(function (k) {
        var key = el(asButtons ? 'button' : 'span', 'bg-key' + (k[1].length > 1 ? ' bg-key--mod' : ''));
        if (asButtons) { key.type = 'button'; key.tabIndex = -1; }
        key.setAttribute('data-k', k[0]);
        if (k[0] === 'Backspace') key.setAttribute('aria-label', 'Backspace');
        if (k[0] === 'Space') key.setAttribute('aria-label', 'Space');
        if (k[2]) key.style.setProperty('--w', k[2]);
        key.appendChild(el('span', 'bg-key__main', k[1].length === 1 ? k[1].toUpperCase() : k[1]));
        row.appendChild(key);
      });
      host.appendChild(row);
    });
  }
  function press(host, code, shift) {
    if (!host) return;
    var keys = [];
    var k = host.querySelector('[data-k="' + code + '"]');
    if (k) keys.push(k);
    if (shift) { var s = host.querySelector('[data-k="ShiftLeft"]'); if (s) keys.push(s); }
    keys.forEach(function (x) {
      x.classList.add('is-down');
      clearTimeout(x._t);
      x._t = setTimeout(function () { x.classList.remove('is-down'); }, 120);
    });
  }
  function pressChar(host, ch) {
    var c = CHAR_CODE[ch.toLowerCase()];
    if (!c && APOS.indexOf(ch) >= 0) c = ['Quote'];
    if (!c) return;
    press(host, c[0], c[1] || ch !== ch.toLowerCase());
  }

  // ---------- yozuvchi: eski yoziladi, so'z tugashi bilan yangiga o'tadi ----------
  var WORD = /[\p{L}\p{N}]/u;
  function isWordChar(ch) { return WORD.test(ch) || APOS.indexOf(ch) >= 0; }
  function Run() { this.alive = true; }
  Run.prototype.stop = function () { this.alive = false; };
  function typeLine(field, src, map, o) {
    o = o || {};
    var run = o.run || new Run(), kb = o.kb, host = o.watch || field;
    field.textContent = '';
    var caret = el('span', 'bg-caret');
    field.appendChild(caret);
    if (reduced()) {
      // harakatsiz: natija darhol
      src.split(/(\s+|[,.!?:])/).forEach(function (part) {
        if (!part) return;
        if (isWordChar(part[0])) {
          var w = el('span', 'bg-w', map[part] || part);
          field.insertBefore(w, caret);
          if (map[part]) { w.className = 'bg-w'; if (o.onConvert) o.onConvert(w, part); }
        } else field.insertBefore(doc.createTextNode(part), caret);
      });
      return Promise.resolve(true);
    }
    var cur = null, i = 0, chars = Array.from(src);
    function endWord() {
      if (!cur) return Promise.resolve();
      var w = cur; cur = null;
      var key = w.textContent;
      return convertWord(w, map[key]).then(function (changed) {
        if (changed && o.onConvert) o.onConvert(w, key);
        return changed ? sleep(180) : null;
      });
    }
    function step() {
      if (!run.alive) return Promise.resolve(false);
      if (i >= chars.length) {
        return (o.finalBoundary === false ? Promise.resolve() : sleep(260).then(endWord)).then(function () { return true; });
      }
      return seen(host).then(function () {
        if (!run.alive) return false;
        var ch = chars[i++];
        pressChar(kb, ch);
        var p;
        if (isWordChar(ch)) {
          if (!cur) { cur = el('span', 'bg-w'); field.insertBefore(cur, caret); }
          cur.textContent += ch;
          p = Promise.resolve();
        } else {
          p = endWord().then(function () { field.insertBefore(doc.createTextNode(ch), caret); });
        }
        var d = (o.speed || 62) + Math.random() * 46 + (ch === ' ' ? 40 : 0);
        return p.then(function () { return sleep(d); }).then(step);
      });
    }
    return step();
  }
  function caretOff(field) { var c = field.querySelector('.bg-caret'); if (c) c.remove(); }
  function popIn(node) {
    if (!reduced()) node.animate([{ transform: 'translateY(12px) scale(.96)', opacity: 0 }, { transform: 'none', opacity: 1 }], { duration: 380, easing: EASE });
  }

  // ---------- hero: chat + 3D klaviatura ----------
  var heroKb = $('#bg-hero-kb'), heroChat = $('#bg-hero-chat'), heroField = $('#bg-hero-field'), stage = $('#bg-stage');
  buildKb(heroKb, false);
  watch(stage);
  var HERO = [
    { me: 'Ha! Soat beshda chorrahada uchrashamiz', map: { beshda: 'beşda', chorrahada: 'çorrahada', uchrashamiz: 'uçraşamiz' }, reply: 'Zör, körişgunça!' },
    { me: 'Choy ham olib kelaman', map: { Choy: 'Çoy' }, reply: null }
  ];
  function bubble(cls, nodes) {
    var b = el('div', 'bg-msg ' + cls);
    nodes.forEach(function (n) { b.appendChild(n); });
    heroChat.appendChild(b);
    while (heroChat.children.length > 4) heroChat.firstElementChild.remove();
    popIn(b);
    return b;
  }
  var heroFirst = heroChat.innerHTML, heroK = 0;
  function heroNext() {
    var s = HERO[heroK % HERO.length];
    return typeLine(heroField, s.me, s.map, { kb: heroKb, watch: stage }).then(function () {
      return sleep(reduced() ? 0 : 700);
    }).then(function () {
      caretOff(heroField);
      var nodes = Array.prototype.slice.call(heroField.childNodes);
      heroField.textContent = '';
      press(heroKb, 'Enter');
      bubble('bg-msg--out', nodes);
      if (reduced()) {
        if (s.reply) bubble('bg-msg--in', [doc.createTextNode(s.reply)]);
        heroField.appendChild(el('span', 'bg-caret'));
        return;
      }
      return sleep(900).then(function () {
        if (s.reply) bubble('bg-msg--in', [doc.createTextNode(s.reply)]);
        return sleep(s.reply ? 1600 : 2800);
      }).then(function () {
        heroK++;
        if (heroK % HERO.length === 0) heroChat.innerHTML = heroFirst;
        return seen(stage).then(heroNext);
      });
    });
  }
  sleep(reduced() ? 0 : 900).then(heroNext);

  // sichqonchaga javob beruvchi 3D egilish
  (function () {
    var t = $('.bg-stage__tilt', stage), hero = $('#bg-hero');
    var tx = 0, ty = 0, x = 0, y = 0, raf = 0;
    function frame() {
      x += (tx - x) * 0.07; y += (ty - y) * 0.07;
      t.style.setProperty('--rx', x.toFixed(3));
      t.style.setProperty('--ry', y.toFixed(3));
      raf = Math.abs(tx - x) + Math.abs(ty - y) > 0.01 ? requestAnimationFrame(frame) : 0;
    }
    function kick() { if (!raf) raf = requestAnimationFrame(frame); }
    hero.addEventListener('pointermove', function (e) {
      if (reduced() || e.pointerType === 'touch') return;
      var r = stage.getBoundingClientRect();
      tx = Math.max(-1, Math.min(1, (e.clientX - (r.left + r.width / 2)) / (r.width / 1.4))) * 7;
      ty = -Math.max(-1, Math.min(1, (e.clientY - (r.top + r.height / 2)) / (r.height / 1.4))) * 5;
      kick();
    });
    hero.addEventListener('pointerleave', function () { tx = 0; ty = 0; kick(); });
  })();

  // ---------- #qanday: 5 harf ----------
  var letters = $$('.bg-letter');
  letters.forEach(function (b) {
    var from = b.getAttribute('data-from'), to = b.getAttribute('data-to');
    var st = el('span', 'bg-letter__stage');
    var cap = el('span', 'bg-letter__cap');
    Array.from(from).forEach(function (c) { cap.appendChild(el('kbd', null, c)); });
    cap.appendChild(el('span', 'bg-arrow', '→'));
    cap.appendChild(el('kbd', 'bg-new', to));
    b.appendChild(st); b.appendChild(cap);
    function reset() {
      st.textContent = '';
      var ch = Array.from(from);
      st.appendChild(gEl(ch[0], ch.length > 1 ? ch[1] : null));
      b.classList.remove('is-done');
    }
    reset();
    b._play = function () {
      if (b.classList.contains('is-done')) { reset(); return Promise.resolve(); }
      b.classList.add('is-done');
      return fuse(st.firstChild, to, 620);
    };
    b.addEventListener('click', function () { b._play(); });
  });
  var lettersBox = $('.bg-letters');
  watch(lettersBox);
  seen(lettersBox).then(function () { return sleep(300); }).then(function () {
    return letters.reduce(function (p, b) {
      return p.then(function () { b._play(); return sleep(reduced() ? 0 : 240); });
    }, Promise.resolve());
  });

  // ---------- #dasturlar: tablar ----------
  var BAR = '<div class="bg-win__bar"><i></i><i></i><i></i><span>';
  var APPS = {
    chat: {
      html: '<div class="bg-win">' + BAR + 'Xabarlar</span></div>' +
        '<div class="bg-app__body bg-app__chat"><div class="bg-msg bg-msg--in">Qaçon kelasiz?</div><div class="bg-msg bg-msg--out bg-app__out" hidden></div></div>' +
        '<div class="bg-compose"><div class="bg-field" data-f></div><span class="bg-send">' + svg('<path d="M4 12l16-8-6 16-2.5-6.5z"/>') + '</span></div></div>',
      lines: [{ s: 'Ertaga soat beshda chorrahada uchrashamiz', m: { beshda: 'beşda', chorrahada: 'çorrahada', uchrashamiz: 'uçraşamiz' } }],
      send: true
    },
    doc: {
      html: '<div class="bg-win">' + BAR + 'Hujjat.docx</span></div>' +
        '<div class="bg-tool"><i></i><i></i><i></i><i></i><i></i><i></i></div>' +
        '<div class="bg-app__body"><div class="bg-doc"><div class="bg-doc__h bg-field" data-f></div><div class="bg-doc__p bg-field" data-f2></div></div></div></div>',
      lines: [
        { s: 'Ochiq eshiklar kuni', m: { Ochiq: 'Oçiq', eshiklar: 'eşiklar' } },
        { s: "Shu hafta chorshanba kuni ko'rishamiz.", m: { Shu: 'Şu', chorshanba: 'çorşanba', "ko'rishamiz": 'körişamiz' }, f: 'f2' }
      ]
    },
    web: {
      html: '<div class="bg-win">' + BAR + 'Brauzer</span></div>' +
        '<div class="bg-web__bar">' + svg('<path d="M15 6l-6 6 6 6"/>') + '<div class="bg-web__url">qidiruv</div></div>' +
        '<div class="bg-app__body"><div class="bg-web__search">' + svg('<circle cx="11" cy="11" r="6.5"/><path d="M16 16l4.5 4.5"/>') + '<div class="bg-field" data-f></div></div>' +
        '<div class="bg-web__res"><i></i><i></i><i></i></div></div></div>',
      lines: [{ s: 'Toshkent shahri', m: { Toshkent: 'Toşkent', shahri: 'şahri' } }]
    },
    sheet: {
      html: '<div class="bg-win">' + BAR + 'Röyxat.xlsx</span></div>' +
        '<div class="bg-sheet">' +
        '<div class="bg-sh-h"></div><div class="bg-sh-h">A</div><div class="bg-sh-h">B</div><div class="bg-sh-h bg-sh-x">C</div>' +
        '<div class="bg-sh-n">1</div><div><b>Mahsulot</b></div><div><b>Miqdor</b></div><div class="bg-sh-x"></div>' +
        '<div class="bg-sh-n">2</div><div class="bg-field" data-c="0"></div><div>2 kg</div><div class="bg-sh-x"></div>' +
        '<div class="bg-sh-n">3</div><div class="bg-field" data-c="1"></div><div>1 kg</div><div class="bg-sh-x"></div>' +
        '<div class="bg-sh-n">4</div><div class="bg-field" data-c="2"></div><div>3 kg</div><div class="bg-sh-x"></div>' +
        '<div class="bg-sh-n">5</div><div class="bg-field" data-c="3"></div><div>5 kg</div><div class="bg-sh-x"></div>' +
        '<div class="bg-sh-n">6</div><div></div><div></div><div class="bg-sh-x"></div>' +
        '</div></div>',
      cells: [['Choy', 'Çoy'], ['Shakar', 'Şakar'], ["Go'sht", 'Göşt'], ['Guruch', 'Guruç']]
    },
    code: {
      html: '<div class="bg-win">' + BAR + 'Terminal</span></div>' +
        '<div class="bg-term"><div><span class="bg-pr">PS C:\\loyiha&gt;</span> <span class="bg-field" data-f></span></div>' +
        '<div class="bg-term__chip" hidden>' + svg(ICON_LOCK) + '<span>Terminalda bitg öçiq: buyruq va parol tegmaydi</span></div></div></div>',
      lines: [{ s: 'git push origin main', m: {} }],
      after: function (scr) {
        var c = $('.bg-term__chip', scr);
        c.hidden = false;
        popIn(c);
      }
    }
  };
  var appScreen = $('#bg-apps-screen'), appTabs = $$('.bg-tab'), appRun = null, appAuto = true;
  function showApp(name, user) {
    if (user) appAuto = false;
    if (appRun) appRun.stop();
    var run = appRun = new Run();
    appTabs.forEach(function (t) {
      var on = t.getAttribute('data-app') === name;
      t.setAttribute('aria-selected', String(on));
      t.tabIndex = on ? 0 : -1;
      var bar = $('.bg-tab__bar', t);
      bar.getAnimations().forEach(function (a) { a.cancel(); });
      bar.style.transform = on ? 'scaleX(1)' : 'scaleX(0)';
      if (on && appAuto && !reduced()) {
        bar.animate([{ transform: 'scaleX(0)' }, { transform: 'scaleX(1)' }], { duration: 7000, easing: 'linear', fill: 'forwards' });
      }
    });
    var a = APPS[name];
    var old = appScreen.firstElementChild;
    var scr = el('div', 'bg-app bg-app--' + name);
    scr.setAttribute('role', 'tabpanel');
    scr.setAttribute('aria-labelledby', 'bg-tab-' + name);
    scr.innerHTML = a.html;
    appScreen.appendChild(scr);
    if (old) {
      $$('.bg-app', appScreen).forEach(function (x) { if (x !== scr && x !== old) x.remove(); });
      if (reduced()) old.remove();
      else old.animate([{ opacity: 1, transform: 'none' }, { opacity: 0, transform: 'translateY(-10px) scale(.985)' }], { duration: 220, easing: 'ease-in', fill: 'forwards' })
        .finished.then(function () { old.remove(); }, function () { old.remove(); });
    }
    if (!reduced()) scr.animate([{ opacity: 0, transform: 'translateY(14px) scale(.985)' }, { opacity: 1, transform: 'none' }], { duration: 380, delay: old ? 120 : 0, easing: EASE, fill: 'backwards' });
    var chain = sleep(reduced() ? 0 : 520);
    if (a.cells) {
      a.cells.forEach(function (c, i) {
        chain = chain.then(function () {
          if (!run.alive) return;
          var f = $('[data-c="' + i + '"]', scr);
          f.classList.add('is-active');
          var m = {}; m[c[0]] = c[1];
          return typeLine(f, c[0], m, { run: run, watch: appScreen }).then(function () {
            caretOff(f); f.classList.remove('is-active');
          });
        });
      });
    } else {
      a.lines.forEach(function (ln, li) {
        chain = chain.then(function () {
          if (!run.alive) return;
          var f = $('[data-' + (ln.f || 'f') + ']', scr);
          return typeLine(f, ln.s, ln.m, { run: run, watch: appScreen }).then(function () {
            if (li < a.lines.length - 1) caretOff(f);
          });
        });
      });
    }
    chain.then(function () {
      if (!run.alive) return;
      if (a.send) {
        var f = $('[data-f]', scr), out = $('.bg-app__out', scr);
        caretOff(f);
        Array.prototype.slice.call(f.childNodes).forEach(function (n) { out.appendChild(n); });
        out.hidden = false;
        f.appendChild(el('span', 'bg-caret'));
        popIn(out);
      }
      if (a.after) a.after(scr);
      if (!appAuto || reduced()) return;
      return sleep(2600).then(function () { return seen(appScreen); }).then(function () {
        if (!run.alive || !appAuto) return;
        var names = Object.keys(APPS);
        showApp(names[(names.indexOf(name) + 1) % names.length]);
      });
    });
  }
  appTabs.forEach(function (t, i) {
    t.id = 'bg-tab-' + t.getAttribute('data-app');
    t.addEventListener('click', function () { showApp(t.getAttribute('data-app'), true); });
    t.addEventListener('keydown', function (e) {
      var d = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0;
      if (!d) return;
      e.preventDefault();
      var n = appTabs[(i + d + appTabs.length) % appTabs.length];
      n.focus();
      showApp(n.getAttribute('data-app'), true);
    });
  });
  watch(appScreen);
  seen(appScreen).then(function () { showApp('chat'); });

  // ---------- #himoya: har so'z tekshiriladi ----------
  var GUARD = [
    ['Shu', 'c', 'Şu'], ['hafta'], ['chorshanba', 'c', 'çorşanba'], ['kuni'], ['push', 'e'], ['qilaman,'],
    ['linkni'], ['https://meet.google.com/abc', 'p'], ['ga'], ['tashlayman,', 'c', 'taşlayman,'],
    ['pochta:', 'c', 'poçta:'], ['ali@example.uz', 'p']
  ];
  var guard = $('#bg-guard'), gLine = $('#bg-guard-line'), gScan = $('.bg-guard__scan', guard);
  var counts = {};
  $$('[data-count]').forEach(function (b) { counts[b.getAttribute('data-count')] = b; });
  function guardLoop() {
    gLine.textContent = '';
    var n = { c: 0, e: 0, p: 0 };
    Object.keys(counts).forEach(function (k) { counts[k].textContent = '0'; });
    var toks = GUARD.map(function (g, i) {
      var t = el('span', 'bg-tok');
      if (g[1]) t.setAttribute('data-r', g[1]);
      t.appendChild(el('span', 'bg-w', g[0]));
      if (g[1]) {
        var tag = el('span', 'bg-tok__tag');
        tag.innerHTML = g[1] === 'c' ? 'yangi' : g[1] === 'e' ? 'EN' : svg(ICON_LOCK);
        t.appendChild(tag);
      }
      gLine.appendChild(t);
      if (i < GUARD.length - 1) gLine.appendChild(doc.createTextNode(' '));
      return t;
    });
    var pos = null;
    gScan.style.opacity = reduced() ? '0' : '1';
    return toks.reduce(function (p, t, i) {
      return p.then(function () { return seen(guard); }).then(function () {
        var g = GUARD[i];
        if (!reduced()) {
          var gr = guard.getBoundingClientRect(), r = t.getBoundingClientRect();
          var next = { x: r.left - gr.left - 3, y: r.top - gr.top - 1, w: r.width + 6, h: r.height + 2 };
          var from = pos || next;
          gScan.animate([
            { transform: 'translate(' + from.x + 'px,' + from.y + 'px)', width: from.w + 'px', height: from.h + 'px' },
            { transform: 'translate(' + next.x + 'px,' + next.y + 'px)', width: next.w + 'px', height: next.h + 'px' }
          ], { duration: 260, easing: EASE, fill: 'forwards' });
          pos = next;
        }
        return sleep(reduced() ? 0 : 280).then(function () {
          if (!g[1]) return sleep(reduced() ? 0 : 60);
          t.classList.add('is-seen');
          n[g[1]]++;
          if (counts[g[1]]) {
            counts[g[1]].textContent = String(n[g[1]]);
            if (!reduced()) counts[g[1]].animate([{ transform: 'scale(1.35)' }, { transform: 'none' }], { duration: 320, easing: EASE });
          }
          var job = g[1] === 'c' ? convertWord($('.bg-w', t), g[2]) : Promise.resolve();
          return job.then(function () { return sleep(reduced() ? 0 : 340); });
        });
      });
    }, Promise.resolve()).then(function () {
      gScan.style.opacity = '0';
      if (reduced()) return;
      return sleep(3800).then(function () { return seen(guard); }).then(guardLoop);
    });
  }
  watch(guard);
  seen(guard).then(function () { return sleep(250); }).then(guardLoop);

  // ---------- #ortga: Backspace bilan qaytarish ----------
  var undoBox = $('#bg-undo'), undoField = $('#bg-undo-field'), undoKey = $('#bg-undo-key'), toast = $('#bg-undo-toast');
  function undoLoop() {
    toast.classList.remove('is-on');
    var target = null;
    return typeLine(undoField, 'Chorsu ', { Chorsu: 'Çorsu' }, {
      watch: undoBox, finalBoundary: false, speed: 90,
      onConvert: function (w) { target = w; }
    }).then(function () { return sleep(reduced() ? 0 : 900); }).then(function () {
      undoKey.classList.add('is-down');
      setTimeout(function () { undoKey.classList.remove('is-down'); }, 160);
      if (target) revertWord(target, 'Chorsu');
      toast.classList.add('is-on');
      return sleep(reduced() ? 0 : 700);
    }).then(function () {
      var caret = $('.bg-caret', undoField);
      var rest = 'bozoriga boraman';
      if (reduced()) { undoField.insertBefore(doc.createTextNode(rest), caret); return; }
      var i = 0;
      return (function more() {
        if (i >= rest.length) return Promise.resolve();
        return seen(undoBox).then(function () {
          undoField.insertBefore(doc.createTextNode(rest[i++]), caret);
          return sleep(70 + Math.random() * 40).then(more);
        });
      })();
    }).then(function () {
      if (reduced()) return;
      return sleep(3200).then(function () { return seen(undoBox); }).then(undoLoop);
    });
  }
  watch(undoBox);
  seen(undoBox).then(function () { return sleep(300); }).then(undoLoop);

  // ---------- #klaviatura: tayyor tartiblar (crates/bitg-tsf/src/keymap.rs, preset()) ----------
  // [oddiy, Shift, AltGr, AltGr+Shift]; N — o'zgarmaydi
  var N = null;
  var PRESETS = {
    tilmoch: {
      Backquote: ['’', '"', '‘', '`'], Digit3: [N, N, N, '№'], Digit9: [N, N, '‘', N], Digit0: [N, N, '’', N],
      Minus: [N, N, '–', '—'], BracketLeft: ['ö', 'Ö', '[', '{'], BracketRight: ['ğ', 'Ğ', ']', '}'],
      Backslash: [';', ':', '\\', '|'], Semicolon: ['ş', 'Ş', ';', ':'], Quote: ['ç', 'Ç', "'", '"'],
      Comma: [',', '“', '<', '«'], Period: ['.', '”', '>', '»']
    },
    turkish: {
      KeyI: ['ı', 'I', N, N], BracketLeft: ['ğ', 'Ğ', '[', '{'], BracketRight: ['ü', 'Ü', ']', '}'],
      Semicolon: ['ş', 'Ş', ';', ':'], Quote: ['i', 'İ', "'", '"'], Comma: ['ö', 'Ö', '<', N],
      Period: ['ç', 'Ç', '>', N], Slash: ['.', ':', '/', '?'], Backslash: [',', ';', '\\', '|']
    },
    altgr: {
      KeyO: [N, N, 'ö', 'Ö'], KeyG: [N, N, 'ğ', 'Ğ'], KeyS: [N, N, 'ş', 'Ş'], KeyC: [N, N, 'ç', 'Ç'],
      Quote: [N, N, 'ʼ', '"'], Minus: [N, N, '–', '—'], Comma: [N, N, '«', '“'], Period: [N, N, '»', '”']
    },
    kirill: (function () {
      var m = {};
      var L = { Backquote: 'ё', Minus: 'ғ', Equal: 'ҳ', KeyQ: 'й', KeyW: 'ц', KeyE: 'у', KeyR: 'к', KeyT: 'е', KeyY: 'н', KeyU: 'г', KeyI: 'ш', KeyO: 'ў', KeyP: 'з', BracketLeft: 'х', BracketRight: 'ъ', KeyA: 'ф', KeyS: 'қ', KeyD: 'в', KeyF: 'а', KeyG: 'п', KeyH: 'р', KeyJ: 'о', KeyK: 'л', KeyL: 'д', Semicolon: 'ж', Quote: 'э', KeyZ: 'я', KeyX: 'ч', KeyC: 'с', KeyV: 'м', KeyB: 'и', KeyN: 'т', KeyM: 'ь', Comma: 'б', Period: 'ю' };
      Object.keys(L).forEach(function (k) { m[k] = [L[k], L[k].toUpperCase(), N, N]; });
      function slot(k) { return m[k] || (m[k] = [N, N, N, N]); }
      var S = { Digit2: '"', Digit3: '№', Digit4: ';', Digit6: ':', Digit7: '?', Slash: ',' };
      Object.keys(S).forEach(function (k) { slot(k)[1] = S[k]; });
      slot('Slash')[0] = '.';
      var U = { Backquote: ['`', '~'], Minus: ['-', '_'], Equal: ['=', '+'], BracketLeft: ['[', '{'], BracketRight: [']', '}'], Semicolon: [';', ':'], Quote: ["'", '"'], Comma: [',', '<'], Period: ['.', '>'], Slash: ['/', '?'] };
      Object.keys(U).forEach(function (k) { slot(k)[2] = U[k][0]; slot(k)[3] = U[k][1]; });
      return m;
    })()
  };
  var HINTS = {
    tilmoch: '<kbd>[</kbd> ö &nbsp; <kbd>]</kbd> ğ &nbsp; <kbd>;</kbd> ş &nbsp; <kbd>\'</kbd> ç &nbsp; · &nbsp;belgilar <kbd>AltGr</kbd> bilan',
    turkish: '<kbd>,</kbd> ö &nbsp; <kbd>[</kbd> ğ &nbsp; <kbd>;</kbd> ş &nbsp; <kbd>.</kbd> ç &nbsp; <kbd>i</kbd> ı',
    altgr: 'Hamma tugma joyida: <kbd>AltGr</kbd> + <kbd>o</kbd> → ö',
    kirill: 'Kirillcha yozing: so‘z oxirida lotinga o‘giriladi'
  };
  var layKb = $('#bg-layout-kb'), layIn = $('#bg-layout-input'), hint = $('#bg-preset-hint'), hintHtml = '', preset = 'tilmoch';
  buildKb(layKb, true);
  function face(c) { return c.toUpperCase() !== c ? c.toUpperCase() : c; }
  function paintPreset(animate) {
    var p = PRESETS[preset];
    $$('.bg-key', layKb).forEach(function (k) {
      var code = k.getAttribute('data-k'), base = LABEL[code];
      if (!base) return;
      var lv = p[code] || [N, N, N, N];
      var m = $('.bg-key__main', k);
      var main = face(lv[0] || base);
      var isNew = !!(lv[0] || lv[2]);
      var changed = m.textContent !== main;
      $$('small', k).forEach(function (s) { s.remove(); });
      m.textContent = main;
      k.classList.toggle('is-new', isNew);
      if (lv[2]) k.appendChild(el('small', 'bg-k-br', lv[2]));
      if (lv[1] && lv[0] && lv[1] !== lv[0].toUpperCase()) k.appendChild(el('small', 'bg-k-tl', lv[1]));
      if (animate && changed && !reduced()) {
        m.classList.remove('is-flip'); void m.offsetWidth; m.classList.add('is-flip');
      }
    });
    hintHtml = HINTS[preset];
    hint.innerHTML = T(hintHtml);
  }
  paintPreset(false);
  var presetBtns = $$('#bg-presets .bg-seg__b');
  presetBtns.forEach(function (b) {
    b.addEventListener('click', function () {
      preset = b.getAttribute('data-preset');
      presetBtns.forEach(function (x) { x.setAttribute('aria-selected', String(x === b)); });
      paintPreset(true);
    });
  });
  function insert(text) {
    var s = layIn.selectionStart, e = layIn.selectionEnd;
    layIn.setRangeText(text, s, e, 'end');
    if (preset === 'kirill' && /[\s,.!?;:]/.test(text)) kirillWord();
  }
  // kirill tartibda so'z tugasa — yadro uni yangi lotinga o'giradi (IME'dagidek)
  function kirillWord() {
    var v = layIn.value, end = layIn.selectionStart - 1;
    var m = /([\u0400-\u04FF]+)$/.exec(v.slice(0, end));
    if (!m) return;
    core().then(function (c) {
      var o = c.toNew(m[1]).out;
      if (o === m[1] || layIn.value !== v) return;
      var st = end - m[1].length;
      layIn.value = v.slice(0, st) + o + v.slice(end);
      var pos = st + o.length + 1;
      layIn.setSelectionRange(pos, pos);
    }, function () {});
  }
  layIn.addEventListener('keydown', function (e) {
    if (e.isComposing || e.metaKey) return;
    var altgr = (e.getModifierState && e.getModifierState('AltGraph')) || (e.ctrlKey && e.altKey);
    if (e.ctrlKey && !altgr) return;
    press(layKb, e.code, e.shiftKey);
    var lv = (PRESETS[preset][e.code] || [])[(altgr ? 2 : 0) + (e.shiftKey ? 1 : 0)];
    if (lv) { e.preventDefault(); insert(lv); return; }
    if (preset === 'kirill' && e.key === ' ') { e.preventDefault(); insert(' '); }
  });
  layKb.addEventListener('click', function (e) {
    var k = e.target.closest('.bg-key');
    if (!k) return;
    var code = k.getAttribute('data-k');
    press(layKb, code);
    var lv = PRESETS[preset][code];
    if (code === 'Backspace') {
      var s = layIn.selectionStart, en = layIn.selectionEnd;
      if (s === en && s > 0) s--;
      layIn.setRangeText('', s, en, 'end');
    } else if (code === 'Space' || code === 'Enter') insert(' ');
    else if (lv && lv[0]) insert(lv[0]);
    else if (LABEL[code]) insert(LABEL[code]);
  });
  $('#bg-layout-clear').addEventListener('click', function () { layIn.value = ''; layIn.focus(); });

  // ---------- #sinov: jonli yadro ----------
  var SAMPLES = [
    "Assalomu alaykum! Ertaga choyxonada ko'rishamiz",
    'Shu hafta chorshanba kuni push qilaman, linkni https://meet.google.com/abc ga tashlayman, pochta: ali@example.uz',
    'Ассалому алайкум! Эртага чойхонада кўришамиз',
    'Ozbekiston gozal, goshtli osh'
  ];
  var tryIn = $('#bg-try-in'), tryOut = $('#bg-try-out'), tryState = $('#bg-try-state'), tryBox = $('#bg-try');
  var dir = 'new', tryTimer = 0, tryCore = null;
  var samples = $$('[data-sample]', tryBox), flagBoxes = $$('[data-flag]', tryBox);
  function flags() {
    var f = 0;
    flagBoxes.forEach(function (c) { if (!c.checked) f |= +c.getAttribute('data-flag'); });
    return f;
  }
  var WHY = { e: 'inglizcha so‘z', p: 'himoyalangan', x: 'istisno' };
  function render() {
    if (!tryCore) return;
    var src = tryIn.value, t0 = performance.now();
    tryOut.textContent = '';
    tryOut.classList.remove('is-loading');
    if (dir === 'old') {
      tryOut.textContent = tryCore.toOld(src);
    } else {
      var r = tryCore.toNew(src, flags());
      var ip = src.split(/(\s+)/), op = r.out.split(/(\s+)/);
      var words = ip.filter(function (p, i) { return i % 2 === 0 && p; });
      if (ip.length === op.length && r.toks.length === words.length) {
        var w = 0;
        op.forEach(function (part, i) {
          if (i % 2 === 1 || !part) { tryOut.appendChild(doc.createTextNode(part)); return; }
          var why = r.toks[w++][1];
          if (why === 'c') tryOut.appendChild(el('mark', null, part));
          else if (WHY[why]) {
            var s = el('span', 'bg-o-' + (why === 'e' ? 'e' : 'p'), part);
            s.title = T(WHY[why]);
            tryOut.appendChild(s);
          } else tryOut.appendChild(doc.createTextNode(part));
        });
      } else tryOut.textContent = r.out;
    }
    var ms = performance.now() - t0;
    tryState.textContent = T('yadro brauzerda: ') + (ms < 1 ? '<1' : ms.toFixed(1)) + ' ms';
    tryState.classList.add('is-ok');
  }
  tryIn.value = SAMPLES[0];
  tryOut.classList.add('is-loading');
  samples[0].classList.add('is-on');
  tryIn.addEventListener('input', function () {
    clearTimeout(tryTimer);
    tryTimer = setTimeout(render, 40);
    samples.forEach(function (b) { b.classList.remove('is-on'); });
  });
  flagBoxes.forEach(function (c) { c.addEventListener('change', render); });
  samples.forEach(function (b) {
    b.addEventListener('click', function () {
      samples.forEach(function (x) { x.classList.toggle('is-on', x === b); });
      if (dir === 'old') setDir('new', true);
      tryIn.value = SAMPLES[+b.getAttribute('data-sample')];
      render();
    });
  });
  function setDir(d, keepInput) {
    if (d === dir) return;
    if (tryCore && !keepInput) tryIn.value = tryOut.textContent;
    dir = d;
    $$('[data-dir]', tryBox).forEach(function (b) { b.setAttribute('aria-pressed', String(b.getAttribute('data-dir') === d)); });
    flagBoxes.forEach(function (c) { c.disabled = d === 'old'; });
    render();
  }
  $$('[data-dir]', tryBox).forEach(function (b) { b.addEventListener('click', function () { setDir(b.getAttribute('data-dir')); }); });
  var copyBtn = $('#bg-try-copy');
  copyBtn.addEventListener('click', function () {
    var lbl = $('span', copyBtn), was = lbl.textContent;
    if (!navigator.clipboard) return;
    navigator.clipboard.writeText(tryOut.textContent).then(function () {
      lbl.textContent = T('Nusxalandi');
      setTimeout(function () { lbl.textContent = was; }, 1500);
    }, function () {});
  });
  var started = false;
  function startCore() {
    if (started) return;
    started = true;
    core().then(function (c) {
      coreApi = c; tryCore = c;
      render();
      if (abc === 'new') applyAbc(false);
    }, function () {
      started = false;
      tryState.textContent = T('yadro yuklanmadi, sahifani yangilang');
      tryOut.classList.remove('is-loading');
    });
  }
  if (abc === 'new') { setAbc('new', false); startCore(); }
  else if (doc.readyState === 'complete') idle(startCore);
  else window.addEventListener('load', function () { idle(startCore); });
  var trySec = $('#sinov');
  watch(trySec);
  seen(trySec).then(startCore);

  // ---------- versiya ----------
  var ver = $('#bg-ver'), appVer = 'v' + (ver.getAttribute('data-ver') || '').replace(/^v/, '');
  fetch('version.txt', { cache: 'no-store' }).then(function (r) { return r.ok ? r.text() : ''; }).then(function (t) {
    var sha = (t || '').trim();
    ver.textContent = appVer + (/^[0-9a-f]{7,40}$/.test(sha) ? ' · ' + sha.slice(0, 7) : '');
  }, function () { ver.textContent = appVer; });
})();
