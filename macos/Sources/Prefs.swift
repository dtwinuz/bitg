// bitg macOS: settings.ini (Windows va Linux bilan bir xil format, key=value). Dvigatel faylni o'zi o'qiydi
// (crates/bitg-ime/src/config.rs); bu yerda faqat Mac menyusi o'zgartiradigan kalitlar: enabled, undo_remember,
// disabled_apps. Boshqa qatorlar, izohlar va tartib saqlanadi. Foundation'dan boshqa narsaga bog'liq emas — macos/test.
import Foundation

/// Terminallar: buyruq va parol yoziladi — standart holatda bitg tegmaydi (Windows'dagi DEFAULT_DISABLED_APPS kabi).
/// `disabled_apps` kaliti bo'lsa, ro'yxat aynan o'sha (foydalanuvchi terminalda ham yoqishi mumkin).
let defaultDisabledApps = [
    "com.apple.terminal", "com.googlecode.iterm2", "dev.warp.warp-stable", "net.kovidgoyal.kitty",
    "io.alacritty", "org.alacritty", "com.github.wez.wezterm", "com.mitchellh.ghostty",
]

struct Prefs {
    private(set) var lines: [String]

    init(text: String) {
        var t = text
        if t.hasPrefix("\u{feff}") {
            t.removeFirst()
        }
        lines = t.isEmpty ? [] : t.components(separatedBy: "\n").map { $0.hasSuffix("\r") ? String($0.dropLast()) : $0 }
        if lines.last == "" {
            lines.removeLast()
        }
    }

    var text: String {
        lines.isEmpty ? "" : lines.joined(separator: "\n") + "\n"
    }

    // MARK: fayl

    /// `~/Library/Application Support/bitg` — dvigatelning `config_dir()` i bilan bir xil: $HOME `environ` dan
    /// (getenv), xuddi Rust'dagi `std::env::var_os` kabi — ikkalasi doim bitta papkani ko'radi.
    static var dir: URL {
        let env = getenv("HOME").map { String(cString: $0) } ?? ""
        let home = env.isEmpty ? NSHomeDirectory() : env
        return URL(fileURLWithPath: home).appendingPathComponent("Library/Application Support/bitg", isDirectory: true)
    }

    static var settingsURL: URL { dir.appendingPathComponent("settings.ini") }
    static var exceptionsURL: URL { dir.appendingPathComponent("exceptions.txt") }

    static func load() -> Prefs {
        Prefs(text: (try? String(contentsOf: settingsURL, encoding: .utf8)) ?? "")
    }

    /// Atomik yozish: dvigatel yarim yozilgan faylni o'qib qolmaydi.
    func save() throws {
        try FileManager.default.createDirectory(at: Self.dir, withIntermediateDirectories: true)
        try Data(text.utf8).write(to: Self.settingsURL, options: .atomic)
    }

    // MARK: kalitlar

    private static func parse(_ line: String) -> (key: String, value: String)? {
        let l = line.trimmingCharacters(in: .whitespaces)
        guard let f = l.first, !"#;[".contains(f), let eq = l.firstIndex(of: "=") else { return nil }
        let k = l[..<eq].trimmingCharacters(in: .whitespaces).lowercased()
        let v = l[l.index(after: eq)...].trimmingCharacters(in: .whitespaces)
        return (k, v)
    }

    /// Dvigateldagidek: kalit bir necha marta yozilgan bo'lsa, oxirgisi yutadi.
    func value(_ key: String) -> String? {
        var out: String?
        for l in lines {
            if let p = Self.parse(l), p.key == key {
                out = p.value
            }
        }
        return out
    }

    /// `crates/bitg-ime/src/config.rs` `flag()` bilan bir xil qiymatlar; noto'g'ri qiymat — standart.
    func bool(_ key: String, default d: Bool) -> Bool {
        switch value(key)?.lowercased() {
        case "1", "true", "yes", "on", "ha": return true
        case "0", "false", "no", "off", "yoq", "yo'q", "yo‘q": return false
        default: return d
        }
    }

    /// Mavjud qatorni almashtiradi (takrorlari o'chiriladi) yoki oxiriga qo'shadi.
    mutating func set(_ key: String, _ value: String) {
        var done = false
        lines = lines.compactMap { l in
            guard let p = Self.parse(l), p.key == key else { return l }
            if done {
                return nil
            }
            done = true
            return "\(key)=\(value)"
        }
        if !done {
            lines.append("\(key)=\(value)")
        }
    }

    var enabled: Bool {
        get { bool("enabled", default: true) }
        set { set("enabled", newValue ? "1" : "0") }
    }

    var undoRemember: Bool {
        get { bool("undo_remember", default: false) }
        set { set("undo_remember", newValue ? "1" : "0") }
    }

    /// Bundle ID'lar, kichik harfda. Kalit yo'q — standart terminallar.
    var disabledApps: [String] {
        get {
            guard let v = value("disabled_apps") else { return defaultDisabledApps }
            return v.split(whereSeparator: { $0 == "," || $0 == ";" })
                .map { $0.trimmingCharacters(in: .whitespaces).lowercased() }
                .filter { !$0.isEmpty }
        }
        set { set("disabled_apps", newValue.joined(separator: ", ")) }
    }

    func isDisabled(bundle: String?) -> Bool {
        guard let b = bundle?.lowercased(), !b.isEmpty else { return false }
        return disabledApps.contains(b)
    }

    mutating func setDisabled(bundle: String, _ off: Bool) {
        let b = bundle.lowercased()
        var apps = disabledApps.filter { $0 != b }
        if off {
            apps.append(b)
        }
        disabledApps = apps
    }

    /// Shu dasturda bitg hech narsaga tegmasinmi (umumiy o'chirish yoki dastur ro'yxatda).
    func suspended(bundle: String?) -> Bool {
        !enabled || isDisabled(bundle: bundle)
    }
}
