// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

fn main() {
    let protos = [
        "src/proto/mempool.proto",
        "src/proto/shared/mempool_status.proto",
    ];

    let includes = ["../types/src/proto", "src/proto", "src/proto/shared"];

    grpcio_compiler::prost_codegen::compile_protos(
        &protos,
        &includes,
        &std::env::var("OUT_DIR").unwrap(),
    )
    .unwrap();
}
