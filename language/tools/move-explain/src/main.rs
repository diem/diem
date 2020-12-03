// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
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

    match move_explain::get_explanation(&module_id, args.abort_code) {
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
