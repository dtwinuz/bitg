#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

# bitg.uz: faqat origin/main deploy qilinadi.
# web/ + web/pages/ (tools/site/build.py) + WASM yadro (origin/main'ning toza worktree'sida) + MSI + Linux .deb
# + Mac .pkg (GitHub Release'dan, attestatsiya tekshiruvi bilan) + imzolangan apt ombori + statistika xizmati
# -> /opt/bitg-site -> health -> IndexNow.
set -euo pipefail

HOST="${1:-dtwin}"
REMOTE_DIR="${REMOTE_DIR:-/opt/bitg-site}"
URL="${URL:-https://bitg.uz}"
CARGO="${CARGO:-cargo +stable-x86_64-pc-windows-msvc}"

fail() { echo "deploy to'xtatildi: $*" >&2; exit 1; }
cd "$(dirname "$0")/.."
ROOT="$(pwd)"

git fetch --quiet origin main
SHA="$(git rev-parse origin/main)"
[ "$(git rev-parse HEAD)" = "$SHA" ] || fail "faqat origin/main (HEAD $(git rev-parse --short HEAD))"
[ -z "$(git status --porcelain -- web deploy tools/site tools/apt)" ] || fail "web/, deploy/ yoki tools/ da commit qilinmagan o'zgarish bor"

STAGE="$(mktemp -d)"
cleanup() { git -C "$ROOT" worktree remove --force "$STAGE/src" 2>/dev/null || true; rm -rf "$STAGE"; }
trap cleanup EXIT

# yadro boshqa ishlardan ta'sirlanmasin: origin/main'ning o'zidan yig'iladi
git worktree add --quiet --detach "$STAGE/src" "$SHA"
( cd "$STAGE/src" && $CARGO build -q -p bitg-core --release --target wasm32-unknown-unknown --target-dir "$ROOT/target/site" ) \
  || fail "wasm build (crates/bitg-core/src/wasm.rs origin/main'da bormi?)"
WASM="$ROOT/target/site/wasm32-unknown-unknown/release/bitg_core.wasm"

VER="$(sed -n 's/^version = "\(.*\)"/\1/p' "$STAGE/src/Cargo.toml" | head -1)"
# Reliz fayllari faqat GitHub Release v$VER dan olinadi va CI qurganiga attestatsiya bilan tekshiriladi:
# sayt aynan release.yml yig'gan faylni tarqatadi, lokal yig'ilgan yoki almashtirilgan faylni emas.
DIST="$STAGE/dist"
MSI="$DIST/bitg-$VER-x64.msi"
PKGS="$DIST/bitg_${VER}_amd64.deb $DIST/fcitx5-bitg_${VER}_amd64.deb $DIST/bitg-${VER}-macos.pkg"
gh release download "v$VER" -R dtwinuz/bitg -D "$DIST" -p '*.msi' -p '*.msi.sha256' -p '*.deb' -p '*.deb.sha256' \
  -p '*.pkg' -p '*.pkg.sha256' || fail "GitHub Release v$VER yuklanmadi (gh auth status?)"
for d in $MSI $PKGS; do
  [ -f "$d" ] || fail "$(basename "$d") Release v$VER da yo'q"
  [ "$(sha256sum < "$d" | cut -d' ' -f1)" = "$(tr -d '\r' < "$d.sha256" | cut -d' ' -f1)" ] || fail "$d sha256 mos emas"
  gh attestation verify "$d" -R dtwinuz/bitg --signer-workflow dtwinuz/bitg/.github/workflows/release.yml \
    --source-ref "refs/tags/v$VER" --deny-self-hosted-runners >/dev/null \
    || fail "$(basename "$d"): attestatsiya tekshiruvi o'tmadi (release.yml v$VER qurmagan)"
done
echo "reliz fayllari tekshirildi: attestatsiya (release.yml, v$VER)"

SITE="$STAGE/out/site"
# css/js uzoq keshlanadi: har deployda manzil yangilansin (?v=<sha8>)
V="${SHA:0:8}"
# sahifalar web/pages/ dan yig'iladi: shablon, sitemap, 404, JSON-LD (tools/site/build.py)
"${PYTHON:-python}" -I "$STAGE/src/tools/site/build.py" --out "$SITE" --v "$V" --msi "$MSI" \
  || fail "sayt yig'ilmadi (tools/site/build.py)"
mkdir -p "$SITE/download"
cp "$WASM" "$SITE/bitg_core.wasm"
cp "$MSI" "$MSI.sha256" "$SITE/download/"
for d in $PKGS; do cp "$d" "$d.sha256" "$SITE/download/"; done
printf '%s\n' "$SHA" > "$SITE/version.txt"

# apt ombori (bitg.uz/apt): .deb'lar imzolangan indeks bilan. Maxfiy kalit faqat shu kompyuterda (tools/apt/genkey.sh).
APT_GNUPG="${BITG_APT_GNUPG:-$HOME/.bitg-apt-gnupg}"
[ -d "$APT_GNUPG" ] || fail "apt imzo kaliti yo'q: $APT_GNUPG (zaxiradan tiklang yoki tools/apt/genkey.sh)"
"${PYTHON:-python}" -I "$STAGE/src/tools/apt/apt_repo.py" --out "$SITE/apt" --gnupg "$APT_GNUPG" \
  "$DIST/bitg_${VER}_amd64.deb" "$DIST/fcitx5-bitg_${VER}_amd64.deb" || fail "apt ombori yig'ilmadi"

cp "$STAGE/src/deploy/docker-compose.yml" "$STAGE/src/deploy/Caddyfile" "$STAGE/src/deploy/set-stats-env.sh" \
  "$STAGE/src/deploy/remote.sh" "$STAGE/out/"
mkdir -p "$STAGE/out/stats" && cp -r "$STAGE/src/deploy/stats/stats.py" "$STAGE/src/deploy/stats/posts" "$STAGE/out/stats/"
"${PYTHON:-python}" -I "$STAGE/out/stats/stats.py" --selftest >/dev/null || fail "stats.py selftest"

# uzilishsiz: reliz papkasi + symlink + caddy reload, konteyner qayta yaratilmaydi (deploy/remote.sh)
tar -czf - -C "$STAGE/out" site stats docker-compose.yml Caddyfile set-stats-env.sh remote.sh \
  | ssh "$HOST" "set -e; mkdir -p '$REMOTE_DIR'; cd '$REMOTE_DIR'
rm -rf new; mkdir new
tar --no-same-owner -xzf - -C new
install -m 755 new/remote.sh remote.sh
./remote.sh '$SHA'"

# IndexNow: barcha URL'lar Bing va Yandex'ga (INDEXNOW=0 — o'tkazib yuborish). Xato deploy'ni to'xtatmaydi.
indexnow() {
  [ "${INDEXNOW:-1}" = "1" ] || return 0
  local key list
  key="$(ls "$STAGE/src/web" | sed -n 's/^\([0-9a-f]\{32\}\)\.txt$/\1/p' | head -1)"
  [ -n "$key" ] || { echo "indexnow: kalit fayli yo'q"; return 0; }
  list="$(sed 's/.*/"&"/' "$SITE/urls.txt" | paste -sd, -)"
  curl -s -o /dev/null -w "indexnow: %{http_code}\n" -m 30 -X POST https://api.indexnow.org/indexnow \
    -H 'Content-Type: application/json; charset=utf-8' \
    -d "{\"host\":\"bitg.uz\",\"key\":\"$key\",\"keyLocation\":\"https://bitg.uz/$key.txt\",\"urlList\":[$list]}" || true
}

# konteyner o'zi javob bersin (edge ulanmagan bo'lsa ham), keyin ommaviy manzil
for _ in $(seq 1 20); do
  if ssh "$HOST" "docker exec bitg-site wget -qO- http://127.0.0.1/version.txt" 2>/dev/null | grep -q "$SHA"; then
    if curl -fsS "$URL/version.txt" 2>/dev/null | grep -q "$SHA"; then echo "deploy tayyor: $SHA -> $URL"
    else echo "konteyner tayyor: $SHA ($URL hali javob bermadi: dtwin edge'da bitg bloki bormi?)"; fi
    indexnow
    exit 0
  fi
  sleep 3
done
fail "bitg-site $SHA ni qaytarmadi (orqaga: ssh $HOST, cd $REMOTE_DIR, tail -n2 www/history, keyin ln -sfn releases/<oldingi> www/current.next && mv -T www/current.next www/current)"
