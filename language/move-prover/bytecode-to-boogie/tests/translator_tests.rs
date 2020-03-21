// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod driver;
use driver::*;

#[test]
fn test3() {
    test(NO_VERIFY, &["test_mvir/test3.mvir"]);
}

#[test]
fn test_arithmetic() {
    test(NO_VERIFY, &["test_mvir/test-arithmetic.mvir"]);
}

#[test]
fn test_control_flow() {
    test(NO_VERIFY, &["test_mvir/test-control-flow.mvir"]);
}

#[test]
fn test_func_call() {
    test(NO_VERIFY, &["test_mvir/test-func-call.mvir"]);
}

#[test]
fn test_reference() {
    test(NO_VERIFY, &["test_mvir/test-reference.mvir"]);
}

#[test]
fn test_struct() {
    test(NO_VERIFY, &["test_mvir/test-struct.mvir"]);
}

#[test]
fn test_generics() {
    test(
        NO_VERIFY,
        &[&verified_std_mvir("vector"), "test_mvir/test-generics.mvir"],
    );
}

#[test]
fn test_specs_translate() {
    test(NO_VERIFY, &["test_mvir/test-specs-translate.mvir"]);
}
