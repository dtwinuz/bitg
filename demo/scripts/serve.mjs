// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// demo/dist ni /demo/bitg/ ostida beradi (wonmor.io dagi kabi papka): `node scripts/serve.mjs [port]`.
import { createReadStream, existsSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { dirname, extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

export const BASE = "/demo/bitg/";
const DIST = join(dirname(fileURLToPath(import.meta.url)), "..", "dist");
const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8",
  ".svg": "image/svg+xml", ".wasm": "application/wasm", ".json": "application/json" };

export function serve(port = 0) {
  const server = createServer((req, res) => {
    const url = new URL(req.url, "http://x");
    if (url.pathname === "/" || url.pathname === BASE.slice(0, -1)) { res.writeHead(302, { location: BASE }); return res.end(); }
    if (!url.pathname.startsWith(BASE)) { res.writeHead(404); return res.end("not found"); }
    let rel = normalize(decodeURIComponent(url.pathname.slice(BASE.length))).replace(/^([/\\]|\.\.)+/, "");
    let file = join(DIST, rel);
    if (!rel || !existsSync(file) || statSync(file).isDirectory()) file = join(DIST, "index.html");
    res.writeHead(200, { "content-type": TYPES[extname(file)] || "application/octet-stream" });
    createReadStream(file).pipe(res);
  });
  return new Promise((ok) => server.listen(port, "127.0.0.1", () => ok(server)));
}

if (process.argv[1] && fileURLToPath(import.meta.url) === normalize(process.argv[1])) {
  const s = await serve(Number(process.argv[2] || 4178));
  console.log(`http://127.0.0.1:${s.address().port}${BASE}`);
}
