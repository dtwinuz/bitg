/* redesign2.bitg.uz — tab/theme/abc switching, hero typer, fuse, playground (real WASM core), reveal, 3D loader. */
const V = new URL(import.meta.url).searchParams.get('v') || '';
const root = document.documentElement;
root.classList.add('r2-js'); // reveal animatsiyasi faqat JS bor bo'lsa yashiradi
const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => [...r.querySelectorAll(s)];
const store = {
  get(k){ try { return localStorage.getItem(k); } catch { return null; } },
  set(k,v){ try { localStorage.setItem(k,v); } catch {} },
};

/* ---------- direction tabs (A/B/C) ---------- */
{
  const tabs = $$('[data-dir-btn]');
  const url = new URL(location.href);
  const initial = url.searchParams.get('dir') || (location.hash.match(/^#([abc])$/)||[])[1] || store.get('r2-dir') || 'a';
  const apply = (d, push) => {
    if (!['a','b','c'].includes(d)) d = 'a';
    root.setAttribute('data-dir', d);
    tabs.forEach(b => b.setAttribute('aria-selected', b.dataset.dirBtn === d));
    store.set('r2-dir', d);
    if (push) { const u = new URL(location.href); u.searchParams.set('dir', d); history.replaceState(null,'',u); }
    document.dispatchEvent(new CustomEvent('r2:dir', { detail: d }));
  };
  tabs.forEach(b => b.addEventListener('click', () => apply(b.dataset.dirBtn, true)));
  apply(initial, false);
}

/* ---------- theme ---------- */
{
  const btn = $('[data-theme-btn]');
  const saved = store.get('r2-theme');
  if (saved) root.setAttribute('data-theme', saved);
  btn?.addEventListener('click', () => {
    const dark = root.getAttribute('data-theme') === 'dark'
      || (!root.getAttribute('data-theme') && matchMedia('(prefers-color-scheme: dark)').matches);
    const next = dark ? 'light' : 'dark';
    root.setAttribute('data-theme', next); store.set('r2-theme', next);
  });
}

/* ---------- bitg yadrosi (Rust → WASM), sayt bilan bir xil ---------- */
let corePromise = null;
function core() {
  if (corePromise) return corePromise;
  const url = '/bitg_core.wasm' + (V ? '?v=' + V : '');
  const slow = () => fetch(url).then(r => { if (!r.ok) throw new Error('wasm ' + r.status); return r.arrayBuffer(); })
    .then(b => WebAssembly.instantiate(b, {}));
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
    return { toNew: t => (run(t, 0) || { out: t }).out, toOld: t => { const r = run(t, 1); return r ? r.out : t; } };
  });
  corePromise.catch(() => { corePromise = null; });
  return corePromise;
}
const idle = f => (window.requestIdleCallback || (c => setTimeout(c, 400)))(f);

/* ---------- sahifa alifbosi: Yangi / Eski ---------- */
{
  const btns = $$('[data-abc-btn]');
  let abc = root.getAttribute('data-abc') === 'new' ? 'new' : 'old';
  let orig = null, api = null;
  const nodes = () => {
    const out = [], w = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, {
      acceptNode(n){
        if (!n.data.trim()) return NodeFilter.FILTER_REJECT;
        const p = n.parentElement;
        if (!p || p.closest('[data-no-conv],script,style,kbd,code,svg')) return NodeFilter.FILTER_REJECT;
        return NodeFilter.FILTER_ACCEPT;
      }
    });
    while (w.nextNode()) out.push(w.currentNode);
    return out;
  };
  const paint = () => {
    if (!orig) { orig = new Map(); nodes().forEach(n => orig.set(n, n.data)); }
    orig.forEach((txt, n) => { n.data = (abc === 'new' && api) ? api.toNew(txt) : txt; });
  };
  const set = v => {
    abc = v; root.setAttribute('data-abc', v);
    btns.forEach(b => b.setAttribute('aria-pressed', b.dataset.abcBtn === v));
    if (v === 'new' && !api) core().then(a => { api = a; paint(); }); else paint();
  };
  btns.forEach(b => b.addEventListener('click', () => set(b.dataset.abcBtn)));
}

/* ---------- hero typer ---------- */
{
  const el = $('[data-typer]'); const caret = '<span class="r2-caret"></span>';
  const phrases = ['Bugun shaharda uchrashuv bor', "O'zbekiston tog'lari", 'Please check — keyin choy', 'push() va @bitguzbot'];
  if (el && !matchMedia('(prefers-reduced-motion: reduce)').matches) {
    core().then(api => {
      let pi = 0;
      const esc = s => s.replace(/[&<>]/g, c => ({ '&':'&amp;','<':'&lt;','>':'&gt;' }[c]));
      const step = async () => {
        const src = phrases[pi % phrases.length]; pi++;
        for (let i = 1; i <= src.length; i++) { el.innerHTML = esc(src.slice(0, i)) + caret; await wait(45); }
        await wait(550);
        el.innerHTML = '<span class="r2-new">' + esc(api.toNew(src)) + '</span>' + caret;
        await wait(1700);
        step();
      };
      step();
    });
  }
  const wait = ms => new Promise(r => setTimeout(r, ms));
}

/* ---------- fuse letters ---------- */
$$('[data-fuse] li').forEach(li => {
  const from = li.dataset.from, to = li.dataset.to;
  li.addEventListener('click', () => {
    if (matchMedia('(prefers-reduced-motion: reduce)').matches) { li.firstChild.textContent = li.firstChild.textContent === from ? to : from; return; }
    li.animate([{ transform:'scale(1)' }, { transform:'scale(.82)', offset:.5 }, { transform:'scale(1)' }], { duration:420, easing:'cubic-bezier(.22,.61,.36,1)' });
    const cur = li.childNodes[0];
    setTimeout(() => { cur.textContent = cur.textContent.trim() === from ? to : from; }, 180);
  });
});

/* ---------- playground (real WASM) ---------- */
{
  const play = $('[data-play]');
  if (play) {
    const input = $('[data-in]', play), out = $('[data-out]', play), modes = $$('[data-mode]', play);
    let mode = '0', api = null, timer = 0;
    const render = () => {
      if (!api) { out.textContent = 'yadro yuklanmoqda…'; return; }
      const t = input.value;
      out.textContent = (mode === '1') ? api.toOld(t) : api.toNew(t);
    };
    const debounce = () => { clearTimeout(timer); timer = setTimeout(render, 90); };
    input.addEventListener('input', debounce);
    modes.forEach(b => b.addEventListener('click', () => {
      mode = b.dataset.mode; modes.forEach(m => m.setAttribute('aria-selected', m === b)); render();
    }));
    $$('[data-ex]', play).forEach(c => c.addEventListener('click', () => {
      input.value = c.dataset.ex;
      const m = c.dataset.exMode || '0';
      const mb = modes.find(x => x.dataset.mode === m); if (mb) mb.click(); else render();
    }));
    const load = () => core().then(a => { api = a; render(); });
    new IntersectionObserver((e, o) => { if (e[0].isIntersecting) { o.disconnect(); load(); } }, { rootMargin:'200px' }).observe(play);
  }
}

/* ---------- reveal on scroll ---------- */
{
  const io = new IntersectionObserver((es, o) => es.forEach(e => { if (e.isIntersecting) { e.target.classList.add('is-in'); o.unobserve(e.target); } }), { rootMargin:'0px 0px -8% 0px' });
  $$('.r2-rev').forEach(el => io.observe(el));
}

/* ---------- 3D keyboard (lazy; next phase wires scroll-journey) ---------- */
{
  const stage = $('[data-stage]'), canvas = $('[data-kb]'), fallback = $('[data-stage-fallback]');
  const ok = (() => { try { return !!document.createElement('canvas').getContext('webgl2'); } catch { return false; } })();
  if (stage && canvas && ok) {
    const go = () => import(`/redesign2/kb3d.js${V ? '?v=' + V : ''}`)
      .then(m => m.start?.(canvas, { dir: () => root.getAttribute('data-dir') }))
      .catch(() => { canvas.hidden = true; fallback.hidden = false; });
    if (document.readyState === 'complete') idle(go); else addEventListener('load', () => idle(go), { once:true });
  } else if (canvas) { canvas.hidden = true; if (fallback) fallback.hidden = false; }
}
