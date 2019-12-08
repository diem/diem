// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod transaction_test_helpers;

pub fn assert_canonical_encode_decode<T>(t: T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + PartialEq,
{
    let bytes = lcs::to_bytes(&t).unwrap();
    let s: T = lcs::from_bytes(&bytes).unwrap();
    assert_eq!(t, s);
}
