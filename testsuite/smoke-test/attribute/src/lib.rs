// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;

/// This macro allows tests to be defined using the "smoke_test" attribute.
/// What this does is declare each test as a "rusty_fork_test" (so that it's
/// spawned in a separate process) and sets a test timeout of 2 minutes (so that
/// if the test takes longer than 2 minutes to run, it will automatically be
/// killed and marked as a test failure).
#[proc_macro_attribute]
pub fn smoke_test(_: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let output = quote! {
        ::rusty_fork::rusty_fork_test! {
            #![rusty_fork(timeout_ms = 120000)] // 2 minute timeout
            #[test]
            #input
        }
    };
    TokenStream::from(output)
}
