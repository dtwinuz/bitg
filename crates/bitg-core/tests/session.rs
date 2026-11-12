// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Yozish paytidagi (IME) o'girish: so'zlar birma-bir, kontekst faqat oldingi so'zlar.

use bitg_core::{Options, Session};

fn typed(words: &[&str]) -> Vec<String> {
    let opts = Options::default();
    let mut s = Session::new();
    words
        .iter()
        .map(|w| {
            s.complete_word(w, &opts)
                .map(|e| e.insert)
                .unwrap_or_else(|| w.to_string())
        })
        .collect()
}

#[test]
fn missing_apostrophe_is_restored_while_typing() {
    assert_eq!(typed(&["qoshimcha"]), ["qöşimça"]);
    assert_eq!(typed(&["shunday", "qoshimcha"]), ["şunday", "qöşimça"]);
    assert_eq!(typed(&["şunday", "qoshimcha"]), ["şunday", "qöşimça"]);
    assert_eq!(typed(&["ozbekcha", "togri"]), ["özbekça", "töğri"]);
    assert_eq!(
        typed(&["goshtni", "songra", "qoshib"]),
        ["göştni", "söngra", "qöşib"]
    );
    assert_eq!(typed(&["Bu", "ish", "yaxshi", "boladi"])[3], "böladi");
    assert_eq!(
        typed(&["qollab-quvvatlash", "togridan-togri"]),
        ["qöllab-quvvatlaş", "töğridan-töğri"]
    );
}

#[test]
fn writer_without_apostrophes_gets_ambiguous_words_restored() {
    // oldingi so'z tiklandi — muallif apostrof qo'ymayapti: oz → o'z, yoq → yo'q
    assert_eq!(
        typed(&["ozbek", "tilida", "oz", "fikrimiz"]),
        ["özbek", "tilida", "öz", "fikrimiz"]
    );
    assert_eq!(typed(&["Togri,", "hech", "narsa", "yoq."])[3], "yöq.");
    // birinchi so'z bo'lsa yoki muallif o'/g' yozsa — taxmin qilinmaydi
    assert_eq!(typed(&["oz", "qoldi"]), ["oz", "qoldi"]);
    assert_eq!(typed(&["o'zbek", "oz", "qoldi"]), ["özbek", "oz", "qoldi"]);
}

#[test]
fn uzbek_word_after_english_word_still_converts_while_typing() {
    assert_eq!(typed(&["check", "shapaloq"]), ["check", "şapaloq"]);
    assert_eq!(typed(&["ok", "choynak"]), ["ok", "çoynak"]);
    // Inglizcha gap davom etsa — o'zgarmaydi
    assert_eq!(typed(&["check", "the", "shop"])[2], "shop");
}

#[test]
fn english_words_and_brands_stay_while_typing() {
    for w in [
        "chrome",
        "Chrome",
        "shell",
        "Photoshop",
        "Photoshopda",
        "Chevrolet",
        "Chevroletni",
        "Toshiba",
    ] {
        assert_eq!(typed(&[w]), [w], "{w} alone");
        assert_eq!(typed(&["men", w, "ochdim"])[1], w, "{w} in a sentence");
    }
}

#[test]
fn foreign_names_while_typing() {
    assert_eq!(typed(&["Michelle", "keldi"]), ["Michelle", "keldi"]);
    // chap qo'shni xorijiy: Chaplin ham
    assert_eq!(
        typed(&["Charlie", "Chaplin", "va", "Chicago"]),
        ["Charlie", "Chaplin", "va", "Chicago"]
    );
    assert_eq!(
        typed(&["Michelle", "va", "Richard"]),
        ["Michelle", "va", "Richard"]
    );
    assert_eq!(typed(&["Richard", "Gir"]), ["Riçard", "Gir"]);
    assert_eq!(typed(&["eshikni", "ochmaydi"]), ["eşikni", "oçmaydi"]);
}
