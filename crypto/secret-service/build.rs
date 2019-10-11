// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This compiles all the `.proto` files under `src/` directory.
//!
//! For example, if there is a file `src/a/b/c.proto`, it will generate `src/a/b/c.rs` and
//! `src/a/b/c_grpc.rs`.

fn main() {
    let protos = ["src/proto/secret_service.proto"];

    grpcio_compiler::prost_codegen::compile_protos(
        &protos,
        &["src/proto"],
        &std::env::var("OUT_DIR").unwrap(),
    )
    .unwrap();
}
