// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![no_main]

use lazy_static::lazy_static;
use libfuzzer_sys::fuzz_target;
use libra_fuzzer::FuzzTarget;
use std::process;

lazy_static! {
    static ref FUZZ_TARGET: FuzzTarget = {
        match FuzzTarget::from_env() {
            Ok(target) => target,
            Err(err) => {
                // lazy_static behaves poorly with panics, so abort here.
                eprintln!("*** [fuzz_runner] Error while determining fuzz target: {}", err);
                process::abort();
            }
        }
    };
}

fuzz_target!(|data: &[u8]| {
    FUZZ_TARGET.fuzz(data);
});
