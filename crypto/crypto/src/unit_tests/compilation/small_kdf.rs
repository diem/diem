// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

fn main() {
    // Test for ripemd160, output_length < 256
    let ripemd = libra_crypto::hkdf::Hkdf::<ripemd160::Ripemd160>::extract(None, &[]);
    assert!(ripemd.is_ok());
}
