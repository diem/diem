// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]
use crate::{
    access_path::AccessPath,
    account_config::AccountEvent,
    ledger_info::LedgerInfo,
    proof::{verify_event, EventProof},
    transaction::Version,
};
use canonical_serialization::{
    CanonicalSerialize, CanonicalSerializer, SimpleDeserializer, SimpleSerializer,
};
use crypto::{
    hash::{ContractEventHasher, CryptoHash, CryptoHasher},
    HashValue,
};
use failure::prelude::*;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};

/// Entry produced via a call to the `emit_event` builtin.
#[derive(Clone, Default, Eq, PartialEq, FromProto, IntoProto)]
#[ProtoType(crate::proto::events::Event)]
pub struct ContractEvent {
    /// The path that the event was emitted to
    access_path: AccessPath,
    /// The number of messages that have been emitted to the path previously
    sequence_number: u64,
    /// The data payload of the event
    event_data: Vec<u8>,
}

impl ContractEvent {
    pub fn new(access_path: AccessPath, sequence_number: u64, event_data: Vec<u8>) -> Self {
        ContractEvent {
            access_path,
            sequence_number,
            event_data,
        }
    }

    pub fn access_path(&self) -> &AccessPath {
        &self.access_path
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn event_data(&self) -> &[u8] {
        &self.event_data
    }
}

impl std::fmt::Debug for ContractEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ContractEvent {{ access_path: {:?}, index: {:?}, event_data: {:?} }}",
            self.access_path,
            self.sequence_number,
            hex::encode(&self.event_data)
        )
    }
}

impl std::fmt::Display for ContractEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(payload) = SimpleDeserializer::deserialize::<AccountEvent>(&self.event_data) {
            write!(
                f,
                "ContractEvent {{ access_path: {}, index: {:?}, event_data: {:?} }}",
                self.access_path, self.sequence_number, payload,
            )
        } else {
            write!(f, "{:?}", self)
        }
    }
}

impl CanonicalSerialize for ContractEvent {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_struct(&self.access_path)?
            .encode_u64(self.sequence_number)?
            .encode_variable_length_bytes(&self.event_data)?;
        Ok(())
    }
}

impl CryptoHash for ContractEvent {
    type Hasher = ContractEventHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&SimpleSerializer::<Vec<u8>>::serialize(self).expect("Failed to serialize."));
        state.finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, FromProto, IntoProto)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[ProtoType(crate::proto::events::EventWithProof)]
pub struct EventWithProof {
    pub transaction_version: u64, // Should be `Version`, but FromProto derive won't work that way.
    pub event_index: u64,
    pub event: ContractEvent,
    pub proof: EventProof,
}

impl std::fmt::Display for EventWithProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EventWithProof {{ \n\ttransaction_version: {}, \n\tevent_index: {}, \
             \n\tevent: {}, \n\tproof: {:?} \n}}",
            self.transaction_version, self.event_index, self.event, self.proof
        )
    }
}

impl EventWithProof {
    /// Constructor.
    pub fn new(
        transaction_version: Version,
        event_index: u64,
        event: ContractEvent,
        proof: EventProof,
    ) -> Self {
        Self {
            transaction_version,
            event_index,
            event,
            proof,
        }
    }

    /// Verifies the event with the proof, both carried by `self`.
    ///
    /// Two things are ensured if no error is raised:
    ///   1. This event exists in the ledger represented by `ledger_info`.
    ///   2. And this event has the same `event_key`, `sequence_number`, `transaction_version`,
    /// and `event_index` as indicated in the parameter list. If any of these parameter is unknown
    /// to the call site and is supposed to be informed by this struct, get it from the struct
    /// itself, such as: `event_with_proof.event.access_path()`, `event_with_proof.event_index()`,
    /// etc.
    pub fn verify(
        &self,
        ledger_info: &LedgerInfo,
        event_key: &AccessPath,
        sequence_number: u64,
        transaction_version: Version,
        event_index: u64,
    ) -> Result<()> {
        ensure!(
            self.event.access_path() == event_key,
            "Event key ({}) not expected ({}).",
            self.event.access_path(),
            *event_key,
        );
        ensure!(
            self.event.sequence_number == sequence_number,
            "Sequence number ({}) not expected ({}).",
            self.event.sequence_number(),
            sequence_number,
        );
        ensure!(
            self.transaction_version == transaction_version,
            "Transaction version ({}) not expected ({}).",
            self.transaction_version,
            transaction_version,
        );
        ensure!(
            self.event_index == event_index,
            "Event index ({}) not expected ({}).",
            self.event_index,
            event_index,
        );

        verify_event(
            ledger_info,
            self.event.hash(),
            transaction_version,
            event_index,
            &self.proof,
        )?;

        Ok(())
    }
}
