// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

fn main() {
    let proto_files = [
        "src/proto/access_path.proto",
        "src/proto/events.proto",
        "src/proto/mempool_status.proto",
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
