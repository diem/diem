// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod retry;

/// Terminates the program with a panic that is tagged as being an unrecoverable error.
/// Use this for errors that arise in correct programs due to external factors.
/// For example, if a file that is essential for running cannot be found for some reason.
#[macro_export]
macro_rules! unrecoverable {
    ($fmt:expr, $($arg:tt)+) => {{
        panic!(concat!("unrecoverable: ", stringify!($fmt)), $($arg)+);
    }};
}
