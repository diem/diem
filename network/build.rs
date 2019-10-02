// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// Builds the proto files needed for the network crate.
fn main() {
    let proto_files = [
        "src/proto/consensus.proto",
        "src/proto/admission_control.proto",
    ];

    let proto_files_prost = [
        "src/proto/network.proto",
        "src/proto/mempool.proto",
        "src/proto/state_synchronizer.proto",
    ];

    let includes = ["../types/src/proto", "src/proto"];

    for file in &proto_files {
        println!("cargo:rerun-if-changed={}", file);
    }

    for file in &proto_files_prost {
        println!("cargo:rerun-if-changed={}", file);
    }

    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/proto",
        input: &proto_files,
        includes: &includes,
        customize: protoc_rust::Customize {
            carllerche_bytes_for_bytes: Some(true),
            carllerche_bytes_for_string: Some(true),
            ..Default::default()
        },
    })
    .expect("protoc");

    prost_build::compile_protos(&proto_files_prost, &includes).unwrap();
}
