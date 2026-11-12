#!/usr/bin/env python3
"""winget manifesti va Homebrew cask'ni dist/ dagi release fayllaridan yaratadi.

  gh release download v<v> -R dtwinuz/bitg -D dist --clobber
  python tools/release/packaging.py            # versiya Cargo.toml'dan, fayllar dist/ dan

Natija (repoda saqlanadi):
  packaging/winget/manifests/d/dtwinuz/bitg/<v>/  -> microsoft/winget-pkgs ga PR ("Update: dtwinuz.bitg to <v>");
                                                     packaging workflow uni Windows'da o'rnatib sinaydi
  packaging/homebrew/Casks/bitg.rb                -> dtwinuz/homebrew-tap ga PR; tap CI macOS'da o'rnatib sinaydi

Ikkala havola ham GitHub Release'ga: o'zgarmas, SHA-256 hech qachon siljimaydi.
"""
import argparse
import hashlib
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPO = "https://github.com/dtwinuz/bitg"
SCHEMA = "1.28.0"


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def write(p, text):
    p.write_text(text.lstrip("\n"), encoding="utf-8", newline="\n")


def upgrade_code():
    wxs = (ROOT / "installer" / "Package.wxs").read_text(encoding="utf-8")
    return re.search(r'UpgradeCode="(\{[0-9A-F-]{36}\})"', wxs).group(1)


def product_code(msi):
    """MSI Property jadvalidan ProductCode (har build'da yangi). Windows'da COM orqali, boshqa joyda --product-code."""
    ps = (
        "$i=New-Object -ComObject WindowsInstaller.Installer;"
        "$d=$i.GetType().InvokeMember('OpenDatabase','InvokeMethod',$null,$i,@($env:BITG_MSI,0));"
        "$v=$d.GetType().InvokeMember('OpenView','InvokeMethod',$null,$d,@(\"SELECT Value FROM Property WHERE Property='ProductCode'\"));"
        "$v.GetType().InvokeMember('Execute','InvokeMethod',$null,$v,$null);"
        "$r=$v.GetType().InvokeMember('Fetch','InvokeMethod',$null,$v,$null);"
        "$r.GetType().InvokeMember('StringData','GetProperty',$null,$r,1)"
    )
    try:
        out = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", ps], capture_output=True,
                             text=True, timeout=60, env={**os.environ, "BITG_MSI": str(msi.resolve())}).stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return None
    return out if re.fullmatch(r"\{[0-9A-F-]{36}\}", out) else None


def winget(ver, msi_sha, pcode, ucode, out):
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
    ProductCode: "{pcode}"
    AppsAndFeaturesEntries:
      - DisplayName: bitg
        Publisher: dtwinuz
        DisplayVersion: {ver}
        ProductCode: "{pcode}"
        UpgradeCode: "{ucode}"
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
LicenseUrl: {REPO}/blob/main/LICENSE-MIT
Copyright: Copyright (c) 2026 dtwinuz
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


def cask(ver, pkg_sha, bundle, out):
    d = out / "homebrew" / "Casks"
    d.mkdir(parents=True, exist_ok=True)
    p = d / "bitg.rb"
    write(p, f'''cask "bitg" do
  version "{ver}"
  sha256 "{pkg_sha}"

  url "{REPO}/releases/download/v#{{version}}/bitg-#{{version}}-macos.pkg",
      verified: "github.com/dtwinuz/bitg/"
  name "bitg"
  desc "Uzbek input method for the new 2026 Latin alphabet"
  homepage "https://bitg.uz/"

  livecheck do
    url "{REPO}"
    strategy :github_latest
  end

  depends_on :macos

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
    ap.add_argument("--product-code", default=None, help="MSI ProductCode (Windows'da MSI'dan o'zi o'qiladi)")
    a = ap.parse_args()
    ver = re.search(r'^version = "(.+?)"', (ROOT / "Cargo.toml").read_text(encoding="utf-8"), re.M).group(1)
    plist = (ROOT / "macos" / "Info.plist").read_text(encoding="utf-8")
    bundle = re.search(r"<key>CFBundleIdentifier</key>\s*<string>([^<]+)</string>", plist).group(1)
    msi, pkg = a.dist / f"bitg-{ver}-x64.msi", a.dist / f"bitg-{ver}-macos.pkg"
    for f in (msi, pkg):
        if not f.exists():
            sys.exit(f"packaging: {f} yo'q (GitHub Release'dan dist/ ga yuklang)")
    pcode = a.product_code or product_code(msi)
    if not pcode:
        sys.exit("packaging: MSI ProductCode o'qilmadi (--product-code bering)")
    w = winget(ver, sha(msi), pcode, upgrade_code(), a.out)
    c = cask(ver, sha(pkg), bundle, a.out)
    print(f"packaging: {w}, {c}")


if __name__ == "__main__":
    main()
