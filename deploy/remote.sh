#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
#
# SPDX-License-Identifier: MIT OR Apache-2.0

# Serverda, /opt/bitg-site ichida ishlaydi: deploy.sh tar'ni new/ ga ochadi, shu faylni o'rnatadi va
# `./remote.sh <sha>` ni chaqiradi.
#
# Uzilishsiz deploy: sayt www/releases/<reliz> ga yoziladi, www/current symlinki bitta rename bilan (mv -T)
# almashtiriladi. bitg-site konteyneri qayta yaratilmaydi, shuning uchun ochiq yuklashlar eski fayldan
# oxirigacha davom etadi, yangi so'rovlar esa yangi relizni oladi. Caddyfile `caddy reload` bilan
# yangilanadi, bu ulanishlarni uzmaydi. Konteyner faqat docker-compose.yml dagi ta'rif o'zgarganda
# qayta yaratiladi (masalan eski tartibdan birinchi ko'chishda).
#
# Orqaga qaytish (relizlar ro'yxati www/history da, oxirgisi joriy):
#   ln -sfn releases/<reliz> www/current.next && mv -T www/current.next www/current
set -euo pipefail

SHA="${1:?foydalanish: remote.sh <sha>}"
KEEP="${KEEP_RELEASES:-3}"
cd "$(dirname "$0")"
[ -d new/site ] && [ -f new/Caddyfile ] && [ -f new/docker-compose.yml ] || { echo "new/ to'liq emas" >&2; exit 1; }

# 1) Yangi Caddyfile avval alohida, tarmoqsiz konteynerda tekshiriladi: xato config jonli saytga yetmasin.
docker run --rm --network none --tmpfs /var/log/caddy -v "$PWD/new:/c:ro" caddy:2-alpine \
  caddy validate --config /c/Caddyfile --adapter caddyfile >/dev/null

mkdir -p logs stats-data stats-public conf www/releases
install -m 755 new/set-stats-env.sh set-stats-env.sh
rm -rf stats && mv new/stats stats

# 2) Reliz: yangi papka (nomida vaqt bor, shuning uchun bir xil sha qayta deploy qilinsa ham joriy papkaga
#    tegilmaydi), keyin symlink atomik almashadi.
REL="${SHA:0:12}-$(date -u +%Y%m%dT%H%M%SZ)"
mv new/site "www/releases/$REL"
prev="$(readlink www/current 2>/dev/null || true)"
ln -sfn "releases/$REL" www/current.next
mv -T www/current.next www/current

# 3) Config: bir fayl tizimida mv atomik. Caddyfile papka orqali ulanadi (conf/ -> /etc/caddy), shuning
#    uchun almashtirilgan fayl konteynerga ko'rinadi.
mv new/Caddyfile conf/Caddyfile.next
mv -T conf/Caddyfile.next conf/Caddyfile
mv new/docker-compose.yml docker-compose.yml
rm -rf new

# 4) Konteynerlar. bitg-site ta'rifi o'zgarmagan bo'lsa tegilmaydi. bitg-stats ./stats ni ulaydi va u
#    yuqorida almashtirildi, shuning uchun doim qayta yaratiladi (saytga ta'siri yo'q).
docker compose up -d --remove-orphans
docker compose up -d --no-deps --force-recreate bitg-stats
docker exec bitg-site caddy reload --config /etc/caddy/Caddyfile --adapter caddyfile

# 5) Eski tartib (site/, site.prev/, ./Caddyfile) ko'chishdan keyin ishlatilmaydi.
rm -rf site site.prev Caddyfile

# 6) Tarix va tozalash: oxirgi KEEP ta reliz saqlanadi, joriy doim ular ichida.
echo "$REL" >> www/history
tail -n "$KEEP" www/history > www/history.keep
mv www/history.keep www/history
for d in www/releases/*/; do
  n="$(basename "$d")"
  grep -qx "$n" www/history || rm -rf "www/releases/$n"
done
echo "$SHA" > REVISION
echo "reliz: $REL (oldingi: ${prev#releases/})"
