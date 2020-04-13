// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::{
        received_payment_tag, sent_payment_tag, ReceivedPaymentEvent, SentPaymentEvent,
    },
    event::EventKey,
    language_storage::TypeTag,
    ledger_info::LedgerInfo,
    proof::EventProof,
    transaction::Version,
};
use anyhow::{ensure, format_err, Error, Result};
use libra_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};
use libra_crypto_derive::CryptoHasher;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    ops::Deref,
};

/// Support versioning of the data structure.
#[derive(Hash, Clone, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
pub enum ContractEvent {
    V0(ContractEventV0),
}

impl ContractEvent {
    pub fn new(
        key: EventKey,
        sequence_number: u64,
        type_tag: TypeTag,
        event_data: Vec<u8>,
    ) -> Self {
        ContractEvent::V0(ContractEventV0::new(
            key,
            sequence_number,
            type_tag,
            event_data,
        ))
    }
}

// Temporary hack to avoid massive changes, it won't work when new variant comes and needs proper
// dispatch at that time.
impl Deref for ContractEvent {
    type Target = ContractEventV0;

    fn deref(&self) -> &Self::Target {
        match self {
            ContractEvent::V0(event) => event,
        }
    }
}

/// Entry produced via a call to the `emit_event` builtin.
#[derive(Hash, Clone, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
pub struct ContractEventV0 {
    /// The unique key that the event was emitted to
    key: EventKey,
    /// The number of messages that have been emitted to the path previously
    #[serde(with = "lcs::fixed_size")]
    sequence_number: u64,
    /// The type of the data
    type_tag: TypeTag,
    /// The data payload of the event
    event_data: Vec<u8>,
}

impl ContractEventV0 {
    pub fn new(
        key: EventKey,
        sequence_number: u64,
        type_tag: TypeTag,
        event_data: Vec<u8>,
    ) -> Self {
        Self {
            key,
            sequence_number,
            type_tag,
            event_data,
        }
    }

    pub fn key(&self) -> &EventKey {
        &self.key
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn event_data(&self) -> &[u8] {
        &self.event_data
    }

    pub fn type_tag(&self) -> &TypeTag {
        &self.type_tag
    }
}

impl TryFrom<&ContractEvent> for SentPaymentEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(sent_payment_tag()) {
            anyhow::bail!("Expected Sent Payment")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for ReceivedPaymentEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(received_payment_tag()) {
            anyhow::bail!("Expected Received Payment")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl std::fmt::Debug for ContractEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ContractEvent {{ key: {:?}, index: {:?}, type: {:?}, event_data: {:?} }}",
            self.key,
            self.sequence_number,
            self.type_tag,
            hex::encode(&self.event_data)
        )
    }
}

impl std::fmt::Display for ContractEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(payload) = SentPaymentEvent::try_from(self) {
            write!(
                f,
                "ContractEvent {{ key: {}, index: {:?}, type: {:?}, event_data: {:?} }}",
                self.key, self.sequence_number, self.type_tag, payload,
            )
        } else if let Ok(payload) = ReceivedPaymentEvent::try_from(self) {
            write!(
                f,
                "ContractEvent {{ key: {}, index: {:?}, type: {:?}, event_data: {:?} }}",
                self.key, self.sequence_number, self.type_tag, payload,
            )
        } else {
            write!(f, "{:?}", self)
        }
    }
}

impl CryptoHash for ContractEvent {
    type Hasher = ContractEventHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&lcs::to_bytes(self).expect("Failed to serialize."));
        state.finish()
    }
}

impl TryFrom<crate::proto::types::Event> for ContractEvent {
    type Error = Error;

    fn try_from(event: crate::proto::types::Event) -> Result<Self> {
        let key = EventKey::try_from(event.key.as_ref())?;
        let sequence_number = event.sequence_number;
        let type_tag = lcs::from_bytes(&event.type_tag)?;
        let event_data = event.event_data;
        Ok(Self::new(key, sequence_number, type_tag, event_data))
    }
}

impl From<ContractEvent> for crate::proto::types::Event {
    fn from(event: ContractEvent) -> Self {
        Self {
            key: event.key.to_vec(),
            sequence_number: event.sequence_number,
            type_tag: lcs::to_bytes(&event.type_tag).expect("Failed to serialize."),
            event_data: event.event_data.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct EventWithProof {
    #[serde(with = "lcs::fixed_size")]
    pub transaction_version: u64, // Should be `Version`
    #[serde(with = "lcs::fixed_size")]
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
        event_key: &EventKey,
        sequence_number: u64,
        transaction_version: Version,
        event_index: u64,
    ) -> Result<()> {
        ensure!(
            self.event.key() == event_key,
            "Event key ({}) not expected ({}).",
            self.event.key(),
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

        self.proof.verify(
            ledger_info,
            self.event.hash(),
            transaction_version,
            event_index,
        )?;

        Ok(())
    }
}

impl TryFrom<crate::proto::types::EventWithProof> for EventWithProof {
    type Error = Error;

    fn try_from(event: crate::proto::types::EventWithProof) -> Result<Self> {
        Ok(Self::new(
            event.transaction_version,
            event.event_index,
            event
                .event
                .ok_or_else(|| format_err!("Missing event"))?
                .try_into()?,
            event
                .proof
                .ok_or_else(|| format_err!("Missing proof"))?
                .try_into()?,
        ))
    }
}

impl From<EventWithProof> for crate::proto::types::EventWithProof {
    fn from(event: EventWithProof) -> Self {
        Self {
            transaction_version: event.transaction_version,
            event_index: event.event_index,
            event: Some(event.event.into()),
            proof: Some(event.proof.into()),
        }
    }
}
