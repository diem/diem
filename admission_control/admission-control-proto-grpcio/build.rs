// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

fn main() {
    let protos = ["../admission-control-proto/src/proto/admission_control.proto"];

    let includes = [
        "../../types/src/proto",
        "../admission-control-proto/src/proto",
        "../../mempool/mempool-shared-proto/src/proto",
    ];

    grpcio_compiler::prost_codegen::compile_protos(
        &protos,
        &includes,
        &std::env::var("OUT_DIR").unwrap(),
    )
    .unwrap();
}
