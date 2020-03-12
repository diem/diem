// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod create_account;
mod peer_to_peer;
mod rotate_key;

use crate::{
    account_universe::{
        default_num_accounts, default_num_transactions, log_balance_strategy, AUTransactionGen,
        AccountCurrent, AccountPairGen, AccountPickStyle, AccountUniverse, AccountUniverseGen,
        RotateKeyGen,
    },
    executor::FakeExecutor,
    transaction_status_eq,
};
use proptest::{collection::vec, prelude::*};
use std::sync::Arc;

proptest! {
    // These tests are pretty slow but quite comprehensive, so run a smaller number of them.
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Ensure that account pair generators return the correct indexes.
    #[test]
    fn account_pair_gen(
        universe in AccountUniverseGen::strategy(2..default_num_accounts(), 0u64..10000),
        pairs in vec(any::<AccountPairGen>(), 0..default_num_transactions()),
    ) {
        let mut executor = FakeExecutor::from_genesis_file();
        let mut universe = universe.setup(&mut executor);

        for pair in pairs {
            let (idx_1, idx_2, account_1, account_2) = {
                let pick = pair.pick(&mut universe);
                (
                    pick.idx_1,
                    pick.idx_2,
                    // Need to convert to raw pointers to avoid holding a mutable reference
                    // (pick_mut below borrows universe mutably, which would conflict.)
                    // This is safe as all we're doing is comparing pointer equality.
                    pick.account_1 as *const AccountCurrent,
                    pick.account_2 as *const AccountCurrent,
                )
            };

            prop_assert_eq!(account_1, &universe.accounts()[idx_1] as *const AccountCurrent);
            prop_assert_eq!(account_2, &universe.accounts()[idx_2] as *const AccountCurrent);
        }
    }

    #[test]
    fn all_transactions(
        universe in AccountUniverseGen::strategy(
            2..default_num_accounts(),
            log_balance_strategy(10_000_000),
        ),
        transactions in vec(all_transactions_strategy(1, 1_000_000), 0..default_num_transactions()),
    ) {
        run_and_assert_universe(universe, transactions)?;
    }

    #[test]
    fn all_transactions_limited(
        mut universe in AccountUniverseGen::strategy(
            4..default_num_accounts(),
            log_balance_strategy(10_000_000),
        ),
        mut transactions in vec(
            all_transactions_strategy(1, 1_000_000),
            0..default_num_transactions(),
        ),
    ) {
        universe.set_pick_style(AccountPickStyle::Limited(4));
        // Each transaction consumes up to 2 slots, and there are (4 * universe.num_accounts())
        // slots. Use only 3/4 of the slots to allow for some tolerance against edge cases. So
        // the maximum number of transactions is (3 * universe.num_accounts()) / 2.
        let max_transactions = (3 * universe.num_accounts()) / 2;
        if transactions.len() >= max_transactions {
            transactions.drain(max_transactions..);
        }

        run_and_assert_universe(universe, transactions)?;
    }
}

/// A strategy that returns a random transaction.
fn all_transactions_strategy(
    min: u64,
    max: u64,
) -> impl Strategy<Value = Arc<dyn AUTransactionGen + 'static>> {
    prop_oneof![
        // Most transactions should be p2p payments.
        8 => peer_to_peer::p2p_strategy(min, max),
        1 => create_account::create_account_strategy(min, max),
        1 => any::<RotateKeyGen>().prop_map(RotateKeyGen::arced),
    ]
}

/// Run these transactions and make sure that they all cost the same amount of gas.
pub(crate) fn run_and_assert_gas_cost_stability(
    universe: AccountUniverseGen,
    transaction_gens: Vec<impl AUTransactionGen + Clone>,
) -> Result<(), TestCaseError> {
    let mut executor = FakeExecutor::from_genesis_file();
    let mut universe = universe.setup_gas_cost_stability(&mut executor);
    let (transactions, expected_values): (Vec<_>, Vec<_>) = transaction_gens
        .iter()
        .map(|transaction_gen| transaction_gen.clone().apply(&mut universe))
        .unzip();
    let outputs = executor.execute_block(transactions).unwrap();

    for (idx, (output, expected_value)) in outputs.iter().zip(&expected_values).enumerate() {
        prop_assert!(
            transaction_status_eq(output.status(), &expected_value.0),
            "unexpected status for transaction {}",
            idx
        );
        prop_assert_eq!(
            output.gas_used(),
            expected_value.1,
            "transaction at idx {} did not have expected gas cost",
            idx,
        );
    }
    Ok(())
}

/// Run these transactions and verify the expected output.
pub(crate) fn run_and_assert_universe(
    universe: AccountUniverseGen,
    transaction_gens: Vec<impl AUTransactionGen + Clone>,
) -> Result<(), TestCaseError> {
    let mut executor = FakeExecutor::from_genesis_file();
    let mut universe = universe.setup(&mut executor);
    let (transactions, expected_values): (Vec<_>, Vec<_>) = transaction_gens
        .iter()
        .map(|transaction_gen| transaction_gen.clone().apply(&mut universe))
        .unzip();
    let outputs = executor.execute_block(transactions).unwrap();

    prop_assert_eq!(outputs.len(), expected_values.len());

    for (idx, (output, expected)) in outputs.iter().zip(&expected_values).enumerate() {
        prop_assert!(
            transaction_status_eq(output.status(), &expected.0),
            "unexpected status for transaction {}",
            idx
        );
        executor.apply_write_set(output.write_set());
    }

    assert_accounts_match(&universe, &executor)
}

/// Verify that the account information in the universe matches the information in the executor.
pub(crate) fn assert_accounts_match(
    universe: &AccountUniverse,
    executor: &FakeExecutor,
) -> Result<(), TestCaseError> {
    for (idx, account) in universe.accounts().iter().enumerate() {
        let (resource, resource_balance) = executor
            .read_account_info(&account.account())
            .expect("resource for this account must exist");
        let auth_key = account.account().auth_key();
        prop_assert_eq!(
            auth_key.as_slice(),
            resource.authentication_key(),
            "account {} should have correct auth key",
            idx
        );
        prop_assert_eq!(
            account.balance(),
            resource_balance.coin(),
            "account {} should have correct balance",
            idx
        );
        // XXX These two don't work at the moment because the VM doesn't bump up event counts.
        //        prop_assert_eq!(
        //            account.received_events_count(),
        //            AccountResource::read_received_events_count(&resource),
        //            "account {} should have correct received_events_count",
        //            idx
        //        );
        //        prop_assert_eq!(
        //            account.sent_events_count(),
        //            AccountResource::read_sent_events_count(&resource),
        //            "account {} should have correct sent_events_count",
        //            idx
        //        );
        prop_assert_eq!(
            account.sequence_number(),
            resource.sequence_number(),
            "account {} should have correct sequence number",
            idx
        );
    }
    Ok(())
}
