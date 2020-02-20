// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod baseline_test;

// =================================================================================================
// Constants

pub const DEFAULT_SENDER: &str = "0x8675309";

// =================================================================================================
// Env vars

pub fn read_env_var(v: &str) -> String {
    std::env::var(v).unwrap_or_else(|_| "".into())
}

pub fn read_bool_env_var(v: &str) -> bool {
    let val = read_env_var(v);
    val == "1" || val == "true"
}
