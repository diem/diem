// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::{
        AdminTransactionEvent, BaseUrlRotationEvent, BurnEvent, CancelBurnEvent,
        ComplianceKeyRotationEvent, CreateAccountEvent, MintEvent, NewBlockEvent, NewEpochEvent,
        PreburnEvent, ReceivedMintEvent, ReceivedPaymentEvent, SentPaymentEvent,
        ToXDXExchangeRateUpdateEvent, VASPDomainEvent,
    },
    event::EventKey,
    ledger_info::LedgerInfo,
    proof::EventProof,
    transaction::Version,
};
use anyhow::{ensure, Context, Error, Result};
use diem_crypto::hash::CryptoHash;
use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};
use move_core_types::{language_storage::TypeTag, move_resource::MoveStructType};

#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, ops::Deref};

/// Support versioning of the data structure.
#[derive(Hash, Clone, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, BCSCryptoHash)]
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
    sequence_number: u64,
    /// The type of the data
    type_tag: TypeTag,
    /// The data payload of the event
    #[serde(with = "serde_bytes")]
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
        if event.type_tag != TypeTag::Struct(SentPaymentEvent::struct_tag()) {
            anyhow::bail!("Expected Sent Payment")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for ReceivedPaymentEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(ReceivedPaymentEvent::struct_tag()) {
            anyhow::bail!("Expected Received Payment")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for ToXDXExchangeRateUpdateEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(ToXDXExchangeRateUpdateEvent::struct_tag()) {
            anyhow::bail!("Expected ToXDXExchangeRateUpdateEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for MintEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(MintEvent::struct_tag()) {
            anyhow::bail!("Expected MintEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for ReceivedMintEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(ReceivedMintEvent::struct_tag()) {
            anyhow::bail!("Expected ReceivedMintEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for BurnEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(BurnEvent::struct_tag()) {
            anyhow::bail!("Expected BurnEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for PreburnEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(PreburnEvent::struct_tag()) {
            anyhow::bail!("Expected PreburnEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for CancelBurnEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(CancelBurnEvent::struct_tag()) {
            anyhow::bail!("Expected CancelBurnEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for AdminTransactionEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(Self::struct_tag()) {
            anyhow::bail!("Expected AdminTransactionEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for NewBlockEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(Self::struct_tag()) {
            anyhow::bail!("Expected NewBlockEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for NewEpochEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(Self::struct_tag()) {
            anyhow::bail!("Expected NewEpochEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for ComplianceKeyRotationEvent {
    type Error = Error;
    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(Self::struct_tag()) {
            anyhow::bail!("Expected ComplianceKeyRotationEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for BaseUrlRotationEvent {
    type Error = Error;
    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(Self::struct_tag()) {
            anyhow::bail!("Expected BaseUrlRotationEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for CreateAccountEvent {
    type Error = Error;
    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(Self::struct_tag()) {
            anyhow::bail!("Expected CreateAccountEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for VASPDomainEvent {
    type Error = Error;
    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(Self::struct_tag()) {
            anyhow::bail!("Expected VASPDomainEvent")
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct EventWithProof {
    pub transaction_version: u64, // Should be `Version`
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

/// The response type for `get_event_by_version_with_proof`, which contains lower
/// and upper bound events surrounding the requested version along with proofs
/// for each event.
///
/// ### Why do we need two events?
///
/// If we could always get the event count _at the requested event_version_, we
/// could return only the lower bound event. With the event count we could verify
/// that the returned event is actually the latest event at the historical ledger
/// view just by checking that `event.sequence_number == event_count`.
///
/// Unfortunately, the event count is only (verifiably) accessible via the
/// on-chain state. While we can easily acquire the event count near the chain
/// HEAD, historical event counts (at versions below HEAD for more than the prune
/// window) may be inaccessible after most non-archival fullnodes have pruned past
/// that version.
///
/// ### Including the Upper Bound Event
///
/// In contrast, if we also return the upper bound event, then we can always
/// verify the request even if the version is past the prune window and we don't
/// know the event_count (at event_version). The upper bound event lets us prove
/// that there is no untransmitted event that is actually closer to the requested
/// event_version than the lower bound.
///
/// For example, consider the case where there are three events at versions 10,
/// 20, and 30. A client asks for the latest event at or below version 25. If we
/// just returned the lower bound event, then a malicious server could return
/// the event at version 10; the client would not be able to distinguish this
/// response from the correct response without the event count (2) at version 25.
///
/// If we also return the upper bound event (the event at version 30), the client
/// can verify that the upper bound is the next event after the lower bound and
/// that the upper bound comes after their requested version. This proves that
/// the lower bound is actually the latest event at or below their requested
/// version.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct EventByVersionWithProof {
    pub lower_bound_incl: Option<EventWithProof>,
    pub upper_bound_excl: Option<EventWithProof>,
}

impl EventByVersionWithProof {
    pub fn new(
        lower_bound_incl: Option<EventWithProof>,
        upper_bound_excl: Option<EventWithProof>,
    ) -> Self {
        Self {
            lower_bound_incl,
            upper_bound_excl,
        }
    }

    /// Verify that the `lower_bound_incl` [`EventWithProof`] is the latest event
    /// at or below the requested `event_version`.
    ///
    /// The `ledger_info` is the client's latest know ledger info (will be near
    /// chain HEAD if the client is synced).
    ///
    /// The `latest_event_count` is the event count at the `ledger_info` version
    /// (not the `event_version`) and is needed to verify the empty event stream
    /// and version after last event cases. In some select instances
    /// (e.g. [`NewBlockEvent`]s) we can determine these cases more efficiently
    /// and so this parameter is left optional.
    pub fn verify(
        &self,
        ledger_info: &LedgerInfo,
        event_key: &EventKey,
        latest_event_count: Option<u64>,
        event_version: Version,
    ) -> Result<()> {
        ensure!(
            event_version <= ledger_info.version(),
            "request event_version {} must be <= LedgerInfo version {}",
            event_version,
            ledger_info.version(),
        );

        // If the verifier didn't provide a latest_event_count, choose a value
        // that will pass the checks in each case.
        let latest_event_count = latest_event_count.unwrap_or_else(|| {
            let upper = self.upper_bound_excl.as_ref();
            let lower = self.lower_bound_incl.as_ref();
            upper /* (None, Some), (Some, Some) */
                .or(lower) /* (Some, None) */
                .map(|proof| proof.event.sequence_number.saturating_add(1))
                .unwrap_or(0) /* (None, None) */
        });

        match (&self.lower_bound_incl, &self.upper_bound_excl) {
            // no events at all yet
            (None, None) => {
                ensure!(latest_event_count == 0);
            }
            // event_version comes before first ever event, so in the range: [0, event_0.version)
            //
            // event_version
            //      v
            // |---------|
            //            (event_0)----->(event_1)---->
            (None, Some(first_event)) => {
                let txn_version = first_event.transaction_version;
                let seq_num = first_event.event.sequence_number;
                ensure!(event_version < txn_version);
                ensure!(seq_num == 0);
                ensure!(latest_event_count > 0);

                first_event
                    .verify(
                        ledger_info,
                        event_key,
                        seq_num,
                        first_event.transaction_version,
                        first_event.event_index,
                    )
                    .context("failed to verify first event")?;
            }
            // event_version is between two events, specifically, it must be in
            // the range: [event_i.version, event_{i+1}.version)
            //
            //              event_version
            //                    v
            //            |---------------|
            //      ----->(event_{i})----->(event_{i+1})----->
            (Some(lower_bound_incl), Some(upper_bound_excl)) => {
                ensure!(lower_bound_incl.transaction_version <= event_version);
                ensure!(event_version < upper_bound_excl.transaction_version);

                let start_seq_num = lower_bound_incl.event.sequence_number;
                let end_seq_num = upper_bound_excl.event.sequence_number;
                ensure!(start_seq_num.saturating_add(1) == end_seq_num);
                ensure!(latest_event_count > end_seq_num);

                lower_bound_incl
                    .verify(
                        ledger_info,
                        event_key,
                        start_seq_num,
                        lower_bound_incl.transaction_version,
                        lower_bound_incl.event_index,
                    )
                    .context("failed to verify lower bound event")?;
                upper_bound_excl
                    .verify(
                        ledger_info,
                        event_key,
                        end_seq_num,
                        upper_bound_excl.transaction_version,
                        upper_bound_excl.event_index,
                    )
                    .context("failed to verify upper bound event")?;
            }
            // event_version is after the latest event, meaning it's in the range:
            // (event_N.version, ledger_version]
            //
            //                         event_version
            //                               v
            //                       |----------------->
            //      ----->(event_{N})
            (Some(latest_event), None) => {
                let txn_version = latest_event.transaction_version;
                let seq_num = latest_event.event.sequence_number;
                ensure!(txn_version <= event_version);
                ensure!(seq_num.saturating_add(1) == latest_event_count);
                latest_event
                    .verify(
                        ledger_info,
                        event_key,
                        seq_num,
                        txn_version,
                        latest_event.event_index,
                    )
                    .context("failed to verify latest event")?;
            }
        }

        Ok(())
    }
}
