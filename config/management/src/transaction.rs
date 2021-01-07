// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::constants;
use diem_time_service::{TimeService, TimeServiceTrait};
use diem_types::{
    account_address::AccountAddress,
    chain_id::ChainId,
    transaction::{RawTransaction, Script},
};

/// Builds a `RawTransaction` to handle common transaction values
pub fn build_raw_transaction(
    chain_id: ChainId,
    account: AccountAddress,
    sequence_number: u64,
    script: Script,
) -> RawTransaction {
    RawTransaction::new_script(
        account,
        sequence_number,
        script,
        constants::MAX_GAS_AMOUNT,
        constants::GAS_UNIT_PRICE,
        constants::GAS_CURRENCY_CODE.to_owned(),
        TimeService::real().now().as_secs() + constants::TXN_EXPIRATION_SECS,
        chain_id,
    )
}
