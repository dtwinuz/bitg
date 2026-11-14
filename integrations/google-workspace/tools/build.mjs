// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Google Workspace add-on'ni yig'ish: integrations/google-workspace/dist/ — `clasp push` shu papkani yuboradi.
// Yadro — bitg.uz va brauzer kengaytmasi bilan bir xil wasm (deploy/deploy.sh dagi buyruq):
//
//   cargo build -p bitg-core --release --target wasm32-unknown-unknown --target-dir target/site
//   node integrations/google-workspace/tools/build.mjs [--cargo] [--wasm <bitg_core.wasm>] [--out <papka>]
//
// Apps Script loyihasi ikkilik fayl saqlamaydi: wasm gzip (mtime 0 — takrorlanuvchan) + base64 qilib
// Wasm.gs ga bo'laklab yoziladi va Engine.gs uni Utilities.base64Decode + Utilities.ungzip bilan ochadi.
// Tarmoqdan hech narsa yuklanmaydi (UrlFetchApp yo'q, tashqi URL yo'q). Bog'liqlik yo'q: faqat Node >= 20.
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import zlib from 'node:zlib';

const HERE = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const ROOT = path.resolve(HERE, '..', '..');

const args = process.argv.slice(2);
function opt(name, def) {
  const i = args.indexOf(name);
  return i >= 0 && args[i + 1] ? args[i + 1] : def;
}
const OUT = path.resolve(opt('--out', path.join(HERE, 'dist')));
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

const SOURCES = ['appsscript.json', 'Code.gs', 'Engine.gs', 'Sidebar.html'];
// Bitta .gs fayl juda katta bo'lmasin (Apps Script muharriri va clasp uchun): ~512 KB bo'laklar
const PART = 512 * 1024;

const wasm = fs.readFileSync(WASM);
new WebAssembly.Module(wasm); // buzilgan wasm'ni shu yerda to'xtatamiz
const sha = createHash('sha256').update(wasm).digest('hex');
const gz = zlib.gzipSync(wasm, { level: 9 });
const b64 = gz.toString('base64');
const parts = [];
for (let i = 0; i < b64.length; i += PART) parts.push(b64.slice(i, i + PART));

fs.rmSync(OUT, { recursive: true, force: true });
fs.mkdirSync(OUT, { recursive: true });
for (const f of SOURCES) fs.copyFileSync(path.join(HERE, 'src', f), path.join(OUT, f));

const wasmGs = [
  '// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors',
  '//',
  '// SPDX-License-Identifier: MIT OR Apache-2.0',
  '',
  '// GENERATSIYA QILINGAN (integrations/google-workspace/tools/build.mjs) — qo\'lda tahrirlamang.',
  `// bitg_core.wasm: ${wasm.length} bayt, sha256 ${sha}; gzip ${gz.length} bayt, base64 ${b64.length} belgi, ${parts.length} bo'lak.`,
  '',
  `var BITG_WASM_SHA256 = '${sha}';`,
  '',
  'function bitgWasmBase64_() {',
  `  return [${parts.map((_, i) => `bitgWasmPart${i}_()`).join(', ')}].join('');`,
  '}',
  '',
  ...parts.map((p, i) => `function bitgWasmPart${i}_() {\n  return '${p}';\n}\n`)
].join('\n');
fs.writeFileSync(path.join(OUT, 'Wasm.gs'), wasmGs);

let total = 0;
for (const f of fs.readdirSync(OUT)) {
  const n = fs.statSync(path.join(OUT, f)).size;
  total += n;
  console.log(`${f.padEnd(16)} ${String(n).padStart(9)} B`);
}
console.log(`jami              ${String(total).padStart(9)} B  (wasm sha256 ${sha})`);
