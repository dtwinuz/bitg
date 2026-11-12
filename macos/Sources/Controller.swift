// bitg macOS: InputMethodKit kontrolleri. Mantiq — Session.swift va crates/bitg-ime; bu fayl faqat IMK bilan bog'laydi.
import AppKit
import InputMethodKit

/// IMK mijozini `TextClient` ga moslaydi.
final class IMKClient: TextClient {
    private let c: IMKTextInput & NSObjectProtocol

    init(_ c: IMKTextInput & NSObjectProtocol) {
        self.c = c
    }

    func selection() -> NSRange {
        c.selectedRange()
    }

    func text(in range: NSRange) -> String? {
        c.attributedSubstring(from: range)?.string
    }

    func insert(_ text: String, replacing range: NSRange) {
        c.insertText(text, replacementRange: range)
    }

    func setMarked(_ text: String) {
        let s = NSAttributedString(string: text, attributes: [.underlineStyle: NSUnderlineStyle.single.rawValue])
        c.setMarkedText(s, selectionRange: NSRange(location: (text as NSString).length, length: 0),
                        replacementRange: NSRange(location: NSNotFound, length: 0))
    }

    var bundleID: String? {
        c.bundleIdentifier()
    }
}

enum Keys {
    /// macOS virtual key code'lari (Carbon kVK_*).
    static func map(_ e: NSEvent) -> (BitgKey, UInt32) {
        let m = e.modifierFlags.intersection(.deviceIndependentFlagsMask)
        if m.contains(.command) || m.contains(.control) {
            return (.other, 0)
        }
        switch e.keyCode {
        case 49: // Space; Option+Space — NBSP
            return (.space, e.characters?.unicodeScalars.first?.value == 0xA0 ? 0xA0 : 0x20)
        case 36, 76: // Return, keypad Enter
            return (.enter, 0)
        case 48:
            return (.tab, 0)
        case 51: // Delete (Backspace)
            return (.backspace, 0)
        case 53, 115, 116, 117, 119, 121, 123, 124, 125, 126: // Esc, Home, PgUp, Fwd Delete, End, PgDn, strelkalar
            return (.navigate, 0)
        default:
            break
        }
        // tartib qo'llangan belgi; o'lik tugma (Option+U) yoki bir nechta belgi — so'z bilan bog'lamaymiz
        guard let s = e.characters?.unicodeScalars, s.count == 1, let u = s.first else {
            return (.other, 0)
        }
        let v = u.value
        if v < 0x20 || v == 0x7F || (0xF700...0xF8FF).contains(v) { // funksional tugmalar
            return (.other, 0)
        }
        return (.char, v)
    }
}

@objc(BitgController)
final class BitgController: IMKInputController {
    private let session = BitgSession()

    private func wrap(_ sender: Any!) -> IMKClient? {
        (sender as? (IMKTextInput & NSObjectProtocol)).map(IMKClient.init)
    }

    override func recognizedEvents(_ sender: Any!) -> Int {
        Int(NSEvent.EventTypeMask.keyDown.rawValue)
    }

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event, event.type == .keyDown, let c = wrap(sender) else { return false }
        let (kind, cp) = Keys.map(event)
        return session.key(kind, cp, c)
    }

    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        if let c = wrap(sender) {
            session.activate(c)
        }
    }

    override func deactivateServer(_ sender: Any!) {
        if let c = wrap(sender) {
            session.commitComposition(c)
        }
        super.deactivateServer(sender)
    }

    override func commitComposition(_ sender: Any!) {
        if let c = wrap(sender) {
            session.commitComposition(c)
        }
    }
}
