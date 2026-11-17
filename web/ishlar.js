// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// /qilingan-ishlar/: raqamlar sanalib chiqadi, "bir bosish ichida" chizg'ichi scroll bilan kattalashadi,
// to'liq ro'yxat filtrlanadi. JS bo'lmasa sahifa to'liq o'qiladi (hamma raqam va band HTML'da bor).
(function () {
  'use strict';

  var doc = document;
  var $$ = function (s, r) { return Array.prototype.slice.call((r || doc).querySelectorAll(s)); };
  var still = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  var io = 'IntersectionObserver' in window;

  function fmt(n, dec) {
    var s = n.toFixed(dec);
    var p = s.split('.');
    p[0] = p[0].replace(/\B(?=(\d{3})+(?!\d))/g, ' ');
    return p.join(',');
  }

  // ---------- raqamlar sanalib chiqishi ----------
  function countUp(el) {
    var to = parseFloat(el.getAttribute('data-count'));
    var dec = parseInt(el.getAttribute('data-dec') || '0', 10);
    if (isNaN(to) || still) return;
    var t0 = null, dur = 1400;
    function step(t) {
      if (t0 === null) t0 = t;
      var k = Math.min((t - t0) / dur, 1);
      var e = 1 - Math.pow(1 - k, 4);
      el.textContent = fmt(to * e, dec);
      if (k < 1) requestAnimationFrame(step);
      else el.textContent = fmt(to, dec);
    }
    el.textContent = fmt(0, dec);
    requestAnimationFrame(step);
  }

  function onSeen(els, fn, margin) {
    if (!io) { els.forEach(fn); return; }
    var ob = new IntersectionObserver(function (es) {
      es.forEach(function (e) {
        if (e.isIntersecting) { ob.unobserve(e.target); fn(e.target); }
      });
    }, { rootMargin: margin || '0px 0px -12% 0px' });
    els.forEach(function (el) { ob.observe(el); });
  }

  onSeen($$('[data-count]'), countUp);

  // SVG ustunlari va halqalar: .nt-anim faqat JS bor va harakat ruxsat etilganda qo'yiladi
  var anim = $$('[data-anim]');
  if (!still) {
    anim.forEach(function (el) {
      $$('[data-off]', el).forEach(function (c) { c.style.setProperty('--nt-off', c.getAttribute('data-off')); });
      el.classList.add('nt-anim');
    });
    onSeen(anim, function (el) {
      requestAnimationFrame(function () { el.classList.add('is-in'); });
    });
  }

  // ---------- bir bosish ichida ----------
  var zoom = doc.querySelector('.nt-zoom');
  if (zoom) {
    var FULL = 100000;          // µs: ikki tugma bosish orasidagi odatiy vaqt
    var WORK = parseFloat(zoom.getAttribute('data-work')) || 2.7; // µs: bitg bir so'zni o'girishi
    var END = 25;               // µs: oxirgi kadrda ekran kengligi
    var slice = zoom.querySelector('.nt-slice');
    var spanEl = zoom.querySelector('[data-span]');
    var timesEl = zoom.querySelector('[data-times]');
    var bpos = zoom.querySelector('.nt-bpos');
    var gap = zoom.querySelector('.nt-gap');
    var ticks = $$('.nt-tl', zoom);

    var human = function (us) {
      if (us >= 1000) return fmt(us / 1000, us >= 10000 ? 0 : 1) + ' ms';
      return fmt(us, us >= 100 ? 0 : 1) + ' µs';
    };

    var paint = function (p) {
      var span = FULL * Math.pow(END / FULL, p);
      var frac = Math.min(WORK / span, 1);
      slice.style.transform = 'scaleX(' + Math.max(frac, 0.0008).toFixed(5) + ')';
      // 2-tugma: span = FULL bo'lganda o'ng chetda, kattalashgan sari o'ngga chiqib ketadi
      var out = Math.min(FULL / span, 4);
      if (bpos) bpos.style.transform = 'translateX(' + (out * 100).toFixed(2) + '%)';
      if (gap) gap.style.transform = 'scaleX(' + out.toFixed(4) + ')';
      ticks.forEach(function (el, i) { el.textContent = i === 0 ? '0' : human(span * i / 4); });
      if (spanEl) spanEl.textContent = human(span);
      if (timesEl) timesEl.textContent = fmt(Math.round(FULL / span), 0) + '×';
      zoom.classList.toggle('is-close', frac > 0.04);
    };

    if (still) {
      zoom.classList.add('is-static');
      paint(1);
    } else {
      var queued = false;
      var update = function () {
        queued = false;
        var r = zoom.getBoundingClientRect();
        var total = r.height - window.innerHeight;
        var p = total > 0 ? Math.min(Math.max(-r.top / total, 0), 1) : 1;
        // boshida va oxirida biroz to'xtash: o'quvchi ikkala holatni ko'rib ulguradi
        p = Math.min(Math.max((p - 0.08) / 0.8, 0), 1);
        p = p * p * (3 - 2 * p);
        paint(p);
      };
      var req = function () { if (!queued) { queued = true; requestAnimationFrame(update); } };
      zoom.classList.remove('is-static');
      window.addEventListener('scroll', req, { passive: true });
      window.addEventListener('resize', req);
      update();
    }
  }

  // ---------- to'liq ro'yxat: filtr va qidiruv ----------
  var log = doc.querySelector('.nt-log');
  var tools = doc.querySelector('.nt-tools');
  if (log && tools) {
    tools.hidden = false;
    var items = $$('.nt-it', log);
    var days = $$('.nt-day', log);
    var countEl = tools.querySelector('.nt-count');
    var empty = log.querySelector('.nt-empty');
    var search = tools.querySelector('.nt-search');
    var state = { cat: 'all', st: 'all', q: '' };

    var norm = function (s) {
      return (s || '').toLowerCase().replace(/[‘’ʻʼ'`]/g, '')
        .replace(/ö/g, 'o').replace(/ğ/g, 'g').replace(/ş/g, 'sh').replace(/ç/g, 'ch');
    };
    items.forEach(function (it) { it.setAttribute('data-q', norm(it.textContent)); });

    var apply = function () {
      var q = norm(state.q.trim()), n = 0;
      items.forEach(function (it) {
        var ok = (state.cat === 'all' || it.getAttribute('data-cat') === state.cat)
          && (state.st === 'all' || it.getAttribute('data-st') === state.st)
          && (!q || it.getAttribute('data-q').indexOf(q) !== -1);
        it.hidden = !ok;
        if (ok) n++;
      });
      days.forEach(function (d) { d.hidden = !d.querySelector('.nt-it:not([hidden])'); });
      if (countEl) countEl.textContent = countEl.getAttribute('data-tpl').replace('{n}', n);
      if (empty) empty.hidden = n !== 0;
    };

    $$('[data-f]', tools).forEach(function (b) {
      b.addEventListener('click', function () {
        var key = b.getAttribute('data-f');
        state[key] = b.getAttribute('data-v');
        $$('[data-f="' + key + '"]', tools).forEach(function (o) {
          o.setAttribute('aria-pressed', o === b ? 'true' : 'false');
        });
        apply();
      });
    });
    if (search) search.addEventListener('input', function () { state.q = search.value; apply(); });
    apply();
  }
})();
