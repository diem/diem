// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

pub struct StringError(String);

pub const STD_LIB: &str = "stdlib/modules";
pub const SENDER: &str = "0x8675309";

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl std::fmt::Debug for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl std::error::Error for StringError {
    fn description(&self) -> &str {
        &self.0
    }
}

pub fn stdlib_files() -> Vec<String> {
    let dirfiles = datatest_stable::utils::iterate_directory(Path::new(STD_LIB));
    dirfiles
        .flat_map(|path| {
            if path.extension()?.to_str()? == "move" {
                path.into_os_string().into_string().ok()
            } else {
                None
            }
        })
        .collect()
}

pub fn read_env_var(v: &str) -> String {
    std::env::var(v).unwrap_or_else(|_| "".into())
}

pub fn read_bool_var(v: &str) -> bool {
    let val = read_env_var(v);
    val == "1" || val == "true"
}

pub fn error(s: String) -> datatest_stable::Result<()> {
    Err(Box::new(StringError(s)))
}
