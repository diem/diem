// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate afl;

use consensus::chained_bft::event_processor::event_processor_fuzzing::{
    fuzz_proposal, generate_corpus_proposal,
};

fn main() {
    fuzz!(|data: &[u8]| {
        fuzz_proposal(data);
    });
}
