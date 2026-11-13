// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Belgilangan matnni o'girish (kontekst menyu yoki tezkor tugma). Tahrir maydonida — joyida almashtiriladi,
// oddiy matnda — o'girilgani buferga nusxalanadi. Talab bo'yicha `scripting.executeScript` bilan qo'yiladi
// (activeTab): doimiy kontent skript yo'q.
(function () {
  'use strict';
  if (window.bitgSelection) return;
  var L = window.bitgLib, E = window.bitgEdit;

  function toast(m) { if (window.bitgToast) window.bitgToast(m); }

  async function inControl(el, msg) {
    var s = el.selectionStart, e = el.selectionEnd;
    if (s === null || s === undefined) return 'skip';
    if (s === e) { s = 0; e = el.value.length; } // belgilanmagan bo'lsa — butun maydon
    var src = el.value.slice(s, e);
    if (!L.hasLetters(src)) { toast(msg.toastSelect); return 'empty'; }
    var out = await E.convert(src);
    if (typeof out !== 'string') return 'error';
    if (out === src) { toast(msg.toastNothing); return 'same'; }
    if (el.value.slice(s, e) !== src) { toast(msg.toastStale); return 'stale'; }
    E.replaceInControl(el, s, e, out);
    el.setSelectionRange(s, s + out.length);
    toast(msg.toastReplaced);
    return 'replaced';
  }

  function textNodesIn(range) {
    var rootNode = range.commonAncestorContainer;
    if (rootNode.nodeType === 3) return [rootNode];
    var out = [], w = document.createTreeWalker(rootNode, NodeFilter.SHOW_TEXT);
    while (w.nextNode()) if (range.intersectsNode(w.currentNode)) out.push(w.currentNode);
    return out;
  }

  async function inRich(sel, msg) {
    var range = sel.getRangeAt(0).cloneRange();
    var nodes = textNodesIn(range), parts = [];
    nodes.forEach(function (n) {
      var p = L.textPiece(n.data.length, n === range.startContainer, range.startOffset,
        n === range.endContainer, range.endOffset);
      if (p && L.hasLetters(n.data.slice(p.from, p.to))) parts.push({ node: n, from: p.from, to: p.to, src: n.data.slice(p.from, p.to) });
    });
    if (!parts.length) { toast(msg.toastSelect); return 'empty'; }
    var outs = await E.convertMany(parts.map(function (p) { return p.src; }));
    if (!Array.isArray(outs)) return 'error';
    var changed = parts.filter(function (p, i) { p.out = outs[i]; return typeof p.out === 'string' && p.out !== p.src; });
    if (!changed.length) { toast(msg.toastNothing); return 'same'; }
    if (changed.some(function (p) { return !p.node.isConnected || p.node.data.slice(p.from, p.to) !== p.src; })) {
      toast(msg.toastStale);
      return 'stale';
    }
    if (parts.length === 1) {
      // bitta tugun — muharrir uchun odatdagi kiritish (bekor qilish tarixi bilan)
      var p = parts[0], r = document.createRange();
      r.setStart(p.node, p.from);
      r.setEnd(p.node, p.to);
      E.replaceRangeRich(r, p.out);
    } else {
      // bir nechta tugun (qalin/kursiv aralash) — formatlash saqlanishi uchun har tugun alohida
      changed.forEach(function (q) { q.node.data = q.node.data.slice(0, q.from) + q.out + q.node.data.slice(q.to); });
      var host = range.commonAncestorContainer;
      host = host.nodeType === 1 ? host : host.parentElement;
      var ce = host && host.closest('[contenteditable]');
      if (ce) ce.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertReplacementText' }));
    }
    toast(msg.toastReplaced);
    return 'replaced';
  }

  async function copy(text) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch (e) {
      // zaxira: clipboardWrite ruxsati bilan execCommand('copy'); foydalanuvchi belgilagani qaytariladi
      var sel = window.getSelection(), saved = [];
      for (var i = 0; i < sel.rangeCount; i++) saved.push(sel.getRangeAt(i));
      var ta = document.createElement('textarea');
      ta.setAttribute('data-bitg-skip', '');
      ta.style.cssText = 'position:fixed;top:0;left:0;width:1px;height:1px;opacity:0;';
      ta.value = text;
      (document.body || document.documentElement).appendChild(ta);
      ta.select();
      var ok = false;
      try { ok = document.execCommand('copy'); } catch (err) { ok = false; }
      ta.remove();
      sel.removeAllRanges();
      saved.forEach(function (r) { sel.addRange(r); });
      return ok;
    }
  }

  /** `strict` — bir nechta freymga qo'yilganda faqat fokusdagisi ishlaydi. */
  async function run(msg, strict) {
    if (strict && !document.hasFocus()) return 'skip';
    var el = E.activeElement();
    // fokus ichki freymda — u freym o'zi ishlaydi
    if (strict && el && /^(IFRAME|FRAME)$/.test(el.tagName)) return 'skip';
    if (el && L.isTextControl(el)) return inControl(el, msg);
    var sel = window.getSelection();
    var text = sel && sel.rangeCount ? sel.toString() : '';
    if (el && el.isContentEditable && sel && sel.rangeCount) {
      if (!L.hasLetters(text)) { toast(msg.toastSelect); return 'empty'; }
      return inRich(sel, msg);
    }
    if (!L.hasLetters(text)) { toast(msg.toastSelect); return 'empty'; }
    var out = await E.convert(text);
    if (typeof out !== 'string') return 'error';
    if (await copy(out)) toast(out === text ? msg.toastNothing : msg.toastCopied);
    else toast(msg.toastCopyFailed);
    return 'copied';
  }

  window.bitgSelection = { run: run };
})();
