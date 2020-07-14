// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use inject_error::inject_error;

#[inject_error(probability = 1.0)]
fn foo() -> anyhow::Result<u64> {
    Ok(1)
}

fn main() {
    foo().unwrap_err();
    println!("Successfully injected error");
}
