// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Popup: matn o'giruvchi (yadro shu sahifaning o'zida, oflayn), shu sahifa uchun tugmalar, til tanlash.
// Matn hech qayerda saqlanmaydi; storage'da faqat til va avto-o'girish yoqilgan saytlar ro'yxati.
/* global bitgCore, bitgI18n, bitgLib */
(function () {
  'use strict';
  var api = globalThis.browser || globalThis.chrome;
  var $ = function (id) { return document.getElementById(id); };
  var input = $('in'), out = $('out'), stat = $('stat'), copyBtn = $('copy');
  var lang = 'uz', core = null, last = null, tab = null, pattern = null, timer = 0;

  function t(key, vars) { return bitgI18n.t(lang, key, vars); }

  function paintLang() {
    document.documentElement.lang = lang;
    document.querySelectorAll('[data-i18n]').forEach(function (el) { el.textContent = t(el.getAttribute('data-i18n')); });
    document.querySelectorAll('[data-i18n-ph]').forEach(function (el) { el.placeholder = t(el.getAttribute('data-i18n-ph')); });
    document.querySelectorAll('[data-i18n-label]').forEach(function (el) { el.setAttribute('aria-label', t(el.getAttribute('data-i18n-label'))); });
    document.querySelectorAll('[data-lang]').forEach(function (b) {
      b.setAttribute('aria-pressed', String(b.getAttribute('data-lang') === lang));
      b.title = bitgI18n.STRINGS[b.getAttribute('data-lang')].langName;
    });
    $('page-label').textContent = t($('page').getAttribute('aria-pressed') === 'true' ? 'pageOff' : 'pageOn');
    paintShortcut();
    render();
  }

  // ---------- o'giruvchi ----------
  function span(cls, text, title) {
    var s = document.createElement(cls === 'mark' ? 'mark' : 'span');
    if (cls !== 'mark') s.className = cls;
    s.textContent = text;
    if (title) s.title = title;
    return s;
  }

  function render() {
    var src = input.value;
    out.textContent = '';
    stat.textContent = '';
    stat.className = 'p-stat';
    if (!src.trim()) {
      last = null;
      out.classList.add('is-empty');
      out.textContent = t('outEmpty');
      copyBtn.disabled = true;
      return;
    }
    if (!core) return;
    var r = core.toNew(src);
    last = r.out;
    out.classList.remove('is-empty');
    copyBtn.disabled = false;
    var ip = src.split(/(\s+)/), op = r.out.split(/(\s+)/);
    var words = ip.filter(function (p, i) { return i % 2 === 0 && p; });
    var changed = 0, alts = 0;
    r.toks.forEach(function (k) { if (k[1] === 'c') changed++; if (k[2]) alts++; });
    if (ip.length === op.length && r.toks.length === words.length) {
      // so'zma-so'z: o'girilgani belgilanadi, ikki ma'noli so'z — "tekshiring" (bitg.uz sinov maydonidagidek)
      var w = 0;
      op.forEach(function (part, i) {
        if (i % 2 === 1 || !part) { out.appendChild(document.createTextNode(part)); return; }
        var k = r.toks[w++];
        if (k[2]) out.appendChild(span('is-alt', part, t('altTitle', { alt: k[2] })));
        else if (k[1] === 'c') out.appendChild(span('mark', part));
        else if (k[1] === 'e' || k[1] === 'p' || k[1] === 'x') out.appendChild(span('is-kept', part, t('protectedTitle')));
        else out.appendChild(document.createTextNode(part));
      });
    } else {
      out.textContent = r.out;
    }
    if (alts) { stat.textContent = t('checkAlt', { n: alts }); stat.classList.add('is-warn'); }
    else if (changed) { stat.textContent = t('converted', { n: changed }); stat.classList.add('is-ok'); }
    else stat.textContent = t('unchanged');
  }

  input.addEventListener('input', function () {
    clearTimeout(timer);
    // uzun matnda har tugmada emas, to'xtaganda
    if (input.value.length > 20000) timer = setTimeout(render, 150);
    else render();
  });

  $('clear').addEventListener('click', function () { input.value = ''; render(); input.focus(); });

  copyBtn.addEventListener('click', function () {
    if (last === null) return;
    navigator.clipboard.writeText(last).then(function () {
      var label = copyBtn.querySelector('span');
      label.textContent = t('copied');
      copyBtn.classList.add('is-done');
      setTimeout(function () { label.textContent = t('copy'); copyBtn.classList.remove('is-done'); }, 1400);
    }, function () {});
  });

  // ---------- til ----------
  document.querySelectorAll('[data-lang]').forEach(function (b) {
    b.addEventListener('click', function () {
      lang = bitgI18n.pick(b.getAttribute('data-lang'));
      api.storage.sync.set({ lang: lang });
      paintLang();
    });
  });

  // ---------- tezkor tugma ----------
  function browserKind() {
    if (api.runtime.getURL('').indexOf('moz-extension:') === 0) return 'firefox';
    return /\bEdg\//.test(navigator.userAgent) ? 'edge' : 'chrome';
  }
  var shortcutKey = '';
  function paintShortcut() {
    $('shortcut').textContent = shortcutKey ? t('shortcut', { key: shortcutKey }) : t('shortcutNone');
  }
  if (api.commands && api.commands.getAll) {
    api.commands.getAll().then(function (cmds) {
      var c = cmds.filter(function (x) { return x.name === 'convert-selection'; })[0];
      shortcutKey = (c && c.shortcut) || '';
      paintShortcut();
    }, function () {});
  }
  if (browserKind() !== 'firefox') {
    $('shortcut-edit').hidden = false;
    $('shortcut-edit').addEventListener('click', function () {
      api.tabs.create({ url: browserKind() === 'edge' ? 'edge://extensions/shortcuts' : 'chrome://extensions/shortcuts' });
    });
  }

  document.querySelectorAll('[data-link]').forEach(function (a) {
    a.addEventListener('click', function (e) { e.preventDefault(); api.tabs.create({ url: a.href }); window.close(); });
  });

  // ---------- shu sahifa ----------
  function setPage(active) {
    $('page').setAttribute('aria-pressed', String(active));
    $('page-label').textContent = t(active ? 'pageOff' : 'pageOn');
  }
  $('page').addEventListener('click', function () {
    if (!tab) return;
    $('page').disabled = true;
    api.runtime.sendMessage({ t: 'page', tabId: tab.id }).then(function (active) {
      setPage(!!active);
    }, function () {}).then(function () { $('page').disabled = false; });
  });

  var auto = $('auto');
  auto.addEventListener('click', function () {
    if (!pattern || !tab) return;
    if (auto.checked) {
      // permissions.request foydalanuvchi bosishi ichida, hech qanday await'dan oldin chaqirilishi shart
      api.permissions.request({ origins: [pattern] }).then(function (ok) {
        if (!ok) { auto.checked = false; return null; }
        return api.runtime.sendMessage({ t: 'siteAdd', pattern: pattern, tabId: tab.id });
      }).then(function (ok) { if (ok === false) auto.checked = false; }, function () { auto.checked = false; });
    } else {
      api.runtime.sendMessage({ t: 'siteRemove', pattern: pattern }).catch(function () {});
    }
  });

  function initSite(url) {
    $('site').hidden = false;
    var host = '';
    try { host = new URL(url).hostname; } catch (e) { host = ''; }
    $('host').textContent = host ? '· ' + host : '';
    if (bitgLib.restrictedUrl(url)) {
      $('restricted').hidden = false;
      $('site-tools').hidden = true;
      return;
    }
    pattern = bitgLib.originPattern(url);
    var docs = bitgLib.canvasEditor(url);
    $('docs').hidden = !docs;
    if (!pattern || docs) {
      auto.disabled = true;
      $('auto-ask').hidden = true;
    }
    api.runtime.sendMessage({ t: 'pageActive', tabId: tab.id }).then(function (a) { setPage(!!a); }, function () {});
    if (!pattern) return;
    Promise.all([api.storage.sync.get({ sites: [] }), api.permissions.contains({ origins: [pattern] })]).then(function (v) {
      var on = v[1] && Array.isArray(v[0].sites) && v[0].sites.indexOf(pattern) >= 0;
      auto.checked = on && !docs;
      $('auto-ask').hidden = on || docs;
    }, function () {});
  }

  // ---------- ishga tushish ----------
  api.storage.sync.get({ lang: 'uz' }).then(function (s) {
    lang = bitgI18n.pick(s.lang);
    paintLang();
  }, function () { paintLang(); });

  api.tabs.query({ active: true, currentWindow: true }).then(function (tabs) {
    tab = tabs && tabs[0];
    if (tab && tab.url) initSite(tab.url);
  }, function () {});

  bitgCore.load(api.runtime.getURL('core/bitg_core.wasm')).then(function (c) {
    core = c;
    render();
  }, function () {
    out.classList.remove('is-empty');
    out.textContent = t('loadFailed');
  });

  input.focus();
})();
