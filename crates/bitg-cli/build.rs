// SPDX-FileCopyrightText: 2026 dtwinuz and bitg contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "../../tools/winres/versioninfo.rs"]
mod versioninfo;

fn main() {
    if let Some(rc) = versioninfo::write(&versioninfo::Info {
        description: "bitg command line",
        file_name: "bitg.exe",
        dll: false,
        extra: "",
    }) {
        embed_resource::compile(&rc, embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
