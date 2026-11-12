#!/usr/bin/env bash
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

cargo build --release --locked -p bitg-ime -p bitg-cli
make -C linux/ibus clean
make -C linux/ibus VERSION="$VER"

rm -rf "$PKG"
install -d "$PKG/DEBIAN" "$PKG/usr/bin" "$PKG/usr/libexec" "$PKG/usr/share/ibus/component" \
  "$PKG/usr/share/icons/hicolor/scalable/apps" "$PKG/usr/share/doc/bitg"
install -m 755 -s linux/ibus/ibus-engine-bitg "$PKG/usr/libexec/ibus-engine-bitg"
install -m 755 -s target/release/bitg "$PKG/usr/bin/bitg"
sed "s/@VERSION@/$VER/" linux/ibus/bitg.xml > "$PKG/usr/share/ibus/component/bitg.xml"
chmod 644 "$PKG/usr/share/ibus/component/bitg.xml"
install -m 644 web/icon.svg "$PKG/usr/share/icons/hicolor/scalable/apps/bitg.svg"
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
Depends: $DEPS, ibus (>= 1.5.20)
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
