// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// bitg.uz yuklab olish: tashrif buyuruvchining tizimini aniqlab, mos o'rnatuvchini birinchi qo'yadi.
// Sahifa JS'siz ham to'liq ishlaydi (Windows standart, barcha havolalar to'g'ri fayllarga).
// JS bo'lsa: yuklash tugmasi /yuklab-olish/rahmat/ sahifasini ochadi, fayl o'sha yerdan yuklanadi.
(function () {
  'use strict';

  var doc = document, nav = navigator;
  var $ = function (s, r) { return (r || doc).querySelector(s); };
  var $$ = function (s, r) { return Array.prototype.slice.call((r || doc).querySelectorAll(s)); };
  // faqat shu saytdagi yo'l ("/..."); boshqa qiymat havolaga qo'yilmaydi
  var local = function (u) { return typeof u === 'string' && /^\/(?![\/\\])[\w\-.\/#?=&%]*$/.test(u) ? u : null; };

  // ---------- tizimni aniqlash ----------
  // userAgentData (Chromium) aniqroq; bo'lmasa userAgent. iPad Safari o'zini Mac deb ataydi:
  // sensorli ekranli "Mac" = iPad.
  function detect() {
    var ua = nav.userAgent || '';
    var p = ((nav.userAgentData && nav.userAgentData.platform) || nav.platform || '').toLowerCase();
    var mobile = nav.userAgentData ? nav.userAgentData.mobile : /Mobi|Android|iPhone|iPod/.test(ua);
    if (/android/i.test(ua) || p === 'android') return 'phone';
    if (/iPhone|iPad|iPod/.test(ua) || p === 'ios') return 'phone';
    if ((p.indexOf('mac') === 0 || /Macintosh/.test(ua)) && nav.maxTouchPoints > 1) return 'phone';
    if (mobile) return 'phone';
    if (/CrOS/.test(ua) || p === 'chrome os' || p === 'chromeos') return 'other';
    if (p.indexOf('win') === 0 || /Windows NT/.test(ua)) return 'win';
    if (p.indexOf('mac') === 0 || /Macintosh|Mac OS X/.test(ua)) return 'mac';
    if (p.indexOf('linux') === 0 || /Linux|X11/.test(ua)) return 'linux';
    return 'other';
  }

  var os = detect();
  try {
    // sinov uchun: ?tizim=mac
    var forced = new URLSearchParams(location.search).get('tizim');
    if (/^(win|mac|linux|phone|other)$/.test(forced || '')) os = forced;
  } catch (e) {}
  doc.documentElement.setAttribute('data-os', os);

  // Windows ARM (Snapdragon): x64 kiritish usuli ARM64 dasturlarga yuklanmasligi mumkin — ogohlantiramiz.
  function armWindows() {
    if (os !== 'win' || !nav.userAgentData || !nav.userAgentData.getHighEntropyValues) return Promise.resolve(false);
    return nav.userAgentData.getHighEntropyValues(['architecture']).then(function (v) {
      return v.architecture === 'arm';
    }, function () { return false; });
  }
  function put(slot, name) {
    var t = slot && $('template[data-os-tpl="' + name + '"]');
    if (t) slot.appendChild(t.content.cloneNode(true));
    return !!t;
  }

  // ---------- yuklash tugmasi -> rahmat sahifasi ----------
  // Ctrl/Shift/o'rta tugma bilan bosilsa — brauzerning odatiy xatti-harakati (faylni o'zi yuklaydi).
  doc.addEventListener('click', function (e) {
    var a = e.target.closest && e.target.closest('a[data-dl]');
    if (!a || e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    e.preventDefault();
    location.href = '/yuklab-olish/rahmat/?os=' + encodeURIComponent(a.getAttribute('data-dl'));
  });

  // ---------- "kompyuterga yuborish" (telefondan kirganlar uchun) ----------
  doc.addEventListener('click', function (e) {
    var b = e.target.closest && e.target.closest('[data-share]');
    if (!b) return;
    var url = b.getAttribute('data-share');
    var lbl = b.lastElementChild || b;
    var done = function () {
      var o = lbl.textContent; lbl.textContent = b.getAttribute('data-done') || '✓';
      setTimeout(function () { lbl.textContent = o; }, 1600);
    };
    if (nav.share) { nav.share({ title: 'bitg', url: url }).then(function () {}, function () {}); return; }
    if (nav.clipboard) nav.clipboard.writeText(url).then(done, function () {});
  });

  // ---------- bosh sahifa ----------
  var plat = $('.bg-plat');
  if (plat) {
    var cards = $$('.bg-plat__card', plat);
    var pick = os === 'other' ? null : $('.bg-plat__card[data-os="' + os + '"]', plat);
    if (pick && !pick.classList.contains('bg-plat__card--main')) {
      cards.forEach(function (c) {
        var on = c === pick;
        c.classList.toggle('bg-plat__card--main', on);
        var go = $('.bg-plat__go', c);
        if (go) { go.classList.toggle('bg-btn--pri', on); go.classList.toggle('bg-btn--sm', !on); }
        // asosiy karta to'g'ridan-to'g'ri faylga, qolganlari yuklab olish sahifasidagi bo'limga
        var file = local(c.getAttribute('data-file'));
        var page = local(c.getAttribute('data-page'));
        if (file && page) {
          if (on) {
            c.href = file; c.setAttribute('download', ''); c.setAttribute('data-dl', c.getAttribute('data-os'));
          } else {
            c.href = page; c.removeAttribute('download'); c.removeAttribute('data-dl');
          }
        }
      });
      plat.insertBefore(pick, plat.firstChild);
    }

    // hero tugmasi va pastdagi eslatma tizimga mos
    var cta = $('#bg-cta'), ctaTxt = $('#bg-cta-txt'), note = $('#bg-dl-note');
    if (cta && ctaTxt) {
      var label = cta.getAttribute('data-label-' + os);
      if (label) ctaTxt.textContent = label;
      if (os === 'phone') cta.setAttribute('href', '/yangi-harflarni-yozish/#telefon');
    }
    if (note && $('template[data-os-tpl="' + os + '"]')) { note.textContent = ''; put(note, os); }
    armWindows().then(function (arm) { if (arm) put(note, 'win-arm'); });

    // ochiq hisoblagich: /stats.json (statistika xizmati yozadi). Son kichik bo'lsa ko'rsatilmaydi.
    var cnt = $('#bg-count');
    if (cnt && window.fetch) {
      fetch('/stats.json', { cache: 'no-cache' }).then(function (r) { return r.ok ? r.json() : null; }).then(function (s) {
        var n = s && +s.downloads;
        if (!(n >= 500)) return;
        var shown = n >= 10000 ? Math.floor(n / 1000) * 1000 : Math.floor(n / 100) * 100;
        $('b', cnt).textContent = String(shown).replace(/\B(?=(\d{3})+(?!\d))/g, ' ') + '+';
        cnt.hidden = false;
      }, function () {});
    }
  }

  // ---------- yuklab olish sahifasi: "sizning tizimingiz" eslatmasi ----------
  put($('[data-os-slot]'), 'hint-' + os);

  // ---------- rahmat sahifasi: fayl shu yerdan yuklanadi, o'rnatish qadamlari ko'rsatiladi ----------
  var ty = $('#dc-ty');
  if (ty) {
    var want = '';
    try { want = new URLSearchParams(location.search).get('os') || ''; } catch (e) {}
    if (['win', 'mac', 'linux', 'fcitx'].indexOf(want) < 0) return; // to'g'ridan-to'g'ri ochildi: barcha bo'limlar ko'rinib turadi
    var src = $('[data-file-for="' + want + '"]', ty);
    var file = src && local(src.getAttribute('href'));
    if (!file) return;
    ty.setAttribute('data-show', want === 'fcitx' ? 'linux' : want);
    $$('[data-direct]', ty).forEach(function (a) { a.href = file; });
    $$('[data-file-name]', ty).forEach(function (n) { n.textContent = src.getAttribute('data-name'); });
    var state = $('#dc-ty-state');
    // sahifa yangilansa fayl ikkinchi marta yuklanmasin
    var key = 'bitg-dl:' + file, again = false;
    try { again = sessionStorage.getItem(key) === '1'; sessionStorage.setItem(key, '1'); } catch (e) {}
    if (!again) setTimeout(function () { location.assign(file); }, 400);
    if (state) state.textContent = state.getAttribute(again ? 'data-again' : 'data-start');
  }
})();
