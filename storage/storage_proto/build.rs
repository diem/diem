// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This compiles all the `.proto` files under `src/` directory.
//!
//! For example, if there is a file `src/a/b/c.proto`, it will generate `src/a/b/c.rs` and
//! `src/a/b/c_grpc.rs`.

fn main() {
    let proto_root = "src/proto";
    let dependent_root = "../../types/src/proto";

    build_helpers::build_helpers::compile_proto(
        proto_root,
        vec![dependent_root],
        false, /* generate_client_code */
    );

    let protos = ["src/proto/storage.proto"];

    let includes = ["../../types/src/proto", "src/proto"];

    prost_build::compile_protos(&protos, &includes).unwrap();
}
