// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

// TODO: There are no negative tests at the moment (e.g. deriving NumVariants on a struct or union).
// Add some, possibly using trybuild: https://docs.rs/trybuild/1.0.21/trybuild/

use num_variants::NumVariants;

#[derive(NumVariants)]
enum BasicEnum {
    A,
    B(usize),
    C { foo: String },
}

#[derive(NumVariants)]
enum ZeroEnum {}

#[derive(NumVariants)]
#[num_variants(CUSTOM_NAME)]
enum CustomName {
    Foo,
    Bar,
    Baz,
}

#[test]
fn basic_enum() {
    assert_eq!(BasicEnum::NUM_VARIANTS, 3);
}

#[test]
fn zero_enum() {
    assert_eq!(ZeroEnum::NUM_VARIANTS, 0);
}

#[test]
fn custom_name() {
    assert_eq!(CustomName::CUSTOM_NAME, 3);
}
