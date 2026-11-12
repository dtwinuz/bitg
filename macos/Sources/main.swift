// bitg macOS kiritish usuli: /Library/Input Methods/bitg.app
//   bitg               IMK serveri (macOS o'zi ishga tushiradi)
//   bitg --register    kiritish manbasini ro'yxatdan o'tkazadi va yoqadi (o'rnatuvchi postinstall chaqiradi)
import AppKit
import Carbon
import InputMethodKit

func register() -> Int32 {
    let url = Bundle.main.bundleURL as CFURL
    let st = TISRegisterInputSource(url)
    guard st == noErr, let id = Bundle.main.bundleIdentifier else {
        FileHandle.standardError.write("bitg: TISRegisterInputSource \(st)\n".data(using: .utf8)!)
        return 1
    }
    let filter = [kTISPropertyBundleID as String: id] as CFDictionary
    let sources = TISCreateInputSourceList(filter, true)?.takeRetainedValue() as? [TISInputSource] ?? []
    for s in sources {
        TISEnableInputSource(s)
    }
    print("bitg: \(sources.count) ta kiritish manbasi yoqildi")
    return 0
}

if CommandLine.arguments.contains("--register") {
    exit(register())
}

let name = Bundle.main.infoDictionary?["InputMethodConnectionName"] as? String ?? "uz.bitg.inputmethod.bitg_Connection"
guard let server = IMKServer(name: name, bundleIdentifier: Bundle.main.bundleIdentifier) else {
    FileHandle.standardError.write("bitg: IMKServer yaratilmadi\n".data(using: .utf8)!)
    exit(1)
}
_ = server
NSApplication.shared.run()
