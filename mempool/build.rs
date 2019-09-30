// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This compiles all the `.proto` files under `src/` directory.
//!
//! For example, if there is a file `src/a/b/c.proto`, it will generate `src/a/b/c.rs` and
//! `src/a/b/c_grpc.rs`.

fn main() {
    let proto_root = "src/proto";
    let proto_shared_root = "src/proto/shared";
    let dependent_root = "../types/src/proto";
    // Build shared directory without further dependencies.
    libra_build_helpers::build_helpers::compile_proto(proto_shared_root, vec![], false);
    libra_build_helpers::build_helpers::compile_proto(
        proto_root,
        vec![dependent_root, proto_shared_root],
        true,
    );
}
