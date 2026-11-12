#!/usr/bin/env python3
"""winget manifesti va Homebrew cask'ni dist/ dagi release fayllaridan yaratadi.

  python tools/release/packaging.py            # versiya Cargo.toml'dan, fayllar dist/ dan

Natija (repoda saqlanadi, yuborish qo'lda):
  packaging/winget/manifests/d/dtwinuz/bitg/<v>/  -> microsoft/winget-pkgs ga PR (wingetcreate submit yoki qo'lda)
  packaging/homebrew/Casks/bitg.rb                -> dtwinuz/homebrew-tap reposiga nusxa

Havolalar: MSI — GitHub Release (winget faqat barqaror URL qabul qiladi), Mac .pkg — Release'da bo'lsa o'sha,
bo'lmasa bitg.uz/download (v0.2.0 Release'i o'zgarmas, pkg unga kirmagan).
"""
import argparse
import hashlib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPO = "https://github.com/dtwinuz/bitg"
SCHEMA = "1.9.0"


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def write(p, text):
    p.write_text(text.lstrip("\n"), encoding="utf-8", newline="\n")


def winget(ver, msi_sha, out):
    ident = "dtwinuz.bitg"
    d = out / "winget" / "manifests" / "d" / "dtwinuz" / "bitg" / ver
    d.mkdir(parents=True, exist_ok=True)
    head = "# yaml-language-server: $schema=https://aka.ms/winget-manifest.{kind}.{s}.schema.json\n\n"
    write(d / f"{ident}.yaml", head.format(kind="version", s=SCHEMA) + f"""PackageIdentifier: {ident}
PackageVersion: {ver}
DefaultLocale: en-US
ManifestType: version
ManifestVersion: {SCHEMA}
""")
    write(d / f"{ident}.installer.yaml", head.format(kind="installer", s=SCHEMA) + f"""PackageIdentifier: {ident}
PackageVersion: {ver}
MinimumOSVersion: 10.0.17763.0
InstallerType: wix
Scope: machine
ElevationRequirement: elevatesSelf
UpgradeBehavior: install
InstallModes:
  - interactive
  - silent
  - silentWithProgress
Installers:
  - Architecture: x64
    InstallerUrl: {REPO}/releases/download/v{ver}/bitg-{ver}-x64.msi
    InstallerSha256: {msi_sha.upper()}
ManifestType: installer
ManifestVersion: {SCHEMA}
""")
    write(d / f"{ident}.locale.en-US.yaml", head.format(kind="defaultLocale", s=SCHEMA) + f"""PackageIdentifier: {ident}
PackageVersion: {ver}
PackageLocale: en-US
Publisher: dtwinuz
PublisherUrl: https://github.com/dtwinuz
PublisherSupportUrl: https://bitg.uz/fikr/
PrivacyUrl: https://bitg.uz/maxfiylik/
PackageName: bitg
PackageUrl: https://bitg.uz/
License: MIT OR Apache-2.0
LicenseUrl: {REPO}#litsenziya
ShortDescription: Uzbek keyboard that converts text to the new 2026 Latin alphabet (ö ğ ş ç) as you type.
Description: |-
  bitg is a Windows input method (TSF) for Uzbek. Type as you always did (o', g', sh, ch) and each word is
  converted to the new 2026 Latin alphabet when you finish it. English words, links and code are left alone,
  Backspace right after a replacement restores the original word, and everything works offline.
Moniker: bitg
Tags:
  - uzbek
  - keyboard
  - input-method
  - alphabet
  - transliteration
  - ime
ReleaseNotesUrl: {REPO}/releases/tag/v{ver}
ManifestType: defaultLocale
ManifestVersion: {SCHEMA}
""")
    return d


def cask(ver, pkg_sha, url, bundle, out):
    d = out / "homebrew" / "Casks"
    d.mkdir(parents=True, exist_ok=True)
    url = url.replace(ver, "#{version}")
    p = d / "bitg.rb"
    write(p, f'''cask "bitg" do
  version "{ver}"
  sha256 "{pkg_sha}"

  url "{url}"
  name "bitg"
  desc "Uzbek input method for the new 2026 Latin alphabet"
  homepage "https://bitg.uz/"

  livecheck do
    url "{REPO}"
    strategy :github_latest
  end

  depends_on macos: ">= :big_sur"

  pkg "bitg-#{{version}}-macos.pkg"

  uninstall quit:    "{bundle}",
            pkgutil: "uz.bitg.inputmethod.pkg",
            delete:  "/Library/Input Methods/bitg.app"

  zap trash: "~/Library/Application Support/bitg"

  caveats <<~EOS
    Log out and back in, then choose bitg with Control + Space
    (System Settings > Keyboard > Input Sources > + > bitg).
  EOS
end
''')
    return p


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dist", type=Path, default=ROOT / "dist")
    ap.add_argument("--out", type=Path, default=ROOT / "packaging")
    ap.add_argument("--pkg-url", default=None, help="Mac .pkg manzili (standart: bitg.uz/download)")
    a = ap.parse_args()
    ver = re.search(r'^version = "(.+?)"', (ROOT / "Cargo.toml").read_text(encoding="utf-8"), re.M).group(1)
    plist = (ROOT / "macos" / "Info.plist").read_text(encoding="utf-8")
    bundle = re.search(r"<key>CFBundleIdentifier</key>\s*<string>([^<]+)</string>", plist).group(1)
    msi, pkg = a.dist / f"bitg-{ver}-x64.msi", a.dist / f"bitg-{ver}-macos.pkg"
    for f in (msi, pkg):
        if not f.exists():
            sys.exit(f"packaging: {f} yo'q (GitHub Release'dan dist/ ga yuklang)")
    w = winget(ver, sha(msi), a.out)
    c = cask(ver, sha(pkg), a.pkg_url or f"https://bitg.uz/download/bitg-{ver}-macos.pkg", bundle, a.out)
    print(f"packaging: {w}, {c}")


if __name__ == "__main__":
    main()
