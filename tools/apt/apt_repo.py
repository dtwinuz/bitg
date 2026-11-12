#!/usr/bin/env python3
"""bitg apt ombori: dist/*.deb -> imzolangan apt repo (https://bitg.uz/apt, suite `stable`, component `main`).

  python tools/apt/apt_repo.py --out <papka> --gnupg ~/.bitg-apt-gnupg dist/bitg_0.2.0_amd64.deb dist/fcitx5-bitg_0.2.0_amd64.deb

Natija:
  bitg.gpg                                   — ochiq kalit (binary; /etc/apt/keyrings/ uchun)
  pool/main/<h>/<paket>/<fayl>.deb
  dists/stable/main/binary-amd64/Packages(.gz)
  dists/stable/Release, InRelease (clearsign), Release.gpg (detached)

Kalit: deploy/apt/fingerprint.txt dagi barmoq izi; maxfiy kalit faqat --gnupg papkasida (repoda emas).
Bog'liqlik: Python 3.14+ (compression.zstd) yoki PATH'da `zstd`; `gpg`.
"""
import argparse
import gzip
import hashlib
import io
import lzma
import os
import shutil
import subprocess
import sys
import tarfile
import time
from email.utils import formatdate
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SUITE, COMPONENT, ARCH = "stable", "main", "amd64"


def fail(msg):
    print(f"apt_repo: {msg}", file=sys.stderr)
    sys.exit(1)


def unzstd(data):
    try:
        from compression import zstd
        return zstd.decompress(data)
    except ImportError:
        return subprocess.run(["zstd", "-dc"], input=data, capture_output=True, check=True).stdout


def deb_control(path):
    """.deb (ar arxiv) ichidagi control faylini matn sifatida qaytaradi."""
    data = path.read_bytes()
    if data[:8] != b"!<arch>\n":
        fail(f"{path.name}: .deb emas")
    i = 8
    while i + 60 <= len(data):
        name = data[i:i + 16].decode().strip().rstrip("/")
        size = int(data[i + 48:i + 58])
        body = data[i + 60:i + 60 + size]
        i += 60 + size + (size & 1)
        if not name.startswith("control.tar"):
            continue
        raw = (unzstd(body) if name.endswith(".zst") else lzma.decompress(body) if name.endswith(".xz")
               else gzip.decompress(body) if name.endswith(".gz") else body)
        with tarfile.open(fileobj=io.BytesIO(raw)) as t:
            for m in t.getmembers():
                if m.name in ("./control", "control"):
                    return t.extractfile(m).read().decode("utf-8")
    fail(f"{path.name}: control topilmadi")


def field(control, key):
    for line in control.splitlines():
        if line.startswith(key + ":"):
            return line.split(":", 1)[1].strip()
    return ""


def hashes(data):
    return {"MD5sum": hashlib.md5(data).hexdigest(), "SHA1": hashlib.sha1(data).hexdigest(),
            "SHA256": hashlib.sha256(data).hexdigest()}


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


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--gnupg", required=True, type=Path)
    ap.add_argument("--fingerprint", default=None)
    ap.add_argument("debs", nargs="+", type=Path)
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
    stanzas = []
    for deb in sorted(a.debs):
        control = deb_control(deb).strip("\n")
        pkg, arch = field(control, "Package"), field(control, "Architecture")
        if not pkg or arch not in (ARCH, "all"):
            fail(f"{deb.name}: Package/Architecture ({pkg}/{arch})")
        rel = f"pool/{COMPONENT}/{pkg[0]}/{pkg}/{deb.name}"
        dst = a.out / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(deb, dst)
        data = deb.read_bytes()
        stanzas.append(control + f"\nFilename: {rel}\nSize: {len(data)}\n"
                       + "".join(f"{k}: {v}\n" for k, v in hashes(data).items()))
    packages = "\n".join(stanzas).encode("utf-8")

    bdir = a.out / "dists" / SUITE / COMPONENT / f"binary-{ARCH}"
    bdir.mkdir(parents=True)
    (bdir / "Packages").write_bytes(packages)
    (bdir / "Packages.gz").write_bytes(gzip.compress(packages, mtime=0))

    files = {}
    for name in ("Packages", "Packages.gz"):
        data = (bdir / name).read_bytes()
        files[f"{COMPONENT}/binary-{ARCH}/{name}"] = (len(data), hashes(data))
    release = [
        "Origin: bitg", "Label: bitg", f"Suite: {SUITE}", f"Codename: {SUITE}",
        f"Date: {formatdate(time.time(), usegmt=True)}", f"Architectures: {ARCH}", f"Components: {COMPONENT}",
        "Description: bitg - Uzbek input method for the 2026 Latin alphabet (https://bitg.uz)",
    ]
    for algo, key in (("MD5Sum", "MD5sum"), ("SHA1", "SHA1"), ("SHA256", "SHA256")):
        release.append(f"{algo}:")
        release += [f" {h[key]} {size:>10} {path}" for path, (size, h) in sorted(files.items())]
    rdir = a.out / "dists" / SUITE
    (rdir / "Release").write_bytes(("\n".join(release) + "\n").encode("utf-8"))

    gpg(a.gnupg, "--local-user", fpr, "--digest-algo", "SHA512", "--clearsign",
        "-o", tool_path(rdir / "InRelease"), tool_path(rdir / "Release"))
    gpg(a.gnupg, "--local-user", fpr, "--digest-algo", "SHA512", "--armor", "--detach-sign",
        "-o", tool_path(rdir / "Release.gpg"), tool_path(rdir / "Release"))
    (a.out / "bitg.gpg").write_bytes(gpg(a.gnupg, "--export", fpr).stdout)
    gpg(a.gnupg, "--verify", tool_path(rdir / "InRelease"))
    print(f"apt_repo: {len(stanzas)} paket -> {a.out} (kalit ...{fpr[-16:]})")


if __name__ == "__main__":
    main()
