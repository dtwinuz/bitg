// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Zaxira yo'l uchun: dastur matnni TSF'ga bermasa (Telegram Desktop), bosilgan harflardan joriy so'zni yig'amiz.
//! Faqat so'z chegarasidan (probel, Enter, fokus) keyin ketma-ket yozilgan harflar ishonchli — navigatsiya,
//! Ctrl+..., noma'lum belgi bo'lsa so'z "noaniq" bo'ladi va o'girilmaydi (begona matnni o'chirib yubormaslik uchun).

/// Bitta so'z uchun eng ko'p belgi: bundan uzun — noaniq.
const MAX: usize = 40;

#[derive(Debug, Default)]
pub struct Typed {
    word: Vec<char>,
    /// So'z boshi ko'rilgan (chegaradan keyin boshlangan) va oraliqda shubhali hodisa bo'lmagan.
    trusted: bool,
    /// Ishonch nega yo'qoldi (log uchun, matnsiz).
    why: &'static str,
}

impl Typed {
    /// Fokus o'zgardi yoki so'z tugadi: keyingi harf — yangi ishonchli so'z boshi.
    pub fn boundary(&mut self) {
        self.word.clear();
        self.trusted = true;
        self.why = "";
    }

    /// Navigatsiya, sichqoncha, Ctrl+V, noma'lum belgi: kursor qayerdaligini endi bilmaymiz.
    pub fn lost(&mut self, why: &'static str) {
        self.word.clear();
        self.trusted = false;
        self.why = why;
    }

    pub fn push(&mut self, c: char) {
        if self.word.len() >= MAX {
            self.trusted = false;
            self.why = "word too long";
        }
        self.word.push(c);
    }

    pub fn backspace(&mut self) {
        if self.word.pop().is_none() {
            // so'zdan oldingi matn o'chirilmoqda — keyingi so'z boshi noma'lum
            self.trusted = false;
            self.why = "backspace past word start";
        }
    }

    /// Oxirgi `finish` so'zni nega bermadi (bo'sh bo'lsa — sabab yo'q).
    pub fn why(&self) -> &'static str {
        if self.why.is_empty() {
            "no word start seen"
        } else {
            self.why
        }
    }

    pub fn len(&self) -> usize {
        self.word.len()
    }

    pub fn is_empty(&self) -> bool {
        self.word.is_empty()
    }

    /// Kursor oldidagi matn hozir yozilayotgan so'z bilan tugaydimi (kamida 2 harf): kontekst matnni
    /// haqiqatan beradi. Bir marta ko'rilgan eski matn (masalan qoralama) bunga mos kelmaydi.
    pub fn mirrored_in(&self, before: &[u16]) -> bool {
        if self.word.len() < 2 {
            return false;
        }
        let w: Vec<u16> = self
            .word
            .iter()
            .collect::<String>()
            .encode_utf16()
            .collect();
        before.ends_with(&w)
    }

    /// Probel bosildi: ishonchli so'z bo'lsa — uni qaytaradi. Har holda chegara qo'yiladi.
    pub fn finish(&mut self) -> Option<String> {
        let w = (self.trusted && !self.word.is_empty() && self.word.len() <= MAX)
            .then(|| self.word.iter().collect());
        if w.is_some() {
            self.boundary();
        } else {
            // sabab log uchun saqlanadi; keyingi so'z yana ishonchli
            self.word.clear();
            self.trusted = true;
        }
        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(s: &str) -> Typed {
        let mut t = Typed::default();
        t.boundary();
        s.chars().for_each(|c| t.push(c));
        t
    }

    #[test]
    fn collects_word_after_boundary() {
        let mut t = typed("choy");
        assert_eq!(t.finish().as_deref(), Some("choy"));
        "shahar".chars().for_each(|c| t.push(c));
        assert_eq!(t.finish().as_deref(), Some("shahar"));
    }

    #[test]
    fn backspace_edits_word() {
        let mut t = typed("chox");
        t.backspace();
        t.push('y');
        assert_eq!(t.finish().as_deref(), Some("choy"));
    }

    #[test]
    fn untrusted_after_navigation_or_deleting_past_start() {
        let mut t = typed("ch");
        t.lost("test");
        "oy".chars().for_each(|c| t.push(c));
        assert_eq!(t.finish(), None);

        let mut t = typed("a");
        t.backspace();
        t.backspace(); // oldingi so'zga kirdi
        "choy".chars().for_each(|c| t.push(c));
        assert_eq!(t.finish(), None);
        // keyingi so'z yana ishonchli
        "choy".chars().for_each(|c| t.push(c));
        assert_eq!(t.finish().as_deref(), Some("choy"));
    }

    #[test]
    fn mirrored_only_when_context_ends_with_typed_word() {
        let u = |s: &str| s.encode_utf16().collect::<Vec<u16>>();
        let t = typed("cho");
        assert!(t.mirrored_in(&u("Salom, cho")));
        // fokusda bir marta berilgan eski matn
        assert!(!t.mirrored_in(&u("Kecha yozgan xabarim")));
        // bitta harf tasodifan mos kelishi mumkin — yetarli emas
        assert!(!typed("m").mirrored_in(&u("xabarim")));
    }

    #[test]
    fn nothing_before_first_boundary() {
        let mut t = Typed::default();
        "choy".chars().for_each(|c| t.push(c));
        assert_eq!(t.finish(), None);
        assert_eq!(t.why(), "no word start seen");
    }

    #[test]
    fn too_long_or_empty_word_is_not_returned() {
        let mut t = typed(&"a".repeat(MAX));
        assert_eq!(t.len(), MAX);
        t.push('b');
        assert_eq!(t.len(), MAX + 1);
        assert_eq!(t.finish(), None);
        assert_eq!(t.why(), "word too long");
        assert!(t.is_empty());

        let mut t = typed("");
        assert_eq!(t.finish(), None);
        t.lost("mouse");
        assert_eq!(t.why(), "mouse");
        t.boundary();
        t.backspace();
        assert_eq!(t.why(), "backspace past word start");
    }
}
