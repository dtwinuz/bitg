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
# qayta yaratiladi (masalan image yangilanganda).
#
# Yarim yo'lda qolmaydi: konteyner ko'tarilmasa, reload o'tmasa yoki sayt yangi sha'ni qaytarmasa symlink,
# Caddyfile va docker-compose.yml oldingisiga qaytariladi. Bir vaqtda ikkita deploy bo'lmaydi (flock).
#
# Qo'lda orqaga qaytish (relizlar ro'yxati www/history da, oxirgisi joriy):
#   ln -sfn releases/<reliz> www/current.next && mv -T www/current.next www/current
# -E: ERR trap funksiyalar ichida ham ishlaydi (aks holda `set -e` funksiyadagi xatoda rollback'siz chiqadi)
set -Eeuo pipefail

SHA="${1:?foydalanish: remote.sh <sha>}"
KEEP="${KEEP_RELEASES:-3}"
cd "$(dirname "$0")"

exec 9>.deploy.lock
flock -n 9 || { echo "boshqa deploy ishlayapti (.deploy.lock)" >&2; exit 1; }

[ -d new/site ] && [ -f new/Caddyfile ] && [ -f new/docker-compose.yml ] || { echo "new/ to'liq emas" >&2; exit 1; }

# 1) Tekshiruv jonli saytga tegmasdan: compose fayli o'qiladi, yangi Caddyfile esa bitg-site bilan bir xil
#    image va cheklovlar (read-only, faqat NET_BIND_SERVICE) ostida, tarmoqsiz konteynerda tekshiriladi.
docker compose -f new/docker-compose.yml config --quiet
IMAGE="$(docker compose -f new/docker-compose.yml config --images | grep -m1 '^caddy')"
docker run --rm --network none --read-only --tmpfs /data --tmpfs /config --tmpfs /var/log/caddy \
  --cap-drop ALL --cap-add NET_BIND_SERVICE --security-opt no-new-privileges:true \
  -v "$PWD/new:/c:ro" "$IMAGE" caddy validate --config /c/Caddyfile --adapter caddyfile >/dev/null

mkdir -p logs stats-data stats-public conf www/releases
install -m 755 new/set-stats-env.sh set-stats-env.sh
rm -rf stats && mv new/stats stats

# 2) Reliz: yangi papka (nomida vaqt bor, shuning uchun bir xil sha qayta deploy qilinsa ham joriy papkaga
#    tegilmaydi), keyin symlink atomik almashadi. Oldingi holat orqaga qaytish uchun saqlanadi.
REL="${SHA:0:12}-$(date -u +%Y%m%dT%H%M%SZ)"
mv new/site "www/releases/$REL"
prev="$(readlink www/current 2>/dev/null || true)"
rm -f conf/Caddyfile.prev docker-compose.yml.prev
if [ -f conf/Caddyfile ]; then cp -p conf/Caddyfile conf/Caddyfile.prev; fi
if [ -f docker-compose.yml ]; then cp -p docker-compose.yml docker-compose.yml.prev; fi

rollback() {
  trap - ERR
  set +e
  echo "deploy o'tmadi: oldingi holatga qaytarilmoqda (${prev#releases/})" >&2
  if [ -n "$prev" ]; then ln -sfn "$prev" www/current.next && mv -T www/current.next www/current; fi
  if [ -f conf/Caddyfile.prev ]; then mv -T conf/Caddyfile.prev conf/Caddyfile; fi
  if [ -f docker-compose.yml.prev ]; then mv -T docker-compose.yml.prev docker-compose.yml; fi
  docker compose up -d --remove-orphans
  docker exec bitg-site caddy reload --config /etc/caddy/Caddyfile --adapter caddyfile
  rm -rf "www/releases/$REL"
  exit 1
}
trap rollback ERR

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

# 5) Sayt shu relizni qaytaryaptimi (konteyner ichidan, edge'siz); 20 s ichida bo'lmasa — orqaga
served() { docker exec bitg-site wget -qO- http://127.0.0.1/version.txt 2>/dev/null | grep -qx "$SHA"; }
for _ in $(seq 1 20); do served && break; sleep 1; done
served
trap - ERR
rm -f conf/Caddyfile.prev docker-compose.yml.prev

# 6) Eski tartib (site/, site.prev/, ./Caddyfile) ko'chishdan keyin ishlatilmaydi.
rm -rf site site.prev Caddyfile

# 7) Tarix va tozalash: oxirgi KEEP ta reliz saqlanadi, joriy doim ular ichida.
echo "$REL" >> www/history
tail -n "$KEEP" www/history > www/history.keep
mv www/history.keep www/history
for d in www/releases/*/; do
  n="$(basename "$d")"
  grep -qx "$n" www/history || rm -rf "www/releases/$n"
done
echo "$SHA" > REVISION
echo "reliz: $REL (oldingi: ${prev#releases/})"
