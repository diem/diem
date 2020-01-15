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

#[test]
fn verify_ref_param() {
    test(VERIFY, &["test_mvir/verify-ref-param.mvir"]);
}

#[test]
fn verify_libra_coin() {
    test(VERIFY, &[verified_std_mvir("libra_coin").as_str()])
}

#[test]
fn verify_libra_account() {
    test(
        VERIFY,
        &[
            verified_std_mvir("libra_coin").as_str(),
            verified_std_mvir("hash").as_str(),
            verified_std_mvir("u64_util").as_str(),
            verified_std_mvir("address_util").as_str(),
            verified_std_mvir("bytearray_util").as_str(),
            verified_std_mvir("libra_account").as_str(),
        ],
    )
}
