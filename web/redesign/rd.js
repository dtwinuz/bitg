// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// redesign.bitg.uz: hero yozuvi (natijalar bitg yadrosidan olingan), 3D klaviatura (kb3d.js, three.js),
// sinov maydoni (bitg_core.wasm — ilovalardagi bilan bir xil yadro), tablar, sanoq, sahifani yangi alifboga o'girish.
// CSP: inline style yo'q — faqat klass va el.style.setProperty.

const doc = document;
const root = doc.documentElement;
const $ = (s, r = doc) => r.querySelector(s);
const $$ = (s, r = doc) => Array.from(r.querySelectorAll(s));
const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
const V = (new URL(import.meta.url)).searchParams.get('v') || '';
root.classList.add('rd-js');

const store = {
  get(k) { try { return localStorage.getItem(k); } catch { return null; } },
  set(k, v) { try { localStorage.setItem(k, v); } catch { /* xususiy oyna */ } },
};

function fmt(n, dec = 0) {
  const p = n.toFixed(dec).split('.');
  p[0] = p[0].replace(/\B(?=(\d{3})+(?!\d))/g, ' ');
  return p.join(',');
}

function onSeen(els, fn, margin = '0px 0px -12% 0px') {
  if (!('IntersectionObserver' in window)) { els.forEach(fn); return; }
  const io = new IntersectionObserver((es) => es.forEach((e) => {
    if (e.isIntersecting) { io.unobserve(e.target); fn(e.target); }
  }), { rootMargin: margin });
  els.forEach((el) => io.observe(el));
}

// ---------- qonun kuchga kirishiga qolgan kunlar (2027-04-22, Toshkent) ----------
{
  const left = Math.max(0, Math.ceil((Date.parse('2027-04-22T00:00:00+05:00') - Date.now()) / 864e5));
  $$('[data-days]').forEach((el) => { el.textContent = fmt(left); });
}

// ---------- header ----------
{
  const top = $('[data-top]');
  const burger = $('[data-burger]');
  const sheet = $('[data-sheet]');
  let lastY = scrollY, queued = false;
  const upd = () => {
    queued = false;
    const y = scrollY;
    top.classList.toggle('is-solid', y > 24);
    top.classList.toggle('is-hidden', y > 600 && y > lastY + 4 && sheet.hidden);
    if (y < lastY - 4 || y < 600) top.classList.remove('is-hidden');
    lastY = y;
  };
  addEventListener('scroll', () => { if (!queued) { queued = true; requestAnimationFrame(upd); } }, { passive: true });
  upd();
  burger.addEventListener('click', () => {
    const open = sheet.hidden;
    sheet.hidden = !open;
    burger.setAttribute('aria-expanded', String(open));
    top.classList.add('is-solid');
  });
  sheet.addEventListener('click', (e) => { if (e.target.closest('a')) { sheet.hidden = true; burger.setAttribute('aria-expanded', 'false'); } });

  const links = $$('.rd-nav a');
  const map = new Map(links.map((a) => [a.getAttribute('href').slice(1), a]));
  if ('IntersectionObserver' in window) {
    const io = new IntersectionObserver((es) => es.forEach((e) => {
      const a = map.get(e.target.id);
      if (a && e.isIntersecting) { links.forEach((l) => l.classList.remove('is-on')); a.classList.add('is-on'); }
    }), { rootMargin: '-45% 0px -50% 0px' });
    map.forEach((_, id) => { const s = doc.getElementById(id); if (s) io.observe(s); });
  }
}

// ---------- hero: yozuv sahnasi ----------
// Har so'z: [yozilgan, natija, sabab]. Natijalar bitg yadrosining (bitg_run) haqiqiy chiqishi:
// c — o'girildi, u — o'zgarmadi, e — inglizcha, p — havola/kod.
const SCRIPT = [
  [['Bugun', 'Bugun', 'u'], ['shaharda', 'şaharda', 'c'], ['choy', 'çoy', 'c'], ['ichdik,', 'içdik,', 'c'], ['keyin', 'keyin', 'u'], ['check', 'check', 'e'], ['qildim.', 'qildim.', 'u']],
  [["O'zbekiston", 'Özbekiston', 'c'], ["tog'lari", 'toğlari', 'c'], ['haqida', 'haqida', 'u'], ['bitg.uz', 'bitg.uz', 'p'], ['saytida', 'saytida', 'u'], ["o'qing", 'öqing', 'c']],
  [['Michael', 'Michael', 'e'], ['Jordan', 'Jordan', 'e'], ['Toshkentga', 'Toşkentga', 'c'], ['keldi', 'keldi', 'u']],
];
const NOTE = { c: 'o‘girildi', e: 'inglizcha — tegilmadi', p: 'havola — himoyalangan' };

function fuseHTML(typed, out) {
  // ikki belgi bitta harfga aylangan joylarni topadi: sh->ş, ch->ç, o'->ö, g'->ğ
  let i = 0, j = 0, html = '';
  const letters = [];
  const esc = (s) => s.replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' })[c]);
  while (i < typed.length && j < out.length) {
    if (typed[i] === out[j]) { html += esc(out[j]); i++; j++; continue; }
    html += `<span class="rd-fz"><span class="rd-fz__n">${esc(out[j])}</span></span>`;
    letters.push(out[j]);
    i += 2; j += 1;
  }
  html += esc(out.slice(j));
  return { html, letters };
}

function typer() {
  const line = $('[data-typer]');
  const note = $('[data-typer-note]');
  if (!line) return;
  const caret = doc.createElement('span');
  caret.className = 'rd-caret';
  const key = (ch) => dispatchEvent(new CustomEvent('bitg:key', { detail: { ch } }));
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  let visible = true;
  if ('IntersectionObserver' in window) {
    new IntersectionObserver((es) => { visible = es[0].isIntersecting; }).observe(line);
  }
  const waitVisible = async () => { while (!visible || doc.hidden) await sleep(400); };

  if (still) {
    // harakatsiz: birinchi gapning natijasi tayyor holda
    line.textContent = SCRIPT[0].map((w) => w[1]).join(' ');
    note.textContent = 'shaharda → şaharda · choy → çoy · check — inglizcha';
    return;
  }

  (async () => {
    await sleep(1600);
    for (let s = 0; ; s = (s + 1) % SCRIPT.length) {
      line.textContent = '';
      line.append(caret);
      note.textContent = ' ';
      note.className = 'rd-editor__note';
      for (const [typed, out, why] of SCRIPT[s]) {
        const w = doc.createElement('span');
        w.className = 'rd-w';
        line.insertBefore(w, caret);
        for (const ch of typed) {
          await waitVisible();
          w.textContent += ch;
          key(ch);
          await sleep(105 + Math.random() * 95 + (/[aeiou]/.test(ch) ? 0 : 20));
        }
        await sleep(140);
        key(' ');
        if (why === 'c') {
          const { html, letters } = fuseHTML(typed, out);
          w.classList.add('is-c');
          w.innerHTML = html;
          dispatchEvent(new CustomEvent('bitg:fuse', { detail: { letters } }));
        } else if (why !== 'u') {
          w.classList.add('is-' + why);
        }
        if (NOTE[why]) {
          note.textContent = why === 'c' ? `${typed} → ${out}` + '  ·  ' + NOTE.c : `${typed} — ${NOTE[why]}`;
          note.className = 'rd-editor__note is-' + why;
        }
        line.insertBefore(doc.createTextNode(' '), caret);
        setTimeout(() => w.classList.add('is-done'), 1500);
        await sleep(why === 'u' ? 260 : 720);
      }
      await sleep(2600);
    }
  })();
}
typer();

// ---------- 3D klaviatura (kerak bo'lganda yuklanadi) ----------
{
  const canvas = $('[data-kb]');
  const stage = $('[data-stage]');
  let gl = false;
  try { gl = !!doc.createElement('canvas').getContext('webgl2'); } catch { gl = false; }
  const save = navigator.connection && navigator.connection.saveData;
  if (canvas && gl && !save) {
    const go = () => import(`./kb3d.js${V ? '?v=' + V : ''}`)
      .then((m) => doc.fonts.load('500 64px "rd Sans"').catch(() => null).then(() => m.start(canvas, { still })))
      .then((api) => { if (!api) stage.classList.add('is-flat'); })
      .catch(() => stage.classList.add('is-flat'));
    if (doc.readyState === 'complete') requestAnimationFrame(go);
    else addEventListener('load', () => requestAnimationFrame(go), { once: true });
  } else if (stage) {
    stage.classList.add('is-flat');
  }
}

// ---------- bitg yadrosi (Rust → WASM), sayt bilan bir xil fayl ----------
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
      run,
    };
  });
  corePromise.catch(() => { corePromise = null; });
  return corePromise;
}

// ---------- sinov maydoni ----------
{
  const play = $('[data-play]');
  if (play) {
    const input = $('[data-in]', play);
    const out = $('[data-out]', play);
    const stat = $('[data-stat]', play);
    const modes = $$('[data-mode]', play);
    let mode = '0', api = null, timer = 0;
    const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);

    const measure = (text, m) => {
      // brauzeringizda o'lchanadi: bir xil matnni ~25 ms davomida qayta o'giradi
      const words = (text.match(/\S+/g) || []).length;
      if (!words) return '';
      let n = 0;
      const t0 = performance.now();
      while (performance.now() - t0 < 25 && n < 2000) { api.run(text, m); n++; }
      const us = ((performance.now() - t0) * 1000) / (n * words);
      return `${words} so‘z · <b>${fmt(us, us < 10 ? 1 : 0)} µs</b> so‘ziga — brauzeringizda, WebAssembly’da o‘lchandi (ilovada ≈ 2,7 µs)`;
    };

    const render = () => {
      if (!api) return;
      const text = input.value;
      if (mode === '1') {
        out.textContent = api.toOld(text);
        stat.innerHTML = measure(text, 1);
        return;
      }
      const r = api.toNew(text);
      const parts = r.out.split(/(\s+)/);
      const words = parts.filter((p, i) => i % 2 === 0 && p !== '');
      if (words.length !== r.toks.length) { out.textContent = r.out; stat.innerHTML = measure(text, 0); return; }
      let k = 0;
      out.innerHTML = parts.map((p, i) => {
        if (i % 2 === 1 || p === '') return esc(p);
        const t = r.toks[k++];
        const why = t[1];
        const cls = why === 'c' || why === 'r' ? 't-c' : why === 'e' ? 't-e' : why === 'p' ? 't-p' : why === 'x' ? 't-x' : '';
        const alt = t[2] ? ` class="${cls} alt" title="yoki: ${esc(t[2])}"` : (cls ? ` class="${cls}"` : '');
        return cls ? `<mark${alt}>${esc(p)}</mark>` : esc(p);
      }).join('');
      stat.innerHTML = measure(text, 0);
    };

    const setMode = (m) => {
      mode = m === 'cyr' ? '0' : m;
      modes.forEach((b) => {
        const on = b.getAttribute('data-mode') === m;
        b.setAttribute('aria-selected', String(on));
        b.tabIndex = on ? 0 : -1;
      });
    };
    modes.forEach((b) => b.addEventListener('click', () => {
      const m = b.getAttribute('data-mode');
      setMode(m);
      if (m === 'cyr' && !/[Ѐ-ӿ]/.test(input.value)) input.value = 'Ўзбекистон Республикаси пойтахти — Тошкент шаҳри.';
      if (m === '1' && !/[öğşçÖĞŞÇ]/.test(input.value)) input.value = 'Özbekiston şaharlari: Toşkent, Samarqand. Çoyxonada çoy içdik.';
      if (m === '0' && /[öğşçÖĞŞÇЀ-ӿ]/.test(input.value)) input.value = "Bugun shaharda choy ichdik, keyin check qildim. O'zbekiston tog'lari haqida bitg.uz saytida o'qing.";
      render();
    }));
    $$('[data-ex]', play).forEach((b) => b.addEventListener('click', () => {
      setMode(b.getAttribute('data-ex-mode') || '0');
      input.value = b.getAttribute('data-ex');
      render();
      input.focus({ preventScroll: true });
    }));
    input.addEventListener('input', () => { clearTimeout(timer); timer = setTimeout(render, 60); });
    $('[data-copy-out]', play).addEventListener('click', (e) => copy(out.textContent, e.currentTarget));

    onSeen([play], () => core().then((a) => { api = a; render(); })
      .catch(() => { stat.textContent = 'yadro yuklanmadi — sahifani yangilang'; }), '400px 0px');
  }
}

// ---------- nusxa olish ----------
function copy(text, btn) {
  const done = () => { const t = btn.textContent; btn.textContent = 'Nusxalandi ✓'; setTimeout(() => { btn.textContent = t; }, 1600); };
  if (navigator.clipboard) navigator.clipboard.writeText(text).then(done, () => {});
}
$$('[data-copy]').forEach((b) => b.addEventListener('click', () => copy(b.previousElementSibling.textContent, b)));

// ---------- tablar (klaviatura bilan ham) ----------
function tabs(box, onChange) {
  const list = $('[role="tablist"]', box);
  const btns = $$('[role="tab"]', list);
  const select = (b, focus) => {
    btns.forEach((x) => {
      const on = x === b;
      x.setAttribute('aria-selected', String(on));
      x.tabIndex = on ? 0 : -1;
      const p = x.getAttribute('aria-controls') && doc.getElementById(x.getAttribute('aria-controls'));
      if (p) {
        p.hidden = !on;
        p.classList.remove('is-in');
        if (on) { void p.offsetWidth; p.classList.add('is-in'); }
      }
    });
    if (focus) b.focus();
    b.scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: still ? 'auto' : 'smooth' });
    if (onChange) onChange(b);
  };
  btns.forEach((b) => b.addEventListener('click', () => select(b)));
  list.addEventListener('keydown', (e) => {
    const i = btns.indexOf(doc.activeElement);
    if (i < 0) return;
    const vert = list.closest('.rd-demo__nav') && innerWidth > 1080;
    const next = vert ? 'ArrowDown' : 'ArrowRight', prev = vert ? 'ArrowUp' : 'ArrowLeft';
    let j = -1;
    if (e.key === next) j = (i + 1) % btns.length;
    else if (e.key === prev) j = (i - 1 + btns.length) % btns.length;
    else if (e.key === 'Home') j = 0;
    else if (e.key === 'End') j = btns.length - 1;
    if (j >= 0) { e.preventDefault(); select(btns[j], true); }
  });
  return { select, btns };
}
$$('[data-tabs]').forEach((box) => {
  const t = tabs(box);
  const first = $('[role="tabpanel"]:not([hidden])', box);
  if (first) onSeen([first], (p) => p.classList.add('is-in'));
  if (box.hasAttribute('data-os-tabs')) {
    const ua = navigator.userAgent;
    const os = /Mac/.test(ua) && !/iPhone|iPad/.test(ua) ? 'mac' : /Linux|X11/.test(ua) && !/Android/.test(ua) ? 'linux' : /Win/.test(ua) ? 'win' : '';
    const b = os && t.btns.find((x) => x.getAttribute('data-os') === os);
    if (b) {
      b.setAttribute('aria-selected', 'true');
      t.btns.forEach((x) => { if (x !== b) { x.setAttribute('aria-selected', 'false'); x.tabIndex = -1; } });
      t.btns.forEach((x) => { const p = doc.getElementById(x.getAttribute('aria-controls')); if (p) p.hidden = x !== b; });
      b.tabIndex = 0;
      const label = { mac: 'macOS uchun yuklab olish', linux: 'Linux uchun yuklab olish', win: 'Windows uchun yuklab olish' }[os];
      const main = $('[data-dl-main]');
      if (main && label) main.textContent = label;
    }
  }
});

// ---------- ilova oynasi: haqiqiy skrinshotlar ----------
{
  const demo = $('[data-demo]');
  const SHOTS = ['home', 'try', 'text', 'rules', 'keyboard', 'apps', 'dictionary', 'news', 'feedback'];
  if (demo) {
    const img = $('[data-shot-img]', demo);
    const cap = $('[data-shot-cap]', demo);
    const list = $('[role="tablist"]', demo);
    const btns = $$('[role="tab"]', list);
    const show = (b) => {
      btns.forEach((x) => { x.setAttribute('aria-selected', String(x === b)); x.tabIndex = x === b ? 0 : -1; });
      // faqat build.py nusxalagan rasmlar (DOM qiymati URL'ga to'g'ridan tushmaydi)
      const name = SHOTS.find((s) => s === b.getAttribute('data-shot'));
      if (!name) return;
      const src = `/redesign/shots/${name}.jpg`;
      const pre = new Image();
      pre.src = src;
      img.classList.add('is-swap');
      const swap = () => {
        img.src = src;
        img.alt = 'bitg sozlamalar oynasi: ' + b.textContent.toLowerCase();
        cap.textContent = b.getAttribute('data-cap');
        requestAnimationFrame(() => img.classList.remove('is-swap'));
      };
      (pre.decode ? pre.decode() : Promise.resolve()).then(() => setTimeout(swap, still ? 0 : 220), swap);
    };
    btns.forEach((b) => b.addEventListener('click', () => show(b)));
    list.addEventListener('keydown', (e) => {
      const i = btns.indexOf(doc.activeElement);
      if (i < 0) return;
      const d = { ArrowDown: 1, ArrowRight: 1, ArrowUp: -1, ArrowLeft: -1 }[e.key];
      if (d) { e.preventDefault(); const b = btns[(i + d + btns.length) % btns.length]; b.focus(); show(b); }
    });
    // o'zi aylanadi (foydalanuvchi bosmaguncha), faqat ko'rinib turganda
    if (!still) {
      let auto = true, vis = false, i = 0;
      btns.forEach((b) => b.addEventListener('pointerdown', () => { auto = false; }));
      list.addEventListener('keydown', () => { auto = false; });
      if ('IntersectionObserver' in window) new IntersectionObserver((es) => { vis = es[0].isIntersecting; }, { threshold: 0.4 }).observe(demo);
      setInterval(() => { if (auto && vis && !doc.hidden) { i = (i + 1) % btns.length; show(btns[i]); } }, 4200);
    }
  }
}

// ---------- paydo bo'lish va sanoq ----------
onSeen($$('.rd-head, .rd-fuse__t, [data-ruler], [data-bars], [data-tl]'), (el) => el.classList.add('is-in'));
$$('.rd-fuse__t').forEach((t) => {
  const replay = () => { t.classList.remove('is-in'); void t.offsetWidth; t.classList.add('is-in'); };
  t.addEventListener('mouseenter', replay);
  t.addEventListener('focus', replay);
});
function countUp(el) {
  const to = parseFloat(el.getAttribute('data-count'));
  const dec = parseInt(el.getAttribute('data-dec') || '0', 10);
  if (isNaN(to) || still) return;
  const dur = 1800;
  let t0 = null;
  const step = (t) => {
    if (t0 === null) t0 = t;
    const k = Math.min((t - t0) / dur, 1);
    el.textContent = fmt(to * (1 - Math.pow(1 - k, 4)), dec);
    if (k < 1) requestAnimationFrame(step);
  };
  el.textContent = fmt(0, dec);
  requestAnimationFrame(step);
}
onSeen($$('[data-count]'), countUp);

// ---------- ekotizim orbitasi ----------
{
  const orbit = $('[data-orbit]');
  if (orbit) {
    const rings = [
      { els: $$('.rd-orbit__ring--1 li', orbit), rx: 0.30, ry: 0.29, speed: 0.000045, phase: 0.3 },
      { els: $$('.rd-orbit__ring--2 li', orbit), rx: 0.47, ry: 0.45, speed: -0.00003, phase: 0 },
    ];
    let vis = false, raf = 0, mx = 0, my = 0;
    const place = (now) => {
      raf = 0;
      const w = orbit.clientWidth, h = orbit.clientHeight;
      rings.forEach((r) => {
        r.els.forEach((li, i) => {
          const a = r.phase + (i / r.els.length) * Math.PI * 2 + (still ? 0 : now * r.speed);
          const x = Math.cos(a) * r.rx * w + mx * 10;
          const y = Math.sin(a) * r.ry * h + my * 8;
          li.style.setProperty('--x', x.toFixed(1) + 'px');
          li.style.setProperty('--y', y.toFixed(1) + 'px');
          li.style.setProperty('z-index', y > 0 ? '2' : '0');
        });
      });
      if (vis && !still) raf = requestAnimationFrame(place);
    };
    orbit.addEventListener('pointermove', (e) => {
      const r = orbit.getBoundingClientRect();
      mx = (e.clientX - r.left) / r.width - 0.5;
      my = (e.clientY - r.top) / r.height - 0.5;
    });
    if ('IntersectionObserver' in window) {
      new IntersectionObserver((es) => { vis = es[0].isIntersecting; if (vis && !raf) raf = requestAnimationFrame(place); }).observe(orbit);
    }
    place(performance.now());
    addEventListener('resize', () => place(performance.now()));
  }
}

// ---------- ochiq yuklab olish hisobi (bitg.uz/stats.json) ----------
{
  const el = $('[data-dl-count]');
  if (el) {
    fetch('/stats.json', { cache: 'no-store' }).then((r) => (r.ok ? r.json() : null)).then((s) => {
      if (s && s.downloads) el.textContent = `bitg.uz’dan ${fmt(s.downloads)} marta yuklab olingan (ochiq hisob).`;
    }).catch(() => {});
  }
}

// ---------- sahifa alifbosi: Eski / Yangi (bitg yadrosi bilan) ----------
{
  const btns = $$('[data-abc-btn]');
  const orig = new Map();
  let abc = 'old';
  const nodes = () => {
    const out = [];
    const w = doc.createTreeWalker(doc.body, NodeFilter.SHOW_TEXT, {
      acceptNode(n) {
        if (!n.data.trim()) return NodeFilter.FILTER_REJECT;
        const p = n.parentElement;
        const c = p && p.closest('[data-conv],[data-no-conv],script,style,kbd,code,pre,svg,textarea');
        if (!p || (c && !c.hasAttribute('data-conv'))) return NodeFilter.FILTER_REJECT;
        return NodeFilter.FILTER_ACCEPT;
      },
    });
    while (w.nextNode()) out.push(w.currentNode);
    return out;
  };
  const apply = (want) => core().then((api) => {
    abc = want;
    nodes().forEach((n) => {
      if (!orig.has(n)) orig.set(n, n.data);
      n.data = want === 'new' ? api.toNew(orig.get(n)).out : orig.get(n);
    });
    root.setAttribute('data-abc', want);
    btns.forEach((b) => b.setAttribute('aria-pressed', String(b.getAttribute('data-abc-btn') === want)));
    store.set('bitg-abc', want);
  });
  btns.forEach((b) => b.addEventListener('click', () => {
    const want = b.getAttribute('data-abc-btn');
    if (want !== abc) apply(want).catch(() => {});
  }));
  if (store.get('bitg-abc') === 'new') {
    (window.requestIdleCallback || ((f) => setTimeout(f, 600)))(() => apply('new').catch(() => {}));
  }
}
