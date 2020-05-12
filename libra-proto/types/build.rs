// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

fn main() {
    let proto_files = [
        "src/access_path.proto",
        "src/events.proto",
        "src/mempool_status.proto",
        "src/proof.proto",
        "src/transaction.proto",
        "src/validator_info.proto",
        "src/vm_errors.proto",
        "src/account_state_blob.proto",
        "src/get_with_proof.proto",
        "src/ledger_info.proto",
        "src/transaction_info.proto",
        "src/epoch_change.proto",
        "src/validator_set.proto",
        "src/vm_errors.proto",
    ];

    let includes = ["src"];

    prost_build::compile_protos(&proto_files, &includes).unwrap();
}
