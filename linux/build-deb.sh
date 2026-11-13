#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

# bitg_<versiya>_<arch>.deb: IBus dvigateli + bitg CLI. Ubuntu 22.04 da yig'iladi (22.04, 24.04, Debian 12 uchun).
#   sudo apt install libibus-1.0-dev dpkg-dev && linux/build-deb.sh   ->  target/deb/bitg_X_amd64.deb (+ .sha256)
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$(pwd)"

VER="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
ARCH="$(dpkg --print-architecture)"
NAME="bitg_${VER}_${ARCH}"
OUT="$ROOT/target/deb"
PKG="$OUT/$NAME"

# Takrorlanadigan build: bir commit'dan har qanday papkada bir xil bayt (CI `Reproducible build` qadami tekshiradi).
# Vaqt — commit vaqti (dpkg-deb ar/tar vaqtlarini shunga qisadi), yo'llar — doimiy prefiks.
export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}"
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$ROOT=/build/bitg --remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo"
export CFLAGS="${CFLAGS:--O2 -g} -ffile-prefix-map=$ROOT=/build/bitg"
export CXXFLAGS="${CXXFLAGS:--O2 -g} -ffile-prefix-map=$ROOT=/build/bitg"

cargo build --release --locked -p bitg-ime -p bitg-cli
make -C linux/ibus clean
make -C linux/ibus VERSION="$VER"

rm -rf "$PKG"
install -d "$PKG/DEBIAN" "$PKG/usr/bin" "$PKG/usr/libexec" "$PKG/usr/share/ibus/component" \
  "$PKG/usr/share/icons/hicolor/scalable/apps" "$PKG/usr/share/doc/bitg" "$PKG/usr/share/applications" "$PKG/etc/xdg/autostart"
install -m 755 -s linux/ibus/ibus-engine-bitg "$PKG/usr/libexec/ibus-engine-bitg"
install -m 755 -s target/release/bitg "$PKG/usr/bin/bitg"
sed "s/@VERSION@/$VER/" linux/ibus/bitg.xml > "$PKG/usr/share/ibus/component/bitg.xml"
chmod 644 "$PKG/usr/share/ibus/component/bitg.xml"
install -m 644 web/icon.svg "$PKG/usr/share/icons/hicolor/scalable/apps/bitg.svg"
install -m 644 linux/indicator/bitg-disabled.svg "$PKG/usr/share/icons/hicolor/scalable/apps/bitg-disabled.svg"
install -m 755 linux/indicator/bitg-indicator "$PKG/usr/bin/bitg-indicator"
install -m 644 linux/indicator/bitg.desktop "$PKG/usr/share/applications/bitg.desktop"
install -m 644 linux/indicator/bitg-autostart.desktop "$PKG/etc/xdg/autostart/bitg.desktop"
install -m 644 linux/copyright "$PKG/usr/share/doc/bitg/copyright"
# native paket: changelog git tarixidan emas — release sahifasiga havola
printf 'bitg (%s) stable; urgency=medium\n\n  * Release %s: https://github.com/dtwinuz/bitg/releases\n\n -- dtwinuz <info@dtwin.uz>  %s\n' \
  "$VER" "$VER" "$(date -R -d "@$(git log -1 --format=%ct)")" | gzip -9n > "$PKG/usr/share/doc/bitg/changelog.gz"
chmod 644 "$PKG/usr/share/doc/bitg/changelog.gz"

# bog'liqliklar binarning o'zidan (libc6, libglib2.0-0, libibus-1.0-5 ...)
SH="$(mktemp -d)"
mkdir "$SH/debian" && printf 'Source: bitg\n' > "$SH/debian/control"
DEPS="$(cd "$SH" && dpkg-shlibdeps -O "$PKG/usr/libexec/ibus-engine-bitg" "$PKG/usr/bin/bitg" | sed -n 's/^shlibs:Depends=//p')"
rm -rf "$SH"

SIZE="$(du -sk --exclude=DEBIAN "$PKG" | cut -f1)"
cat > "$PKG/DEBIAN/control" <<EOF
Package: bitg
Version: $VER
Architecture: $ARCH
Maintainer: dtwinuz <info@dtwin.uz>
Installed-Size: $SIZE
Depends: $DEPS, ibus (>= 1.5.20), python3-gi, gir1.2-gtk-3.0, gir1.2-ayatanaappindicator3-0.1
Section: utils
Priority: optional
Homepage: https://bitg.uz
Description: Uzbek input method for the new 2026 Latin alphabet
 bitg converts Uzbek text to the new Latin alphabet (o' g' sh ch to
 the new letters) as you type, in any IBus application. English words,
 links and code are left alone; Backspace right after a replacement
 restores the original word. Works offline. Includes the bitg CLI.
EOF
cat > "$PKG/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
if [ "$1" = configure ] && command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q -t /usr/share/icons/hicolor || true
fi
exit 0
EOF
chmod 755 "$PKG/DEBIAN/postinst"

dpkg-deb --root-owner-group --build "$PKG" "$OUT/$NAME.deb" >/dev/null
( cd "$OUT" && sha256sum "$NAME.deb" > "$NAME.deb.sha256" )
echo "$OUT/$NAME.deb"

# fcitx5-bitg: Fcitx5 qo'shimchasi (libfcitx5core-dev bor bo'lsa)
pkg-config --exists Fcitx5Core || { echo "Fcitx5 dev yo'q — fcitx5-bitg o'tkazib yuborildi"; exit 0; }
make -C linux/fcitx5 clean
make -C linux/fcitx5
FNAME="fcitx5-bitg_${VER}_${ARCH}"
FPKG="$OUT/$FNAME"
MA="$(dpkg-architecture -qDEB_HOST_MULTIARCH)"
rm -rf "$FPKG"
install -d "$FPKG/DEBIAN" "$FPKG/usr/lib/$MA/fcitx5" "$FPKG/usr/share/fcitx5/addon" "$FPKG/usr/share/fcitx5/inputmethod" \
  "$FPKG/usr/share/icons/hicolor/scalable/apps" "$FPKG/usr/share/doc/fcitx5-bitg"
install -m 644 -s linux/fcitx5/bitg.so "$FPKG/usr/lib/$MA/fcitx5/bitg.so"
sed "s/@VERSION@/$VER/" linux/fcitx5/bitg-addon.conf > "$FPKG/usr/share/fcitx5/addon/bitg.conf"
install -m 644 linux/fcitx5/bitg-im.conf "$FPKG/usr/share/fcitx5/inputmethod/bitg.conf"
chmod 644 "$FPKG/usr/share/fcitx5/addon/bitg.conf"
install -m 644 web/icon.svg "$FPKG/usr/share/icons/hicolor/scalable/apps/fcitx-bitg.svg"
sed -i 's/^Icon=bitg$/Icon=fcitx-bitg/' "$FPKG/usr/share/fcitx5/inputmethod/bitg.conf"
install -m 644 linux/copyright "$FPKG/usr/share/doc/fcitx5-bitg/copyright"
cp "$PKG/usr/share/doc/bitg/changelog.gz" "$FPKG/usr/share/doc/fcitx5-bitg/changelog.gz"

SH="$(mktemp -d)"
mkdir "$SH/debian" && printf 'Source: bitg\n' > "$SH/debian/control"
FDEPS="$(cd "$SH" && dpkg-shlibdeps -O "$FPKG/usr/lib/$MA/fcitx5/bitg.so" | sed -n 's/^shlibs:Depends=//p')"
rm -rf "$SH"
FSIZE="$(du -sk --exclude=DEBIAN "$FPKG" | cut -f1)"
cat > "$FPKG/DEBIAN/control" <<EOF
Package: fcitx5-bitg
Version: $VER
Architecture: $ARCH
Maintainer: dtwinuz <info@dtwin.uz>
Installed-Size: $FSIZE
Depends: $FDEPS, fcitx5
Section: utils
Priority: optional
Homepage: https://bitg.uz
Description: Uzbek input method for the new 2026 Latin alphabet (Fcitx5)
 bitg converts Uzbek text to the new Latin alphabet (o' g' sh ch to
 the new letters) as you type, in any Fcitx5 application. English words,
 links and code are left alone; Backspace right after a replacement
 restores the original word. Works offline.
EOF
dpkg-deb --root-owner-group --build "$FPKG" "$OUT/$FNAME.deb" >/dev/null
( cd "$OUT" && sha256sum "$FNAME.deb" > "$FNAME.deb.sha256" )
echo "$OUT/$FNAME.deb"
