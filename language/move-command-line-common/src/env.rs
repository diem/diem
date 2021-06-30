// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub fn read_env_var(v: &str) -> String {
    std::env::var(v).unwrap_or_else(|_| String::new())
}

pub fn read_bool_env_var(v: &str) -> bool {
    let val = read_env_var(v).to_lowercase();
    val.parse::<bool>() == Ok(true) || val.parse::<usize>() == Ok(1)
}
