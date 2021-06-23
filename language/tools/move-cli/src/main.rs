// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{account_address::AccountAddress, errmap::ErrorMapping};

fn main() -> Result<()> {
    let error_descriptions: ErrorMapping = bcs::from_bytes(move_stdlib::error_descriptions())?;
    move_cli::move_cli(
        move_stdlib::natives::all_natives(AccountAddress::from_hex_literal("0x1").unwrap()),
        &error_descriptions,
    )
}
