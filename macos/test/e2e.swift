// bitg macOS uchidan-uchiga sinov: haqiqiy Cocoa matn maydoni (NSTextView) + haqiqiy InputMethodKit + o'rnatilgan
// /Library/Input Methods/bitg.app. Tugmalar dasturning o'z hodisa navbatiga qo'yiladi (NSApp.postEvent) va
// NSTextView -> NSTextInputContext -> IMK -> BitgController yo'lidan o'tadi, xuddi foydalanuvchi bosgandek.
// TCC ruxsati (Accessibility) kerak emas.
// Faqat haqiqiy Mac'da: macOS 15 uchinchi tomon kiritish usulini dastur orqali tanlashni rad etadi
// (TISSelectInputSource = -50, HIToolbox sozlamasi ham yordam bermaydi), shuning uchun CI uni faqat yig'adi.
//   1) bitg .pkg ni o'rnating, menyu satridan bitg ni tanlang
//   2) swiftc -O macos/test/e2e.swift -o bitg-e2e && ./bitg-e2e
//   chiqish kodi 0 = hammasi o'tdi, 1 = farq, 2 = bitg tanlanmagan, 3 = vaqt tugadi
import AppKit
import Carbon

let modeID = "uz.bitg.inputmethod.bitg.latin"

/// (nom, tugmalar, kutilgan matn). "\u{8}" = Backspace, "\n" = Return. backspace_undo so'zni eslab qoladi — oxirida.
let cases: [(String, String, String)] = [
    ("direct_converts", "keyin choy ichamiz ", "keyin çoy içamiz "),
    ("english_untouched", "Please check the schedule ", "Please check the schedule "),
    ("enter_keeps_newline", "salom shahar\n", "salom şahar\n"),
    ("backspace_undo", "choy \u{8}", "choy"),
]

/// ANSI (US) virtual key code'lari.
let keyCodes: [Character: UInt16] = [
    "a": 0, "s": 1, "d": 2, "f": 3, "h": 4, "g": 5, "z": 6, "x": 7, "c": 8, "v": 9, "b": 11, "q": 12, "w": 13,
    "e": 14, "r": 15, "y": 16, "t": 17, "o": 31, "u": 32, "i": 34, "p": 35, "l": 37, "j": 38, "k": 40, "n": 45,
    "m": 46, "'": 39, " ": 49, "\n": 36, "\u{8}": 51,
]

func log(_ s: String) {
    FileHandle.standardOutput.write((s + "\n").data(using: .utf8)!)
}

func sourceID(_ s: TISInputSource) -> String {
    guard let p = TISGetInputSourceProperty(s, kTISPropertyInputSourceID) else { return "?" }
    return Unmanaged<CFString>.fromOpaque(p).takeUnretainedValue() as String
}

func flag(_ s: TISInputSource, _ key: CFString) -> String {
    guard let p = TISGetInputSourceProperty(s, key) else { return "-" }
    return CFBooleanGetValue(Unmanaged<CFBoolean>.fromOpaque(p).takeUnretainedValue()) ? "1" : "0"
}

func currentID() -> String {
    sourceID(TISCopyCurrentKeyboardInputSource().takeRetainedValue())
}

/// bitg manbalarini (bundle va rejim) xossalari bilan chiqaradi, rejimni yoqib tanlaydi; tayyor bo'lsa true.
func selectBitg(attempt: Int) -> Bool {
    let all = TISCreateInputSourceList(nil, true)?.takeRetainedValue() as? [TISInputSource] ?? []
    let ours = all.filter { sourceID($0).hasPrefix("uz.bitg") }
    for s in ours where attempt == 0 {
        log("setup: \(sourceID(s)) enabled=\(flag(s, kTISPropertyInputSourceIsEnabled))"
            + " selectCapable=\(flag(s, kTISPropertyInputSourceIsSelectCapable))"
            + " enableCapable=\(flag(s, kTISPropertyInputSourceIsEnableCapable))")
    }
    guard let mode = ours.first(where: { sourceID($0) == modeID }) else {
        log("setup: \(modeID) topilmadi (o'rnatilganmi, --register qilinganmi?)")
        return false
    }
    for s in ours where flag(s, kTISPropertyInputSourceIsEnabled) == "0" {
        _ = TISEnableInputSource(s)
    }
    let sel = TISSelectInputSource(mode)
    log("setup[\(attempt)]: select=\(sel) current=\(currentID())")
    return currentID() == modeID
}

final class Runner: NSObject, NSApplicationDelegate {
    let window = NSWindow(contentRect: NSRect(x: 200, y: 200, width: 640, height: 240),
                          styleMask: [.titled], backing: .buffered, defer: false)
    let tv = NSTextView(frame: NSRect(x: 0, y: 0, width: 640, height: 240))
    var failed = 0
    var index = 0

    func applicationDidFinishLaunching(_ n: Notification) {
        tv.isRichText = false
        tv.isAutomaticQuoteSubstitutionEnabled = false
        tv.isAutomaticDashSubstitutionEnabled = false
        tv.isAutomaticTextReplacementEnabled = false
        tv.isAutomaticSpellingCorrectionEnabled = false
        tv.isContinuousSpellCheckingEnabled = false
        tv.smartInsertDeleteEnabled = false
        window.contentView = tv
        window.makeKeyAndOrderFront(nil)
        window.makeFirstResponder(tv)
        NSApp.activate(ignoringOtherApps: true)
        DispatchQueue.main.asyncAfter(deadline: .now() + 120) {
            log("FAIL timeout")
            exit(3)
        }
        prepare(0)
    }

    /// Tizim manbasini tanlaydi (TIS keshi yangi o'rnatilgan IM'ni kechikib ko'radi, shuning uchun qayta urinadi),
    /// so'ng shu maydonning NSTextInputContext'ida ham tanlaydi.
    func prepare(_ attempt: Int) {
        let system = selectBitg(attempt: attempt)
        let ctx = tv.inputContext
        ctx?.selectedKeyboardInputSource = modeID
        let local = ctx?.selectedKeyboardInputSource == modeID
        if attempt == 0 || system || local {
            let listed = ctx?.keyboardInputSources?.contains(modeID) ?? false
            log("setup[\(attempt)]: inputContext=\(ctx?.selectedKeyboardInputSource ?? "nil") listed=\(listed)")
        }
        if system || local {
            // IMK serveri birinchi marta ishga tushishi uchun vaqt
            DispatchQueue.main.asyncAfter(deadline: .now() + 3) { self.next() }
        } else if attempt < 10 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 1) { self.prepare(attempt + 1) }
        } else {
            log("FAIL setup: bitg tanlanmadi")
            exit(2)
        }
    }

    func next() {
        guard index < cases.count else {
            log("\(cases.count - failed)/\(cases.count) o'tdi")
            exit(failed == 0 ? 0 : 1)
        }
        let (name, keys, want) = cases[index]
        tv.string = ""
        // fokusni qayta berish IMK'da deactivate/activate chaqiradi: har holat yangi so'z boshidan
        window.makeFirstResponder(nil)
        window.makeFirstResponder(tv)
        typeKeys(Array(keys), 0) {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.8) {
                let got = self.tv.string
                if got == want {
                    log("ok   \(name)")
                } else {
                    self.failed += 1
                    log("FAIL \(name): \(got.debugDescription) != \(want.debugDescription)")
                }
                self.index += 1
                self.next()
            }
        }
    }

    func typeKeys(_ keys: [Character], _ i: Int, done: @escaping () -> Void) {
        guard i < keys.count else { return done() }
        post(keys[i])
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.08) { self.typeKeys(keys, i + 1, done: done) }
    }

    func post(_ ch: Character) {
        guard let code = keyCodes[Character(ch.lowercased())] else {
            log("FAIL setup: '\(ch)' uchun tugma yo'q")
            exit(2)
        }
        let chars: String
        switch ch {
        case "\n": chars = "\r"
        case "\u{8}": chars = "\u{7f}"
        default: chars = String(ch)
        }
        for type in [NSEvent.EventType.keyDown, .keyUp] {
            guard let e = NSEvent.keyEvent(with: type, location: .zero, modifierFlags: ch.isUppercase ? .shift : [],
                                           timestamp: ProcessInfo.processInfo.systemUptime,
                                           windowNumber: window.windowNumber, context: nil, characters: chars,
                                           charactersIgnoringModifiers: chars.lowercased(), isARepeat: false,
                                           keyCode: code)
            else { continue }
            NSApp.postEvent(e, atStart: false)
        }
    }
}

let app = NSApplication.shared
app.setActivationPolicy(.regular)
let runner = Runner()
app.delegate = runner
app.run()
