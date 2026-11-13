# Ubuntu / GNOME tekshiruvi (#75)

Bu yo'riqnoma haqiqiy Ubuntu sessiyasida xatoni takrorlash uchun. Headless IBus sinovi
XKB tartibi, GNOME paneli va real GTK/Qt dasturlarini tekshirmaydi.

## Diagnostika

```bash
bash linux/test/ubuntu_diagnostics.sh
```

Skript sozlamalarni o'zgartirmaydi va yozilgan matnni yig'maydi. Natijani issue'ga
qo'shishdan oldin shaxsiy ma'lumotlar yo'qligini tekshiring.

1. Ubuntu 22.04 va 24.04; GNOME Wayland va X11 sessiyalarini alohida qayd eting.
2. Sozlamalar → Klaviatura → Kiritish manbalari orqali bitg'ni tanlang.
   Avval lotin, keyin kirill tartibidan o'tib ko'ring. `ibus engine` bitg
   tanlanganini ko'rsatishi kerak (IBus ishlatilganda).
3. GTK matn muharriri, brauzer matn maydoni va Telegram/Qt'da
   `choy shahar ` → `çoy şahar `, `чой шаҳар ` → `çoy şahar `
   holatlarini tekshiring. Probelgacha preedit va probeldan keyingi natijani
   alohida yozing. Backspace, fokus va input source almashishini ham sinang.
4. `~/.config/bitg/settings.ini` mavjud bo'lsa, `enabled=1`,
   `auto_convert=1`, `cyrillic=1` ekanini tekshiring. Sinov uchun
   faylni vaqtincha boshqa nomga ko'chirish mumkin; keyin tiklang.
   `XDG_CONFIG_HOME` belgilangan bo'lsa, sozlamalar shu joydagi `bitg/` da.
5. IBus uchun `dbus-run-session -- python3 linux/test/ibus_e2e.py` headless
   regression sinovini ishga tushiring. Bu GUI sinovi o'rnini bosmaydi.

## Yuqori panel belgisi

Hozirgi Linux .deb faqat IBus dvigateli va CLI'ni o'rnatadi; mustaqil
bitg tray/StatusNotifier jarayoni, kontekst menyusi, .desktop launcher va
avtostart yo'q. GNOME'ning input source indikatori mustaqil bitg tray
belgisi emas. AppIndicator/StatusNotifier qo'llab-quvvatlanishi GNOME
versiyasi va o'rnatilgan shell extension'ga bog'liq; ularni alohida tekshiring.

Bu diagnostika bosqichi indikatorni yaratmaydi va real Ubuntu xatosining
ildiz sababini tasdiqlamaydi. GUI natijalariga asoslanib keyingi tuzatish
va regression testlar yozilishi kerak.
