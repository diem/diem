// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![no_main]

use libfuzzer_sys::fuzz_target;
use libra_fuzzer::FuzzTarget;
use once_cell::sync::Lazy;
use std::process;

static FUZZ_TARGET: Lazy<FuzzTarget> = Lazy::new(|| {
    match FuzzTarget::from_env() {
        Ok(target) => target,
        Err(err) => {
            // Lazy behaves poorly with panics, so abort here.
            eprintln!(
                "*** [fuzz_runner] Error while determining fuzz target: {}",
                err
            );
            process::abort();
        }
    }
});

fuzz_target!(|data: &[u8]| {
    FUZZ_TARGET.fuzz(data);
});
