// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use bitg_core::{OldStyle, Options, Session, convert, to_old};

fn c(s: &str) -> String {
    convert(s, &Options::default())
}

#[test]
fn uzbek_sentences() {
    assert_eq!(
        c("O'zbekiston shaharlari: Toshkent"),
        "Özbekiston şaharlari: Toşkent"
    );
    assert_eq!(
        c("Bugun choy ichdik, keyin bog'ga chiqdik."),
        "Bugun çoy içdik, keyin boğga çiqdik."
    );
    assert_eq!(
        c("Yaxshi kishi hech qachon shoshilmaydi"),
        "Yaxşi kişi heç qaçon şoşilmaydi"
    );
    assert_eq!(c("ma'no va san'at"), "ma\u{02BC}no va san\u{02BC}at");
    assert_eq!(c("tog' va bog'"), "toğ va boğ");
}

#[test]
fn missing_apostrophes_are_restored() {
    assert_eq!(
        c("Men ozbek tilida togri yozaman, ogil bola kop kitob"),
        "Men özbek tilida töğri yozaman, öğil bola köp kitob"
    );
    assert_eq!(c("Ozbekiston, Ozim bilaman"), "Özbekiston, Özim bilaman");
    // apostrofsiz shakl faqat o'zak orqali "ma'lum" (organ|ib, soz|lar) yoki korpusda xato yozuv (ogzaki)
    assert_eq!(
        c("organib qolibman ogzaki sozlar"),
        "örganib qolibman oğzaki sözlar"
    );
    // apostrofsiz shakl korpusda kirill xato yozuvi sifatida uchraydi (кора, оша, огир, гарб)
    assert_eq!(
        c("Shu kitobga kora yozdim, osha kuni ogir boldi. Garb mamlakatlari"),
        "Şu kitobga köra yozdim, öşa kuni oğir böldi. Ğarb mamlakatlari"
    );
    // inglizcha lug'atdagi so'z: o'zbekcha gapda tiklanadi, inglizcha gapda — yo'q
    assert_eq!(
        c("Bundan song men uyga keldim"),
        "Bundan söng men uyga keldim"
    );
    assert_eq!(c("I love this song"), "I love this song");
    // ikkala shakl ham haqiqiy: toy/to'y, top/to'p
    assert_eq!(c("toy kuni top oynadik"), "toy kuni top oynadik");
    // haqiqiy so'z: oshganlar (oshmoq), ortayotgani (ortmoq) — tegilmaydi
    assert_eq!(
        c("narxlar oshganlar, ortayotgani"),
        "narxlar oşganlar, ortayotgani"
    );
    // ikkala shakl ham haqiqiy so'z — taxmin qilinmaydi
    assert_eq!(
        c("Uyda qol, oz qoldi, ozod bor"),
        "Uyda qol, oz qoldi, ozod bor"
    );
    // inglizcha so'z va gap
    assert_eq!(c("good song, go home"), "good song, go home");
    // o'chirib qo'yish
    let off = Options {
        restore_apostrophes: false,
        ..Options::default()
    };
    assert_eq!(convert("ozbek tili", &off), "ozbek tili");
}

#[test]
fn common_english_word_inside_uzbek_sentence_is_kept() {
    assert_eq!(
        c("salom choy o'zbek check, keyin shahar"),
        "salom çoy özbek check, keyin şahar"
    );
}

#[test]
fn initials_and_abbreviations_convert() {
    assert_eq!(c("A.Lukashenko va Sh.Sharipov"), "A.Lukaşenko va Ş.Şaripov");
    assert_eq!(c("M.Jo'rayev, O'zA, O'zR"), "M.Jörayev, ÖzA, ÖzR");
    // kod va domenlar avvalgidek tegilmaydi
    assert_eq!(
        c("obj.show kun.uz myShop I'm"),
        "obj.show kun.uz myShop I'm"
    );
}

#[test]
fn whitespace_and_lines_are_preserved() {
    assert_eq!(c("  choy\n\tshahar  "), "  çoy\n\tşahar  ");
}

#[test]
fn english_words_and_sentences_are_kept() {
    for s in [
        "check",
        "How are you shah",
        "Please check the schedule",
        "I wish you much success",
        "The shop is closed",
        "Chat with the team",
        "Don't touch the cash",
        "Children should watch",
        "Rock'n'roll at nine o'clock",
    ] {
        assert_eq!(c(s), s, "{s}");
    }
}

#[test]
fn mixed_language_text() {
    assert_eq!(c("Thanks! Bugun choy ichdik"), "Thanks! Bugun çoy içdik");
    assert_eq!(c("please qo'shiq ayt"), "please qöşiq ayt");
    // chap: yolg'iz — noaniq (inglizcha "chap" ham bor), o'zbekcha kontekstda o'giriladi
    assert_eq!(c("Menga chap tomonda"), "Menga çap tomonda");
    // chat: o'zbek korpusida deyarli yo'q, inglizchada keng tarqalgan — tegilmaydi
    assert_eq!(
        c("Menga chat yozing, shahar markazida"),
        "Menga chat yozing, şahar markazida"
    );
}

#[test]
fn protected_tokens() {
    assert_eq!(c("kun.uz/choy va shahar.uz"), "kun.uz/choy va shahar.uz");
    assert_eq!(
        c("sherzod@gmail.com ga yozing"),
        "sherzod@gmail.com ga yozing"
    );
    assert_eq!(c("@shahzod #shahar"), "@shahzod #shahar");
    assert_eq!(c("my_shop.check()"), "my_shop.check()");
    assert_eq!(c("parol: shahar123"), "parol: shahar123");
    assert_eq!(c("O'Brien choy ichdi"), "O'Brien çoy içdi");
}

#[test]
fn lone_letters() {
    assert_eq!(
        c("G' YOKO O' HARFLAR, o' va g' harfi"),
        "Ğ YOKO Ö HARFLAR, ö va ğ harfi"
    );
    assert_eq!(c("O'"), "Ö");
    // inglizcha va tirnoq ichida — tegilmaydi
    assert_eq!(
        c("Rock 'n' roll at nine o' the clock"),
        "Rock 'n' roll at nine o' the clock"
    );
    assert_eq!(c("'o' dedi"), "'o' dedi");
}

#[test]
fn quotes() {
    assert_eq!(c("'Yaxshi tong' dedi"), "'Yaxşi tong' dedi");
    assert_eq!(c("«Choyxona»"), "«Çoyxona»");
}

#[test]
fn case() {
    assert_eq!(c("SHAHAR CHOY O'ZBEK"), "ŞAHAR ÇOY ÖZBEK");
    assert_eq!(c("Shahnoza va Chorsu"), "Şahnoza va Çorsu");
}

#[test]
fn cyrillic() {
    assert_eq!(c("Ўзбекистон шаҳарлари"), "Özbekiston şaharlari");
    assert_eq!(c("Ер юзи, поезд, лицей"), "Yer yuzi, poyezd, litsey");
    let o = Options {
        cyrillic: false,
        ..Options::default()
    };
    assert_eq!(convert("Салом choy", &o), "Салом çoy");
}

#[test]
fn acronyms_without_vowels_are_kept() {
    assert_eq!(c("TSH va PTCH bo'yicha"), "TSH va PTCH böyiça");
    assert_eq!(c("CH-III bosqich"), "CH-III bosqiç");
    // O'zbekcha qisqartma — o'giriladi.
    assert_eq!(c("AQSH bilan"), "AQŞ bilan");
    assert_eq!(c("MCHJ va KXSHT"), "MÇJ va KXŞT");
}

#[test]
fn hyphen_compounds_are_split() {
    assert_eq!(c("C-uchli va MR-shartli"), "C-uçli va MR-şartli");
    assert_eq!(c("shahar-qishloq"), "şahar-qişloq");
    assert_eq!(c("o'n-o'n besh"), "ön-ön beş");
    assert_eq!(c("2-chi sinf"), "2-çi sinf");
}

#[test]
fn foreign_letters_and_german_sch_are_kept() {
    assert_eq!(c("Pöschl va Tröltsch"), "Pöschl va Tröltsch");
    assert_eq!(c("Müller shahri"), "Müller şahri");
    assert_eq!(c("Fischer va Schmidt"), "Fischer va Schmidt");
    // O'zbekcha s+ch: o'giriladi.
    assert_eq!(c("asoschi bokschi"), "asosçi boksçi");
    // ö tugmadan yozilgan bo'lsa ham qolgani o'giriladi.
    assert_eq!(c("köcha"), "köça");
}

#[test]
fn dis_h_is_split() {
    assert_eq!(c("dishormonogenetik holat"), "dishormonogenetik holat");
}

#[test]
fn english_name_with_uzbek_suffix_is_kept() {
    assert_eq!(c("Chromeda ochildi"), "Chromeda oçildi");
    assert_eq!(c("Shellga yozdim"), "Shellga yozdim");
    assert_eq!(c("Toshkentda boshda"), "Toşkentda boşda");
}

#[test]
fn already_converted_text_is_stable() {
    let once = c("O'zbekiston shaharlari, choy va ma'no");
    assert_eq!(c(&once), once);
}

#[test]
fn exceptions() {
    let o = Options {
        exceptions: vec!["shohjahon".into()],
        ..Options::default()
    };
    assert_eq!(convert("Shohjahon bilan choy", &o), "Shohjahon bilan çoy");
}

#[test]
fn guard_off_converts_everything() {
    let o = Options {
        english_guard: false,
        ..Options::default()
    };
    assert_eq!(convert("check", &o), "çeck");
}

#[test]
fn reverse() {
    assert_eq!(
        to_old("Özbekiston şaharlari", OldStyle::Ascii),
        "O'zbekiston shaharlari"
    );
}

#[test]
fn session_typing() {
    let o = Options::default();
    let mut s = Session::new();
    assert_eq!(
        s.complete_word("choy", &o).map(|e| e.insert),
        Some("çoy".into())
    );
    let mut en = Session::new();
    assert_eq!(en.complete_word("How", &o), None);
    assert_eq!(en.complete_word("are", &o), None);
    assert_eq!(en.complete_word("you", &o), None);
    assert_eq!(en.complete_word("shah", &o), None);
    let mut uz = Session::new();
    uz.complete_word("Menga", &o);
    let e = uz
        .complete_word("chap", &o)
        .expect("o'zbekcha kontekstda chap o'giriladi");
    assert_eq!((e.delete, e.insert.as_str()), (4, "çap"));
}

#[test]
fn cheap_fuzz_never_panics_and_is_idempotent_on_output() {
    // Oddiy LCG — tashqi bog'liqliksiz tasodifiy matn.
    let alphabet: Vec<char> = "aoughscqx'`ʻʼ’ .,!?\n@#/_-1Ш ҳўғ\"«»()OUSHCH"
        .chars()
        .collect();
    let mut seed: u64 = 0x2026_1021;
    for _ in 0..3000 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let len = (seed >> 58) as usize;
        let mut s = String::new();
        for _ in 0..len {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            s.push(alphabet[(seed >> 33) as usize % alphabet.len()]);
        }
        let once = c(&s);
        let _ = to_old(&once, OldStyle::Ascii);
    }
}

#[test]
fn text_written_without_any_apostrophes() {
    // Muallif o'/g' apostrofini umuman qo'ymaydi (boshqa so'z ham tiklandi): ikki ma'noli so'zlar ham tiklanadi.
    assert_eq!(
        c("Biz ozbek tilida oz fikrimizni aytamiz"),
        "Biz özbek tilida öz fikrimizni aytamiz"
    );
    assert_eq!(
        c("Ozbekistonda ish yoq, otgan yili kop odam ketdi"),
        "Özbekistonda iş yöq, ötgan yili köp odam ketdi"
    );
    assert_eq!(c("Togri, hech narsa yoq."), "Töğri, heç narsa yöq.");
    // apostrofsiz shakl kirill korpusda uchramaydi (boladi), o'zak orqali "bola|di" deb hisoblanmaydi
    assert_eq!(c("Bu ish yaxshi boladi"), "Bu iş yaxşi böladi");
    // korpusda o'zi uchragan nomzod o'zak orqali taxmindan ustun: o'g'irlab, og'ir|lab emas
    assert_eq!(c("pulni ogirlab ketishdi"), "pulni öğirlab ketişdi");
    // boshqa so'z tiklanmasa — ikki ma'noli so'zga tegilmaydi
    assert_eq!(
        c("Uyda qol, oz qoldi, ozod bor"),
        "Uyda qol, oz qoldi, ozod bor"
    );
    // matnda o'/g' yozilgan bo'lsa — muallif apostrof qo'yadi, oz "kam" ma'nosida qoladi
    assert_eq!(c("O'zbekistonda oz qoldi"), "Özbekistonda oz qoldi");
}

#[test]
fn hyphenated_words_restore_part_by_part() {
    assert_eq!(
        c("Davlat qollab-quvvatlash dasturi togridan-togri ishlaydi"),
        "Davlat qöllab-quvvatlaş dasturi töğridan-töğri işlaydi"
    );
    // bir bo'lagida o'/g' yozilgan: qolganiga tegilmaydi (ozmi-ko'pmi — "kam-ko'p")
    assert_eq!(c("Ozmi-ko'pmi pul bor"), "Ozmi-köpmi pul bor");
}

#[test]
fn cedilla_used_as_comma() {
    assert_eq!(
        c("Narx qimmat bolsa¸ olmaymiz"),
        "Narx qimmat bölsa¸ olmaymiz"
    );
}

#[test]
fn sh_split_with_suffix() {
    // kirillda s|h alohida (нусҳа, Мусҳаф, Исҳоқ): qo'shimchali shakl ham
    assert_eq!(c("nushalari Ishoqqa Mushaf"), "nushalari Ishoqqa Mushaf");
    assert_eq!(c("Is'hoq va mas'ul shaxs"), "Ishoq va masʼul şaxs");
}

// #6: xorijiy imlodagi nom (o'zbek korpusida yo'q, inglizcha imlo belgisi bor) o'girilmaydi; yonidagi ikki xil
// o'qiladigan nom (Richard, Chaplin) ham shu nomning bir qismi.
#[test]
fn foreign_spelled_names_are_kept() {
    assert_eq!(
        c("Richard va Michelle Washington'ga ketishdi."),
        "Richard va Michelle Washington'ga ketişdi."
    );
    assert_eq!(
        c("Charlie Chaplin va Chicago haqida."),
        "Charlie Chaplin va Chicago haqida."
    );
    // nom gap tilini aytmaydi: qolgan so'zlar o'zbekcha
    assert_eq!(c("Chicagoda yashaydi"), "Chicagoda yaşaydi");
    assert_eq!(c("Michelleni kutdik"), "Michelleni kutdik");
    // #50: boshqa xorijiy imlo belgilari (ff, tte, ey, sch)
    assert_eq!(
        c("Sheffield va Charlotte shaharlari"),
        "Sheffield va Charlotte şaharlari"
    );
    assert_eq!(
        c("Ashley Schumacher bilan gaplashdi"),
        "Ashley Schumacher bilan gaplaşdi"
    );
    assert_eq!(c("Deutsche Bank hisoboti"), "Deutsche Bank hisoboti");
}

#[test]
fn uzbek_spelled_names_are_still_converted() {
    // o'zbek imlosida ham shunday yoziladi (Ричард, Пушкин) — yonida xorijiy nom bo'lmasa o'giriladi
    assert_eq!(c("Richard Gir filmi"), "Riçard Gir filmi");
    assert_eq!(c("Pushkin va Shakira"), "Puşkin va Şakira");
    // o'zbekcha nom xorijiy zanjirga qo'shilmaydi
    assert_eq!(c("Shohruh va Michelle keldi"), "Şohruh va Michelle keldi");
    // vergul zanjirni uzadi
    assert_eq!(c("Michelle, Richard keldi"), "Michelle, Riçard keldi");
    // apostrof o'zbekcha tutuq (Mash'al), inglizcha qo'shimcha emas
    assert_eq!(c("Mash'al yondi"), "Maşʼal yondi");
}

// #5: ikki haqiqiy o'zakdan yasalgan so'z (och / o'ch, ot / o't), apostrofsiz shakli korpusda uchramaydi va
// nomzod siyrak — chastota hech narsa demaydi, muallif yozganicha qoladi.
#[test]
fn ambiguous_words_are_not_restored_silently() {
    assert_eq!(
        c("boshliqdan sora, eshikni ochmaydi"),
        "boşliqdan söra, eşikni oçmaydi"
    );
    assert_eq!(
        c("telefonimni otdim, ot minib kelgan"),
        "telefonimni otdim, ot minib kelgan"
    );
    // nomzod keng tarqalgan (o'qish, bo'yicha) — apostrofsiz shaklning yo'qligi o'zi dalil
    assert_eq!(c("kitob oqish boyicha"), "kitob öqiş böyiça");
    // o't|kazdi: "kazdi" qo'shimcha emas — ikki ma'noli emas
    assert_eq!(c("majlis otkazdi"), "majlis ötkazdi");
    // #50: och / o'ch, ot / o't, olim / o'lim, sol / so'l — muallif yozganicha
    assert_eq!(c("eshikni ochdi va chiqdi"), "eşikni oçdi va çiqdi");
    assert_eq!(c("qushni otib, uyga qaytdi"), "quşni otib, uyga qaytdi");
    assert_eq!(c("olim kitob yozdi"), "olim kitob yozdi");
    assert_eq!(c("choyga shakar sol"), "çoyga şakar sol");
}

#[test]
fn ambiguous_words_carry_an_alternative() {
    let (out, words) =
        bitg_core::convert_words("boshliqdan sora, eshikni ochmaydi", &Options::default());
    assert_eq!(out, "boşliqdan söra, eşikni oçmaydi");
    let alt = |t: &str| {
        words
            .iter()
            .find(|w| w.token == t)
            .and_then(|w| w.alt.clone())
    };
    assert_eq!(alt("ochmaydi").as_deref(), Some("öçmaydi"));
    assert_eq!(alt("sora,"), None);
    // tiklangan ikki ma'noli so'z (loose): muqobili — apostrofsiz o'qilishi
    let (out, words) = bitg_core::convert_words(
        "Ozbekistonda ish yoq, otgan yili kop odam ketdi",
        &Options::default(),
    );
    assert_eq!(out, "Özbekistonda iş yöq, ötgan yili köp odam ketdi");
    let alt = words
        .iter()
        .find(|w| w.token == "otgan")
        .and_then(|w| w.alt.clone());
    assert_eq!(alt.as_deref(), Some("otgan"));
    // muallif apostrof qo'yadi — muqobil ko'rsatilmaydi
    let (_, words) = bitg_core::convert_words("O'zbekistonda oz qoldi", &Options::default());
    assert!(words.iter().all(|w| w.alt.is_none()));
}
