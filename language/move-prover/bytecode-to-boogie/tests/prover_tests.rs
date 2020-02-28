// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod driver;
use driver::*;

#[test]
fn verify_addition() {
    test(VERIFY, &["test_mvir/verify-addition.mvir"]);
}

#[test]
fn verify_cast() {
    test(VERIFY, &["test_mvir/verify-cast.mvir"]);
}

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
fn verify_multiplication() {
    test(VERIFY, &["test_mvir/verify-multiplication.mvir"]);
}

#[test]
fn verify_ref_param() {
    test(VERIFY, &["test_mvir/verify-ref-param.mvir"]);
}

#[test]
fn test_aborts_if() {
    test(VERIFY, &["test_mvir/test-aborts-if.mvir"]);
}

#[test]
fn verify_vector() {
    test(
        VERIFY,
        &[
            "test_mvir/verify-stdlib/vector.mvir",
            "test_mvir/verify-vector.mvir",
        ],
    );
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

#[test]
fn verify_invariants() {
    test(VERIFY, &["test_mvir/verify-invariants.mvir"]);
}

#[test]
fn verify_synthetics() {
    test(VERIFY, &["test_mvir/verify-synthetics.mvir"]);
}

#[test]
fn verify_lifetime() {
    test(VERIFY, &["test_mvir/verify-lifetime.mvir"]);
}
