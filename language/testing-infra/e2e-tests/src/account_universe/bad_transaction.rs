// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_universe::{AUTransactionGen, AccountUniverse},
    common_transactions::{empty_txn, EMPTY_SCRIPT},
    gas_costs,
};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use diem_proptest_helpers::Index;
use diem_types::{
    account_config::XUS_NAME,
    transaction::{Script, SignedTransaction, TransactionStatus},
    vm_status::StatusCode,
};
use move_core_types::gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier, GasConstants};
use move_vm_types::gas_schedule::calculate_intrinsic_gas;
use proptest::prelude::*;
use proptest_derive::Arbitrary;
use std::sync::Arc;

/// Represents a sequence number mismatch transaction
///
#[derive(Arbitrary, Clone, Debug)]
#[proptest(params = "(u64, u64)")]
pub struct SequenceNumberMismatchGen {
    sender: Index,
    #[proptest(strategy = "params.0 ..= params.1")]
    seq: u64,
}

impl AUTransactionGen for SequenceNumberMismatchGen {
    fn apply(
        &self,
        universe: &mut AccountUniverse,
    ) -> (SignedTransaction, (TransactionStatus, u64)) {
        let sender = universe.pick(self.sender).1;

        let seq = if sender.sequence_number == self.seq {
            self.seq + 1
        } else {
            self.seq
        };

        let txn = empty_txn(
            sender.account(),
            seq,
            gas_costs::TXN_RESERVED,
            0,
            XUS_NAME.to_string(),
        );

        (
            txn,
            (
                if seq >= sender.sequence_number {
                    TransactionStatus::Discard(StatusCode::SEQUENCE_NUMBER_TOO_NEW)
                } else {
                    TransactionStatus::Discard(StatusCode::SEQUENCE_NUMBER_TOO_OLD)
                },
                0,
            ),
        )
    }
}

/// Represents a insufficient balance transaction
///
#[derive(Arbitrary, Clone, Debug)]
#[proptest(params = "(u64, u64)")]
pub struct InsufficientBalanceGen {
    sender: Index,
    #[proptest(strategy = "params.0 ..= params.1")]
    gas_unit_price: u64,
}

impl AUTransactionGen for InsufficientBalanceGen {
    fn apply(
        &self,
        universe: &mut AccountUniverse,
    ) -> (SignedTransaction, (TransactionStatus, u64)) {
        let sender = universe.pick(self.sender).1;

        let max_gas_unit = (sender.balance / self.gas_unit_price) + 1;

        let txn = empty_txn(
            sender.account(),
            sender.sequence_number,
            max_gas_unit,
            self.gas_unit_price,
            XUS_NAME.to_string(),
        );

        // TODO: Move such config to AccountUniverse
        let default_constants = GasConstants::default();
        let raw_bytes_len = AbstractMemorySize::new(txn.raw_txn_bytes_len() as GasCarrier);
        let min_cost = GasConstants::default()
            .to_external_units(calculate_intrinsic_gas(
                raw_bytes_len,
                &GasConstants::default(),
            ))
            .get();

        (
            txn,
            (
                if max_gas_unit > default_constants.maximum_number_of_gas_units.get() {
                    TransactionStatus::Discard(
                        StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND,
                    )
                } else if max_gas_unit < min_cost {
                    TransactionStatus::Discard(
                        StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS,
                    )
                } else if self.gas_unit_price > default_constants.max_price_per_gas_unit.get() {
                    TransactionStatus::Discard(StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND)
                } else if self.gas_unit_price < default_constants.min_price_per_gas_unit.get() {
                    TransactionStatus::Discard(StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND)
                } else {
                    TransactionStatus::Discard(StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE)
                },
                0,
            ),
        )
    }
}

/// Represents a authkey mismatch transaction
///
#[derive(Arbitrary, Clone, Debug)]
#[proptest(no_params)]
pub struct InvalidAuthkeyGen {
    sender: Index,
    #[proptest(
        strategy = "diem_crypto::test_utils::uniform_keypair_strategy_with_perturbation(1)"
    )]
    new_keypair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
}

impl AUTransactionGen for InvalidAuthkeyGen {
    fn apply(
        &self,
        universe: &mut AccountUniverse,
    ) -> (SignedTransaction, (TransactionStatus, u64)) {
        let sender = universe.pick(self.sender).1;

        let txn = sender
            .account()
            .transaction()
            .script(Script::new(EMPTY_SCRIPT.clone(), vec![], vec![]))
            .sequence_number(sender.sequence_number)
            .raw()
            .sign(
                &self.new_keypair.private_key,
                self.new_keypair.public_key.clone(),
            )
            .unwrap()
            .into_inner();

        (
            txn,
            (TransactionStatus::Discard(StatusCode::INVALID_AUTH_KEY), 0),
        )
    }
}

pub fn bad_txn_strategy() -> impl Strategy<Value = Arc<dyn AUTransactionGen + 'static>> {
    prop_oneof![
        1 => any_with::<SequenceNumberMismatchGen>((0, 10_000)).prop_map(SequenceNumberMismatchGen::arced),
        1 => any_with::<InvalidAuthkeyGen>(()).prop_map(InvalidAuthkeyGen::arced),
        1 => any_with::<InsufficientBalanceGen>((1, 20_000)).prop_map(InsufficientBalanceGen::arced),
    ]
}
