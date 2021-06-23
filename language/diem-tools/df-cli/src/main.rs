// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::errmap::ErrorMapping;

fn main() -> Result<()> {
    let error_descriptions: ErrorMapping =
        bcs::from_bytes(diem_framework_releases::current_error_descriptions())?;
    move_cli::move_cli(diem_vm::natives::diem_natives(), &error_descriptions)
}
