// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{unit_tests::TestInterface, LibraDebugger};
use anyhow::bail;

#[test]
fn test_bisection() {
    let debugger = LibraDebugger::new(Box::new(TestInterface::empty(100)));
    let check = |v: Vec<bool>, result| {
        assert_eq!(
            debugger
                .bisect_transaction_impl(
                    |version| {
                        if v[version as usize] {
                            Ok(())
                        } else {
                            bail!("Err")
                        }
                    },
                    0,
                    v.len() as u64
                )
                .unwrap(),
            result
        );
    };
    check(vec![true, true, true, true], None);
    check(vec![true, true, true, false], Some(3));
    check(vec![true, true, false, false], Some(2));
    check(vec![false, false, false, false], Some(0));
}
