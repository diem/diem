// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error, Result};

use std::convert::{TryFrom, TryInto};

use libra_types::{
    contract_event::{ContractEvent, EventWithProof},
    event::EventKey,
};

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

impl From<&ContractEvent> for crate::proto::types::Event {
    fn from(event: &ContractEvent) -> Self {
        Self {
            key: event.key().to_vec(),
            sequence_number: event.sequence_number(),
            type_tag: lcs::to_bytes(&event.type_tag()).expect("Failed to serialize."),
            event_data: event.event_data().to_vec(),
        }
    }
}

impl From<ContractEvent> for crate::proto::types::Event {
    fn from(event: ContractEvent) -> Self {
        Self::from(&event)
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
