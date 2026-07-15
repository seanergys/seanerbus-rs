// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

fn main() {
    capnpc::CompilerCommand::new()
        .output_path("src/")
        .src_prefix("schemas")
        .file("schemas/internal.capnp")
        .run()
        .expect("capnp compiles")
}
