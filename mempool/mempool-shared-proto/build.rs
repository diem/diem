// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Builds the proto files needed for the mempool-shared-proto crate.
//!
//! This compiles all the `.proto` files under `src/` directory.
//! For example, if there is a file `src/a/b/c.proto`, it will generate `src/a/b/c.rs` and
//! `src/a/b/c_grpc.rs`.
fn main() {
    let proto_files_prost = ["src/proto/mempool_status.proto"];
    let includes = ["../../types/src/proto", "src/proto"];
    prost_build::compile_protos(&proto_files_prost, &includes).unwrap();
}
