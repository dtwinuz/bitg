// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

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

    // MARK: menyu (kiritish manbalari menyusida, bitg tanlanganda)

    override func menu() -> NSMenu! {
        let p = Prefs.load()
        let m = NSMenu(title: "bitg")
        item(m, "bitg yoqilgan", #selector(toggleEnabled(_:)), on: p.enabled)
        if let id = client()?.bundleIdentifier(), !id.isEmpty {
            item(m, "\(Self.appName(id)) dasturida o‘chirish", #selector(toggleApp(_:)), on: p.isDisabled(bundle: id))
        }
        item(m, "Qaytarilgan so‘zlarni eslab qolish", #selector(toggleRemember(_:)), on: p.undoRemember)
        m.addItem(.separator())
        item(m, "Istisno so‘zlar…", #selector(openExceptions(_:)))
        item(m, "Yordam — bitg.uz", #selector(openHelp(_:)))
        return m
    }

    private func item(_ m: NSMenu, _ title: String, _ action: Selector, on: Bool? = nil) {
        let i = NSMenuItem(title: title, action: action, keyEquivalent: "")
        i.target = self
        if let on {
            i.state = on ? .on : .off
        }
        m.addItem(i)
    }

    private static func appName(_ bundle: String) -> String {
        guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundle) else { return bundle }
        let name = FileManager.default.displayName(atPath: url.path)
        return name.hasSuffix(".app") ? String(name.dropLast(4)) : name
    }

    /// O'zgartiradi, saqlaydi va joriy dasturga darhol qo'llaydi (boshqalari — dvigatel faylni qayta o'qiganda, ≤1 s).
    private func change(_ f: (inout Prefs) -> Void) {
        var p = Prefs.load()
        f(&p)
        do {
            try p.save()
        } catch {
            FileHandle.standardError.write("bitg: settings.ini yozilmadi: \(error)\n".data(using: .utf8)!)
            NSSound.beep()
            return
        }
        if let c = wrap(nil) {
            session.updateSuspended(c, p)
        }
    }

    @objc func toggleEnabled(_ sender: Any?) {
        change { $0.enabled.toggle() }
    }

    @objc func toggleApp(_ sender: Any?) {
        guard let id = client()?.bundleIdentifier(), !id.isEmpty else { return }
        change { $0.setDisabled(bundle: id, !$0.isDisabled(bundle: id)) }
    }

    @objc func toggleRemember(_ sender: Any?) {
        change { $0.undoRemember.toggle() }
    }

    @objc func openExceptions(_ sender: Any?) {
        let url = Prefs.exceptionsURL
        if !FileManager.default.fileExists(atPath: url.path) {
            try? FileManager.default.createDirectory(at: Prefs.dir, withIntermediateDirectories: true)
            let head = "# bitg bu so‘zlarni o‘girmaydi: har qatorda bitta so‘z (masalan, nom yoki atama).\n"
            try? Data(head.utf8).write(to: url, options: .atomic)
        }
        NSWorkspace.shared.open(url)
    }

    @objc func openHelp(_ sender: Any?) {
        if let url = URL(string: "https://bitg.uz/yuklab-olish/#mac") {
            NSWorkspace.shared.open(url)
        }
    }

    // MARK: hodisalar

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
