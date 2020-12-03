// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::*;

pub const DEPENDENCY: &str = "dependency";
pub const DEPENDENCY_SHORT: &str = "d";

pub const SENDER: &str = "sender";
pub const SENDER_SHORT: &str = "s";

pub const OUT_DIR: &str = "out-dir";
pub const OUT_DIR_SHORT: &str = "o";
pub const DEFAULT_OUTPUT_DIR: &str = "build";

pub const SOURCE_MAP: &str = "source-map";
pub const SOURCE_MAP_SHORT: &str = "m";

pub fn parse_address(s: &str) -> Result<Address, String> {
    Address::parse_str(s).map_err(|msg| format!("Invalid argument to '{}': {}", SENDER, msg))
}

pub const COLOR_MODE_ENV_VAR: &str = "COLOR_MODE";

pub fn read_env_var(v: &str) -> String {
    std::env::var(v)
        .unwrap_or_else(|_| "".into())
        .to_uppercase()
}

pub fn read_bool_env_var(v: &str) -> bool {
    let val = read_env_var(v);
    val == "1" || val == "TRUE"
}
