// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

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
    // ro'yxatda ikkita manba: kiritish usulining o'zi va tanlanadigan rejim (.latin). Foydalanuvchiga ko'rinadigani
    // va yoqilishi shart bo'lgani — rejim; usulning o'zini yoqish ko'pincha xato qaytaradi, bu normal.
    let mode = "\(id).latin"
    let filter = [kTISPropertyBundleID as String: id] as CFDictionary
    let sources = TISCreateInputSourceList(filter, true)?.takeRetainedValue() as? [TISInputSource] ?? []
    var modeStatus: OSStatus?
    for s in sources {
        let st = TISEnableInputSource(s)
        if sourceID(s) == mode {
            modeStatus = st
        }
    }
    guard let modeStatus else {
        FileHandle.standardError.write("bitg: \(mode) topilmadi (\(sources.count) ta manba); qayta kiring\n".data(using: .utf8)!)
        return 2
    }
    guard modeStatus == noErr else {
        FileHandle.standardError.write("bitg: \(mode) yoqilmadi: \(modeStatus)\n".data(using: .utf8)!)
        return 3
    }
    print("bitg: \(mode) yoqildi")
    return 0
}

func sourceID(_ s: TISInputSource) -> String? {
    guard let p = TISGetInputSourceProperty(s, kTISPropertyInputSourceID) else { return nil }
    return Unmanaged<CFString>.fromOpaque(p).takeUnretainedValue() as String
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
