#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

"""bitg pacman ombori: *.pkg.tar.zst -> imzolangan Arch repo (https://bitg.uz/arch/$arch, ombor nomi `bitg`).

  python tools/arch/pacman_repo.py --out <papka> --gnupg ~/.bitg-apt-gnupg \\
      dist/bitg-0.4.1-1-x86_64.pkg.tar.zst dist/fcitx5-bitg-0.4.1-1-x86_64.pkg.tar.zst

Natija (repo-add bilan bir xil format, faqat Python + gpg; Windows'dagi deploy'da ham ishlaydi):
  bitg.asc                                       — ochiq kalit (pacman-key --add)
  x86_64/<paket>.pkg.tar.zst + .sig              — paket va uning ajratilgan imzosi
  x86_64/bitg.db.tar.gz, bitg.db (+ .sig)        — ombor bazasi, %PGPSIG% bilan
  x86_64/bitg.files.tar.gz, bitg.files (+ .sig)  — `pacman -F` uchun fayllar ro'yxati

Kalit apt ombori bilan bitta (deploy/apt/fingerprint.txt): maxfiy qismi faqat --gnupg papkasida.
Bog'liqlik: Python 3.14+ (compression.zstd) yoki PATH'da `zstd`; `gpg`.
"""
import argparse
import base64
import gzip
import hashlib
import io
import os
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPO = "bitg"
ARCHES = ("x86_64", "aarch64")

# .PKGINFO kaliti -> desc bo'limi (repo-add tartibida)
LISTS = [("replaces", "REPLACES"), ("conflict", "CONFLICTS"), ("provides", "PROVIDES"), ("depend", "DEPENDS"),
         ("optdepend", "OPTDEPENDS"), ("makedepend", "MAKEDEPENDS"), ("checkdepend", "CHECKDEPENDS")]


def fail(msg):
    print(f"pacman_repo: {msg}", file=sys.stderr)
    sys.exit(1)


def unzstd(data):
    try:
        from compression import zstd
        return zstd.decompress(data)
    except ImportError:
        return subprocess.run(["zstd", "-dc"], input=data, capture_output=True, check=True).stdout


def read_pkg(path):
    """.pkg.tar.zst -> (.PKGINFO lug'ati {kalit: [qiymatlar]}, paket ichidagi fayllar ro'yxati)."""
    with tarfile.open(fileobj=io.BytesIO(unzstd(path.read_bytes()))) as t:
        info, files = None, []
        for m in t.getmembers():
            name = m.name[2:] if m.name.startswith("./") else m.name
            if name == ".PKGINFO":
                info = t.extractfile(m).read().decode("utf-8")
            elif not name.startswith("."):
                files.append(name + ("/" if m.isdir() else ""))
    if info is None:
        fail(f"{path.name}: .PKGINFO yo'q")
    d = {}
    for line in info.splitlines():
        if line and not line.startswith("#") and " = " in line:
            k, v = line.split(" = ", 1)
            d.setdefault(k.strip(), []).append(v.strip())
    for k in ("pkgname", "pkgver", "arch", "size", "builddate"):
        if k not in d:
            fail(f"{path.name}: .PKGINFO da {k} yo'q")
    return d, sorted(files)


def section(key, values):
    return f"%{key}%\n" + "".join(v + "\n" for v in values) + "\n" if values else ""


def desc(info, path, sig):
    one = lambda k: info.get(k, [""])[0]
    data = path.read_bytes()
    out = section("FILENAME", [path.name]) + section("NAME", [one("pkgname")])
    out += section("BASE", [one("pkgbase") or one("pkgname")]) + section("VERSION", [one("pkgver")])
    out += section("DESC", [one("pkgdesc")]) + section("GROUPS", info.get("group", []))
    out += section("CSIZE", [str(len(data))]) + section("ISIZE", [one("size")])
    out += section("SHA256SUM", [hashlib.sha256(data).hexdigest()])
    out += section("PGPSIG", [base64.b64encode(sig).decode("ascii")])
    out += section("URL", [one("url")]) + section("LICENSE", info.get("license", []))
    out += section("ARCH", [one("arch")]) + section("BUILDDATE", [one("builddate")])
    out += section("PACKAGER", [one("packager")])
    for k, name in LISTS:
        out += section(name, info.get(k, []))
    return out


def tar_gz(entries):
    """{yo'l: matn} -> tar.gz baytlari (vaqt va egasi qotirilgan: bir xil kirish -> bir xil fayl)."""
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w", format=tarfile.USTAR_FORMAT) as t:
        for d in sorted({p.split("/")[0] for p in entries}):
            ti = tarfile.TarInfo(d)
            ti.type, ti.mode, ti.mtime = tarfile.DIRTYPE, 0o755, 0
            t.addfile(ti)
            for p in sorted(k for k in entries if k.startswith(d + "/")):
                data = entries[p].encode("utf-8")
                ti = tarfile.TarInfo(p)
                ti.size, ti.mode, ti.mtime = len(data), 0o644, 0
                t.addfile(ti, io.BytesIO(data))
    return gzip.compress(buf.getvalue(), mtime=0)


def tool_path(p):
    """Windows'da Git Bash'ning gpg'si (msys) C:\\... yo'lni tushunmaydi — /c/... ko'rinishiga o'tkaziladi."""
    s = str(Path(p).resolve())
    exe = (shutil.which("gpg") or "").lower().replace("\\", "/")
    if os.name == "nt" and ("/usr/bin/" in exe or "/msys" in exe or "/git/" in exe) and s[1:3] == ":\\":
        return "/" + s[0].lower() + s[2:].replace("\\", "/")
    return s


def gpg(home, *args):
    env = dict(os.environ, GNUPGHOME=tool_path(home), LC_ALL="C")
    return subprocess.run(["gpg", "--batch", "--yes", "--no-tty", *args], env=env, check=True, capture_output=True)


def sign(home, fpr, path):
    """Ajratilgan binar imzo (pacman .sig formati), yozilgandan keyin tekshiriladi."""
    sig = path.with_name(path.name + ".sig")
    gpg(home, "--local-user", fpr, "--digest-algo", "SHA512", "--detach-sign", "-o", tool_path(sig), tool_path(path))
    gpg(home, "--verify", tool_path(sig), tool_path(path))
    return sig.read_bytes()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--gnupg", required=True, type=Path)
    ap.add_argument("--fingerprint", default=None)
    ap.add_argument("pkgs", nargs="+", type=Path)
    a = ap.parse_args()

    fpr = (a.fingerprint or (ROOT / "deploy/apt/fingerprint.txt").read_text()).replace(" ", "").strip().upper()
    try:
        listed = gpg(a.gnupg, "--with-colons", "--list-secret-keys", fpr).stdout.decode()
    except subprocess.CalledProcessError:
        fail(f"maxfiy kalit {fpr} {a.gnupg} da yo'q (tools/apt/genkey.sh)")
    if f":{fpr}:" not in listed:
        fail(f"kalit barmoq izi mos emas: {fpr}")

    if a.out.exists():
        shutil.rmtree(a.out)
    by_arch = {}
    for pkg in sorted(a.pkgs):
        if not pkg.name.endswith(".pkg.tar.zst"):
            fail(f"{pkg.name}: .pkg.tar.zst emas")
        info, files = read_pkg(pkg)
        name, ver, arch = info["pkgname"][0], info["pkgver"][0], info["arch"][0]
        if arch not in ARCHES:
            fail(f"{pkg.name}: arxitektura {arch}")
        if pkg.name != f"{name}-{ver}-{arch}.pkg.tar.zst":
            fail(f"{pkg.name}: nom .PKGINFO ga mos emas ({name}-{ver}-{arch})")
        d = a.out / arch
        d.mkdir(parents=True, exist_ok=True)
        dst = d / pkg.name
        shutil.copyfile(pkg, dst)
        entry = f"{name}-{ver}"
        db, fl = by_arch.setdefault(arch, ({}, {}))
        if entry + "/desc" in db:
            fail(f"{entry} ikki marta berildi")
        text = desc(info, dst, sign(a.gnupg, fpr, dst))
        db[entry + "/desc"] = fl[entry + "/desc"] = text
        fl[entry + "/files"] = section("FILES", files)

    for arch, (db, fl) in by_arch.items():
        d = a.out / arch
        for base, entries in (("db", db), ("files", fl)):
            data = tar_gz(entries)
            # pacman `<ombor>.db` ni yuklaydi; repo-add uni .tar.gz ga symlink qiladi — veb uchun nusxa
            for name in (f"{REPO}.{base}.tar.gz", f"{REPO}.{base}"):
                (d / name).write_bytes(data)
                sign(a.gnupg, fpr, d / name)
    (a.out / "bitg.asc").write_bytes(gpg(a.gnupg, "--armor", "--export", fpr).stdout)
    n = sum(len(db) for db, _ in by_arch.values())
    print(f"pacman_repo: {n} paket ({', '.join(sorted(by_arch))}) -> {a.out} (kalit ...{fpr[-16:]})")


if __name__ == "__main__":
    main()
