// bitg macOS sinovi: Session.swift + dvigatel, soxta matn maydoni bilan (IMK'siz).
// Holatlar Linux'dagi linux/test/ibus_e2e.py bilan bir xil, ustiga UTF-16 (emoji) va terminal.
import Foundation

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

    func selection() -> NSRange {
        surrounding ? NSRange(location: text.length, length: 0) : NSRange(location: NSNotFound, length: 0)
    }

    func text(in range: NSRange) -> String? {
        surrounding ? text.substring(with: range) : nil
    }

    func insert(_ s: String, replacing range: NSRange) {
        if range.location == NSNotFound {
            text.append(s)
        } else {
            precondition(NSMaxRange(range) == text.length, "almashtirish kursor oldida emas: \(range)")
            text.replaceCharacters(in: range, with: s)
        }
        marked = ""
    }

    func setMarked(_ s: String) {
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
func session(_ f: FakeField) -> BitgSession {
    let dir = FileManager.default.temporaryDirectory.appendingPathComponent("bitg-test-\(UUID().uuidString)")
    try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    setenv("HOME", dir.path, 1)
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
