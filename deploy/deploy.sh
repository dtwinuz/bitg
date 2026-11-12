#!/usr/bin/env bash
# bitg.uz: faqat origin/main deploy qilinadi.
# web/ + web/pages/ (tools/site/build.py) + WASM yadro (origin/main'ning toza worktree'sida) + MSI + Linux .deb
# (dist/ da, GitHub Release'dan) -> /opt/bitg-site -> health -> IndexNow.
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
[ -z "$(git status --porcelain -- web deploy tools/site)" ] || fail "web/, deploy/ yoki tools/site/ da commit qilinmagan o'zgarish bor"

STAGE="$(mktemp -d)"
cleanup() { git -C "$ROOT" worktree remove --force "$STAGE/src" 2>/dev/null || true; rm -rf "$STAGE"; }
trap cleanup EXIT

# yadro boshqa ishlardan ta'sirlanmasin: origin/main'ning o'zidan yig'iladi
git worktree add --quiet --detach "$STAGE/src" "$SHA"
( cd "$STAGE/src" && $CARGO build -q -p bitg-core --release --target wasm32-unknown-unknown --target-dir "$ROOT/target/site" ) \
  || fail "wasm build (crates/bitg-core/src/wasm.rs origin/main'da bormi?)"
WASM="$ROOT/target/site/wasm32-unknown-unknown/release/bitg_core.wasm"

VER="$(sed -n 's/^version = "\(.*\)"/\1/p' "$STAGE/src/Cargo.toml" | head -1)"
MSI="dist/bitg-$VER-x64.msi"
[ -f "$MSI" ] || fail "$MSI yo'q (installer/build.ps1)"
[ "$(sha256sum < "$MSI" | cut -d' ' -f1)" = "$(tr -d '\r' < "$MSI.sha256" | cut -d' ' -f1)" ] || fail "$MSI sha256 mos emas"
grep -q "download/bitg-$VER-x64.msi" "$STAGE/src/web/index.html" || fail "index.html $VER MSI'ga havola bermaydi"
PKGS="dist/bitg_${VER}_amd64.deb dist/fcitx5-bitg_${VER}_amd64.deb dist/bitg-${VER}-macos.pkg"
for d in $PKGS; do
  [ -f "$d" ] || fail "$d yo'q (GitHub Release v$VER dan yuklang)"
  [ "$(sha256sum < "$d" | cut -d' ' -f1)" = "$(tr -d '\r' < "$d.sha256" | cut -d' ' -f1)" ] || fail "$d sha256 mos emas"
done

SITE="$STAGE/out/site"
# css/js uzoq keshlanadi: har deployda manzil yangilansin (?v=<sha8>)
V="${SHA:0:8}"
# sahifalar web/pages/ dan yig'iladi: shablon, sitemap, 404, JSON-LD (tools/site/build.py)
"${PYTHON:-python}" -I "$STAGE/src/tools/site/build.py" --out "$SITE" --v "$V" --msi "$ROOT/$MSI" \
  || fail "sayt yig'ilmadi (tools/site/build.py)"
mkdir -p "$SITE/download"
cp "$WASM" "$SITE/bitg_core.wasm"
cp "$MSI" "dist/bitg-$VER-x64.msi.sha256" "$SITE/download/"
for d in $PKGS; do cp "$d" "$d.sha256" "$SITE/download/"; done
printf '%s\n' "$SHA" > "$SITE/version.txt"
cp deploy/docker-compose.yml deploy/Caddyfile "$STAGE/out/"

tar -czf - -C "$STAGE/out" site docker-compose.yml Caddyfile \
  | ssh "$HOST" "set -e; mkdir -p '$REMOTE_DIR'; cd '$REMOTE_DIR'
rm -rf new; mkdir new
tar --no-same-owner -xzf - -C new
mv new/docker-compose.yml new/Caddyfile .
rm -rf site.prev; [ -d site ] && mv site site.prev
mv new/site site; rm -rf new
docker compose up -d --force-recreate --remove-orphans
echo '$SHA' > REVISION"

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
fail "bitg-site $SHA ni qaytarmadi (rollback: ssh $HOST 'cd $REMOTE_DIR && rm -rf site && mv site.prev site')"
