// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{self, Account},
    executor::FakeExecutor,
    keygen::KeyGen,
};
use libra_types::account_config;
use proptest::{collection::vec, prelude::*};
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
        accounts.push((Account::new_libra_root(), 1));
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
    fn fuzz_scripts(
        txns in vec(any::<ScriptCall>(), 0..100),
    ) {
        let mut executor = FakeExecutor::from_genesis_file();
        let mut keygen = KeyGen::from_seed([9u8; 32]);
        let mut accounts = vec![];
        let libra_root = Account::new_libra_root();
        let coins = vec![account::lbr_currency_code(), account::coin1_currency_code(), account::coin2_currency_code()];
        // Create a number of accounts
        for i in 0..10 {
            let account = Account::new();
            let (_, cpubkey) = keygen.generate_keypair();
            executor.execute_and_apply(
                libra_root
                .transaction()
                .script(encode_create_parent_vasp_account_script(
                        account_config::type_tag_for_currency_code(coins[i % coins.len()].clone()),
                        *account.address(),
                        account.auth_key_prefix(),
                        vec![],
                        vec![],
                        cpubkey.to_bytes().to_vec(),
                        i % 2 == 0,
                ))
                .sequence_number(i as u64 + 1)
                .sign(),
            );
            accounts.push((account, 0));
        }
        // Don't include the LR account since txns from that can bork the system
        accounts.push((Account::new_genesis_account(account_config::testnet_dd_account_address()), 0));
        accounts.push((Account::new_blessed_tc(), 0));
        let num_accounts = accounts.len();

        for (i, txn) in txns.into_iter().enumerate() {
            let script = txn.encode();
            let (account, account_sequence_number) = accounts.get_mut(i % num_accounts).unwrap();
            let output = executor.execute_transaction(
                account.transaction()
                .script(script.clone())
                .sequence_number(*account_sequence_number)
                .sign());
                prop_assert!(!output.status().is_discarded());
                executor.apply_write_set(output.write_set());
                *account_sequence_number += 1;
        }
    }
}
