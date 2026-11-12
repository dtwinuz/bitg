// bitg macOS: InputMethodKit kontrolleri. Mantiq — Session.swift, Keys.swift va crates/bitg-ime; bu fayl faqat
// IMK bilan bog'laydi. IMK har bir matn mijozi uchun alohida kontroller yaratadi va hammasini asosiy oqimda chaqiradi.
import AppKit
import InputMethodKit

/// IMK mijozini `TextClient` ga moslaydi. Barcha diapazonlar UTF-16 birligida.
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

@objc(BitgController)
final class BitgController: IMKInputController {
    private let session = BitgSession()

    /// IMK ba'zan `sender` ni nil beradi (masalan Cmd+Space bilan manba almashganda commitComposition) —
    /// unda kontrollerga bog'langan mijoz olinadi, aks holda preedit'dagi so'z yo'qoladi.
    private func wrap(_ sender: Any!) -> IMKClient? {
        ((sender as? (IMKTextInput & NSObjectProtocol)) ?? client()).map(IMKClient.init)
    }

    override func recognizedEvents(_ sender: Any!) -> Int {
        Int(NSEvent.EventTypeMask([.keyDown, .leftMouseDown]).rawValue)
    }

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event, let c = wrap(sender) else { return false }
        switch event.type {
        case .keyDown:
            let (kind, cp) = Keys.map(event)
            return session.key(kind, cp, c)
        case .leftMouseDown:
            // kursor sichqoncha bilan ko'chdi: preedit'dagi so'z o'girilmay yoziladi, so'z boshi noma'lum
            session.commitComposition(c)
            return false
        default:
            return false
        }
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
