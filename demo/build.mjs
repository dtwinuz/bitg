// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// Brauzer demosi: haqiqiy sozlamalar oynasi (crates/bitg-settings/src/ui.html) + xotiradagi IPC (mock-ipc.js)
// + haqiqiy yadro (bitg_core.wasm). Natija demo/dist/ — istalgan statik hostda istalgan papkada ishlaydi
// (barcha havolalar nisbiy, sahifalar #/try kabi hash bilan).
//
//   node demo/build.mjs            # wasm'ni yig'adi va demo/dist/ ni yozadi
//   BITG_WASM=path node build.mjs  # tayyor wasm bilan (cargo'siz)
//
// Ilovaning o'zi (bitg-settings.exe) va bitg.uz sayti bu skriptdan foydalanmaydi va o'zgarmaydi.
import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = join(HERE, "..");
const OUT = join(HERE, "dist");

// 1) yadro: sayt va kengaytma bilan bir xil buyruq (deploy/deploy.sh)
let wasm = process.env.BITG_WASM;
if (!wasm) {
  execFileSync("cargo", ["build", "-q", "-p", "bitg-core", "--release", "--target", "wasm32-unknown-unknown",
    "--target-dir", "target/site"], { cwd: ROOT, stdio: "inherit" });
  wasm = join(ROOT, "target/site/wasm32-unknown-unknown/release/bitg_core.wasm");
}
if (!existsSync(wasm)) throw new Error(`wasm topilmadi: ${wasm}`);

// 2) «Yangiliklar»: NEWS.md -> updates.rs news_json bilan bir xil tuzilma
function plain(s) {
  return s.replace(/\[([^\]]*)\]\([^)]*\)/g, "$1").replace(/\*\*/g, "").replace(/`/g, "").trim();
}
function item(line) {
  const m = line.match(/^\*\*(.+?)\*\*(.*)$/);
  return m ? { lead: plain(m[1]), text: plain(m[2]) } : { lead: "", text: plain(line) };
}
function news(md) {
  const versions = [];
  for (const line of md.split(/\r?\n/)) {
    if (line.startsWith("## ")) {
      const [v, d = ""] = line.slice(3).split("—");
      versions.push({ version: v.trim(), date: d.trim(), groups: [] });
    } else if (line.startsWith("### ") && versions.length) {
      const title = line.slice(4).trim();
      const kind = { yangi: "new", yaxshilandi: "better", tuzatildi: "fixed" }[title.toLowerCase()] || "other";
      versions.at(-1).groups.push({ kind, title, items: [] });
    } else if (line.startsWith("- ") && versions.at(-1)?.groups.length) {
      versions.at(-1).groups.at(-1).items.push(item(line.slice(2)));
    }
  }
  return versions;
}

const cargo = readFileSync(join(ROOT, "Cargo.toml"), "utf8");
const version = cargo.match(/^version = "(.+)"/m)[1];
const fixture = JSON.parse(readFileSync(join(HERE, "fixtures/layouts.json"), "utf8"));
const data = {
  version,
  layouts: fixture.layouts,
  default_apps: fixture.default_apps,
  news: news(readFileSync(join(ROOT, "NEWS.md"), "utf8")),
};

// 3) oyna: o'zgartirilmaydi, faqat IPC va demo belgisi qo'shiladi
let html = readFileSync(join(ROOT, "crates/bitg-settings/src/ui.html"), "utf8");
const head = `<meta name="description" content="bitg sozlamalar oynasi brauzerda: haqiqiy yadro, namuna ma'lumotlar.">
<meta name="robots" content="noindex">
<link rel="icon" href="icon.svg" type="image/svg+xml">
<link rel="stylesheet" href="demo.css">`;
const boot = `<script>window.__BITG_DEMO__ = ${JSON.stringify(data).replace(/</g, "\\u003c")};</script>
<script src="mock-ipc.js"></script>
`;
const pill = `<div class="demo-pill" role="note">Demo · namuna ma'lumotlar, qayta yuklanganda tiklanadi
  <a href="https://bitg.uz/yuklab-olish/" target="_blank" rel="noopener">Yuklab olish</a></div>
<div class="demo-toast" id="demoToast" role="status" aria-live="polite"></div>
`;
const n = html.split("<script>").length - 1;
if (n !== 1) throw new Error(`ui.html: bitta <script> kutilgan, ${n} ta`);
html = html
  .replace("<title>bitg</title>", "<title>bitg demo</title>\n" + head)
  .replace("<script>", boot + "<script>")
  .replace("</body>", pill + "</body>");

rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });
writeFileSync(join(OUT, "index.html"), html);
copyFileSync(join(HERE, "mock-ipc.js"), join(OUT, "mock-ipc.js"));
copyFileSync(join(HERE, "demo.css"), join(OUT, "demo.css"));
copyFileSync(join(ROOT, "web/icon.svg"), join(OUT, "icon.svg"));
copyFileSync(wasm, join(OUT, "bitg_core.wasm"));
console.log(`demo/dist: v${version}, ${data.news.length} versiya, ${Object.keys(data.layouts).length} tartib`);
