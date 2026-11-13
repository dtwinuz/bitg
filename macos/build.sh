#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

# bitg macOS kiritish usuli: universal (arm64 + x86_64) bitg.app va o'rnatuvchi .pkg.
#   macos/build.sh          ->  target/macos/bitg-<versiya>-macos.pkg (+ .sha256), target/macos/bitg-test
# Talab: Xcode Command Line Tools, rustup target add aarch64-apple-darwin x86_64-apple-darwin
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$(pwd)"

VER="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
OUT="$ROOT/target/macos"
APP="$OUT/stage/bitg.app"
export MACOSX_DEPLOYMENT_TARGET=11.0
FRAMEWORKS=(-framework Cocoa -framework InputMethodKit -framework Carbon)
# Takrorlanuvchan qurilish: vaqt — commit vaqti, binarlarda qurilish papkasi va cargo kesh yo'li yo'q
# (Swift #file ham), statik kutubxona a'zolari vaqtsiz. Kafolat ad-hoc imzo uchun; Developer ID imzosi
# (BITG_MAC_SIGN_ID) vaqt va sertifikat ma'lumotini qo'shadi.
export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}"
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$ROOT=/build/bitg --remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo"
export ZERO_AR_DATE=1
# -Xlinker -S: debug map (N_OSO) yozilmaydi — u libbitg_ime.a ning to'liq yo'lini va vaqtinchalik obyekt fayllar
# nomini saqlaydi; belgilar jadvali qoladi.
SWIFT_REPRO=(-file-prefix-map "$ROOT=/build/bitg" -Xlinker -S)
STAMP="$(date -u -r "$SOURCE_DATE_EPOCH" +%Y%m%d%H%M.%S)"

rm -rf "$OUT"
mkdir -p "$OUT/bin"

for pair in "aarch64-apple-darwin arm64" "x86_64-apple-darwin x86_64"; do
  set -- $pair
  cargo build --release --locked -p bitg-ime --target "$1"
  LIB="$ROOT/target/$1/release"
  swiftc -O "${SWIFT_REPRO[@]}" -target "$2-apple-macos11.0" -import-objc-header linux/include/bitg_ime.h \
    macos/Sources/*.swift -L "$LIB" -lbitg_ime "${FRAMEWORKS[@]}" -o "$OUT/bin/bitg-$2"
  # sinov: IMK'siz — Session.swift, Keys.swift va soxta matn maydoni
  swiftc -O "${SWIFT_REPRO[@]}" -target "$2-apple-macos11.0" -import-objc-header linux/include/bitg_ime.h \
    macos/Sources/Session.swift macos/Sources/Keys.swift macos/Sources/Prefs.swift macos/test/main.swift \
    -L "$LIB" -lbitg_ime -framework AppKit -o "$OUT/bin/bitg-test-$2"
done
lipo -create "$OUT/bin/bitg-arm64" "$OUT/bin/bitg-x86_64" -output "$OUT/bin/bitg"
lipo -create "$OUT/bin/bitg-test-arm64" "$OUT/bin/bitg-test-x86_64" -output "$OUT/bitg-test"

# bundle
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources/en.lproj"
install -m 755 "$OUT/bin/bitg" "$APP/Contents/MacOS/bitg"
sed "s/@VERSION@/$VER/g" macos/Info.plist > "$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist"
printf 'APPL????' > "$APP/Contents/PkgInfo"
cp macos/InfoPlist.strings "$APP/Contents/Resources/en.lproj/InfoPlist.strings"
# menyu ikonkasi: 32 px + Retina uchun 64 px, bitta TIFF ichida
sips -s format png -z 32 32 web/logo-512.png --out "$OUT/icon.png" >/dev/null
sips -s format png -z 64 64 web/logo-512.png --out "$OUT/icon@2x.png" >/dev/null
tiffutil -cathidpicheck "$OUT/icon.png" "$OUT/icon@2x.png" -out "$APP/Contents/Resources/bitg.tiff"

# imzo: Developer ID bo'lmasa ad-hoc (Apple Silicon imzosiz binarni ishga tushirmaydi)
codesign --force --sign "${BITG_MAC_SIGN_ID:--}" --timestamp=none "$APP"
codesign --verify --strict --verbose=2 "$APP"

# pkg: /Library/Input Methods/bitg.app; ko'chiriladigan bundle emas (boshqa joydagi nusxani yangilamasin)
COMP="$OUT/component.plist"
pkgbuild --analyze --root "$OUT/stage" "$COMP"
plutil -replace 0.BundleIsRelocatable -bool NO "$COMP"
PKG="$OUT/bitg-$VER-macos.pkg"
# Payload (cpio) va Bom fayl vaqtlarini saqlaydi: stage va skriptlar commit vaqtiga tenglanadi.
SCRIPTS="$OUT/pkg-scripts"
cp -R macos/pkg-scripts "$SCRIPTS"
find "$OUT/stage" "$SCRIPTS" -exec touch -h -t "$STAMP" {} +
pkgbuild --root "$OUT/stage" --component-plist "$COMP" --install-location "/Library/Input Methods" \
  --identifier uz.bitg.inputmethod.pkg --version "$VER" --scripts "$SCRIPTS" "$PKG"
python3 -I tools/xar_normalize.py "$PKG"
pkgutil --payload-files "$PKG" >/dev/null # normallashtirilgan TOC o'qiladimi
(cd "$OUT" && shasum -a 256 "$(basename "$PKG")" > "$(basename "$PKG").sha256")
lipo -info "$APP/Contents/MacOS/bitg"
ls -l "$PKG"
