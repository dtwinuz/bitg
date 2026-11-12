// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// bitg macOS sinovi: Session.swift + Keys.swift + dvigatel, soxta matn maydoni bilan (IMK'siz).
// Holatlar Linux'dagi linux/test/ibus_e2e.py bilan bir xil, ustiga UTF-16 (emoji), terminal, sichqoncha va tugmalar.
import AppKit

/// Matn maydoni: kursor doim oxirda. Diapazonlar UTF-16 (NSString), xuddi IMK mijozidagidek.
final class FakeField: TextClient {
    let text = NSMutableString()
    var marked = ""
    let surrounding: Bool
    let bundleID: String?

    init(surrounding: Bool = true, bundle: String? = "com.apple.TextEdit") {
        self.surrounding = surrounding
        self.bundleID = bundle
    }

    /// Dasturga so'rovlar soni (haqiqiy IMK'da har biri sinxron IPC).
    var queries = 0

    func selection() -> NSRange {
        queries += 1
        return surrounding ? NSRange(location: text.length, length: 0) : NSRange(location: NSNotFound, length: 0)
    }

    func text(in range: NSRange) -> String? {
        surrounding ? text.substring(with: range) : nil
    }

    /// insertText'dan keyin darhol setMarkedText("") — haqiqiy mijozlarda bo'sh marked diapazon qoldiradi.
    var clearsAfterInsert = 0
    private var justInserted = false

    func insert(_ s: String, replacing range: NSRange) {
        justInserted = true
        if range.location == NSNotFound {
            text.append(s)
        } else {
            precondition(NSMaxRange(range) == text.length, "almashtirish kursor oldida emas: \(range)")
            text.replaceCharacters(in: range, with: s)
        }
        marked = ""
    }

    func setMarked(_ s: String) {
        if s.isEmpty, justInserted {
            clearsAfterInsert += 1
        }
        justInserted = false
        marked = s
    }

    var value: String { text as String }
}

struct Case {
    let name: String
    let run: () throws -> Void
}

struct Fail: Error, CustomStringConvertible {
    let description: String
}

func check<T: Equatable>(_ what: String, _ got: T, _ want: T) throws {
    if got != want {
        throw Fail(description: "\(what): \(got) != \(want)")
    }
}

/// Har holat o'z HOME'ida: o'rganilgan so'zlar (exceptions.txt) holatlar orasida o'tmaydi.
/// `ini` — dvigatel ishga tushishidan oldin yoziladigan settings.ini.
func session(_ f: FakeField, ini: String? = nil) -> BitgSession {
    let dir = FileManager.default.temporaryDirectory.appendingPathComponent("bitg-test-\(UUID().uuidString)")
    try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    setenv("HOME", dir.path, 1)
    if let ini {
        try! Prefs(text: ini).save()
    }
    let s = BitgSession()
    precondition(s.ready, "bitg_ime_new NULL qaytardi")
    s.activate(f)
    return s
}

/// Tugmalar: belgi, " ", "\n", "\u{8}" (Backspace). Iste'mol qilinmagan tugmani "dastur" o'zi bajaradi.
func type(_ s: String, _ f: FakeField, _ sess: BitgSession) {
    for u in s.unicodeScalars {
        let kind: BitgKey
        switch u {
        case " ": kind = .space
        case "\n": kind = .enter
        case "\u{8}": kind = .backspace
        default: kind = .char
        }
        if sess.key(kind, u.value, f) {
            continue
        }
        if kind == .backspace {
            if f.text.length > 0 {
                f.text.deleteCharacters(in: f.text.rangeOfComposedCharacterSequence(at: f.text.length - 1))
            }
        } else {
            f.text.append(String(u))
        }
    }
}

let cases: [Case] = [
    Case(name: "direct_converts") {
        let f = FakeField(), s = session(f)
        type("keyin choy ichamiz ", f, s)
        try check("text", f.value, "keyin çoy içamiz ")
        try check("preedit", f.marked, "")
    },
    Case(name: "english_untouched") {
        let f = FakeField(), s = session(f)
        type("Please check the schedule, keyin choy ichamiz ", f, s)
        try check("text", f.value, "Please check the schedule, keyin çoy içamiz ")
    },
    Case(name: "cyrillic") {
        let f = FakeField(), s = session(f)
        type("чой ичамиз ", f, s)
        try check("text", f.value, "çoy içamiz ")
    },
    Case(name: "backspace_undo") {
        let f = FakeField(), s = session(f)
        type("choy ", f, s)
        try check("converted", f.value, "çoy ")
        type("\u{8}", f, s)
        try check("restored", f.value, "choy")
        type(" choy ", f, s)
        try check("learned", f.value, "choy choy ")
    },
    Case(name: "enter_keeps_newline") {
        let f = FakeField(), s = session(f)
        type("salom shahar\n", f, s)
        try check("text", f.value, "salom şahar\n")
    },
    Case(name: "preedit_mode") {
        let f = FakeField(surrounding: false), s = session(f)
        type("shahar", f, s)
        try check("held text", f.value, "")
        try check("held preedit", f.marked, "shahar")
        type(" choy\n", f, s)
        try check("text", f.value, "şahar çoy\n")
        try check("preedit", f.marked, "")
        try check("empty setMarkedText after insertText", f.clearsAfterInsert, 0)
    },
    Case(name: "preedit_kept_on_focus_out") {
        let f = FakeField(surrounding: false), s = session(f)
        type("choy", f, s)
        s.commitComposition(f)
        try check("committed raw", f.value, "choy")
    },
    Case(name: "emoji_utf16") {
        // emoji = 2 UTF-16 birlik: almashtirish diapazoni UTF-16 da to'g'ri hisoblanishi kerak
        let f = FakeField(), s = session(f)
        type("😀 choy 👍🏽 shahar ", f, s)
        try check("text", f.value, "😀 çoy 👍🏽 şahar ")
    },
    Case(name: "terminal_untouched") {
        let f = FakeField(bundle: "com.apple.Terminal"), s = session(f)
        type("choy shahar ", f, s)
        try check("text", f.value, "choy shahar ")
    },
    Case(name: "mouse_moves_cursor") {
        // "cho" dan keyin sichqoncha bosildi: "y" yangi so'z boshi emas, so'z o'girilmaydi
        let f = FakeField(), s = session(f)
        type("cho", f, s)
        s.commitComposition(f)
        type("y ", f, s)
        try check("text", f.value, "choy ")
    },
    Case(name: "commit_twice_writes_once") {
        let f = FakeField(surrounding: false), s = session(f)
        type("choy", f, s)
        s.commitComposition(f)
        s.commitComposition(f)
        try check("text", f.value, "choy")
    },
    Case(name: "navigate_flushes_preedit") {
        let f = FakeField(surrounding: false), s = session(f)
        type("choy", f, s)
        try check("consumed", s.key(.navigate, 0, f), false)
        try check("text", f.value, "choy")
        try check("preedit", f.marked, "")
    },
    Case(name: "few_app_queries") {
        // "choy " — faqat so'z boshi va probelda matn so'raladi (5 emas, 2)
        let f = FakeField(), s = session(f)
        f.queries = 0
        type("choy ", f, s)
        try check("text", f.value, "çoy ")
        try check("queries", f.queries, 2)
        // darhol Backspace — qaytarish uchun matn kerak
        type("\u{8}", f, s)
        try check("restored", f.value, "choy")
        try check("queries", f.queries, 3)
    },
    Case(name: "prefs_keep_other_lines") {
        var p = Prefs(text: "\u{feff}# izoh\r\nEnabled = 1\nfoo=bar\nenabled=1\n")
        try check("read", p.enabled, true)
        p.enabled = false
        try check("text", p.text, "# izoh\nenabled=0\nfoo=bar\n")
        try check("flag words", Prefs(text: "enabled=yo‘q").enabled, false)
        try check("bad value -> default", Prefs(text: "enabled=maybe").enabled, true)
        try check("last wins", Prefs(text: "undo_remember=1\nundo_remember=0").undoRemember, false)
    },
    Case(name: "prefs_disabled_apps") {
        var p = Prefs(text: "")
        try check("terminal default", p.isDisabled(bundle: "com.apple.Terminal"), true)
        try check("textedit default", p.isDisabled(bundle: "com.apple.TextEdit"), false)
        p.setDisabled(bundle: "com.apple.TextEdit", true)
        try check("added", p.isDisabled(bundle: "com.apple.textedit"), true)
        try check("terminal kept", p.isDisabled(bundle: "com.apple.Terminal"), true)
        p.setDisabled(bundle: "com.apple.Terminal", false)
        try check("terminal on", p.isDisabled(bundle: "com.apple.Terminal"), false)
        try check("empty key = none", Prefs(text: "disabled_apps=").isDisabled(bundle: "com.apple.Terminal"), false)
    },
    Case(name: "app_disabled_in_settings") {
        let f = FakeField(), s = session(f, ini: "disabled_apps=com.apple.TextEdit\n")
        type("choy shahar ", f, s)
        try check("text", f.value, "choy shahar ")
    },
    Case(name: "terminal_enabled_by_user") {
        let f = FakeField(bundle: "com.apple.Terminal"), s = session(f, ini: "disabled_apps=\n")
        type("choy ", f, s)
        try check("text", f.value, "çoy ")
    },
    Case(name: "toggle_applies_immediately") {
        let f = FakeField(), s = session(f)
        type("choy ", f, s)
        var p = Prefs.load()
        p.enabled = false
        try p.save()
        s.updateSuspended(f, p)
        type("shahar ", f, s)
        try check("off", f.value, "çoy shahar ")
        p.enabled = true
        try p.save()
        s.updateSuspended(f, p)
        Thread.sleep(forTimeInterval: 1.1) // dvigatel settings.ini ni sekundiga bir marta qayta o'qiydi
        type("choy ", f, s)
        try check("on again", f.value, "çoy shahar çoy ")
    },
    Case(name: "key_map") {
        typealias K = (code: UInt16, flags: NSEvent.ModifierFlags, chars: String?, kind: BitgKey, cp: UInt32)
        let table: [K] = [
            (0, [], "a", .char, 0x61),
            (0, .shift, "A", .char, 0x41),
            (0, [.shift, .capsLock], "A", .char, 0x41),
            (0, [], "ф", .char, 0x444),
            (39, [], "'", .char, 0x27),
            (49, [], " ", .space, 0x20),
            (49, .option, "\u{A0}", .space, 0xA0),
            (36, [], "\r", .enter, 0),
            (76, [], "\u{3}", .enter, 0),
            (48, [], "\t", .tab, 0),
            (51, [], "\u{7F}", .backspace, 0),
            (51, .option, "\u{7F}", .other, 0),
            (51, .command, "\u{7F}", .other, 0),
            (123, [.function, .numericPad], "\u{F702}", .navigate, 0),
            (117, .function, "\u{F728}", .navigate, 0),
            (53, [], "\u{1B}", .navigate, 0),
            (0, .command, "a", .other, 0),
            (0, .control, "\u{1}", .other, 0),
            (32, .option, "", .other, 0), // o'lik tugma Option+U
            (122, .function, "\u{F704}", .other, 0), // F1
            (0, [], nil, .other, 0),
        ]
        for k in table {
            let (kind, cp) = Keys.map(keyCode: k.code, flags: k.flags, characters: k.chars)
            try check("key \(k.code) \(k.chars.map { Array($0.unicodeScalars) } ?? [])", "\(kind) \(cp)", "\(k.kind) \(k.cp)")
        }
    },
]

var failed = 0
for c in cases {
    do {
        try c.run()
        print("ok   \(c.name)")
    } catch {
        failed += 1
        print("FAIL \(c.name): \(error)")
    }
}
print("\(cases.count - failed)/\(cases.count) o'tdi")
exit(failed == 0 ? 0 : 1)
