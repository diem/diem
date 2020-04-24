// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::*;

pub const SOURCE_FILES: &str = "source-files";
pub const SOURCE_FILES_SHORT: &str = "f";

pub const DEPENDENCIES: &str = "dependencies";
pub const DEPENDENCIES_SHORT: &str = "d";

pub const SENDER: &str = "sender";
pub const SENDER_SHORT: &str = "s";

pub const OUT_DIR: &str = "out-dir";
pub const OUT_DIR_SHORT: &str = "o";
pub const DEFAULT_OUTPUT_DIR: &str = "output";

pub const SOURCE_MAP: &str = "source-map";
pub const SOURCE_MAP_SHORT: &str = "m";

pub fn parse_address(s: &str) -> Result<Address, String> {
    Address::parse_str(s).map_err(|msg| format!("Invalid argument to '{}': {}", SENDER, msg))
}
