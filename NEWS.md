# bitg yangiliklari

Foydalanuvchi uchun qisqa: har versiyada nima o'zgardi. Texnik tafsilot — [CHANGELOG.md](CHANGELOG.md).
Bu fayldan ilovadagi «Yangiliklar» sahifasi va yangilanish oynasidagi qaydlar yig'iladi.

## 0.4.0 — 2026-11-13

### Yangi
- **Shaxsiy lug'at.** bitg bilmagan so'zni o'zingiz o'rgating: «shamollash» → «şamollaş». Qo'shimchali shakli ham shunday yoziladi (shamollashdan → şamollaşdan), katta-kichik harf yozganingizdek qoladi.
- **Bitta harfni saqlash.** So'zning har bir harfini bosib, qaysi biri o'girilmasin — tanlaysiz: bog'cha → bog'ça (g' qoladi, ch o'giriladi).
- **Ilova ichidan xabar berish.** Xato, taklif yoki noto'g'ri o'girilgan so'z haqida bir tugma bilan yozing; ekran rasmini ham qo'shish mumkin. Brauzer ochilmaydi.
- **Avtomatik yangilanish.** bitg yangi versiya chiqqanini o'zi aytadi va bir tugma bilan yangilanadi. Fayl GitHub'dan olinadi va SHA-256 bilan tekshiriladi.
- **Yangiliklar sahifasi.** Yangilangandan keyin nima o'zgargani bir marta ko'rsatiladi. Sozlamalarda o'chirsa bo'ladi.
- **Lug'atni ko'chirish.** «Nusxalash» bilan ro'yxatni oling, boshqa kompyuterda «Fayldan qo'shish» bilan yuklang.
- **Aqlli tuzatishlar uchun tayyor holat.** «Ehtiyotkor» — faqat yozganingiz o'giriladi, so'zlar tuzatilmaydi; «Tavsiya» — apostrofsiz so'zni ham o'zi tuzatadi.
- **Ubuntu: yuqori paneldagi bitg belgisi.** Ekranning yuqori panelida bitg ikonkasi va menyusi paydo bo'ladi: yoqish/o'chirish, avtomatik o'girish va kirillni o'girishni shu yerdan almashtirasiz, dasturlarni qayta ochish shart emas. bitg ilovalar menyusida ham bor va tizimga kirganda o'zi ishga tushadi.

### Yaxshilandi
- **Aqlli tuzatishni boshqarish.** «Tushib qolgan apostrofni tiklash» lug'at bo'yicha xato yozilgan so'zni ham tuzatadi (Qirgiziston → Qirğiziston). Endi tavsifda aniq yozilgan: yozganingiz aynan qolsin desangiz — o'chiring yoki so'zni lug'atga qo'shing.

### Tuzatildi
- **Ko'proq xorijiy ism.** 2 000 dan ortiq yangi xorijiy ism va familiya (Chesney kabi) yozilganidek qoladi, bitg ularni o'zbekcha deb o'girmaydi.
- **Brend va texnik atamalar.** fintech, edtech, Semrush, Genshin kabi 55 ta so'z endi buzilmaydi.
- **Kod yoki brend yonidagi so'z.** «fetch() chaqiruvi», «Shohjahon McDonald's» kabi qisqa gapda o'zbekcha so'z inglizcha deb qolib ketardi — endi o'giriladi.

## 0.3.1 — 2026-11-13

### Tuzatildi
- **Parol maydonlari.** WPF va boshqa dasturlardagi parol va maxfiy maydonlarda bitg o'zi o'chadi.
- **Ishonchli o'rnatuvchi.** Reliz fayllari imzolangan kelib chiqish ma'lumoti bilan chiqadi.

## 0.3.0 — 2026-11-12

### Yangi
- **macOS (beta).** Mac uchun klaviatura.
- **Ikki ma'noli so'zlar.** «ochmaydi», «otdim» kabi so'zlar taxmin bilan o'zgartirilmaydi, «tekshiring» deb belgilanadi.

### Tuzatildi
- **Xorijiy ismlar.** Michelle, Chicago kabi nomlar o'girilmaydi.

## 0.2.0 — 2026-11-12

### Yangi
- **Linux.** IBus va Fcitx5 uchun klaviatura (.deb).

## 0.1.0 — 2026-11-12

### Yangi
- **Birinchi reliz.** Windows klaviaturasi: probel bosilganda so'z yangi alifboga o'tadi, inglizchaga tegmaydi.
