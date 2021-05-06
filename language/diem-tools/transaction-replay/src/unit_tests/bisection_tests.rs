// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{unit_tests::TestInterface, DiemDebugger};
use anyhow::bail;
use diem_types::{
    account_address::AccountAddress,
    account_config::AccountResource,
    event::{EventHandle, EventKey},
};
use move_core_types::{effects::ChangeSet, move_resource::MoveStructType};
use std::path::PathBuf;

#[test]
fn test_bisection() {
    let debugger = DiemDebugger::new(Box::new(TestInterface::empty(100)));
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

#[test]
fn test_changeset_override() {
    let debugger = DiemDebugger::new(Box::new(TestInterface::genesis()));
    let address = AccountAddress::random();
    let mut override_changeset = ChangeSet::new();
    override_changeset
        .publish_resource(
            address,
            AccountResource::struct_tag(),
            bcs::to_bytes(&AccountResource::new(
                0,
                vec![],
                None,
                None,
                EventHandle::new(EventKey::new_from_address(&address, 0), 0),
                EventHandle::new(EventKey::new_from_address(&address, 1), 1),
            ))
            .unwrap(),
        )
        .unwrap();

    let mut script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    script_path.push("examples/account_exists.move");

    assert_eq!(
        None,
        debugger
            .bisect_transactions_by_script(script_path.to_str().unwrap(), address, 1, 2, None)
            .unwrap()
    );
    assert_eq!(
        Some(1),
        debugger
            .bisect_transactions_by_script(
                script_path.to_str().unwrap(),
                address,
                1,
                2,
                Some(override_changeset)
            )
            .unwrap()
    );
}
