// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod driver;
use driver::*;

#[test]
fn test3() {
    run_boogie(&generate_boogie("test_mvir/test3.mvir", &["Test3"]));
}

#[test]
fn test_arithmetic() {
    run_boogie(&generate_boogie(
        "test_mvir/test-arithmetic.mvir",
        &["TestArithmetic"],
    ));
}

#[test]
fn test_control_flow() {
    run_boogie(&generate_boogie(
        "test_mvir/test-control-flow.mvir",
        &["TestControlFlow"],
    ));
}

#[test]
fn test_func_call() {
    run_boogie(&generate_boogie(
        "test_mvir/test-func-call.mvir",
        &["TestFuncCall"],
    ));
}

#[test]
fn test_reference() {
    run_boogie(&generate_boogie(
        "test_mvir/test-reference.mvir",
        &["TestReference"],
    ));
}

#[test]
fn test_struct() {
    run_boogie(&generate_boogie(
        "test_mvir/test-struct.mvir",
        &["TestStruct"],
    ));
}

#[test]
fn test_lib() {
    run_boogie(&generate_boogie(
        "test_mvir/test-lib.mvir",
        &[], // empty here means include all deps in output
    ));
}
