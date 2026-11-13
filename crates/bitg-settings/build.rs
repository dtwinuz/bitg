// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "../../tools/winres/versioninfo.rs"]
mod versioninfo;

fn main() {
    println!("cargo:rerun-if-changed=assets/bitg.ico");
    println!("cargo:rerun-if-changed=assets/bitg-off.ico");
    // 1 — asosiy ikonka (exe, oyna, tray), 2 — o'chiq holat (tray).
    let icons = format!(
        "1 ICON \"{}\"\n2 ICON \"{}\"\n",
        versioninfo::manifest_path("assets/bitg.ico"),
        versioninfo::manifest_path("assets/bitg-off.ico"),
    );
    if let Some(rc) = versioninfo::write(&versioninfo::Info {
        description: "bitg settings and tray",
        file_name: "bitg-settings.exe",
        dll: false,
        extra: &icons,
    }) {
        embed_resource::compile(&rc, embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
