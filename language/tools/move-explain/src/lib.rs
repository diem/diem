// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    account_address::AccountAddress, errmap::ErrorMapping, identifier::Identifier,
    language_storage::ModuleId,
};
use std::{io::Write, path::Path};

pub fn parse_and_explain<W: Write>(
    w: &mut W,
    total_location: &str,
    abort_code: u64,
    errmap_path: &Path,
) {
    let mut location = total_location.trim().split("::");
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

    let errmap_bytes = std::fs::read(errmap_path).expect("Could not load errmap from file");
    let errmap: ErrorMapping =
        bcs::from_bytes(&errmap_bytes).expect("Failed to deserialize errmap");

    match errmap.get_explanation(&module_id, abort_code) {
        None => writeln!(
            w,
            "Unable to find a description for {}::{}",
            total_location, abort_code
        ),
        Some(error_desc) => writeln!(
            w,
            "Category:\n  Name: {}\n  Description: {}\nReason:\n  Name: {}\n  Description: {}",
            error_desc.category.code_name,
            error_desc.category.code_description,
            error_desc.reason.code_name,
            error_desc.reason.code_description,
        ),
    }
    .unwrap();
}
