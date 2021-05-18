// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

fn main() {
    let mut conf = prost_build::Config::new();
    conf.type_attribute(".", "#[derive(::serde::Serialize, ::serde::Deserialize)]");

    conf.field_attribute(
        ".",
        "#[serde(default, skip_serializing_if = \"crate::is_default\")]",
    );

    conf.compile_protos(&["src/jsonrpc.proto"], &["src/"])
        .unwrap();
}
