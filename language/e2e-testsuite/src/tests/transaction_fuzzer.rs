// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use compiled_stdlib::transaction_scripts::StdlibScript;
use diem_types::account_config;
use language_e2e_tests::{
    account::{self, Account},
    executor::FakeExecutor,
};
use proptest::{collection::vec, prelude::*};
use std::convert::TryFrom;
use transaction_builder::encode_create_parent_vasp_account_script;
use transaction_builder_generated::stdlib::ScriptCall;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn fuzz_scripts_genesis_state(
        txns in vec(any::<ScriptCall>(), 0..100),
    ) {
        let executor = FakeExecutor::from_genesis_file();
        let mut accounts = vec![];
        accounts.push((Account::new_diem_root(), 1));
        accounts.push((Account::new_blessed_tc(), 0));
        let num_accounts = accounts.len();

        for (i, txn) in txns.into_iter().enumerate() {
            let script = txn.encode();
            let (account, account_sequence_number) = &accounts[i % num_accounts];
            let output = executor.execute_transaction(
                account.transaction()
                .script(script.clone())
                .sequence_number(*account_sequence_number)
                .sign());
                prop_assert!(!output.status().is_discarded());
        }
    }

    #[test]
    #[ignore]
    fn fuzz_scripts(
        txns in vec(any::<ScriptCall>(), 0..100),
    ) {
        let mut executor = FakeExecutor::from_genesis_file();
        let mut accounts = vec![];
        let diem_root = Account::new_diem_root();
        let coins = vec![account::xus_currency_code()];
        // Create a number of accounts
        for i in 0..10 {
            let account = executor.create_raw_account();
            executor.execute_and_apply(
                diem_root
                .transaction()
                .script(encode_create_parent_vasp_account_script(
                        account_config::type_tag_for_currency_code(coins[i % coins.len()].clone()),
                        0,
                        *account.address(),
                        account.auth_key_prefix(),
                        vec![],
                        i % 2 == 0,
                ))
                .sequence_number(i as u64 + 1)
                .sign(),
            );
            accounts.push((account, 0));
        }
        // Don't include the DR account since txns from that can bork the system
        accounts.push((Account::new_genesis_account(account_config::testnet_dd_account_address()), 0));
        accounts.push((Account::new_blessed_tc(), 0));
        let num_accounts = accounts.len();

        for (i, txn) in txns.into_iter().enumerate() {
            let script = txn.encode();
            let (account, account_sequence_number) = accounts.get_mut(i % num_accounts).unwrap();
            let script_is_rotate = StdlibScript::try_from(script.code()).map(|script|
                script == StdlibScript::RotateAuthenticationKey ||
                script == StdlibScript::RotateAuthenticationKeyWithNonce ||
                script == StdlibScript::RotateAuthenticationKeyWithRecoveryAddress
            ).unwrap_or(false);
            let output = executor.execute_transaction(
                account.transaction()
                .script(script.clone())
                .sequence_number(*account_sequence_number)
                .sign());
                prop_assert!(!output.status().is_discarded());
                // Don't apply key rotation transactions since that will bork future txns
                if !script_is_rotate {
                    executor.apply_write_set(output.write_set());
                    *account_sequence_number += 1;
                }
        }
    }
}
