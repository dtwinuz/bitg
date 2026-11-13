// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Yangilanish tekshiruvi (`bitg.uz/update/stable.json`) qaysi manbadan yoqiladi yoki o'chiriladi. Platformaga
//! bog'liq emas — testlanadi; reestrni `system::update_policy` o'qiydi.
//!
//! Ustunlik tartibi:
//! 1. `HKLM` / `HKCU` `\SOFTWARE\Policies\bitg\UpdateCheck` (GPO, Intune) — majburiy, oynadagi tugma qulflanadi;
//! 2. foydalanuvchi tanlovi (`settings.ini` dagi `update_check`);
//! 3. o'rnatuvchi standarti `HKLM\SOFTWARE\bitg\UpdateCheck` (MSI `UPDATECHECK=0|1`, o'rnatishdagi belgi);
//! 4. yoqilgan.

use bitg_tsf::config::{self, Config};

/// Kompyuter darajasidagi qiymatlar (reestrdan).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UpdatePolicy {
    /// O'rnatuvchi yozgan standart: foydalanuvchi hali tanlamagan bo'lsa ishlatiladi.
    pub machine_default: Option<bool>,
    /// Administrator siyosati: bo'lsa, foydalanuvchi o'zgartira olmaydi.
    pub forced: Option<bool>,
}

impl UpdatePolicy {
    /// Yakuniy qiymat. `user` — `settings.ini` da aniq yozilgan tanlov.
    pub fn update_check(&self, user: Option<bool>) -> bool {
        self.forced
            .or(user)
            .or(self.machine_default)
            .unwrap_or(true)
    }

    /// Oynadagi tugma qulflanganmi (administrator siyosati).
    pub fn locked(&self) -> bool {
        self.forced.is_some()
    }

    /// Tekshiruv administrator tomonidan taqiqlanganmi (qo'lda «Tekshirish» ham ishlamaydi).
    pub fn blocked(&self) -> bool {
        self.forced == Some(false)
    }

    /// Faylga yoziladigan qiymat: siyosatsiz (siyosat olib tashlansa, foydalanuvchining o'z tanlovi qaytadi).
    /// Shu sababli yozishda standart ham aniq qiymatga aylanadi — o'rnatuvchi `0` desa, fayl `update_check=0` oladi.
    pub fn stored(&self, user: Option<bool>) -> bool {
        user.or(self.machine_default).unwrap_or(true)
    }

    /// `settings.ini` matnidan yakuniy qiymat.
    pub fn effective(&self, settings_ini: &str) -> bool {
        self.update_check(config::explicit_flag(settings_ini, "update_check"))
    }

    /// `settings.ini` matni bilan o'qilgan `Config` ga ko'rsatiladigan (yakuniy) qiymatni qo'yadi.
    pub fn apply(&self, c: &mut Config, settings_ini: &str) {
        c.update_check = self.effective(settings_ini);
    }

    /// Yozishdan oldin: fayldagi qiymatni siyosatsiz holatga qaytaradi (oyna yoki `parse` standarti emas).
    pub fn for_storage(&self, c: &mut Config, settings_ini: &str) {
        c.update_check = self.stored(config::explicit_flag(settings_ini, "update_check"));
    }
}

/// Reestr qiymati: MSI satr (`"0"`, `"1"`) yozadi, GPO/administrator DWORD qo'yadi. Boshqasi — yo'q hisoblanadi.
pub fn reg_flag(dword: Option<u32>, text: Option<&str>) -> Option<bool> {
    if let Some(d) = dword {
        return Some(d != 0);
    }
    match text.map(str::trim) {
        Some("1") => Some(true),
        Some("0") => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn p(machine_default: Option<bool>, forced: Option<bool>) -> UpdatePolicy {
        UpdatePolicy {
            machine_default,
            forced,
        }
    }

    #[test]
    fn nothing_set_means_on() {
        assert!(UpdatePolicy::default().update_check(None));
        assert!(!UpdatePolicy::default().locked());
    }

    #[test]
    fn installer_default_applies_until_user_chooses() {
        let off = p(Some(false), None);
        assert!(!off.update_check(None));
        assert!(
            off.update_check(Some(true)),
            "foydalanuvchi qayta yoqa oladi"
        );
        assert!(!off.locked() && !off.blocked());
        assert!(!p(Some(true), None).update_check(Some(false)));
    }

    #[test]
    fn policy_wins_and_locks() {
        let off = p(Some(true), Some(false));
        assert!(!off.update_check(Some(true)));
        assert!(off.locked() && off.blocked());
        let on = p(Some(false), Some(true));
        assert!(on.update_check(Some(false)));
        assert!(on.locked() && !on.blocked());
    }

    #[test]
    fn apply_reads_explicit_ini_value_only() {
        let off = p(Some(false), None);
        // Fayl yo'q (yangi o'rnatish) yoki kalit yozilmagan — o'rnatuvchi standarti.
        let mut c = config::parse("", "");
        off.apply(&mut c, "");
        assert!(!c.update_check);
        let ini = "enabled=1\n";
        let mut c = config::parse(ini, "");
        off.apply(&mut c, ini);
        assert!(!c.update_check);
        // Oynada yoqilgan — saqlangan tanlov ustun.
        let ini = config::to_ini(&Config::default());
        let mut c = config::parse(&ini, "");
        off.apply(&mut c, &ini);
        assert!(c.update_check);
    }

    #[test]
    fn storage_ignores_forced_value() {
        // Siyosat o'chirgan, foydalanuvchi avval yoqqan — faylda yoqilgan qoladi.
        let forced_off = p(None, Some(false));
        let mut c = Config {
            update_check: false,
            ..Config::default()
        };
        forced_off.for_storage(&mut c, "update_check=1\n");
        assert!(c.update_check);
        // Tanlov yo'q: o'rnatuvchi standarti aniq qiymat bo'lib yoziladi (`parse` dagi `true` emas).
        let installer_off = p(Some(false), None);
        let mut c = config::parse("", "");
        installer_off.for_storage(&mut c, "");
        assert!(!c.update_check);
        assert!(installer_off.effective(&config::to_ini(&c)) == installer_off.effective(""));
    }

    #[test]
    fn registry_values() {
        assert_eq!(reg_flag(Some(0), None), Some(false));
        assert_eq!(reg_flag(Some(1), Some("0")), Some(true));
        assert_eq!(reg_flag(None, Some("0")), Some(false));
        assert_eq!(reg_flag(None, Some(" 1 ")), Some(true));
        assert_eq!(reg_flag(None, Some("")), None);
        assert_eq!(reg_flag(None, Some("yes")), None);
        assert_eq!(reg_flag(None, None), None);
    }
}
