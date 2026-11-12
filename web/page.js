// bitg.uz ichki sahifalari: mavzu, menyu, sahifa alifbosi, onlayn konvertor, kunlar hisobi.
// bitg yadrosi (WASM) faqat kerak bo'lganda yuklanadi: konvertor sahifasida yoki «Yangi» tanlanganda.
(function () {
  'use strict';

  var doc = document, root = doc.documentElement;
  var $ = function (s, r) { return (r || doc).querySelector(s); };
  var $$ = function (s, r) { return Array.prototype.slice.call((r || doc).querySelectorAll(s)); };

  // ---------- yadro ----------
  var corePromise = null;
  function core() {
    if (corePromise) return corePromise;
    var v = root.getAttribute('data-v');
    var url = '/bitg_core.wasm' + (v ? '?v=' + v : '');
    function slow() {
      return fetch(url).then(function (r) {
        if (!r.ok) throw new Error('wasm ' + r.status);
        return r.arrayBuffer();
      }).then(function (b) { return WebAssembly.instantiate(b, {}); });
    }
    var p = WebAssembly.instantiateStreaming ? WebAssembly.instantiateStreaming(fetch(url), {}).catch(slow) : slow();
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
        toNew: function (t, f) { return run(t, 0, f) || { out: t, toks: [] }; },
        toOld: function (t, ascii) { var r = run(t, ascii ? 2 : 1, 0); return r ? r.out : t; },
        toCyr: function (t) { var r = run(t, 3, 0); return r ? r.out : t; }
      };
    });
    corePromise.catch(function () { corePromise = null; });
    return corePromise;
  }

  // ---------- mavzu ----------
  var themeBtn = $('#bg-theme');
  if (themeBtn) themeBtn.addEventListener('click', function () {
    var cur = root.getAttribute('data-theme') || (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
    var next = cur === 'dark' ? 'light' : 'dark';
    root.setAttribute('data-theme', next);
    try { localStorage.setItem('bitg-theme', next); } catch (e) {}
  });

  // ---------- sarlavha va mobil menyu ----------
  var head = $('#bg-head'), menu = $('#dc-menu');
  function onScroll() { if (head) head.classList.toggle('is-stuck', window.scrollY > 4); }
  window.addEventListener('scroll', onScroll, { passive: true });
  onScroll();
  if (menu && head) {
    menu.addEventListener('click', function () {
      var open = !head.classList.contains('dc-head-open');
      head.classList.toggle('dc-head-open', open);
      menu.setAttribute('aria-expanded', String(open));
    });
    doc.addEventListener('keydown', function (e) {
      if (e.key === 'Escape' && head.classList.contains('dc-head-open')) {
        head.classList.remove('dc-head-open');
        menu.setAttribute('aria-expanded', 'false');
        menu.focus();
      }
    });
  }

  // ---------- sahifa alifbosi: Yangi / Eski (faqat o'zbekcha sahifalar) ----------
  var abcBtns = $$('button[data-abc]');
  if (abcBtns.length) {
    var abc = root.getAttribute('data-abc') === 'new' ? 'new' : 'old';
    var orig = new Map(), api = null, origTitle = doc.title;
    var textNodes = function () {
      var out = [], w = doc.createTreeWalker(doc.body, NodeFilter.SHOW_TEXT, {
        acceptNode: function (n) {
          if (!n.data.trim()) return NodeFilter.FILTER_REJECT;
          var p = n.parentElement;
          var c = p && p.closest('[data-conv],[data-no-conv],script,style,kbd,code,pre,svg,textarea,.dc-cv__out');
          if (!p || (c && !c.hasAttribute('data-conv'))) return NodeFilter.FILTER_REJECT;
          return NodeFilter.FILTER_ACCEPT;
        }
      });
      while (w.nextNode()) out.push(w.currentNode);
      return out;
    };
    var apply = function () {
      textNodes().forEach(function (n) {
        if (!orig.has(n)) orig.set(n, n.data);
        var o = orig.get(n);
        n.data = abc === 'new' && api ? api.toNew(o).out : o;
      });
      doc.title = abc === 'new' && api ? api.toNew(origTitle).out : origTitle;
    };
    var set = function (next) {
      abc = next;
      root.setAttribute('data-abc', next);
      try { localStorage.setItem('bitg-abc', next); } catch (e) {}
      abcBtns.forEach(function (b) { b.setAttribute('aria-pressed', String(b.getAttribute('data-abc') === next)); });
      if (next === 'old' && !api) return;
      core().then(function (c) { api = c; apply(); }, function () {});
    };
    abcBtns.forEach(function (b) {
      b.addEventListener('click', function () { var n = b.getAttribute('data-abc'); if (n !== abc) set(n); });
    });
    if (abc === 'new') set('new');
  }

  // ---------- onlayn konvertor ----------
  var cv = $('#dc-cv');
  if (cv) {
    var inp = $('#dc-cv-in', cv), out = $('#dc-cv-out', cv), state = $('#dc-cv-state', cv);
    var dirBtns = $$('[data-dir]', cv), boxes = $$('input[data-flag]', cv), copy = $('#dc-cv-copy', cv);
    var opts = $('#dc-cv-opts', cv), ambNote = $('#dc-cv-amb', cv);
    var msg = {
      load: cv.getAttribute('data-load') || '…', ready: cv.getAttribute('data-ready') || '',
      fail: cv.getAttribute('data-fail') || 'error', copied: cv.getAttribute('data-copied') || 'OK'
    };
    var dir = 'new', api2 = null, timer = 0;
    var flags = function () {
      var f = 0;
      boxes.forEach(function (c) { if (!c.checked) f |= +c.getAttribute('data-flag'); });
      return f;
    };
    var render = function () {
      if (!api2) return;
      var src = inp.value;
      out.textContent = '';
      if (ambNote) ambNote.hidden = true;
      if (dir === 'old') { out.textContent = api2.toOld(src, false); return; }
      if (dir === 'cyr') { out.textContent = api2.toCyr(src); return; }
      var r = api2.toNew(src, flags());
      // yangi harfli so'zlar belgilanadi (natija matni o'zgarmaydi, faqat ko'rinish);
      // ikki ma'noli so'z (yadro 3-element bilan beradi) — tugma: bosilsa ikkinchi o'qilishiga almashadi
      var parts = r.out.split(/(\s+)/);
      var words = parts.filter(function (p, i) { return i % 2 === 0 && p; }).length;
      var toks = r.toks.length === words ? r.toks : null;
      var frag = doc.createDocumentFragment(), w = 0, amb = 0;
      parts.forEach(function (p, i) {
        if (!p) return;
        if (i % 2 === 1) { frag.appendChild(doc.createTextNode(p)); return; }
        var alt = toks && toks[w] && toks[w][2];
        w++;
        if (alt) {
          amb++;
          var b = doc.createElement('button');
          b.type = 'button'; b.className = 'dc-cv__amb'; b.textContent = p;
          b.setAttribute('data-alt', alt);
          b.title = ambNote ? ambNote.getAttribute('data-title').split('{alt}').join(alt) : alt;
          frag.appendChild(b);
        } else if (/[öğşçÖĞŞÇʼ]/.test(p)) {
          var m = doc.createElement('mark'); m.textContent = p; frag.appendChild(m);
        } else frag.appendChild(doc.createTextNode(p));
      });
      out.appendChild(frag);
      if (ambNote) {
        ambNote.hidden = !amb;
        ambNote.textContent = '';
        if (amb) {
          var n = doc.createElement('b');
          n.textContent = ambNote.getAttribute('data-count').replace('{n}', amb);
          ambNote.append(n, ' ' + ambNote.getAttribute('data-hint'));
        }
      }
    };
    if (ambNote) out.addEventListener('click', function (e) {
      var b = e.target.closest('.dc-cv__amb');
      if (!b) return;
      var cur = b.textContent, alt = b.getAttribute('data-alt');
      b.textContent = alt;
      b.setAttribute('data-alt', cur);
      b.classList.toggle('is-swapped');
      b.title = ambNote.getAttribute('data-title').split('{alt}').join(cur);
    });
    var schedule = function () { clearTimeout(timer); timer = setTimeout(render, 60); };
    dirBtns.forEach(function (b) {
      b.addEventListener('click', function () {
        dir = b.getAttribute('data-dir');
        dirBtns.forEach(function (x) { x.setAttribute('aria-pressed', String(x === b)); });
        if (opts) opts.hidden = dir !== 'new';
        render();
      });
    });
    boxes.forEach(function (c) { c.addEventListener('change', render); });
    inp.addEventListener('input', schedule);
    $$('[data-sample]', cv).forEach(function (b) {
      b.addEventListener('click', function () { inp.value = b.getAttribute('data-sample'); render(); inp.focus(); });
    });
    if (copy) copy.addEventListener('click', function () {
      var t = out.textContent;
      var done = function () { var l = copy.lastElementChild, o = l.textContent; l.textContent = msg.copied; setTimeout(function () { l.textContent = o; }, 1400); };
      if (navigator.clipboard) navigator.clipboard.writeText(t).then(done, function () {});
    });
    state.textContent = msg.load;
    core().then(function (c) { api2 = c; state.textContent = msg.ready; render(); },
      function () { state.textContent = msg.fail; });
  }

  // ---------- nusxalash tugmalari ----------
  $$('[data-copy]').forEach(function (b) {
    b.addEventListener('click', function () {
      var t = $(b.getAttribute('data-copy'));
      if (!t || !navigator.clipboard) return;
      navigator.clipboard.writeText(t.textContent.trim()).then(function () {
        var o = b.textContent; b.textContent = b.getAttribute('data-done') || '✓';
        setTimeout(function () { b.textContent = o; }, 1400);
      }, function () {});
    });
  });

  // ---------- kunlar hisobi: statik raqam bor, JS faqat bugungi kunga yangilaydi ----------
  $$('[data-until]').forEach(function (e) {
    var t = new Date(e.getAttribute('data-until') + 'T00:00:00+05:00').getTime();
    var d = Math.ceil((t - Date.now()) / 864e5);
    if (d >= 0) e.textContent = String(d);
  });
})();
