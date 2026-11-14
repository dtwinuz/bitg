#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

# bitg SRPM yig'uvchi: Source0 (manba tarball) + Source1 (cargo vendor) -> rpmbuild -bs -> .src.rpm yo'li.
# COPR/OBS shu SRPM'dan tarmoqsiz yig'adi. Fedora konteynerida ishlaydi
# (kerak: rpm-build cargo git-core tar xz curl).
#
#   packaging/rpm/make-srpm.sh                     # spec'dagi versiya, Source0 GitHub teg tarball'idan
#   packaging/rpm/make-srpm.sh 0.4.0 --from-git    # Source0 = `git archive HEAD` (CI, teg hali yo'q)
#   packaging/rpm/make-srpm.sh --source0 x.tar.gz  # tayyor tarball (ichida bitg-<V>/)
#   --outdir DIR   natija papkasi (standart: target/rpm)
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
SPEC="$HERE/bitg.spec"
SPEC_VER="$(sed -n 's/^Version:[[:space:]]*//p' "$SPEC" | head -1)"

VER=""
MODE=url
REF=HEAD
SRC0_FILE=""
OUTDIR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --from-git) MODE=git; if [ $# -gt 1 ] && [ "${2#-}" = "$2" ]; then REF="$2"; shift; fi ;;
    --source0) MODE=file; SRC0_FILE="$(realpath "$2")"; shift ;;
    --outdir) OUTDIR="$2"; shift ;;
    -h|--help) sed -n '7,15p' "$0"; exit 0 ;;
    -*) echo "noma'lum parametr: $1" >&2; exit 2 ;;
    *) VER="$1" ;;
  esac
  shift
done
VER="${VER:-$SPEC_VER}"
if [ "$VER" != "$SPEC_VER" ]; then
  echo "versiya $VER, lekin bitg.spec da Version: $SPEC_VER — avval spec'ni (va %changelog) yangilang" >&2
  exit 1
fi

OUTDIR="${OUTDIR:-$HERE/../../target/rpm}"
mkdir -p "$OUTDIR"
OUTDIR="$(cd "$OUTDIR" && pwd)"
TOP="$OUTDIR/rpmbuild"
rm -rf "$TOP"
mkdir -p "$TOP/SOURCES" "$TOP/SPECS" "$TOP/SRPMS"
SRC0="$TOP/SOURCES/bitg-$VER.tar.gz"
SRC1="$TOP/SOURCES/bitg-$VER-vendor.tar.xz"

case "$MODE" in
  url) curl -fsSL --retry 3 -o "$SRC0" "https://github.com/dtwinuz/bitg/archive/refs/tags/v$VER.tar.gz" ;;
  # pastki papkadan `git archive` faqat o'sha papkani oladi — repo ildizidan yurgiziladi
  git) git -C "$(git -C "$HERE" rev-parse --show-toplevel)" -c core.autocrlf=false -c core.eol=lf archive --format=tar --prefix="bitg-$VER/" "$REF" | gzip -9n > "$SRC0" ;;
  file) cp "$SRC0_FILE" "$SRC0" ;;
esac

# Source1: Cargo.lock'dagi aniq versiyalar (--locked), tarball baytma-bayt takrorlanadi (tartib, vaqt, egasi qotirilgan)
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
tar -xzf "$SRC0" -C "$WORK"
[ -f "$WORK/bitg-$VER/Cargo.lock" ] || { echo "Source0 ichida bitg-$VER/Cargo.lock yo'q" >&2; exit 1; }
# manba versiyasi spec'dagi bilan bir xil bo'lsin (eski teg yoki yangilanmagan spec)
SRC_VER="$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$WORK/bitg-$VER/Cargo.toml" | head -1)"
[ "$SRC_VER" = "$VER" ] || { echo "Source0 Cargo.toml versiyasi $SRC_VER, spec'da $VER" >&2; exit 1; }
(
  cd "$WORK/bitg-$VER"
  export CARGO_HOME="$WORK/cargo-home"
  cargo vendor --locked --quiet vendor > /dev/null
  MTIME="$(stat -c %Y Cargo.lock)"
  tar --sort=name --mtime="@$MTIME" --owner=0 --group=0 --numeric-owner --format=gnu -cf - vendor \
    | xz -T1 -9 > "$SRC1"
)

cp "$SPEC" "$TOP/SPECS/bitg.spec"
rpmbuild -bs --define "_topdir $TOP" "$TOP/SPECS/bitg.spec" >&2
SRPM="$(ls "$TOP"/SRPMS/bitg-"$VER"-*.src.rpm)"
mv "$SRPM" "$OUTDIR/"
echo "$OUTDIR/$(basename "$SRPM")"
