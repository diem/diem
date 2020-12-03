// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bindgen::EnumVariation;
use std::{env, path::PathBuf};

fn main() {
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("include/data.h")
        // Finish the builder and generate the bindings.
        .array_pointers_in_arguments(true)
        .size_t_is_usize(true)
        .derive_default(true)
        .derive_eq(true)
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: false,
        })
        .whitelist_type("DiemStatus")
        .whitelist_type("DiemP2PTransferTransactionArgument")
        .whitelist_type("DiemTransactionPayload")
        .whitelist_type("DiemRawTransaction")
        .whitelist_type("DiemSignedTransaction")
        .whitelist_type("TransactionType")
        .whitelist_type("DiemAccountKey")
        .whitelist_var("DIEM_PUBKEY_SIZE")
        .whitelist_var("DIEM_PRIVKEY_SIZE")
        .whitelist_var("DIEM_AUTHKEY_SIZE")
        .whitelist_var("DIEM_ADDRESS_SIZE")
        .whitelist_var("DIEM_SIGNATURE_SIZE")
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings.");

    // Write the bindings to the src/data.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("data.rs"))
        .expect("Couldn't write bindings!");
}
