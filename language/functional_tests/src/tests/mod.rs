// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod checker_tests;
mod global_config_tests;
mod transaction_config_tests;
mod utils_tests;

use crate::errors::*;
use std::str::FromStr;

/// Parses each line in the given input as `T`.
pub fn parse_each_line_as<T>(s: &str) -> Result<Vec<T>>
where
    T: FromStr<Err = Error>,
{
    s.lines()
        .map(|s| s.trim_start().trim_end())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<T>())
        .collect()
}
