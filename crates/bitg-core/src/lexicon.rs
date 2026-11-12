// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Qurish vaqtida tayyorlangan lug'at jadvali (`build.rs`): so'z → og'irlik. Ishga tushganda tahlil va
//! xotira ajratish yo'q; qidiruv — FNV-1a hash + chiziqli probing, bitta so'z solishtiriladi.

pub(crate) struct Lexicon {
    keys: &'static [u8],
    offs: &'static [u8],
    weights: &'static [u8],
    slots: &'static [u8],
}

/// `build.rs` dagi bilan bir xil bo'lishi shart.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([b[i * 4], b[i * 4 + 1], b[i * 4 + 2], b[i * 4 + 3]])
}

impl Lexicon {
    pub(crate) fn len(&self) -> usize {
        self.weights.len() / 4
    }

    pub(crate) fn get(&self, key: &str) -> Option<f32> {
        let size = self.slots.len() / 4;
        let k = key.as_bytes();
        let mut s = fnv1a(k) as usize & (size - 1);
        loop {
            let slot = u32_at(self.slots, s) as usize;
            if slot == 0 {
                return None;
            }
            let i = slot - 1;
            let (a, b) = (
                u32_at(self.offs, i) as usize,
                u32_at(self.offs, i + 1) as usize,
            );
            if &self.keys[a..b] == k {
                return Some(f32::from_bits(u32_at(self.weights, i)));
            }
            s = (s + 1) & (size - 1);
        }
    }

    pub(crate) fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

macro_rules! lexicon {
    ($name:literal) => {
        Lexicon {
            keys: include_bytes!(concat!(env!("OUT_DIR"), "/", $name, ".keys")),
            offs: include_bytes!(concat!(env!("OUT_DIR"), "/", $name, ".off")),
            weights: include_bytes!(concat!(env!("OUT_DIR"), "/", $name, ".w")),
            slots: include_bytes!(concat!(env!("OUT_DIR"), "/", $name, ".tab")),
        }
    };
}

pub(crate) static UZ: Lexicon = lexicon!("uz");
pub(crate) static EN: Lexicon = lexicon!("en");
pub(crate) static SPLIT: Lexicon = lexicon!("sh_split");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_row_is_found_with_its_weight() {
        for (lex, src) in [
            (&UZ, include_str!("data/uz.tsv")),
            (&EN, include_str!("data/en.tsv")),
        ] {
            let mut last = std::collections::HashMap::new();
            for line in src.lines().filter(|l| !l.starts_with('#')) {
                if let Some((w, s)) = line.split_once('\t')
                    && let Ok(v) = s.trim().parse::<f32>()
                {
                    last.insert(w, v);
                }
            }
            assert_eq!(lex.len(), last.len());
            for (w, v) in last {
                assert_eq!(lex.get(w), Some(v), "{w}");
            }
        }
        assert_eq!(UZ.get("bunday-soz-yoq-xyz"), None);
    }
}
