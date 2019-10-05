// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This compiles all the `.proto` files under `src/` directory.
//!
//! For example, if there is a file `src/a/b/c.proto`, it will generate `src/a/b/c.rs` and
//! `src/a/b/c_grpc.rs`.

fn main() {
    let proto_root = "src";
    let dependent_root = "../../types/src/proto";
    let mempool_dependent_root = "../../mempool/src/proto/shared";

    build_helpers::build_helpers::compile_proto(
        proto_root,
        vec![dependent_root, mempool_dependent_root],
        false,
    );
}
