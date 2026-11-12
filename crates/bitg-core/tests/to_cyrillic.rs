// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use bitg_core::{cyrillic::to_new_latin, to_cyrillic};

const T: char = '\u{02BC}';

#[test]
fn old_latin_words() {
    for (lat, cyr) in [
        ("O'zbekiston", "Ўзбекистон"),
        ("shahar", "шаҳар"),
        ("choy", "чой"),
        ("tog'", "тоғ"),
        ("g'alaba", "ғалаба"),
        ("qishloq", "қишлоқ"),
        ("xalq", "халқ"),
        ("hayot", "ҳаёт"),
        ("yangi", "янги"),
        ("ming", "минг"),
        ("ma'no", "маъно"),
        ("ta'lim", "таълим"),
        ("san'at", "санъат"),
        ("she'r", "шеър"),
        ("mo'jiza", "мўжиза"),
        ("Is'hoq", "Исҳоқ"),
        ("mus'haf", "мусҳаф"),
        ("mashhur", "машҳур"),
        ("ishchi", "ишчи"),
        ("yo'l", "йўл"),
        ("yomg'ir", "ёмғир"),
        ("yulduz", "юлдуз"),
        ("dunyo", "дунё"),
        ("siyosat", "сиёсат"),
        ("qayerda", "қаерда"),
        ("suyuq", "суюқ"),
        ("o'yin", "ўйин"),
        ("yer", "ер"),
        ("poyezd", "поезд"),
        ("ertaga", "эртага"),
        ("aeroport", "аэропорт"),
        ("kecha", "кеча"),
        ("obyekt", "объект"),
        ("politsiya", "полиция"),
        ("litsey", "лицей"),
        ("aytsa", "айтса"),
        ("ketsin", "кетсин"),
        ("jamiyat", "жамият"),
    ] {
        assert_eq!(to_cyrillic(lat), cyr, "{lat}");
    }
}

#[test]
fn new_latin_words() {
    for (lat, cyr) in [
        ("Özbekiston", "Ўзбекистон"),
        ("şahar", "шаҳар"),
        ("çoy", "чой"),
        ("toğ", "тоғ"),
        ("maʼno", "маъно"),
        ("yöl", "йўл"),
        ("işçi", "ишчи"),
    ] {
        assert_eq!(to_cyrillic(lat), cyr, "{lat}");
    }
}

#[test]
fn apostrophe_variants() {
    for o in ['\'', '`', '\u{02BB}', '\u{02BC}', '\u{2019}', '\u{2018}'] {
        assert_eq!(to_cyrillic(&format!("O{o}zbek")), "Ўзбек", "{o:?}");
        assert_eq!(to_cyrillic(&format!("ma{o}no")), "маъно", "{o:?}");
    }
}

#[test]
fn case() {
    assert_eq!(to_cyrillic("YANGI HAYOT"), "ЯНГИ ҲАЁТ");
    assert_eq!(to_cyrillic("Yangi Yil"), "Янги Йил");
    assert_eq!(to_cyrillic("SHAHAR"), "ШАҲАР");
    assert_eq!(to_cyrillic("G'ALABA"), "ҒАЛАБА");
    assert_eq!(to_cyrillic("MA'NO"), "МАЪНО");
    assert_eq!(to_cyrillic("Ya'ni"), "Яъни");
    assert_eq!(to_cyrillic("Shahnoza"), "Шаҳноза");
    assert_eq!(to_cyrillic("Ertaga"), "Эртага");
    assert_eq!(to_cyrillic("AQSH va BMT"), "АҚШ ва БМТ");
}

#[test]
fn sentences_and_punctuation() {
    assert_eq!(
        to_cyrillic("O'zbekiston Respublikasi poytaxti Toshkent shahri."),
        "Ўзбекистон Республикаси пойтахти Тошкент шаҳри."
    );
    assert_eq!(
        to_cyrillic("Men 5-sinfda o'qiyman, 2026-yil (rost!)"),
        "Мен 5-синфда ўқийман, 2026-йил (рост!)"
    );
    assert_eq!(
        to_cyrillic("Bugun choy ichdik,\nkeyin bog'ga chiqdik."),
        "Бугун чой ичдик,\nкейин боғга чиқдик."
    );
    assert_eq!(to_cyrillic("Sh.Sharipov"), "Ш.Шарипов");
    assert_eq!(to_cyrillic("«Toshkent»"), "«Тошкент»");
}

#[test]
fn english_links_and_code_stay_latin() {
    assert_eq!(
        to_cyrillic("Please check https://kun.uz/choy"),
        "Please check https://kun.uz/choy"
    );
    assert_eq!(
        to_cyrillic("Hello world, how are you?"),
        "Hello world, how are you?"
    );
    assert_eq!(to_cyrillic("user@mail.uz"), "user@mail.uz");
    assert_eq!(to_cyrillic("my_shop"), "my_shop");
    assert_eq!(
        to_cyrillic("Photoshop va Windows dasturlari"),
        "Photoshop ва Windows дастурлари"
    );
}

#[test]
fn numbers_with_suffixes_and_acronyms() {
    assert_eq!(to_cyrillic("20ga yaqin"), "20га яқин");
    assert_eq!(to_cyrillic("soat 23:30da"), "соат 23:30да");
    assert_eq!(to_cyrillic("1-2-dekabr kunlari"), "1-2-декабр кунлари");
    assert_eq!(to_cyrillic("43chi o'rin"), "43чи ўрин");
    assert_eq!(to_cyrillic("ÖzA va ÖXDP"), "ЎзА ва ЎХДП");
    assert_eq!(to_cyrillic("17 dekabr\u{02BC} kuni"), "17 декабръ куни");
    assert_eq!(to_cyrillic("shahar123"), "shahar123");
}

#[test]
fn cyrillic_input_is_kept() {
    assert_eq!(to_cyrillic("Салом, dunyo"), "Салом, дунё");
}

/// Kirill → lotin → kirill: noaniq harflarsiz (ц щ ь ы, сҳ) gaplar o'zgarmay qaytishi kerak.
#[test]
fn roundtrip_sample() {
    for s in [
        "Ўзбекистон Республикаси пойтахти Тошкент шаҳри.",
        "Барча одамлар эркин, қадр-қиммат ва ҳуқуқларда тенг бўлиб туғилади.",
        "Ашула айтиш яхши кўрасизми?",
        "Сиз хотинни қаерда кўрдингиз?",
        "Қор ёғишга бошлади.",
        "Менинг дўстларим ёш эмас.",
        "Қийинчиликдаги дўст ҳақиқий дўстдир.",
        "Ҳеч гап йўқ.",
        "Соат нечада ўрнингиздан турибсиз?",
        "Ер юзида поезд, аэропорт ва йўллар кўп.",
        "Маъно, таълим, санъат ва шеър ҳақида гаплашдик.",
        "ЯНГИ ЙИЛ БИЛАН!",
        "Эртага биз қишлоққа борамиз, у ерда ҳаво тоза.",
        "Ёмғир ёғди, юлдузлар кўринмади.",
        "Бу объект полиция назоратида.",
    ] {
        let lat = to_new_latin(s, T);
        assert_eq!(to_cyrillic(&lat), s, "{lat}");
    }
}

/// Katta korpusda aylanish barqarorligi: `BITG_CYR_CORPUS=fayl cargo test -- --ignored roundtrip_corpus`.
/// Faylning har qatori (yoki TSV oxirgi ustuni) — kirill gap.
#[test]
#[ignore]
fn roundtrip_corpus() {
    let Ok(path) = std::env::var("BITG_CYR_CORPUS") else {
        return;
    };
    let data = std::fs::read_to_string(&path).expect("corpus");
    let (mut total, mut ok) = (0usize, 0usize);
    let mut fails = Vec::new();
    for line in data.lines() {
        let s = line.rsplit('\t').next().unwrap_or(line).trim();
        if !s
            .chars()
            .any(|c| matches!(c, 'ў' | 'қ' | 'ғ' | 'ҳ' | 'Ў' | 'Қ' | 'Ғ' | 'Ҳ'))
        {
            continue; // o'zbekcha emas (ruscha) bo'lishi mumkin
        }
        if s.chars().any(|c| c.is_ascii_alphabetic())
            || s.to_lowercase().contains(['ц', 'щ', 'ь', 'ы'])
            || s.to_lowercase().contains("сҳ")
        {
            continue; // noaniq
        }
        total += 1;
        let back = to_cyrillic(&to_new_latin(s, T));
        if back == s {
            ok += 1;
        } else if fails.len() < 40 {
            fails.push(format!("{s}\n  -> {back}"));
        }
    }
    for f in &fails {
        println!("{f}");
    }
    let pct = 100.0 * ok as f64 / total.max(1) as f64;
    println!("{ok}/{total} {pct:.2}%");
    assert!(pct >= 95.0, "{pct:.2}%");
}
