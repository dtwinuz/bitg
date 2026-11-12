// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! .docx va .xlsx (Office Open XML) ichidagi matnni formatini buzmasdan o'girish.
//!
//! Faqat matn tugunlari (`w:t`, `t`) baytma-bayt almashtiriladi — qolgan XML, zip yozuvlarining
//! tartibi va siqilishi o'zgarmaydi. Abzats (`w:p`) yoki satr (`si`, `is`) bitta matn sifatida
//! kontekst bilan o'giriladi, natija esa asl run'larga belgi indekslari bo'yicha qaytariladi: o'zgargan
//! bo'lak u boshlangan run'ga yoziladi (bold "O" + "'zbek" → bold "Ö" + "zbek").

use std::io::{Cursor, Read, Write};

use quick_xml::Reader;
use quick_xml::escape::{partial_escape, unescape};
use quick_xml::events::{BytesStart, Event};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::align;

/// Hujjat turi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Word,
    Excel,
}

/// Qaysi elementlar birlik (abzats), qaysilari matn va qaysilari ajratgich.
pub struct Spec {
    unit: &'static str,
    text: &'static str,
    breaks: &'static [(&'static str, char)],
    skip: &'static [&'static str],
    /// Nomni prefikssiz solishtirish (xlsx'da `x:si` ham uchraydi).
    local: bool,
}

pub const WORD: Spec = Spec {
    unit: "w:p",
    text: "w:t",
    breaks: &[("w:tab", '\t'), ("w:br", '\n'), ("w:cr", '\n')],
    skip: &[],
    local: false,
};

/// xl/sharedStrings.xml: `<si>` — oddiy yoki rich text (`<r><t>`); `<rPh>` (fonetik) tegilmaydi.
pub const SHARED: Spec = Spec {
    unit: "si",
    text: "t",
    breaks: &[],
    skip: &["rPh"],
    local: true,
};

/// Varaqdagi inline satrlar (`<c t="inlineStr"><is>`); formulalar (`<f>`) va sonlar (`<v>`) tegilmaydi.
pub const SHEET: Spec = Spec {
    unit: "is",
    text: "t",
    breaks: &[],
    skip: &["rPh"],
    local: true,
};

/// Matn tuguni: ichki matn joyi (bayt; ochuvchi teg `content.0` da `>` bilan tugaydi).
struct Node {
    content: (usize, usize),
    text: String,
    preserve: bool,
}

enum Piece {
    Node(Node),
    /// Hech bir run'ga tegishli bo'lmagan ajratgich (tab, qator) yoki o'qib bo'lmagan tugun.
    Sep(char),
}

/// Bayt oralig'ini almashtirish.
type Edit = (usize, usize, String);

impl Spec {
    fn is(&self, e: &BytesStart, name: &str) -> bool {
        if self.local {
            e.local_name().as_ref() == name
        } else {
            e.name().as_ref() == name
        }
    }

    fn is_end(&self, e: &quick_xml::events::BytesEnd, name: &str) -> bool {
        if self.local {
            e.local_name().as_ref() == name
        } else {
            e.name().as_ref() == name
        }
    }
}

fn has_preserve(e: &BytesStart) -> bool {
    e.attributes()
        .flatten()
        .any(|a| a.key.as_ref() == "xml:space" && a.value == "preserve")
}

/// XML qismini o'giradi. O'zgarish bo'lmasa `None`; o'zgargan so'zlar `changes` ga qo'shiladi.
pub fn convert_xml(
    xml: &str,
    spec: &Spec,
    f: &dyn Fn(&str) -> String,
    changes: &mut Vec<(String, String)>,
) -> Result<Option<String>, String> {
    let mut reader = Reader::from_str(xml);
    let mut units: Vec<Vec<Piece>> = Vec::new();
    let mut open: Option<(usize, bool)> = None;
    let mut skip = 0usize;
    let mut edits: Vec<Edit> = Vec::new();
    loop {
        let before = reader.buffer_position() as usize;
        let ev = reader
            .read_event()
            .map_err(|e| format!("XML xatosi ({before}-bayt): {e}"))?;
        let after = reader.buffer_position() as usize;
        match ev {
            Event::Start(e) => {
                if spec.skip.iter().any(|s| spec.is(&e, s)) {
                    skip += 1;
                } else if skip == 0 {
                    if spec.is(&e, spec.unit) {
                        units.push(Vec::new());
                    } else if spec.is(&e, spec.text) && !units.is_empty() {
                        open = Some((after, has_preserve(&e)));
                    }
                }
            }
            Event::End(e) => {
                if spec.skip.iter().any(|s| spec.is_end(&e, s)) {
                    skip = skip.saturating_sub(1);
                } else if skip == 0 {
                    if spec.is_end(&e, spec.text) {
                        if let (Some((start, preserve)), Some(unit)) =
                            (open.take(), units.last_mut())
                        {
                            let raw = &xml[start..before];
                            unit.push(match unescape(raw) {
                                Ok(text) if !raw.contains('<') => Piece::Node(Node {
                                    content: (start, before),
                                    text: text.into_owned(),
                                    preserve,
                                }),
                                _ => Piece::Sep(' '),
                            });
                        }
                    } else if spec.is_end(&e, spec.unit)
                        && let Some(unit) = units.pop()
                    {
                        convert_unit(&unit, f, &mut edits, changes);
                    }
                }
            }
            Event::Empty(e) if skip == 0 && open.is_none() => {
                if let (Some(&(_, c)), Some(unit)) = (
                    spec.breaks.iter().find(|(n, _)| spec.is(&e, n)),
                    units.last_mut(),
                ) {
                    unit.push(Piece::Sep(c));
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if edits.is_empty() {
        return Ok(None);
    }
    edits.sort_by_key(|e| e.0);
    let mut out = String::with_capacity(xml.len() + 64);
    let mut last = 0;
    for (s, e, r) in edits {
        out.push_str(&xml[last..s]);
        out.push_str(&r);
        last = e;
    }
    out.push_str(&xml[last..]);
    Ok(Some(out))
}

/// Bitta abzatsni o'girib, natijani run'larga taqsimlaydi.
fn convert_unit(
    unit: &[Piece],
    f: &dyn Fn(&str) -> String,
    edits: &mut Vec<Edit>,
    changes: &mut Vec<(String, String)>,
) {
    let mut text = String::new();
    let mut owner: Vec<Option<usize>> = Vec::new();
    for (k, p) in unit.iter().enumerate() {
        match p {
            Piece::Node(n) => {
                text.push_str(&n.text);
                owner.extend(std::iter::repeat_n(Some(k), n.text.chars().count()));
            }
            Piece::Sep(c) => {
                text.push(*c);
                owner.push(None);
            }
        }
    }
    let new = f(&text);
    if new == text {
        return;
    }
    let chars: Vec<char> = text.chars().collect();
    // Bo'lak egasi: shu belgining run'i, bo'lmasa keyingi, bo'lmasa oldingi.
    let owner_at = |i: usize| {
        owner[i.min(owner.len())..]
            .iter()
            .flatten()
            .next()
            .or_else(|| owner[..i.min(owner.len())].iter().rev().flatten().next())
            .copied()
    };
    let mut out: Vec<String> = vec![String::new(); unit.len()];
    let mut hs = align::hunks(&text, &new).into_iter().peekable();
    let mut i = 0;
    while i <= chars.len() {
        if let Some(h) = hs.next_if(|h| h.start == i) {
            if let Some(k) = owner_at(i) {
                out[k].push_str(&h.insert);
            }
            i = i.max(h.end);
            continue;
        }
        if i == chars.len() {
            break;
        }
        if let Some(k) = owner[i] {
            out[k].push(chars[i]);
        }
        i += 1;
    }
    for (p, new_text) in unit.iter().zip(out) {
        let Piece::Node(n) = p else { continue };
        if new_text == n.text {
            continue;
        }
        let gained_edge = |s: &str, o: &str, start: bool| {
            let ws = |t: &str| {
                if start {
                    t.starts_with(char::is_whitespace)
                } else {
                    t.ends_with(char::is_whitespace)
                }
            };
            ws(s) && !ws(o)
        };
        if !n.preserve
            && (gained_edge(&new_text, &n.text, true) || gained_edge(&new_text, &n.text, false))
        {
            let at = n.content.0 - 1;
            edits.push((at, at, " xml:space=\"preserve\"".to_string()));
        }
        edits.push((
            n.content.0,
            n.content.1,
            partial_escape(new_text.as_str()).into_owned(),
        ));
    }
    changes.extend(align::changed_words(&text, &new));
}

/// Paket ichidagi qaysi qism qanday o'giriladi.
pub fn spec_for(kind: Kind, name: &str) -> Option<&'static Spec> {
    match kind {
        Kind::Word => {
            let file = name.strip_prefix("word/")?.strip_suffix(".xml")?;
            let known = [
                "document",
                "header",
                "footer",
                "footnotes",
                "endnotes",
                "comments",
            ];
            (!file.contains('/') && known.iter().any(|k| file.starts_with(k))).then_some(&WORD)
        }
        Kind::Excel => {
            if name == "xl/sharedStrings.xml" {
                Some(&SHARED)
            } else {
                let file = name.strip_prefix("xl/worksheets/")?.strip_suffix(".xml")?;
                (!file.contains('/')).then_some(&SHEET)
            }
        }
    }
}

/// O'zgargan so'z va u topilgan qism (masalan `word/document.xml`).
pub struct Located {
    pub part: String,
    pub old: String,
    pub new: String,
}

/// O'giriladigan bitta XML qismning ochilgandagi eng katta hajmi (zip-bombadan himoya). Haqiqiy hujjatlarda
/// `document.xml` bir necha MiB bo'ladi. Qolgan yozuvlar ochilmaydi, siqilgan holida ko'chiriladi.
#[cfg(not(test))]
const ENTRY_LIMIT: u64 = 256 << 20;
#[cfg(test)]
const ENTRY_LIMIT: u64 = 1 << 20;

/// Butun paketni o'giradi. Natija — yangi zip baytlari; tegilmagan yozuvlar siqilgan holida ko'chiriladi.
pub fn convert_package(
    input: &[u8],
    kind: Kind,
    f: &dyn Fn(&str) -> String,
    changes: &mut Vec<Located>,
) -> Result<Vec<u8>, String> {
    let zerr = |e: zip::result::ZipError| format!("zip: {e}");
    let mut archive = ZipArchive::new(Cursor::new(input)).map_err(zerr)?;
    let mut writer = ZipWriter::new(Cursor::new(Vec::with_capacity(input.len())));
    for i in 0..archive.len() {
        let name = archive
            .by_index_raw(i)
            .map_err(zerr)?
            .name()
            .map_err(zerr)?
            .into_owned();
        let converted = match spec_for(kind, &name) {
            Some(spec) => {
                let mut xml = String::new();
                archive
                    .by_index(i)
                    .map_err(zerr)?
                    .take(ENTRY_LIMIT + 1)
                    .read_to_string(&mut xml)
                    .map_err(|e| format!("{name}: {e}"))?;
                if xml.len() as u64 > ENTRY_LIMIT {
                    return Err(format!(
                        "{name}: ochilgandagi hajmi {} MiB dan katta — fayl buzilgan yoki zip-bomba",
                        ENTRY_LIMIT >> 20
                    ));
                }
                let mut words = Vec::new();
                let out =
                    convert_xml(&xml, spec, f, &mut words).map_err(|e| format!("{name}: {e}"))?;
                changes.extend(words.into_iter().map(|(old, new)| Located {
                    part: name.clone(),
                    old,
                    new,
                }));
                out
            }
            None => None,
        };
        let raw = archive.by_index_raw(i).map_err(zerr)?;
        match converted {
            Some(xml) => {
                let method = match raw.compression() {
                    CompressionMethod::Stored => CompressionMethod::Stored,
                    _ => CompressionMethod::Deflated,
                };
                let mut opts = SimpleFileOptions::default().compression_method(method);
                if let Some(t) = raw.last_modified() {
                    opts = opts.last_modified_time(t);
                }
                drop(raw);
                writer.start_file(name.as_str(), opts).map_err(zerr)?;
                writer
                    .write_all(xml.as_bytes())
                    .map_err(|e| format!("{name}: {e}"))?;
            }
            None => writer.raw_copy_file(raw).map_err(zerr)?,
        }
    }
    let comment = archive.comment().to_vec();
    if !comment.is_empty() {
        writer
            .set_raw_comment(comment.into_boxed_slice())
            .map_err(zerr)?;
    }
    Ok(writer.finish().map_err(zerr)?.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitg_core::Options;

    fn conv(s: &str) -> String {
        bitg_core::convert(s, &Options::default())
    }

    fn run(xml: &str, spec: &Spec) -> (String, Vec<(String, String)>) {
        let mut ch = Vec::new();
        let out = convert_xml(xml, spec, &conv, &mut ch)
            .unwrap()
            .unwrap_or_else(|| xml.to_string());
        (out, ch)
    }

    #[test]
    fn split_runs_keep_formatting() {
        let xml = r#"<w:document><w:body><w:p><w:r><w:rPr><w:b/></w:rPr><w:t>O</w:t></w:r><w:r><w:t>'zbekiston sha</w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:t>harlari</w:t></w:r></w:p></w:body></w:document>"#;
        let (out, ch) = run(xml, &WORD);
        assert_eq!(
            out,
            r#"<w:document><w:body><w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Ö</w:t></w:r><w:r><w:t>zbekiston şa</w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:t>harlari</w:t></w:r></w:p></w:body></w:document>"#
        );
        assert_eq!(ch.len(), 2);
    }

    #[test]
    fn whitespace_gets_preserve() {
        let xml = r#"<w:p><w:r><w:t>bog</w:t></w:r><w:r><w:t>' bor</w:t></w:r></w:p>"#;
        let (out, _) = run(xml, &WORD);
        assert_eq!(
            out,
            r#"<w:p><w:r><w:t>boğ</w:t></w:r><w:r><w:t xml:space="preserve"> bor</w:t></w:r></w:p>"#
        );
    }

    #[test]
    fn existing_preserve_and_entities() {
        let xml = r#"<w:p><w:r><w:t xml:space="preserve">choy &amp; non </w:t></w:r><w:r><w:t>o&apos;g&apos;il</w:t></w:r></w:p>"#;
        let (out, _) = run(xml, &WORD);
        assert_eq!(
            out,
            r#"<w:p><w:r><w:t xml:space="preserve">çoy &amp; non </w:t></w:r><w:r><w:t>öğil</w:t></w:r></w:p>"#
        );
    }

    #[test]
    fn tab_separates_words_and_other_xml_untouched() {
        let xml = r#"<w:p w14:paraId="1"><w:r><w:t>o'z</w:t><w:tab/><w:t>shahar</w:t></w:r><w:r><w:instrText>HYPERLINK "o'zbek"</w:instrText></w:r><w:r><w:delText>o'chdi</w:delText></w:r></w:p>"#;
        let (out, _) = run(xml, &WORD);
        assert_eq!(
            out,
            r#"<w:p w14:paraId="1"><w:r><w:t>öz</w:t><w:tab/><w:t>şahar</w:t></w:r><w:r><w:instrText>HYPERLINK "o'zbek"</w:instrText></w:r><w:r><w:delText>o'chdi</w:delText></w:r></w:p>"#
        );
    }

    #[test]
    fn shared_strings_rich_text_and_phonetic() {
        let xml = r#"<sst count="2"><si><t>Toshkent</t></si><si><r><rPr><b/></rPr><t>G</t></r><r><t>'alaba</t></r><rPh><t>sh</t></rPh></si></sst>"#;
        let (out, ch) = run(xml, &SHARED);
        assert_eq!(
            out,
            r#"<sst count="2"><si><t>Toşkent</t></si><si><r><rPr><b/></rPr><t>Ğ</t></r><r><t>alaba</t></r><rPh><t>sh</t></rPh></si></sst>"#
        );
        assert_eq!(ch[1], ("G'alaba".to_string(), "Ğalaba".to_string()));
    }

    #[test]
    fn sheet_formulas_untouched() {
        let xml = r#"<worksheet><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>choy</t></is></c><c r="B1" t="str"><f>CONCAT("cho","y")</f><v>choy</v></c><c r="C1"><v>42</v></c></row></sheetData></worksheet>"#;
        let (out, _) = run(xml, &SHEET);
        assert_eq!(out, xml.replace("<is><t>choy", "<is><t>çoy"));
    }

    #[test]
    fn nothing_to_change() {
        let mut ch = Vec::new();
        let xml = "<w:p><w:r><w:t>salom dunyo</w:t></w:r></w:p>";
        assert!(convert_xml(xml, &WORD, &conv, &mut ch).unwrap().is_none());
    }

    #[test]
    fn other_attributes_and_nested_markup() {
        let xml = r#"<w:p><w:r><w:t w:rsid="1">choy</w:t></w:r></w:p>"#;
        assert_eq!(
            run(xml, &WORD).0,
            r#"<w:p><w:r><w:t w:rsid="1">çoy</w:t></w:r></w:p>"#
        );
        let xml = r#"<w:p><w:r><w:t>o'<w:br/>z</w:t></w:r><w:r><w:t>choy</w:t></w:r></w:p>"#;
        let (out, ch) = run(xml, &WORD);
        assert_eq!(out, xml.replace("<w:t>choy", "<w:t>çoy"));
        assert_eq!(ch, [("choy".to_string(), "çoy".to_string())]);
    }

    #[test]
    fn empty_element_inside_phonetic_is_skipped() {
        let xml = r#"<si><t>choy</t><rPh><t>sh</t><x/></rPh></si>"#;
        assert_eq!(
            run(xml, &SHARED).0,
            r#"<si><t>çoy</t><rPh><t>sh</t><x/></rPh></si>"#
        );
    }

    #[test]
    fn change_in_separator_only_has_no_run() {
        let mut ch = Vec::new();
        let xml = "<w:p><w:r><w:tab/></w:r></w:p>";
        let out = convert_xml(xml, &WORD, &|_| "x".to_string(), &mut ch).unwrap();
        assert_eq!(out, None);
        assert_eq!(ch, [("\t".to_string(), "x".to_string())]);
    }

    #[test]
    fn trailing_whitespace_gets_preserve() {
        let mut ch = Vec::new();
        let xml = "<w:p><w:r><w:t>a b</w:t></w:r></w:p>";
        let out = convert_xml(xml, &WORD, &|s| s.replace('b', ""), &mut ch).unwrap();
        assert_eq!(
            out.as_deref(),
            Some(r#"<w:p><w:r><w:t xml:space="preserve">a </w:t></w:r></w:p>"#)
        );
    }

    #[test]
    fn parts() {
        assert!(spec_for(Kind::Word, "word/document.xml").is_some());
        assert!(spec_for(Kind::Word, "word/header2.xml").is_some());
        assert!(spec_for(Kind::Word, "word/styles.xml").is_none());
        assert!(spec_for(Kind::Word, "word/glossary/document.xml").is_none());
        assert!(spec_for(Kind::Excel, "xl/worksheets/sheet1.xml").is_some());
        assert!(spec_for(Kind::Excel, "xl/worksheets/_rels/sheet1.xml.rels").is_none());
        assert!(spec_for(Kind::Excel, "xl/styles.xml").is_none());
    }

    pub(crate) fn make_zip(entries: &[(&str, &str, CompressionMethod)]) -> Vec<u8> {
        let mut w = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, body, method) in entries {
            w.start_file(
                *name,
                SimpleFileOptions::default().compression_method(*method),
            )
            .unwrap();
            w.write_all(body.as_bytes()).unwrap();
        }
        w.finish().unwrap().into_inner()
    }

    fn read_zip(bytes: &[u8]) -> Vec<(String, String, CompressionMethod)> {
        let mut a = ZipArchive::new(Cursor::new(bytes)).unwrap();
        (0..a.len())
            .map(|i| {
                let mut f = a.by_index(i).unwrap();
                let mut s = String::new();
                f.read_to_string(&mut s).unwrap();
                (f.name().unwrap().into_owned(), s, f.compression())
            })
            .collect()
    }

    #[test]
    fn docx_round_trip_keeps_order_and_compression() {
        let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="x"><w:body><w:p><w:r><w:t>O'zbekiston</w:t></w:r></w:p></w:body></w:document>"#;
        let header = r#"<w:hdr xmlns:w="x"><w:p><w:r><w:t>Toshkent</w:t></w:r></w:p></w:hdr>"#;
        let styles = r#"<w:styles xmlns:w="x"><w:t>o'zgarmasin</w:t></w:styles>"#;
        let input = make_zip(&[
            (
                "[Content_Types].xml",
                "<Types/>",
                CompressionMethod::Deflated,
            ),
            ("word/styles.xml", styles, CompressionMethod::Stored),
            ("word/document.xml", doc, CompressionMethod::Deflated),
            ("word/header1.xml", header, CompressionMethod::Stored),
        ]);
        let mut ch = Vec::new();
        let out = convert_package(&input, Kind::Word, &conv, &mut ch).unwrap();
        let entries = read_zip(&out);
        let names: Vec<_> = entries.iter().map(|e| e.0.as_str()).collect();
        assert_eq!(
            names,
            [
                "[Content_Types].xml",
                "word/styles.xml",
                "word/document.xml",
                "word/header1.xml"
            ]
        );
        assert_eq!(entries[1].1, styles);
        assert_eq!(entries[1].2, CompressionMethod::Stored);
        assert!(entries[2].1.contains("<w:t>Özbekiston</w:t>"));
        assert!(entries[2].1.starts_with(r#"<?xml version="1.0""#));
        assert_eq!(entries[2].2, CompressionMethod::Deflated);
        assert!(entries[3].1.contains("Toşkent"));
        assert_eq!(entries[3].2, CompressionMethod::Stored);
        assert_eq!(ch.len(), 2);
        assert_eq!(ch[0].part, "word/document.xml");
    }

    #[test]
    fn xlsx_round_trip() {
        let sst = r#"<sst xmlns="x"><si><t>Choy</t></si></sst>"#;
        let sheet = r#"<worksheet xmlns="x"><sheetData><row><c t="s"><v>0</v></c><c><f>SUM(A1:A2)</f><v>3</v></c></row></sheetData></worksheet>"#;
        let input = make_zip(&[
            ("xl/sharedStrings.xml", sst, CompressionMethod::Deflated),
            (
                "xl/worksheets/sheet1.xml",
                sheet,
                CompressionMethod::Deflated,
            ),
        ]);
        let mut ch = Vec::new();
        let out = convert_package(&input, Kind::Excel, &conv, &mut ch).unwrap();
        let entries = read_zip(&out);
        assert_eq!(entries[0].1, sst.replace("Choy", "Çoy"));
        assert_eq!(entries[1].1, sheet);
    }

    #[test]
    fn zip_bomb_entry_is_rejected() {
        // 2 MiB bo'sh joy deflate bilan bir necha KiB ga siqiladi; sinovda chegara 1 MiB
        let body = format!(
            "<w:document><w:t>{}</w:t></w:document>",
            " ".repeat(2 << 20)
        );
        let input = make_zip(&[("word/document.xml", &body, CompressionMethod::Deflated)]);
        assert!(input.len() < 64 << 10, "{}", input.len());
        let err = convert_package(&input, Kind::Word, &conv, &mut Vec::new()).unwrap_err();
        assert!(err.contains("zip-bomba"), "{err}");

        // chegara faqat o'giriladigan qismlarga: katta rasm yoki styles.xml ochilmaydi, o'zgarmay ko'chadi
        let ok = make_zip(&[
            ("word/styles.xml", &body, CompressionMethod::Deflated),
            (
                "word/document.xml",
                "<w:t>choy</w:t>",
                CompressionMethod::Deflated,
            ),
        ]);
        let out = convert_package(&ok, Kind::Word, &conv, &mut Vec::new()).unwrap();
        assert_eq!(read_zip(&out)[0].1.len(), body.len());
    }

    #[test]
    fn archive_comment_is_kept() {
        let mut w = ZipWriter::new(Cursor::new(Vec::new()));
        w.set_comment("izoh").unwrap();
        w.start_file("word/document.xml", SimpleFileOptions::default())
            .unwrap();
        w.write_all(b"<w:p><w:r><w:t>choy</w:t></w:r></w:p>")
            .unwrap();
        let input = w.finish().unwrap().into_inner();
        let out = convert_package(&input, Kind::Word, &conv, &mut Vec::new()).unwrap();
        let a = ZipArchive::new(Cursor::new(&out)).unwrap();
        assert_eq!(a.comment(), b"izoh");
        assert_eq!(read_zip(&out)[0].1, "<w:p><w:r><w:t>çoy</w:t></w:r></w:p>");
    }

    #[test]
    fn invalid_entry_date_is_not_copied() {
        let mut input = make_zip(&[(
            "word/document.xml",
            "<w:p><w:r><w:t>choy</w:t></w:r></w:p>",
            CompressionMethod::Stored,
        )]);
        // MS-DOS sanasi 0 (oy 0) — yaroqsiz
        for (sig, at) in [(b"PK\x03\x04", 12), (b"PK\x01\x02", 14)] {
            let p = input.windows(4).position(|w| w == sig).unwrap();
            input[p + at..p + at + 2].copy_from_slice(&[0, 0]);
        }
        let date = |bytes: &[u8]| {
            ZipArchive::new(Cursor::new(bytes))
                .unwrap()
                .by_index_raw(0)
                .unwrap()
                .last_modified()
        };
        assert_eq!(date(&input), None);
        let out = convert_package(&input, Kind::Word, &conv, &mut Vec::new()).unwrap();
        assert_eq!(date(&out), Some(zip::DateTime::default()));
        assert_eq!(read_zip(&out)[0].1, "<w:p><w:r><w:t>çoy</w:t></w:r></w:p>");
    }

    #[test]
    fn reverse_direction() {
        let old = |s: &str| bitg_core::to_old(s, bitg_core::OldStyle::Official);
        let mut ch = Vec::new();
        let xml = "<w:p><w:r><w:t>Ö</w:t></w:r><w:r><w:t>zbek</w:t></w:r></w:p>";
        let out = convert_xml(xml, &WORD, &old, &mut ch).unwrap().unwrap();
        assert!(out.starts_with("<w:p><w:r><w:t>O"), "{out}");
        assert!(out.ends_with("<w:t>zbek</w:t></w:r></w:p>"), "{out}");
    }
}
