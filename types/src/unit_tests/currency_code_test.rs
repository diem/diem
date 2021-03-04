// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::allowed_currency_code_string;
use proptest::prelude::*;

static ALLOWED_CURRENCY_IDENTIFIERS: &str = r"[A-Z][A-Z0-9]*";

#[test]
fn simple_allowed_currency_codes() {
    let acceptable_names = vec!["A", "AB", "ABC", "A1B", "AB1", "A"];
    let unacceptable_names = vec![
        "a", "Ab", "AbC", "A1b", "aB1", "1AB", "1", "", "´t", "©", "AB†", "ƒA", "AB_1", "A_B",
        "_A", "_a", "_A_1", "0",
    ];

    assert!(
        acceptable_names
            .into_iter()
            .all(allowed_currency_code_string),
        "Acceptable currency code name not accepted"
    );
    assert!(
        unacceptable_names
            .into_iter()
            .all(|code| !allowed_currency_code_string(code)),
        "Unacceptable currency code name accepted"
    );
}

proptest! {
    #[test]
    fn acceptable_currency_codes(s in ALLOWED_CURRENCY_IDENTIFIERS) {
        prop_assert!(allowed_currency_code_string(&s))
    }
}
