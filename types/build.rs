// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This compiles all the `.proto` files under `src/` directory.
//!
//! For example, if there is a file `src/a/b/c.proto`, it will generate `src/a/b/c.rs` and
//! `src/a/b/c_grpc.rs`.

fn main() {
    let proto_root = "src/proto";

    build_helpers::build_helpers::compile_proto(
        proto_root,
        vec![], /* dependent roots */
        false,  /* generate_client_stub */
    );

    let proto_files = [
        "src/proto/access_path.proto",
        "src/proto/events.proto",
        "src/proto/language_storage.proto",
        "src/proto/proof.proto",
        "src/proto/transaction.proto",
        "src/proto/validator_public_keys.proto",
        "src/proto/vm_errors.proto",
        "src/proto/account_state_blob.proto",
        "src/proto/get_with_proof.proto",
        "src/proto/ledger_info.proto",
        "src/proto/transaction_info.proto",
        "src/proto/validator_change.proto",
        "src/proto/validator_set.proto",
        "src/proto/vm_errors.proto",
    ];

    let includes = ["src/proto"];

    prost_build::compile_protos(&proto_files, &includes).unwrap();
}
