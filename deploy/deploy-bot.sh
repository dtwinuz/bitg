#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

# @bitguzbot: faqat origin/main deploy qilinadi. Manba serverga yuboriladi, image o'sha yerda quriladi.
# Token serverda /opt/bitg-bot/.env (BOT_TOKEN=...) — repoga ham, bu skriptga ham kirmaydi.
# Birinchi marta tokenni qo'yish: ssh -t dtwin /opt/bitg-bot/set-token.sh
set -euo pipefail

HOST="${1:-dtwin}"
REMOTE_DIR="${REMOTE_DIR:-/opt/bitg-bot}"

fail() { echo "bot deploy to'xtatildi: $*" >&2; exit 1; }
cd "$(dirname "$0")/.."

git fetch --quiet origin main
SHA="$(git rev-parse origin/main)"
[ "$(git rev-parse HEAD)" = "$SHA" ] || fail "faqat origin/main (HEAD $(git rev-parse --short HEAD))"
[ -z "$(git status --porcelain -- bot deploy)" ] || fail "bot/ yoki deploy/ da commit qilinmagan o'zgarish bor"
ssh "$HOST" "test -s '$REMOTE_DIR/.env'" || fail "serverda $REMOTE_DIR/.env yo'q: ssh -t $HOST $REMOTE_DIR/set-token.sh"

# autocrlf=false: Windows'da archive .sh fayllarni CRLF qilib chiqaradi, Linux'da ishlamaydi.
git -c core.autocrlf=false archive --format=tar "$SHA" Cargo.toml crates/bitg-core bot | gzip \
  | ssh "$HOST" "set -e; cd '$REMOTE_DIR'
rm -rf src; mkdir src
tar --no-same-owner -xzf - -C src
cp src/bot/docker-compose.yml docker-compose.yml
install -m 755 src/bot/set-token.sh set-token.sh
install -d -m 755 data && chown 65534:65534 data
docker build -q -t bitg-bot:${SHA:0:8} -t bitg-bot:latest -f src/bot/Dockerfile src >/dev/null
docker compose up -d --force-recreate --remove-orphans
echo '$SHA' > REVISION
docker image prune -f >/dev/null"

for _ in $(seq 1 15); do
  if ssh "$HOST" "docker logs --since 3m bitg-bot 2>&1 | grep -q 'ishga tushdi'"; then
    ssh "$HOST" "docker logs --since 3m bitg-bot 2>&1 | tail -5"
    echo "bot tayyor: $SHA"
    exit 0
  fi
  sleep 2
done
ssh "$HOST" "docker logs --tail 20 bitg-bot" >&2 || true
fail "bot ishga tushmadi"
