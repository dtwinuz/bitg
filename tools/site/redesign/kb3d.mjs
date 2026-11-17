// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// redesign.bitg.uz hero: 3D klaviatura (three.js). `npm run build` -> web/redesign/kb3d.js.
// rd.js "bitg:key" ({ch}) va "bitg:fuse" ({letters}) hodisalarini yuboradi: tugma bosiladi, yangi harf
// tugmasi (ö ğ ş ç) yonadi. Hech qanday matn bu yerda o'girilmaydi — natijalar rd.js da, yadrodan olingan.
import {
  ACESFilmicToneMapping, CanvasTexture, DirectionalLight, ExtrudeGeometry, Group, HemisphereLight, Mesh,
  MeshBasicMaterial, MeshPhysicalMaterial, MeshStandardMaterial, PCFSoftShadowMap, PerspectiveCamera, PlaneGeometry,
  PMREMGenerator, Scene, Shape, ShadowMaterial, SRGBColorSpace, WebGLRenderer,
} from 'three';
import { RoomEnvironment } from 'three/examples/jsm/environments/RoomEnvironment.js';

const U = 1;          // bitta tugma kengligi
const GAP = 0.13;
const H = 0.5;        // tugma balandligi
const TRAVEL = 0.2;   // bosilganda pastga yurishi

const ROWS = [
  { x: 0, keys: ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', ['⌫', 1.6]] },
  { x: 0.3, keys: ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', "'", ['⏎', 1.3]] },
  { x: 0.75, keys: ['z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', ['⇧', 1.2]] },
  { x: 2.6, keys: [[' ', 6.2]] },
];
const NEW = ['ö', 'ğ', 'ş', 'ç'];

function roundedRect(w, d, r) {
  const s = new Shape();
  const x = -w / 2, y = -d / 2;
  s.moveTo(x + r, y);
  s.lineTo(x + w - r, y); s.quadraticCurveTo(x + w, y, x + w, y + r);
  s.lineTo(x + w, y + d - r); s.quadraticCurveTo(x + w, y + d, x + w - r, y + d);
  s.lineTo(x + r, y + d); s.quadraticCurveTo(x, y + d, x, y + d - r);
  s.lineTo(x, y + r); s.quadraticCurveTo(x, y, x + r, y);
  return s;
}

const geoCache = new Map();
function capGeo(w) {
  const k = w.toFixed(2);
  if (!geoCache.has(k)) {
    const g = new ExtrudeGeometry(roundedRect(w - 0.12, U - 0.12, 0.16), {
      depth: H - 0.1, bevelEnabled: true, bevelThickness: 0.06, bevelSize: 0.06, bevelSegments: 4, curveSegments: 6,
    });
    g.rotateX(-Math.PI / 2);
    geoCache.set(k, g);
  }
  return geoCache.get(k);
}

function label(text, ratio, ink, font) {
  const px = 160;
  const c = document.createElement('canvas');
  c.width = Math.round(px * ratio); c.height = px;
  const x = c.getContext('2d');
  x.fillStyle = ink;
  x.textBaseline = 'middle';
  if (text === ' ') {
    x.font = `500 30px ${font}`; x.globalAlpha = 0.45; x.textAlign = 'center';
    x.fillText('probel', c.width / 2, px * 0.62);
  } else if (text.length === 1 && /[a-zöğşç']/i.test(text)) {
    x.font = `500 ${text === "'" ? 92 : 70}px ${font}`; x.textAlign = 'left';
    x.fillText(text, 30, px * 0.5);
  } else {
    x.font = `500 56px ${font}`; x.textAlign = 'center';
    x.fillText(text, c.width / 2, px * 0.52);
  }
  const t = new CanvasTexture(c);
  t.colorSpace = SRGBColorSpace;
  t.anisotropy = 4;
  return t;
}

export function start(canvas, opts = {}) {
  let renderer;
  try {
    renderer = new WebGLRenderer({ canvas, antialias: true, alpha: true, powerPreference: 'high-performance' });
  } catch {
    return null;
  }
  if (!renderer.getContext()) return null;
  const still = !!opts.still;
  const root = document.documentElement;
  const dark = () => root.getAttribute('data-theme') === 'dark'
    || (root.getAttribute('data-theme') !== 'light' && matchMedia('(prefers-color-scheme: dark)').matches);
  const small = innerWidth < 720;

  renderer.setPixelRatio(Math.min(devicePixelRatio || 1, small ? 1.5 : 1.75));
  renderer.outputColorSpace = SRGBColorSpace;
  renderer.toneMapping = ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.05;
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = PCFSoftShadowMap;

  const scene = new Scene();
  const pm = new PMREMGenerator(renderer);
  scene.environment = pm.fromScene(new RoomEnvironment(), 0.04).texture;
  pm.dispose();
  scene.environmentIntensity = 0.55;

  const cam = new PerspectiveCamera(28, 1, 0.1, 200);

  const hemi = new HemisphereLight(0xfff8ec, 0x2a2a28, 0.55);
  scene.add(hemi);
  const sun = new DirectionalLight(0xffffff, 2.1);
  sun.position.set(-6, 14, 8);
  sun.castShadow = true;
  sun.shadow.mapSize.set(small ? 1024 : 2048, small ? 1024 : 2048);
  Object.assign(sun.shadow.camera, { left: -11, right: 11, top: 8, bottom: -8 });
  sun.shadow.radius = 6; sun.shadow.bias = -0.0004; sun.shadow.normalBias = 0.02;
  scene.add(sun);

  const kb = new Group();
  const font = '"rd Sans", "Segoe UI", system-ui, sans-serif';
  const capMat = new MeshPhysicalMaterial({ color: 0xf3efe6, roughness: 0.52, clearcoat: 0.25, clearcoatRoughness: 0.6 });
  const modMat = new MeshPhysicalMaterial({ color: 0x2b2d33, roughness: 0.6, clearcoat: 0.2, clearcoatRoughness: 0.7 });
  const accentMat = () => new MeshPhysicalMaterial({
    color: 0x2f62c0, roughness: 0.42, clearcoat: 0.5, clearcoatRoughness: 0.35, emissive: 0x3b74de, emissiveIntensity: 0,
  });

  const keys = new Map();
  const all = [];
  function addKey(ch, w, x, z, isNew) {
    const mod = /[⌫⏎⇧]/.test(ch);
    const mat = isNew ? accentMat() : (mod ? modMat : capMat);
    const m = new Mesh(capGeo(w), mat);
    m.castShadow = true; m.receiveShadow = true;
    m.position.set(x + w / 2, 0.08, z);
    const ink = isNew ? '#ffffff' : (mod ? '#d9d6cc' : '#30302e');
    const top = new Mesh(new PlaneGeometry(w - 0.3, U - 0.3),
      new MeshBasicMaterial({ map: label(isNew ? ch.toUpperCase() : ch, (w - 0.3) / (U - 0.3), ink, font), transparent: true, depthWrite: false }));
    top.rotation.x = -Math.PI / 2;
    top.position.y = H + 0.002;
    m.add(top);
    kb.add(m);
    const k = { mesh: m, base: 0.08, v: 0, d: 0, t: 0, until: 0, glow: 0, mat, isNew };
    all.push(k);
    keys.set(ch, k);
  }

  let maxX = 0;
  ROWS.forEach((row, r) => {
    let x = row.x;
    const z = r * (U + GAP);
    row.keys.forEach((k) => {
      const [ch, w] = Array.isArray(k) ? k : [k, 1];
      addKey(ch, w * U + (w - 1) * GAP, x, z, false);
      x += w * U + w * GAP;
    });
    maxX = Math.max(maxX, x);
  });
  // yangi harflar: o'ngda 2x2 ko'k blok
  const nx = maxX + 0.55;
  NEW.forEach((ch, i) => addKey(ch, U, nx + (i % 2) * (U + GAP), (i < 2 ? 0.5 : 1.5) * (U + GAP), true));
  const width = nx + 2 * U + GAP;
  const depth = 4 * (U + GAP) - GAP;

  const body = new Mesh(
    new ExtrudeGeometry(roundedRect(width + 1.1, depth + 1.1, 0.55), { depth: 0.42, bevelEnabled: true, bevelThickness: 0.08, bevelSize: 0.08, bevelSegments: 5, curveSegments: 10 }),
    new MeshStandardMaterial({ color: 0x1b1e26, roughness: 0.38, metalness: 0.55 }),
  );
  body.geometry.rotateX(-Math.PI / 2);
  body.position.set(width / 2, -0.42, depth / 2 - U / 2);
  body.receiveShadow = true; body.castShadow = true;
  kb.add(body);
  const plate = new Mesh(
    new ExtrudeGeometry(roundedRect(width + 0.4, depth + 0.4, 0.3), { depth: 0.05, bevelEnabled: false, curveSegments: 8 }),
    new MeshStandardMaterial({ color: 0x23262f, roughness: 0.7, metalness: 0.2 }),
  );
  plate.geometry.rotateX(-Math.PI / 2);
  plate.position.set(width / 2, 0.02, depth / 2 - U / 2);
  plate.receiveShadow = true;
  kb.add(plate);
  kb.position.set(-width / 2, 0, -depth / 2 + U / 2);

  const holder = new Group();
  holder.add(kb);
  scene.add(holder);

  const ground = new Mesh(new PlaneGeometry(60, 40), new ShadowMaterial({ opacity: 0.16 }));
  ground.rotation.x = -Math.PI / 2;
  ground.position.y = -0.62;
  ground.receiveShadow = true;
  scene.add(ground);

  function theme() {
    const d = dark();
    ground.material.opacity = d ? 0.55 : 0.16;
    hemi.intensity = d ? 0.35 : 0.55;
    scene.environmentIntensity = d ? 0.4 : 0.55;
  }
  theme();

  function fit() {
    const r = canvas.getBoundingClientRect();
    const W = Math.max(1, r.width), Hh = Math.max(1, r.height);
    renderer.setSize(W, Hh, false);
    cam.aspect = W / Hh;
    // butun klaviatura (aylangan holatda ham) kadrga sig'sin; yuqori qismi yozuv kartasi uchun bo'sh
    const span = width + 3.6;
    const fovH = 2 * Math.atan(Math.tan((cam.fov * Math.PI) / 360) * cam.aspect);
    const fovV = (cam.fov * Math.PI) / 180;
    const dist = Math.max((span / 2) / Math.tan(fovH / 2), (depth + 9) / 2 / Math.tan(fovV / 2), 14);
    cam.position.set(0, dist * 0.78, dist * 0.6);
    cam.lookAt(0, 0.4, -2.2);
    cam.updateProjectionMatrix();
  }

  function press(ch) {
    const c = ch.toLowerCase();
    const k = keys.get(c) || (/[‘’`ʻʼ]/.test(c) ? keys.get("'") : null);
    if (!k) return;
    k.t = 1;
    k.until = performance.now() + 95;
    kick();
  }
  function fuse(letters) {
    letters.forEach((l, i) => {
      const k = keys.get(l.toLowerCase());
      if (!k) return;
      setTimeout(() => { k.t = 1; k.until = performance.now() + 150; k.glow = 1; kick(); }, 140 + i * 170);
    });
  }
  addEventListener('bitg:key', (e) => press(e.detail.ch));
  addEventListener('bitg:fuse', (e) => fuse(e.detail.letters || []));

  let px = 0, py = 0, tx = 0, ty = 0;
  if (!still) {
    addEventListener('pointermove', (e) => {
      tx = (e.clientX / innerWidth - 0.5) * 2;
      ty = (e.clientY / innerHeight - 0.5) * 2;
    }, { passive: true });
  }

  let visible = true, raf = 0, last = performance.now();
  function frame(now) {
    raf = 0;
    const dt = Math.min(0.05, Math.max(0.001, (now - last) / 1000));
    last = now;
    px += (tx - px) * Math.min(1, dt * 2.2);
    py += (ty - py) * Math.min(1, dt * 2.2);
    const r = canvas.getBoundingClientRect();
    const sp = Math.min(1, Math.max(0, -r.top / Math.max(1, r.height)));
    const idle = still ? 0 : Math.sin(now / 2600) * 0.025;
    holder.rotation.y = -0.09 + px * 0.08 + idle;
    holder.rotation.x = py * 0.05 + sp * 0.35;
    holder.position.y = (still ? 0 : Math.sin(now / 3100) * 0.06) - sp * 1.2;
    let moving = false;
    const steps = Math.ceil(dt / (1 / 240));
    const h = dt / steps;
    for (const k of all) {
      if (k.t && now > k.until) k.t = 0;
      // prujina (yarim-oshkor Eyler, 240 Gts qadam): sekin kadrda ham barqaror
      for (let s = 0; s < steps; s++) {
        k.v += ((k.t - k.d) * 900 - k.v * 42) * h;
        k.d += k.v * h;
      }
      if (k.d < -0.4) { k.d = -0.4; k.v = 0; }
      if (k.d > 1.15) { k.d = 1.15; k.v = 0; }
      k.mesh.position.y = k.base - k.d * TRAVEL;
      if (k.isNew) {
        k.glow += (0 - k.glow) * Math.min(1, dt * 1.1);
        k.mat.emissiveIntensity = k.glow * 1.4;
        if (k.glow > 0.01) moving = true;
      }
      if (Math.abs(k.v) > 0.001 || k.t) moving = true;
    }
    renderer.render(scene, cam);
    if (visible && (!still || moving)) raf = requestAnimationFrame(frame);
  }
  function kick() { if (!raf && visible) { last = performance.now(); raf = requestAnimationFrame(frame); } }

  fit();
  if ('IntersectionObserver' in window) {
    new IntersectionObserver((es) => {
      visible = es[0].isIntersecting && !document.hidden;
      kick();
    }, { rootMargin: '80px' }).observe(canvas);
  }
  document.addEventListener('visibilitychange', () => { visible = !document.hidden; kick(); });
  new ResizeObserver(() => { fit(); kick(); }).observe(canvas);
  matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => { theme(); kick(); });
  new MutationObserver(() => { theme(); kick(); }).observe(root, { attributes: true, attributeFilter: ['data-theme'] });
  if (still) addEventListener('scroll', kick, { passive: true });

  kick();
  canvas.classList.add('is-on');
  return { press, fuse };
}
