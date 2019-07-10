// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod create_account;
mod peer_to_peer;
mod rotate_key;

use crate::{
    account::AccountResource,
    account_universe::{
        log_balance_strategy, num_accounts, num_transactions, AUTransactionGen, AccountCurrent,
        AccountPairGen, AccountUniverse, AccountUniverseGen, RotateKeyGen,
    },
    executor::FakeExecutor,
};
use proptest::{collection::vec, prelude::*};

proptest! {
    // These tests are pretty slow but quite comprehensive, so run a smaller number of them.
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Ensure that account pair generators return the correct indexes.
    #[test]
    fn account_pair_gen(
        universe in AccountUniverseGen::strategy(2..num_accounts(), 0u64..10000),
        pairs in vec(any::<AccountPairGen>(), 0..num_transactions()),
    ) {
        let mut executor = FakeExecutor::from_genesis_file();
        let mut universe = universe.setup(&mut executor);

        for pair in pairs {
            let (idx_1, idx_2, account_1, account_2) = {
                let pick = pair.pick(&universe);
                prop_assert_eq!(pick.account_1, &universe.accounts()[pick.idx_1]);
                prop_assert_eq!(pick.account_2, &universe.accounts()[pick.idx_2]);
                (
                    pick.idx_1,
                    pick.idx_2,
                    // Need to convert to raw pointers to avoid holding an immutable reference
                    // (pick_mut below borrows universe mutably, which would conflict.)
                    // This is safe as all we're doing is comparing pointer equality.
                    pick.account_1 as *const AccountCurrent,
                    pick.account_2 as *const AccountCurrent,
                )
            };

            let pick_mut = pair.pick_mut(&mut universe);
            prop_assert_eq!(pick_mut.idx_1, idx_1);
            prop_assert_eq!(pick_mut.idx_2, idx_2);
            prop_assert_eq!(pick_mut.account_1 as *const AccountCurrent, account_1);
            prop_assert_eq!(pick_mut.account_2 as *const AccountCurrent, account_2);
        }
    }

    #[test]
    fn all_transactions(
        universe in AccountUniverseGen::strategy(2..num_accounts(), log_balance_strategy(10_000_000)),
        transactions in vec(all_transactions_strategy(1, 1_000_000), 0..num_transactions()),
    ) {
        run_and_assert_universe(universe, transactions)?;
    }
}

/// A strategy that returns a random transaction.
fn all_transactions_strategy(
    min: u64,
    max: u64,
) -> impl Strategy<Value = Box<dyn AUTransactionGen + 'static>> {
    prop_oneof![
        // Most transactions should be p2p payments.
        8 => peer_to_peer::p2p_strategy(min, max),
        1 => create_account::create_account_strategy(min, max),
        1 => any::<RotateKeyGen>().prop_map(RotateKeyGen::boxed),
    ]
}

/// Run these transactions and make sure that they all cost the same amount of gas.
pub(crate) fn run_and_assert_gas_cost_stability(
    universe: AccountUniverseGen,
    transaction_gens: Vec<impl AUTransactionGen>,
    gas_cost: u64,
) -> Result<(), TestCaseError> {
    let mut executor = FakeExecutor::from_genesis_file();
    let mut universe = universe.setup_gas_cost_stability(&mut executor);
    let (transactions, expected_statuses): (Vec<_>, Vec<_>) = transaction_gens
        .into_iter()
        .map(|transaction_gen| transaction_gen.apply(&mut universe))
        .unzip();
    let outputs = executor.execute_block(transactions);

    for (idx, (output, expected)) in outputs.iter().zip(&expected_statuses).enumerate() {
        prop_assert_eq!(
            output.status(),
            expected,
            "unexpected status for transaction {}",
            idx
        );
        prop_assert_eq!(
            output.gas_used(),
            gas_cost,
            "transaction at idx {} did not have expected gas cost",
            idx,
        );
    }

    Ok(())
}

/// Run these transactions and verify the expected output.
pub(crate) fn run_and_assert_universe(
    universe: AccountUniverseGen,
    transaction_gens: Vec<impl AUTransactionGen>,
) -> Result<(), TestCaseError> {
    let mut executor = FakeExecutor::from_genesis_file();
    let mut universe = universe.setup(&mut executor);
    let (transactions, expected_statuses): (Vec<_>, Vec<_>) = transaction_gens
        .into_iter()
        .map(|transaction_gen| transaction_gen.apply(&mut universe))
        .unzip();
    let outputs = executor.execute_block(transactions);

    prop_assert_eq!(outputs.len(), expected_statuses.len());

    for (idx, (output, expected)) in outputs.iter().zip(&expected_statuses).enumerate() {
        prop_assert_eq!(
            output.status(),
            expected,
            "unexpected status for transaction {}",
            idx
        );
        executor.apply_write_set(output.write_set());
    }

    assert_accounts_match(&universe, &executor)?;
    Ok(())
}

/// Verify that the account information in the universe matches the information in the executor.
pub(crate) fn assert_accounts_match(
    universe: &AccountUniverse,
    executor: &FakeExecutor,
) -> Result<(), TestCaseError> {
    for (idx, account) in universe.accounts().iter().enumerate() {
        let resource = executor
            .read_account_resource(&account.account())
            .expect("resource for this account must exist");
        prop_assert_eq!(
            account.account().auth_key(),
            AccountResource::read_auth_key(&resource),
            "account {} should have correct auth key",
            idx
        );
        prop_assert_eq!(
            account.balance(),
            AccountResource::read_balance(&resource),
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
            AccountResource::read_sequence_number(&resource),
            "account {} should have correct sequence number",
            idx
        );
    }
    Ok(())
}
