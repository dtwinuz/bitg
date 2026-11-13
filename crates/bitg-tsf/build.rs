// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "../../tools/winres/versioninfo.rs"]
mod versioninfo;

fn main() {
    // Faqat cdylib (IME DLL) ga: rlib sifatida bitg-settings'ga ulanganda resurs takrorlanmasin.
    if let Some(rc) = versioninfo::write(&versioninfo::Info {
        description: "bitg input method (TSF)",
        file_name: "bitg_tsf.dll",
        dll: true,
        extra: "",
    }) {
        embed_resource::compile_for_cdylib(&rc, embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
