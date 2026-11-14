#!/bin/sh

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

# Serverda: ssh -t dtwin /opt/bitg-bot/set-token.sh
# Tokenni ekranga chiqarmasdan /opt/bitg-bot/.env dagi BOT_TOKEN= qatoriga yozadi (ADMIN_IDS, NEWS_CHANNEL
# va boshqa qatorlar saqlanadi; fayl bo'lmasa yaratiladi) va bot ishlayotgan bo'lsa qayta ishga tushiradi.
set -e
cd "$(dirname "$0")"
printf "@bitguzbot tokeni (BotFather beradi; yozganingiz ko'rinmaydi): "
stty -echo
read -r t
stty echo
echo
case "$t" in
  [0-9]*:*) ;;
  *) echo "Bu token emas: format 123456789:AA..."; exit 1 ;;
esac
umask 077
# vaqtinchalik fayl shu papkada (bir disk: mv atomik); umask 077 bilan huquqi 600
tmp=$(mktemp ./.env.XXXXXX)
trap 'rm -f "$tmp"' EXIT
if [ -f .env ]; then
  grep -v '^BOT_TOKEN=' .env > "$tmp" || true
fi
printf 'BOT_TOKEN=%s\n' "$t" >> "$tmp"
chmod 600 "$tmp"
mv -f "$tmp" .env
trap - EXIT
echo "saqlandi: $(pwd)/.env"
if [ -f docker-compose.yml ] && docker image inspect bitg-bot:latest >/dev/null 2>&1; then
  docker compose up -d --force-recreate >/dev/null && echo "bot qayta ishga tushdi"
fi
