// bitg macOS: dvigatel (crates/bitg-ime, C ABI) va matn maydoni orasidagi bog'lovchi.
// IMK'ga bog'liq emas — test (macos/test) soxta maydon bilan shu kodni sinaydi.
import Foundation

/// Matn maydoni: IMK mijozi (Controller.swift) yoki test maydoni. Barcha diapazonlar UTF-16 birligida.
protocol TextClient: AnyObject {
    /// Tanlov; dastur bermasa `location == NSNotFound`.
    func selection() -> NSRange
    /// Diapazondagi matn yoki nil (dastur matnni bermaydi).
    func text(in range: NSRange) -> String?
    /// `range.location == NSNotFound` — kursor (yoki preedit) o'rniga yoziladi.
    func insert(_ text: String, replacing range: NSRange)
    /// Preedit (tagi chizilgan so'z); "" — yashiriladi.
    func setMarked(_ text: String)
    var bundleID: String? { get }
}

/// `BITG_KEY_*` (linux/include/bitg_ime.h).
enum BitgKey: UInt32 {
    case char = 0, space, enter, tab, backspace, navigate, other
}

final class BitgSession {
    /// Kursor oldidan so'raladigan matn (UTF-16). So'z ko'pi bilan 40 belgi — 128 yetadi.
    static let window = 128
    private static let caret = NSRange(location: NSNotFound, length: 0)

    private let ime: OpaquePointer?

    init() {
        ime = bitg_ime_new()
    }

    deinit {
        bitg_ime_free(ime)
    }

    var ready: Bool { ime != nil }

    /// Yangi maydon yoki dastur: so'z boshi ishonchli; o'chirilgan dasturda (standart: terminallar) bitg tegmaydi.
    func activate(_ c: TextClient) {
        guard let ime else { return }
        bitg_ime_focus_in(ime)
        updateSuspended(c)
    }

    /// Sozlamalardan (`Prefs`) shu dastur uchun o'chiq/yoqiqni qo'llaydi. Menyudan o'zgartirilganda darhol chaqiriladi —
    /// dvigatelning o'zi `settings.ini` ni ko'pi bilan sekundiga bir marta o'qiydi.
    func updateSuspended(_ c: TextClient, _ prefs: Prefs = .load()) {
        guard let ime else { return }
        if let o = bitg_ime_set_suspended(ime, prefs.suspended(bundle: c.bundleID) ? 1 : 0) {
            apply(o.pointee, c, nil)
        }
    }

    /// Fokus ketdi, sichqoncha bosildi: preedit'dagi so'z o'girilmay yoziladi (matn yo'qolmaydi).
    /// IMK buni bir hodisada bir necha marta chaqirishi mumkin — ikkinchisida yoziladigan narsa yo'q.
    func commitComposition(_ c: TextClient) {
        guard let ime, let p = bitg_ime_reset(ime) else { return }
        let pending = String(cString: p)
        if !pending.isEmpty {
            c.insert(pending, replacing: Self.caret)
        }
    }

    /// true — tugma iste'mol qilindi, dasturga yuborilmasin.
    func key(_ kind: BitgKey, _ cp: UInt32, _ c: TextClient) -> Bool {
        guard let ime else { return false }
        // kursor oldidagi matn — dasturga sinxron IPC (Chromium'da sekin): faqat dvigatel uni ishlatadigan tugmada
        // (so'z boshi, so'z oxiri, qaytarish). Natija baribir bir xil — crates/bitg-ime tests::wants_before_is_exact.
        let ctx = bitg_ime_wants_before(ime, kind.rawValue, cp) != 0 ? before(c) : nil
        let out: UnsafePointer<BitgImeOut>?
        if let ctx {
            out = ctx.text.withCString { bitg_ime_key(ime, kind.rawValue, cp, $0) }
        } else {
            out = bitg_ime_key(ime, kind.rawValue, cp, nil)
        }
        guard let o = out?.pointee else { return false }
        apply(o, c, ctx)
        return o.consumed != 0
    }

    /// Kursor oldidagi matn va kursor joyi; dastur bermasa yoki tanlov bo'lsa nil (dvigatel preedit'ga o'tadi).
    private func before(_ c: TextClient) -> (text: String, cursor: Int)? {
        let sel = c.selection()
        guard sel.location != NSNotFound, sel.length == 0 else { return nil }
        let start = max(0, sel.location - Self.window)
        let want = sel.location - start
        guard let t = c.text(in: NSRange(location: start, length: want)) else { return nil }
        // so'ralganidan boshqa uzunlik: kursor hisobi ishonchsiz — "bo'sh matn", dvigatel shu maydonda preedit'ga o'tadi
        guard t.utf16.count == want else { return ("", sel.location) }
        return (t, sel.location)
    }

    /// Tartib: delete_before + commit (bitta almashtirish) -> preedit. Ko'rsatkichlar shu yerning o'zida o'qiladi.
    private func apply(_ o: BitgImeOut, _ c: TextClient, _ ctx: (text: String, cursor: Int)?) {
        let commit = o.commit.map { String(cString: $0) }
        if let commit {
            if o.delete_before > 0, let ctx {
                let tail = String(String.UnicodeScalarView(ctx.text.unicodeScalars.suffix(Int(o.delete_before))))
                let len = tail.utf16.count
                c.insert(commit, replacing: NSRange(location: ctx.cursor - len, length: len))
            } else if o.delete_before == 0 {
                c.insert(commit, replacing: Self.caret)
            }
            // delete_before > 0, lekin matn yo'q — bo'lmaydi (dvigatel faqat matn bilan o'chiradi); takror yozmaymiz
        }
        guard o.preedit_changed != 0 else { return }
        let preedit = o.preedit.map { String(cString: $0) } ?? ""
        // insertText kompozitsiyani o'zi yopadi: undan keyin setMarkedText("") Chromium/Word'da bo'sh marked
        // diapazon qoldiradi va keyingi almashtirish noto'g'ri joyga tushadi
        if preedit.isEmpty, commit != nil {
            return
        }
        c.setMarked(preedit)
    }
}
