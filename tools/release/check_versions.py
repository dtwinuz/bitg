#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Versiya qo'lda yoziladigan hamma joyda Cargo.toml bilan bir xilmi.

    python3 tools/release/check_versions.py          # CI: manifestlar Cargo.toml bilan bir xil
    python3 tools/release/check_versions.py 0.4.1    # teg: Cargo.toml = 0.4.1, CHANGELOG/NEWS bo'limi bor

Reliz fayllarining sha256'i kerak bo'lgan retseptlar (AUR, Nix, winget, Chocolatey, cask) relizdan keyin
yangilanadi, shuning uchun ular faqat eslatma sifatida chiqadi va xato bermaydi.
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(rel):
    return (ROOT / rel).read_text(encoding="utf-8")


def regex(rel, pattern):
    m = re.search(pattern, read(rel), re.M)
    return m.group(1) if m else None


def jsonv(rel):
    return json.loads(read(rel)).get("version")


def cargo_version():
    # [workspace.package] dagi version (rust-version emas)
    return regex("Cargo.toml", r'^version\s*=\s*"([^"]+)"')


# reliz oldidan qo'lda yangilanadi: farq = xato
STRICT = {
    "CITATION.cff": lambda: regex("CITATION.cff", r'^version:\s*"?([^"\s]+)"?'),
    "server.json": lambda: jsonv("server.json"),
    "gemini-extension.json": lambda: jsonv("gemini-extension.json"),
    "integrations/claude/.claude-plugin/plugin.json": lambda: jsonv("integrations/claude/.claude-plugin/plugin.json"),
    "integrations/chatgpt/.codex-plugin/plugin.json": lambda: jsonv("integrations/chatgpt/.codex-plugin/plugin.json"),
    "packaging/rpm/bitg.spec": lambda: regex("packaging/rpm/bitg.spec", r"^Version:\s*(\S+)"),
}

# relizdan keyin (sha256 bilan) yangilanadi: farq = eslatma
AFTER_RELEASE = {
    "packaging/aur/PKGBUILD": lambda: regex("packaging/aur/PKGBUILD", r"^pkgver=(\S+)"),
    "packaging/nix/package.nix": lambda: regex("packaging/nix/package.nix", r'^\s*version\s*=\s*"([^"]+)"'),
}


def check(tag=None):
    errors, notes = [], []
    cargo = cargo_version()
    if cargo is None:
        return ["Cargo.toml: version topilmadi"], notes
    if tag is not None and cargo != tag:
        errors.append(f"Cargo.toml: {cargo}, teg: {tag}")
    want = tag or cargo
    for name, get in STRICT.items():
        got = get()
        if got != want:
            errors.append(f"{name}: {got}, kerak: {want}")
    for name, get in AFTER_RELEASE.items():
        got = get()
        if got != want:
            notes.append(f"{name}: {got} (relizdan keyin {want} ga yangilang)")
    if tag is not None:
        if not re.search(rf"^## \[{re.escape(tag)}\]", read("CHANGELOG.md"), re.M):
            errors.append(f"CHANGELOG.md: '## [{tag}]' bo'limi yo'q")
        if not re.search(rf"^## {re.escape(tag)}\b", read("NEWS.md"), re.M):
            notes.append(f"NEWS.md: '## {tag}' yo'q — e'lon CHANGELOG'dan olinadi")
    return errors, notes


def main(argv):
    tag = argv[1].removeprefix("v") if len(argv) > 1 else None
    errors, notes = check(tag)
    for n in notes:
        print(f"eslatma: {n}")
    for e in errors:
        print(f"xato: {e}", file=sys.stderr)
    if errors:
        return 1
    print(f"versiyalar bir xil: {tag or cargo_version()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
