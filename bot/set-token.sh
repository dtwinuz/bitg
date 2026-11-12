#!/bin/sh

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

# Serverda: ssh -t dtwin /opt/bitg-bot/set-token.sh
# Tokenni ekranga chiqarmasdan /opt/bitg-bot/.env ga yozadi va bot ishlayotgan bo'lsa qayta ishga tushiradi.
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
printf 'BOT_TOKEN=%s\n' "$t" > .env
echo "saqlandi: $(pwd)/.env"
if [ -f docker-compose.yml ] && docker image inspect bitg-bot:latest >/dev/null 2>&1; then
  docker compose up -d --force-recreate >/dev/null && echo "bot qayta ishga tushdi"
fi
