// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use itertools::Itertools;
use prover_mutation::mutator;

fn main() {
    let args = std::env::args().collect_vec();
    mutator::mutate(&args[1..]);
}
