// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Kengaytmani yig'ish: extension/dist/{chrome,firefox}/ (load unpacked) va dist/{chrome,firefox}.zip (do'konlar).
// Edge Chrome arxivini ishlatadi. Yadro — sayt bilan bir xil wasm (deploy/deploy.sh dagi buyruq):
//
//   cargo build -p bitg-core --release --target wasm32-unknown-unknown --target-dir target/site
//   node extension/tools/build.mjs [--cargo] [--wasm <bitg_core.wasm>] [--out <papka>]
//
// --cargo — wasm'ni avval shu buyruq bilan yig'adi. Bog'liqlik yo'q: faqat Node (>= 20) standart modullari.
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { zip } from './zip.mjs';

const EXT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const ROOT = path.resolve(EXT, '..');

const args = process.argv.slice(2);
function opt(name, def) {
  const i = args.indexOf(name);
  return i >= 0 && args[i + 1] ? args[i + 1] : def;
}
const OUT = path.resolve(opt('--out', path.join(EXT, 'dist')));
const WASM = path.resolve(opt('--wasm', path.join(ROOT, 'target', 'site', 'wasm32-unknown-unknown', 'release', 'bitg_core.wasm')));

if (args.includes('--cargo')) {
  execFileSync('cargo', ['build', '-q', '-p', 'bitg-core', '--release', '--target', 'wasm32-unknown-unknown',
    '--target-dir', path.join(ROOT, 'target', 'site')], { cwd: ROOT, stdio: 'inherit' });
}
if (!fs.existsSync(WASM)) {
  console.error(`bitg_core.wasm topilmadi: ${WASM}\n` +
    'Avval: cargo build -p bitg-core --release --target wasm32-unknown-unknown --target-dir target/site  (yoki --cargo)');
  process.exit(1);
}

// Kengaytma fayllari (aniq ro'yxat: dist/, test/, tools/ va hujjatlar arxivga tushmaydi)
const SOURCES = [
  'manifest.json', 'background.js', 'i18n.js', 'core/core.js',
  'content/lib.js', 'content/toast.js', 'content/edit.js', 'content/selection.js', 'content/page.js', 'content/autotype.js',
  'popup/popup.html', 'popup/popup.css', 'popup/popup.js',
  '_locales/uz/messages.json', '_locales/ru/messages.json', '_locales/en/messages.json',
  'icons/icon-16.png', 'icons/icon-32.png', 'icons/icon-48.png', 'icons/icon-128.png'
];
const EXTRA = [
  [WASM, 'core/bitg_core.wasm'],
  [path.join(ROOT, 'web', 'fonts', 'inter-latin.woff2'), 'fonts/inter-latin.woff2'],
  [path.join(ROOT, 'web', 'fonts', 'inter-latin-ext.woff2'), 'fonts/inter-latin-ext.woff2'],
  [path.join(ROOT, 'web', 'fonts', 'inter-cyrillic.woff2'), 'fonts/inter-cyrillic.woff2'],
  [path.join(ROOT, 'web', 'fonts', 'OFL.txt'), 'fonts/OFL.txt'],
  [path.join(ROOT, 'LICENSE-MIT'), 'LICENSE-MIT'],
  [path.join(ROOT, 'LICENSE-APACHE'), 'LICENSE-APACHE']
];

const GECKO_ID = 'bitg@bitg.uz';

function manifestFor(target) {
  const m = JSON.parse(fs.readFileSync(path.join(EXT, 'manifest.json'), 'utf8'));
  if (target === 'firefox') {
    // Firefox MV3 service worker'ni qo'llamaydi: fon — event page, skriptlar ro'yxat bilan
    delete m.minimum_chrome_version;
    m.background = { scripts: ['core/core.js', 'i18n.js', 'background.js'] };
    m.browser_specific_settings = {
      gecko: {
        id: GECKO_ID,
        // optional_host_permissions (128+) va data_collection_permissions (140+)
        strict_min_version: '140.0',
        data_collection_permissions: { required: ['none'] }
      }
    };
  }
  return Buffer.from(JSON.stringify(m, null, 2) + '\n');
}

// matn fayllari LF bilan: Windows (core.autocrlf) va CI arxivi baytma-bayt bir xil bo'lsin
const TEXT = /\.(js|json|html|css|txt)$|^LICENSE-/;
function read(file, rel) {
  const buf = fs.readFileSync(file);
  return TEXT.test(rel) ? Buffer.from(buf.toString('utf8').replace(/\r\n/g, '\n')) : buf;
}

function collect(target) {
  const files = new Map();
  for (const rel of SOURCES) {
    files.set(rel, rel === 'manifest.json' ? manifestFor(target) : read(path.join(EXT, rel), rel));
  }
  for (const [src, rel] of EXTRA) files.set(rel, read(src, rel));
  return files;
}

// manifestdagi har havola arxivda bormi
function check(files, target) {
  const m = JSON.parse(files.get('manifest.json').toString());
  const refs = [
    ...Object.values(m.icons), ...Object.values(m.action.default_icon), m.action.default_popup,
    ...(m.background.service_worker ? [m.background.service_worker] : m.background.scripts)
  ];
  for (const r of refs) if (!files.has(r)) throw new Error(`${target}: manifest ${r} ga ishora qiladi, lekin fayl yo'q`);
  for (const [rel, buf] of files) {
    if (!rel.endsWith('.html')) continue;
    for (const [, ref] of buf.toString().matchAll(/(?:src|href)="([^"#:]+)"/g)) {
      const p = path.posix.normalize(path.posix.join(path.posix.dirname(rel), ref));
      if (!files.has(p)) throw new Error(`${target}: ${rel} → ${ref} topilmadi`);
    }
  }
}

fs.rmSync(OUT, { recursive: true, force: true });
for (const target of ['chrome', 'firefox']) {
  const files = collect(target);
  check(files, target);
  const dir = path.join(OUT, target);
  for (const [rel, buf] of files) {
    const p = path.join(dir, rel);
    fs.mkdirSync(path.dirname(p), { recursive: true });
    fs.writeFileSync(p, buf);
  }
  const archive = zip([...files].map(([name, data]) => ({ name, data })));
  const zipPath = path.join(OUT, `${target}.zip`);
  fs.writeFileSync(zipPath, archive);
  const sha = createHash('sha256').update(archive).digest('hex');
  console.log(`${path.relative(ROOT, zipPath)}  ${(archive.length / 1024).toFixed(0)} KiB  sha256 ${sha}`);
}
