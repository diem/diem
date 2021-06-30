// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_command_line_common::files::MOVE_ERROR_DESC_EXTENSION;
use move_core_types::{
    account_address::AccountAddress, errmap::ErrorMapping, identifier::Identifier,
    language_storage::ModuleId,
};
use structopt::StructOpt;
#[derive(Debug, StructOpt)]
#[structopt(
    name = "Move Explain",
    about = "Explain Move abort codes. Errors are defined as a global category + module-specific reason for the error."
)]
struct Args {
    /// The location (module id) returned with a `MoveAbort` error
    #[structopt(long = "location", short = "l")]
    location: String,
    /// The abort code returned with a `MoveAbort` error
    #[structopt(long = "abort-code", short = "a")]
    abort_code: u64,
    /// Path to the error code mapping file
    #[structopt(long = MOVE_ERROR_DESC_EXTENSION, short = "e")]
    errmap_path: String,
}

fn main() {
    let args = Args::from_args();

    let mut location = args.location.trim().split("::");
    let mut address_literal = location.next().expect("Could not find address").to_string();
    let module_name = location
        .next()
        .expect("Could not find module name")
        .to_string();
    if !address_literal.starts_with("0x") {
        address_literal = format!("0x{}", address_literal);
    }
    let module_id = ModuleId::new(
        AccountAddress::from_hex_literal(&address_literal).expect("Unable to parse module address"),
        Identifier::new(module_name).expect("Invalid module name encountered"),
    );

    let errmap_bytes = std::fs::read(&args.errmap_path).expect("Could not load errmap from file");
    let errmap: ErrorMapping =
        bcs::from_bytes(&errmap_bytes).expect("Failed to deserialize errmap");

    match errmap.get_explanation(&module_id, args.abort_code) {
        None => println!(
            "Unable to find a description for {}::{}",
            args.location, args.abort_code
        ),
        Some(error_desc) => println!(
            "Category:\n  Name: {}\n  Description: {}\nReason:\n  Name: {}\n  Description: {}",
            error_desc.category.code_name,
            error_desc.category.code_description,
            error_desc.reason.code_name,
            error_desc.reason.code_description,
        ),
    }
}
