// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Collection of convenience macros for error handling

/// Exits a function early with an `Error`.
///
/// Equivalent to the `bail!` macro, except a error type is provided instead of
/// a message.
#[macro_export]
macro_rules! bail_err {
    ($e:expr) => {
        return Err(From::from($e));
    };
}

/// Terminates the program with a panic that is tagged as being an unrecoverable error.
/// Use this for errors that arise in correct programs due to external factors.
/// For example, if a file that is essential for running cannot be found for some reason.
#[macro_export]
macro_rules! unrecoverable {
    ($fmt:expr, $($arg:tt)+) => {{
        panic!(concat!("unrecoverable: ", stringify!($fmt)), $($arg)+);
    }};
}
