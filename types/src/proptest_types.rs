// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    byte_array::ByteArray,
    contract_event::ContractEvent,
    get_with_proof::{ResponseItem, UpdateToLatestLedgerResponse},
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    proof::AccumulatorProof,
    transaction::{
        Program, RawTransaction, SignatureCheckedTransaction, SignedTransaction,
        TransactionArgument, TransactionInfo, TransactionListWithProof, TransactionPayload,
        TransactionStatus, TransactionToCommit, Version,
    },
    validator_change::ValidatorChangeEventWithProof,
    vm_error::VMStatus,
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use crypto::{
    hash::CryptoHash,
    signing::{sign_message, PrivateKey as OldPrivateKey, PublicKey as OldPublicKey},
    utils::{keypair_strategy as gen_keypair_strategy, keypair_strategy},
    HashValue, Signature,
};
use proptest::{
    collection::{hash_map, hash_set, vec, SizeRange},
    option,
    prelude::*,
    strategy::Union,
};
use std::{collections::HashMap, time::Duration};

prop_compose! {
    #[inline]
    pub fn arb_byte_array()(byte_array in vec(any::<u8>(), 1..=10)) -> ByteArray {
        ByteArray::new(byte_array)
    }
}

impl Arbitrary for ByteArray {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    #[inline]
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        arb_byte_array().boxed()
    }
}

impl WriteOp {
    pub fn value_strategy() -> impl Strategy<Value = Self> {
        vec(any::<u8>(), 0..64).prop_map(WriteOp::Value)
    }

    pub fn deletion_strategy() -> impl Strategy<Value = Self> {
        Just(WriteOp::Deletion)
    }
}

impl Arbitrary for WriteOp {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        prop_oneof![Self::deletion_strategy(), Self::value_strategy()].boxed()
    }
}

impl WriteSet {
    fn genesis_strategy() -> impl Strategy<Value = Self> {
        vec((any::<AccessPath>(), WriteOp::value_strategy()), 0..64).prop_map(|write_set| {
            let write_set_mut = WriteSetMut::new(write_set);
            write_set_mut
                .freeze()
                .expect("generated write sets should always be valid")
        })
    }
}

impl Arbitrary for WriteSet {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        // XXX there's no checking for repeated access paths here, nor in write_set. Is that
        // important? Not sure.
        vec((any::<AccessPath>(), any::<WriteOp>()), 0..64)
            .prop_map(|write_set| {
                let write_set_mut = WriteSetMut::new(write_set);
                write_set_mut
                    .freeze()
                    .expect("generated write sets should always be valid")
            })
            .boxed()
    }
}

impl RawTransaction {
    fn strategy_impl(
        address_strategy: impl Strategy<Value = AccountAddress>,
        payload_strategy: impl Strategy<Value = TransactionPayload>,
    ) -> impl Strategy<Value = Self> {
        // XXX what other constraints do these need to obey?
        (
            address_strategy,
            any::<u64>(),
            payload_strategy,
            any::<u64>(),
            any::<u64>(),
            any::<u64>(),
        )
            .prop_map(
                |(
                    sender,
                    sequence_number,
                    payload,
                    max_gas_amount,
                    gas_unit_price,
                    expiration_time_secs,
                )| {
                    match payload {
                        TransactionPayload::Program(program) => RawTransaction::new(
                            sender,
                            sequence_number,
                            program,
                            max_gas_amount,
                            gas_unit_price,
                            Duration::from_secs(expiration_time_secs),
                        ),
                        TransactionPayload::WriteSet(write_set) => {
                            // It's a bit unfortunate that max_gas_amount etc is generated but
                            // not used, but it isn't a huge deal.
                            RawTransaction::new_write_set(sender, sequence_number, write_set)
                        }
                    }
                },
            )
    }
}

impl Arbitrary for RawTransaction {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        Self::strategy_impl(any::<AccountAddress>(), any::<TransactionPayload>()).boxed()
    }
}

impl SignatureCheckedTransaction {
    // This isn't an Arbitrary impl because this doesn't generate *any* possible SignedTransaction,
    // just one kind of them.
    pub fn program_strategy(
        keypair_strategy: impl Strategy<Value = (OldPrivateKey, OldPublicKey)>,
    ) -> impl Strategy<Value = Self> {
        Self::strategy_impl(keypair_strategy, TransactionPayload::program_strategy())
    }

    pub fn write_set_stratedy(
        keypair_strategy: impl Strategy<Value = (OldPrivateKey, OldPublicKey)>,
    ) -> impl Strategy<Value = Self> {
        Self::strategy_impl(keypair_strategy, TransactionPayload::write_set_strategy())
    }

    pub fn genesis_stratedy(
        keypair_strategy: impl Strategy<Value = (OldPrivateKey, OldPublicKey)>,
    ) -> impl Strategy<Value = Self> {
        Self::strategy_impl(keypair_strategy, TransactionPayload::genesis_strategy())
    }

    fn strategy_impl(
        keypair_strategy: impl Strategy<Value = (OldPrivateKey, OldPublicKey)>,
        payload_strategy: impl Strategy<Value = TransactionPayload>,
    ) -> impl Strategy<Value = Self> {
        (keypair_strategy, payload_strategy)
            .prop_flat_map(|(keypair, payload)| {
                let address = AccountAddress::from(keypair.1);
                (
                    Just(keypair),
                    RawTransaction::strategy_impl(Just(address), Just(payload)),
                )
            })
            .prop_map(|((private_key, public_key), raw_txn)| {
                raw_txn
                    .sign(&private_key, public_key)
                    .expect("signing should always work")
            })
    }
}

impl Arbitrary for SignatureCheckedTransaction {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        Self::strategy_impl(gen_keypair_strategy(), any::<TransactionPayload>()).boxed()
    }
}

/// This `Arbitrary` impl only generates valid signed transactions. TODO: maybe add invalid ones?
impl Arbitrary for SignedTransaction {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        any::<SignatureCheckedTransaction>()
            .prop_map(|txn| txn.into_inner())
            .boxed()
    }
}

impl TransactionPayload {
    pub fn program_strategy() -> impl Strategy<Value = Self> {
        any::<Program>().prop_map(TransactionPayload::Program)
    }

    pub fn write_set_strategy() -> impl Strategy<Value = Self> {
        any::<WriteSet>().prop_map(TransactionPayload::WriteSet)
    }

    /// Similar to `write_set_strategy` except generates a valid write set for the genesis block.
    pub fn genesis_strategy() -> impl Strategy<Value = Self> {
        WriteSet::genesis_strategy().prop_map(TransactionPayload::WriteSet)
    }
}

prop_compose! {
    fn arb_transaction_status()(vm_status in any::<VMStatus>()) -> TransactionStatus {
        vm_status.into()
    }
}

impl Arbitrary for TransactionStatus {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        arb_transaction_status().boxed()
    }
}

impl Arbitrary for TransactionPayload {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        // Most transactions in practice will be programs, but other parts of the system should
        // at least not choke on write set strategies so introduce them with decent probability.
        // The figures below are probability weights.
        prop_oneof![
            9 => Self::program_strategy(),
            1 => Self::write_set_strategy(),
        ]
        .boxed()
    }
}

impl Arbitrary for Program {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        // XXX This should eventually be an actually valid program, maybe?
        // How should we generate random modules?
        // The vector sizes are picked out of thin air.
        (
            vec(any::<u8>(), 0..100),
            vec(any::<Vec<u8>>(), 0..100),
            vec(any::<TransactionArgument>(), 0..10),
        )
            .prop_map(|(code, modules, args)| Program::new(code, modules, args))
            .boxed()
    }
}

impl Arbitrary for TransactionArgument {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        prop_oneof![
            any::<u64>().prop_map(TransactionArgument::U64),
            any::<AccountAddress>().prop_map(TransactionArgument::Address),
            any::<ByteArray>().prop_map(TransactionArgument::ByteArray),
            ".*".prop_map(TransactionArgument::String),
        ]
        .boxed()
    }
}

prop_compose! {
    fn arb_validator_signature_for_hash(hash: HashValue)(
        hash in Just(hash),
        (private_key, public_key) in keypair_strategy(),
    ) -> (AccountAddress, Signature) {
        let signature = sign_message(hash, &private_key).unwrap();
        (AccountAddress::from(public_key), signature)
    }
}

impl Arbitrary for LedgerInfoWithSignatures {
    type Parameters = SizeRange;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(num_validators_range: Self::Parameters) -> Self::Strategy {
        (any::<LedgerInfo>(), Just(num_validators_range))
            .prop_flat_map(|(ledger_info, num_validators_range)| {
                let hash = ledger_info.hash();
                (
                    Just(ledger_info),
                    prop::collection::vec(
                        arb_validator_signature_for_hash(hash),
                        num_validators_range,
                    ),
                )
            })
            .prop_map(|(ledger_info, signatures)| {
                LedgerInfoWithSignatures::new(ledger_info, signatures.into_iter().collect())
            })
            .boxed()
    }
}

prop_compose! {
    fn arb_update_to_latest_ledger_response()(
        response_items in vec(any::<ResponseItem>(), 0..10),
        ledger_info_with_sigs in any::<LedgerInfoWithSignatures>(),
        validator_change_events in vec(any::<ValidatorChangeEventWithProof>(), 0..10),
    ) -> UpdateToLatestLedgerResponse {
        UpdateToLatestLedgerResponse::new(
            response_items, ledger_info_with_sigs, validator_change_events)
    }
}

impl Arbitrary for UpdateToLatestLedgerResponse {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        arb_update_to_latest_ledger_response().boxed()
    }
}

#[allow(clippy::implicit_hasher)]
pub fn renumber_events(
    events: &[ContractEvent],
    next_seq_num_by_access_path: &mut HashMap<AccessPath, u64>,
) -> Vec<ContractEvent> {
    events
        .iter()
        .map(|e| {
            let next_seq_num = next_seq_num_by_access_path
                .entry(e.access_path().clone())
                .or_insert(0);
            *next_seq_num += 1;
            ContractEvent::new(
                e.access_path().clone(),
                *next_seq_num - 1,
                e.event_data().to_vec(),
            )
        })
        .collect::<Vec<_>>()
}

pub fn arb_txn_to_commit_batch(
    num_accounts: usize,
    num_event_paths: usize,
    num_transactions: usize,
) -> impl Strategy<Value = Vec<TransactionToCommit>> {
    (
        vec(gen_keypair_strategy(), num_accounts),
        hash_set(any::<Vec<u8>>(), num_event_paths),
        Just(num_transactions),
    )
        .prop_flat_map(|(keypairs, event_paths, num_transactions)| {
            let keypair_strategy = Union::new(keypairs.into_iter().map(Just)).boxed();
            let event_path_strategy = Union::new(event_paths.into_iter().map(Just));
            vec(
                TransactionToCommit::strategy_impl(keypair_strategy, event_path_strategy),
                num_transactions,
            )
        })
        .prop_map(|txns_to_commit| {
            // re- number events to make it logical
            let mut next_seq_num_by_access_path = HashMap::new();
            txns_to_commit
                .into_iter()
                .map(|t| {
                    let events = renumber_events(t.events(), &mut next_seq_num_by_access_path);
                    TransactionToCommit::new(
                        t.signed_txn().clone(),
                        t.account_states().clone(),
                        events,
                        t.gas_used(),
                    )
                })
                .collect::<Vec<_>>()
        })
}

impl ContractEvent {
    pub fn strategy_impl(
        access_path_strategy: impl Strategy<Value = AccessPath>,
    ) -> impl Strategy<Value = Self> {
        (access_path_strategy, any::<u64>(), vec(any::<u8>(), 1..10)).prop_map(
            |(access_path, seq_num, event_data)| {
                ContractEvent::new(access_path, seq_num, event_data)
            },
        )
    }
}

impl Arbitrary for ContractEvent {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        ContractEvent::strategy_impl(any::<AccessPath>()).boxed()
    }
}

impl TransactionToCommit {
    fn strategy_impl(
        keypair_strategy: BoxedStrategy<(OldPrivateKey, OldPublicKey)>,
        event_path_strategy: impl Strategy<Value = Vec<u8>>,
    ) -> impl Strategy<Value = Self> {
        // signed_txn
        let txn_strategy = SignatureCheckedTransaction::strategy_impl(
            keypair_strategy.clone(),
            any::<TransactionPayload>(),
        );

        // account_states
        let address_strategy = keypair_strategy
            .clone()
            .prop_map(|(_, public_key)| AccountAddress::from(public_key));
        let account_states_strategy =
            hash_map(address_strategy.clone(), any::<AccountStateBlob>(), 1..10);

        // events
        let access_path_strategy = (address_strategy, event_path_strategy)
            .prop_map(|(address, path)| AccessPath::new(address, path));
        let events_strategy = vec(ContractEvent::strategy_impl(access_path_strategy), 0..10);

        // gas_used
        let gas_used_strategy = any::<u64>();

        // Combine the above into result.
        (
            txn_strategy,
            account_states_strategy,
            events_strategy,
            gas_used_strategy,
        )
            .prop_map(|(txn, account_states, events, gas_used)| {
                let signed_txn = txn.into_inner();
                Self::new(signed_txn, account_states, events, gas_used)
            })
    }
}

impl Arbitrary for TransactionToCommit {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        TransactionToCommit::strategy_impl(gen_keypair_strategy().boxed(), any::<Vec<u8>>()).boxed()
    }
}

fn arb_transaction_list_with_proof() -> impl Strategy<Value = TransactionListWithProof> {
    vec(
        (
            any::<SignedTransaction>(),
            any::<TransactionInfo>(),
            vec(any::<ContractEvent>(), 0..10),
        ),
        0..10,
    )
    .prop_flat_map(|transaction_and_infos_and_events| {
        let transaction_and_infos: Vec<_> = transaction_and_infos_and_events
            .clone()
            .into_iter()
            .map(|(transaction, info, _event)| (transaction, info))
            .collect();
        let events: Vec<_> = transaction_and_infos_and_events
            .into_iter()
            .map(|(_transaction, _info, event)| event)
            .collect();

        (
            Just(transaction_and_infos),
            option::of(Just(events)),
            any::<Version>(),
            any::<AccumulatorProof>(),
            any::<AccumulatorProof>(),
        )
    })
    .prop_map(
        |(
            transaction_and_infos,
            events,
            first_txn_version,
            proof_of_first_txn,
            proof_of_last_txn,
        )| {
            match transaction_and_infos.len() {
                0 => TransactionListWithProof::new_empty(),
                1 => TransactionListWithProof::new(
                    transaction_and_infos,
                    events,
                    Some(first_txn_version),
                    Some(proof_of_first_txn),
                    None,
                ),
                _ => TransactionListWithProof::new(
                    transaction_and_infos,
                    events,
                    Some(first_txn_version),
                    Some(proof_of_first_txn),
                    Some(proof_of_last_txn),
                ),
            }
        },
    )
}

impl Arbitrary for TransactionListWithProof {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        arb_transaction_list_with_proof().boxed()
    }
}
