// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate afl;

use canonical_serialization::{SimpleDeserializer, SimpleSerializer};
use failure::prelude::Result;
use types::transaction::SignedTransaction;

fn main() {
    fuzz!(|data: &[u8]| {
        let _: Result<SignedTransaction> = SimpleDeserializer::deserialize(&data);
    });
}
