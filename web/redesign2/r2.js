/* SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
 * SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * redesign2.bitg.uz — signature (jonli harf-specimen), real WASM playground,
 * kontekst-demo, countdown, tezlik-bar, yumshoq motion. CSP-xavfsiz:
 * inline style YO'Q; o'lcham/transform faqat CSSOM (el.style) va klass orqali.
 */
const V = new URL(import.meta.url).searchParams.get('v') || '';
const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));
const root = document.documentElement;
const RM = matchMedia('(prefers-reduced-motion: reduce)').matches;
root.classList.add('r2-js');

/* ---------- theme ---------- */
(() => {
  let saved = null;
  try { saved = localStorage.getItem('r2-theme'); } catch (e) {}
  if (saved) root.setAttribute('data-theme', saved);
  const btn = $('[data-theme-btn]');
  if (!btn) return;
  btn.addEventListener('click', () => {
    const dark = matchMedia('(prefers-color-scheme: dark)').matches;
    const cur = root.getAttribute('data-theme') || (dark ? 'dark' : 'light');
    const next = cur === 'dark' ? 'light' : 'dark';
    root.setAttribute('data-theme', next);
    try { localStorage.setItem('r2-theme', next); } catch (e) {}
  });
})();

/* ---------- header stuck ---------- */
(() => {
  const top = $('[data-top]');
  if (!top) return;
  const on = () => top.classList.toggle('is-stuck', window.scrollY > 8);
  on(); addEventListener('scroll', on, { passive: true });
})();

/* ---------- bitg yadrosi (Rust -> WASM), sayt bilan bir xil fayl ---------- */
let corePromise = null;
function core() {
  if (corePromise) return corePromise;
  const url = '/bitg_core.wasm' + (V ? '?v=' + V : '');
  const slow = () => fetch(url).then((r) => { if (!r.ok) throw new Error('wasm ' + r.status); return r.arrayBuffer(); })
    .then((b) => WebAssembly.instantiate(b, {}));
  const p = WebAssembly.instantiateStreaming ? WebAssembly.instantiateStreaming(fetch(url), {}).catch(slow) : slow();
  corePromise = p.then(({ instance }) => {
    const x = instance.exports, enc = new TextEncoder(), dec = new TextDecoder();
    const run = (text, mode) => {
      const b = enc.encode(text);
      const ptr = x.bitg_alloc(b.length);
      new Uint8Array(x.memory.buffer, ptr, b.length).set(b);
      const n = x.bitg_run(ptr, b.length, mode, 0);
      x.bitg_dealloc(ptr, b.length);
      return n ? JSON.parse(dec.decode(new Uint8Array(x.memory.buffer, x.bitg_out(), n))) : null;
    };
    return {
      toNew: (t) => run(t, 0) || { out: t, toks: [] },
      toOld: (t) => { const r = run(t, 1); return r ? r.out : t; },
    };
  });
  corePromise.catch(() => { corePromise = null; });
  return corePromise;
}

/* ---------- seen (IntersectionObserver) ---------- */
function onSeen(els, cb, margin = '0px 0px -12% 0px', once = true) {
  if (!('IntersectionObserver' in window)) { els.forEach(cb); return; }
  const io = new IntersectionObserver((ents) => {
    ents.forEach((e) => { if (e.isIntersecting) { cb(e.target); if (once) io.unobserve(e.target); } });
  }, { rootMargin: margin, threshold: 0.12 });
  els.forEach((el) => io.observe(el));
}

/* ---------- reveal ---------- */
onSeen($$('.r2-rev'), (el) => el.classList.add('is-in'));

/* ---------- esc ---------- */
const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

/* ---------- count-up (proof numbers) ---------- */
onSeen($$('[data-count-to]'), (el) => {
  const to = parseFloat(el.getAttribute('data-count-to'));
  const suf = el.getAttribute('data-suffix') || '';
  const dec = (to % 1 !== 0) ? (String(to).split('.')[1] || '').length : 0;
  if (RM) { el.textContent = to.toFixed(dec).replace('.', ',') + suf; return; }
  const dur = 1100, t0 = performance.now();
  const step = (t) => {
    const k = Math.min(1, (t - t0) / dur);
    const e = 1 - Math.pow(1 - k, 3);
    el.textContent = (to * e).toFixed(dec).replace('.', ',') + suf;
    if (k < 1) requestAnimationFrame(step);
  };
  requestAnimationFrame(step);
});

/* ---------- speed bars ---------- */
onSeen($$('[data-speed]'), (box) => {
  $$('.r2-speed__fill', box).forEach((f, i) => {
    const w = f.getAttribute('data-w') || '0';
    setTimeout(() => { f.style.width = Math.max(3, +w) + '%'; }, RM ? 0 : 120 * i);
  });
});

/* ---------- countdown ---------- */
(() => {
  const DEADLINE = Date.UTC(2027, 3, 22, 0, 0, 0); // 2027-04-22, O'RQ-1184
  const box = $('[data-count]');
  const days = $$('[data-days]');
  const u = box ? {
    d: $('[data-unit="d"]', box), h: $('[data-unit="h"]', box),
    m: $('[data-unit="m"]', box), s: $('[data-unit="s"]', box),
  } : null;
  const pad = (n) => String(n).padStart(2, '0');
  const tick = () => {
    let diff = Math.max(0, DEADLINE - Date.now());
    const D = Math.floor(diff / 864e5); diff -= D * 864e5;
    const H = Math.floor(diff / 36e5); diff -= H * 36e5;
    const M = Math.floor(diff / 6e4); diff -= M * 6e4;
    const S = Math.floor(diff / 1e3);
    days.forEach((e) => { e.textContent = D.toLocaleString('uz'); });
    if (u) { u.d.textContent = D; u.h.textContent = pad(H); u.m.textContent = pad(M); u.s.textContent = pad(S); }
  };
  tick(); setInterval(tick, 1000);
})();

/* ---------- playground ---------- */
(() => {
  const play = $('[data-play]');
  if (!play) return;
  const input = $('[data-in]', play), out = $('[data-out]', play);
  const modes = $$('[data-mode]', play);
  let mode = '0', api = null, timer = 0;
  const mark = (p, cls) => cls ? `<span class="${cls}">${esc(p)}</span>` : esc(p);
  const render = () => {
    if (!api) return;
    const text = input.value;
    if (mode === '1') { out.textContent = api.toOld(text); return; }
    const r = api.toNew(text);
    const inW = text.split(/(\s+)/);
    const parts = r.out.split(/(\s+)/);
    const words = parts.filter((p, i) => i % 2 === 0 && p !== '');
    if (words.length !== r.toks.length) { out.textContent = r.out; return; }
    let k = 0;
    out.innerHTML = parts.map((p, i) => {
      if (i % 2 === 1 || p === '') return esc(p);
      const t = r.toks[k]; const why = t[1]; const orig = inW[i];
      k++;
      if (why === 'e' || why === 'p') return mark(p, 'r2-keep');
      if (p !== orig) return mark(p, 'r2-new');
      return esc(p);
    }).join('');
  };
  const setMode = (m) => {
    mode = m === 'cyr' ? '0' : m;
    modes.forEach((b) => b.setAttribute('aria-selected', String(b.getAttribute('data-mode') === m)));
  };
  modes.forEach((b) => b.addEventListener('click', () => {
    const m = b.getAttribute('data-mode'); setMode(m);
    if (m === 'cyr' && !/[Ѐ-ӿ]/.test(input.value)) input.value = 'Ўзбекистон Республикаси пойтахти — Тошкент шаҳри.';
    if (m === '1' && !/[öğşçÖĞŞÇ]/.test(input.value)) input.value = 'Özbekiston şaharlari: Toşkent, Samarqand. Çoyxonada çoy içdik.';
    if (m === '0' && /[öğşçÖĞŞÇЀ-ӿ]/.test(input.value)) input.value = "Bugun shaharda choy ichdik, keyin \"check\" qildim. O'zbekiston tog'lari haqida bitg.uz da o'qing.";
    render();
  }));
  $$('[data-ex]', play).forEach((b) => b.addEventListener('click', () => {
    setMode(b.getAttribute('data-ex-mode') || '0');
    input.value = b.getAttribute('data-ex'); render();
    input.focus({ preventScroll: true });
  }));
  input.addEventListener('input', () => { clearTimeout(timer); timer = setTimeout(render, 60); });
  onSeen([play], () => core().then((a) => { api = a; render(); })
    .catch(() => { out.textContent = 'yadro yuklanmadi — sahifani yangilang'; }), '500px 0px');
})();

/* ---------- context demo (differentiator) ---------- */
(() => {
  const box = $('[data-ctx]'); if (!box) return;
  const line = $('[data-ctx-line]', box);
  const SENT = "Toshkentda choyxonada o'tirib, @ustoz ga https://bitg.uz havolasini yubordim.";
  const build = (api) => {
    const r = api.toNew(SENT);
    const inW = SENT.split(/(\s+)/);
    const parts = r.out.split(/(\s+)/);
    const words = parts.filter((p, i) => i % 2 === 0 && p !== '');
    if (words.length !== r.toks.length) { line.textContent = r.out; return []; }
    let k = 0; const nodes = [];
    line.innerHTML = parts.map((p, i) => {
      if (i % 2 === 1 || p === '') return esc(p);
      const t = r.toks[k]; const why = t[1]; const orig = inW[i]; k++;
      let cls = 'r2-tok';
      if (why === 'e' || why === 'p') cls += ' r2-ctx-keep';
      else if (p !== orig) cls += ' r2-ctx-new';
      return `<span class="${cls}" data-old="${esc(orig)}" data-new="${esc(p)}">${esc(orig)}</span>`;
    }).join('');
    return $$('.r2-tok', line);
  };
  const play = (toks) => {
    if (!toks.length) return;
    toks.forEach((s, idx) => {
      const isNew = s.classList.contains('r2-ctx-new');
      const isKeep = s.classList.contains('r2-ctx-keep');
      setTimeout(() => {
        if (isKeep) { s.classList.add('is-keep'); return; }
        if (isNew) {
          s.textContent = s.getAttribute('data-new');
          s.classList.add('is-new');
          s.style.transform = 'translateY(-3px)';
          setTimeout(() => { s.style.transform = ''; }, 320);
        }
      }, RM ? 0 : 260 + idx * 230);
    });
  };
  onSeen([box], () => core().then((api) => play(build(api))).catch(() => { line.textContent = SENT; }), '200px 0px');
})();

/* ---------- signature: living type specimen ---------- */
(() => {
  const el = $('[data-specimen]'); if (!el) return;
  const SENTS = [
    "Shahar markazidagi choyxonada o'tirdik.",
    "O'zbekiston tog'lari — g'oyat go'zal manzara.",
    "Eski yozuv bugun yangicha chiqadi.",
  ];
  let api = null, idx = 0, alive = false;
  const renderPair = (sentence) => {
    const out = api.toNew(sentence);
    const inW = sentence.split(/(\s+)/);
    const nw = out.out.split(/(\s+)/);
    if (inW.length !== nw.length) { el.textContent = out.out; return []; }
    el.innerHTML = inW.map((p, i) => {
      if (i % 2 === 1 || p === '') return esc(p);
      const n = nw[i];
      return `<span class="r2-tok r2-old" data-new="${esc(n)}" data-chg="${p !== n ? 1 : 0}">${esc(p)}</span>`;
    }).join('');
    return $$('.r2-tok', el);
  };
  const cycle = () => {
    if (!alive) return;
    const toks = renderPair(SENTS[idx]);
    const changed = toks.filter((t) => t.getAttribute('data-chg') === '1');
    let d = 420;
    toks.forEach((t) => {
      if (t.getAttribute('data-chg') !== '1') return;
      setTimeout(() => {
        t.textContent = t.getAttribute('data-new');
        t.classList.remove('r2-old'); t.classList.add('r2-new');
        t.style.transform = 'translateY(-4px) scale(1.02)';
        setTimeout(() => { t.style.transform = ''; }, 360);
      }, d);
      d += 300;
    });
    idx = (idx + 1) % SENTS.length;
    setTimeout(() => { el.style.opacity = '0'; setTimeout(() => { el.style.opacity = ''; cycle(); }, 480); }, d + 2400);
  };
  onSeen([el], () => core().then((a) => {
    api = a; alive = true;
    if (RM) { el.innerHTML = ''; const r = api.toNew(SENTS[0]); el.innerHTML = `<span class="r2-new">${esc(r.out)}</span>`; return; }
    el.style.transition = 'opacity .5s var(--soft)';
    cycle();
  }).catch(() => {}), '300px 0px');
})();

/* ---------- page alphabet toggle (Yangi / Eski) ---------- */
(() => {
  const btns = $$('[data-abc-btn]'); if (!btns.length) return;
  const roots = ['main', 'header', 'footer'].map((s) => $(s)).filter(Boolean);
  const stash = new WeakMap();
  const nodes = () => {
    const list = [];
    roots.forEach((r) => {
      const w = document.createTreeWalker(r, NodeFilter.SHOW_TEXT, {
        acceptNode(n) {
          if (!n.data.trim()) return NodeFilter.FILTER_REJECT;
          let p = n.parentElement;
          while (p) { if (p.hasAttribute('data-no-conv') || p.tagName === 'SCRIPT' || p.tagName === 'STYLE') return NodeFilter.FILTER_REJECT; p = p.parentElement; }
          return NodeFilter.FILTER_ACCEPT;
        },
      });
      let n; while ((n = w.nextNode())) list.push(n);
    });
    return list;
  };
  let cached = null;
  const apply = (which) => {
    const setPressed = () => btns.forEach((b) => b.setAttribute('aria-pressed', String(b.getAttribute('data-abc-btn') === which)));
    root.setAttribute('data-abc', which);
    setPressed();
    try { localStorage.setItem('r2-abc', which); } catch (e) {}
    if (which === 'old') { // restore
      if (!cached) return;
      cached.forEach((n) => { if (stash.has(n)) n.data = stash.get(n); });
      return;
    }
    core().then((api) => {
      cached = cached || nodes();
      cached.forEach((n) => {
        if (!stash.has(n)) stash.set(n, n.data);
        n.data = api.toNew(stash.get(n)).out;
      });
    }).catch(() => {});
  };
  btns.forEach((b) => b.addEventListener('click', () => apply(b.getAttribute('data-abc-btn'))));
  let saved = null; try { saved = localStorage.getItem('r2-abc'); } catch (e) {}
  if (saved === 'new') apply('new');
})();
