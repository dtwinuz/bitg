// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Kichik xabarcha: yopiq shadow DOM ichida, uslub CSSOM orqali (sahifa CSP'si inline <style> ni to'sishi mumkin).
(function () {
  'use strict';
  if (window.bitgToast) return;
  var host = null, box = null, timer = 0;

  function make() {
    host = document.createElement('div');
    host.setAttribute('data-bitg-skip', '');
    host.style.cssText = 'all:initial;position:fixed;z-index:2147483647;left:50%;bottom:24px;' +
      'transform:translateX(-50%);pointer-events:none;';
    var shadow = host.attachShadow({ mode: 'closed' });
    box = document.createElement('div');
    box.setAttribute('role', 'status');
    box.setAttribute('aria-live', 'polite');
    var dark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
    box.style.cssText = [
      'font:500 13px/1.4 Inter,"Segoe UI",system-ui,-apple-system,sans-serif',
      'max-width:min(440px,calc(100vw - 32px))', 'padding:10px 14px', 'border-radius:10px',
      'display:flex', 'gap:10px', 'align-items:center',
      dark ? 'background:#161922;color:#ecebe6;border:1px solid #363d4c'
        : 'background:#1b1e26;color:#f3efe7;border:1px solid #1b1e26',
      'box-shadow:0 1px 0 rgb(20 24 32 / .04),0 18px 40px -18px rgb(20 24 32 / .45)',
      'opacity:0', 'transform:translateY(6px)', 'transition:opacity .18s ease,transform .18s ease'
    ].join(';');
    var dot = document.createElement('span');
    dot.style.cssText = 'flex:none;width:7px;height:7px;border-radius:50%;background:#8fb0ea';
    var txt = document.createElement('span');
    box.appendChild(dot);
    box.appendChild(txt);
    shadow.appendChild(box);
  }

  window.bitgToast = function (message) {
    if (!message) return;
    if (!host) make();
    if (!host.isConnected) (document.body || document.documentElement).appendChild(host);
    box.lastChild.textContent = message;
    // keyingi kadrda ko'rinadi (o'tish animatsiyasi uchun)
    requestAnimationFrame(function () { box.style.opacity = '1'; box.style.transform = 'translateY(0)'; });
    clearTimeout(timer);
    timer = setTimeout(function () {
      box.style.opacity = '0';
      box.style.transform = 'translateY(6px)';
      setTimeout(function () { if (box.style.opacity === '0' && host) host.remove(); }, 250);
    }, 2600);
  };
})();
