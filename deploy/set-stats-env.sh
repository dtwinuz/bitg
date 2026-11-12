#!/bin/sh
# Serverda: ssh -t dtwin /opt/bitg-site/set-stats-env.sh <ADMIN_CHAT_ID[,ID2...]>
# Statistika, uptime va sertifikat ogohlantirishlari @bitgdevbot (ishchi bot) orqali adminlarga boradi.
# Ommaviy @bitguzbot tokeni bu yerda ishlatilmaydi. Har bir admin avval @bitgdevbot'ga /start yozgan bo'lishi kerak.
# Token so'raladi (ekranda ko'rinmaydi); Enter bosilsa avvalgi token qoladi.
# Terminalsiz ham bo'ladi: ssh dtwin /opt/bitg-site/set-stats-env.sh <ID> < ~/.bitgdevbot-token
# O'chirish: rm /opt/bitg-site/stats.env && docker compose up -d --force-recreate bitg-stats
set -e
cd "$(dirname "$0")"
chat="${1:-}"
case "$chat" in
  -[0-9]*|[0-9]*) ;;
  *) echo "ishlatish: $0 <ADMIN_CHAT_ID[,ID2...]>   (raqam; guruh uchun -100...)"; exit 1 ;;
esac
old="$(sed -n 's/^BOT_TOKEN=//p' stats.env 2>/dev/null | head -1)"
if [ -t 0 ]; then
  printf "@bitgdevbot tokeni (BotFather; yozganingiz ko'rinmaydi, Enter = avvalgisi): "
  stty -echo; read -r t || true; stty echo; echo
else
  read -r t || true
fi
t="$(printf '%s' "$t" | tr -d '\r ')"
[ -n "$t" ] || t="$old"
case "$t" in
  [0-9]*:*) ;;
  *) echo "Bu token emas: format 123456789:AA..."; exit 1 ;;
esac
umask 077
printf 'BOT_TOKEN=%s\nADMIN_CHAT_ID=%s\n' "$t" "$chat" > stats.env
echo "saqlandi: $(pwd)/stats.env"
docker compose up -d --force-recreate bitg-stats >/dev/null
echo "sinov xabari:"
docker exec bitg-stats python -I /app/stats.py --digest-now
