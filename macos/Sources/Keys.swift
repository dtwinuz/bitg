// bitg macOS: NSEvent tugmasini dvigatel tugmasiga (BITG_KEY_*) aylantirish. Sof funksiya — macos/test sinaydi.
import AppKit

enum Keys {
    /// macOS virtual key code'lari (Carbon kVK_*).
    enum Code {
        static let space: UInt16 = 49, ret: UInt16 = 36, keypadEnter: UInt16 = 76, tab: UInt16 = 48
        static let delete: UInt16 = 51
        /// Esc, Home, PgUp, Fwd Delete, End, PgDn, ←, →, ↓, ↑
        static let navigate: Set<UInt16> = [53, 115, 116, 117, 119, 121, 123, 124, 125, 126]
    }

    static func map(_ e: NSEvent) -> (BitgKey, UInt32) {
        map(keyCode: e.keyCode, flags: e.modifierFlags, characters: e.characters)
    }

    /// `characters` — tartib qo'llangan belgi(lar); o'lik tugma (Option+U) uchun "".
    static func map(keyCode: UInt16, flags: NSEvent.ModifierFlags, characters: String?) -> (BitgKey, UInt32) {
        let m = flags.intersection(.deviceIndependentFlagsMask)
        if m.contains(.command) || m.contains(.control) {
            return (.other, 0)
        }
        if Code.navigate.contains(keyCode) {
            return (.navigate, 0)
        }
        switch keyCode {
        case Code.space: // Option+Space — NBSP
            return (.space, characters?.unicodeScalars.first?.value == 0xA0 ? 0xA0 : 0x20)
        case Code.ret, Code.keypadEnter:
            return (.enter, 0)
        case Code.tab:
            return (.tab, 0)
        case Code.delete:
            // Option+Delete so'zni butunlay o'chiradi: bu "qaytarish" emas, kursor joyi endi noma'lum
            return m.contains(.option) ? (.other, 0) : (.backspace, 0)
        default:
            break
        }
        guard let s = characters?.unicodeScalars, s.count == 1, let u = s.first else {
            return (.other, 0)
        }
        let v = u.value
        if v < 0x20 || v == 0x7F || (0xF700...0xF8FF).contains(v) { // boshqaruv va funksional tugmalar (F1…)
            return (.other, 0)
        }
        return (.char, v)
    }
}
