#!/bin/sh
# Serverda: ssh -t dtwin /opt/bitg-site/set-stats-env.sh <ADMIN_CHAT_ID>
# Statistika xulosasi qaysi Telegram chatga borishini belgilaydi. Token @bitguzbot'niki
# (/opt/bitg-bot/.env) — ekranga chiqmaydi. Chat ID: botga /start yozing, so'ng @userinfobot dan ID'ni oling.
# O'chirish: rm /opt/bitg-site/stats.env && docker compose up -d --force-recreate bitg-stats
set -e
cd "$(dirname "$0")"
chat="${1:-}"
case "$chat" in
  -[0-9]*|[0-9]*) ;;
  *) echo "ishlatish: $0 <ADMIN_CHAT_ID>   (raqam; guruh uchun -100...)"; exit 1 ;;
esac
tok="$(sed -n 's/^BOT_TOKEN=//p' /opt/bitg-bot/.env 2>/dev/null | head -1)"
[ -n "$tok" ] || { echo "/opt/bitg-bot/.env da BOT_TOKEN yo'q (avval: /opt/bitg-bot/set-token.sh)"; exit 1; }
umask 077
printf 'BOT_TOKEN=%s\nADMIN_CHAT_ID=%s\n' "$tok" "$chat" > stats.env
echo "saqlandi: $(pwd)/stats.env"
docker compose up -d --force-recreate bitg-stats >/dev/null
echo "sinov xabari:"
docker exec bitg-stats python -I /app/stats.py --digest-now
