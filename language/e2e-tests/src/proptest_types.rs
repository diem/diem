// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account::{lbr_currency_code, Account, AccountData};
use proptest::prelude::*;

impl Arbitrary for Account {
    type Parameters = ();
    fn arbitrary_with(_params: ()) -> Self::Strategy {
        // Provide Account::new as the canonical strategy. This means that no shrinking will happen,
        // but that's fine as accounts have nothing to shrink inside them anyway.
        Account::new as Self::Strategy
    }

    type Strategy = fn() -> Account;
}

impl AccountData {
    /// Returns a [`Strategy`] that creates `AccountData` instances.
    pub fn strategy(balance_strategy: impl Strategy<Value = u64>) -> impl Strategy<Value = Self> {
        // Pick sequence numbers and event counts in a smaller range so that valid transactions can
        // be generated.
        // XXX should we also test edge cases around large sequence numbers?
        let sequence_strategy = 0u64..(1 << 32);
        let event_count_strategy = 0u64..(1 << 32);

        (
            any::<Account>(),
            balance_strategy,
            sequence_strategy,
            event_count_strategy.clone(),
            event_count_strategy,
        )
            .prop_map(
                |(account, balance, sequence_number, sent_events_count, received_events_count)| {
                    AccountData::with_account_and_event_counts(
                        account,
                        balance,
                        lbr_currency_code(), // TODO: Vary account balance currency?
                        sequence_number,
                        sent_events_count,
                        received_events_count,
                        false, // TODO: vary withdrawal capability param?
                        false, // TODO: vary rotation capability param?
                        false, // TODO: Vary account type?
                        false,
                    )
                },
            )
    }
}
