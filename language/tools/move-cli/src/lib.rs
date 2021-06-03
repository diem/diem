// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod base;
pub mod experimental;
pub mod sandbox;

/// Default directory where saved Move resources live
pub const DEFAULT_STORAGE_DIR: &str = "storage";

/// Default directory where Move modules live
pub const DEFAULT_SOURCE_DIR: &str = "src";

/// Default directory where Move packages live under build_dir
pub const DEFAULT_PACKAGE_DIR: &str = "package";

/// Default dependency inclusion mode
pub const DEFAULT_DEP_MODE: &str = "stdlib";

/// Default directory for build output
pub use move_lang::command_line::DEFAULT_OUTPUT_DIR as DEFAULT_BUILD_DIR;

/// Extension for resource and event files, which are in BCS format
const BCS_EXTENSION: &str = "bcs";
