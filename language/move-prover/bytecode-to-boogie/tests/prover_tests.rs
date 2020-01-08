// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod driver;
use driver::*;

#[test]
fn verify_create_resource() {
    test(VERIFY, &["test_mvir/verify-create-resource.mvir"]);
}

#[test]
fn verify_div() {
    test(VERIFY, &["test_mvir/verify-div.mvir"]);
}

#[test]
fn verify_local_ref() {
    test(VERIFY, &["test_mvir/verify-local-ref.mvir"]);
}
