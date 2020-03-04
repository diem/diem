// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Error;
use hex;
use libra_types::{
    account_config::{
        received_payment_tag, sent_payment_tag, AccountResource, ReceivedPaymentEvent,
        SentPaymentEvent,
    },
    contract_event::ContractEvent,
    language_storage::TypeTag,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AccountView {
    balance: u64,
    sequence_number: u64,
    authentication_key: BytesView,
    pub sent_events_key: BytesView,
    pub received_events_key: BytesView,
    delegated_key_rotation_capability: bool,
    delegated_withdrawal_capability: bool,
}

impl AccountView {
    pub fn new(account: &AccountResource) -> Self {
        Self {
            balance: account.balance(),
            sequence_number: account.sequence_number(),
            authentication_key: BytesView::from(account.authentication_key()),
            sent_events_key: BytesView::from(account.sent_events().key().as_bytes()),
            received_events_key: BytesView::from(account.received_events().key().as_bytes()),

            delegated_key_rotation_capability: account.delegated_key_rotation_capability(),
            delegated_withdrawal_capability: account.delegated_withdrawal_capability(),
        }
    }

    // TODO remove test tag once used by CLI client
    #[cfg(test)]
    pub(crate) fn balance(&self) -> u64 {
        self.balance
    }
}

#[derive(Serialize, Deserialize)]
pub struct EventView {
    pub key: BytesView,
    pub sequence_number: u64,
    pub transaction_version: u64,
    pub data: EventDataView,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventDataView {
    ReceivedPayment {
        amount: u64,
        sender: BytesView,
        metadata: BytesView,
    },
    SentPayment {
        amount: u64,
        receiver: BytesView,
        metadata: BytesView,
    },
    SystemEvent,
}

impl TryFrom<(u64, ContractEvent)> for EventView {
    type Error = Error;

    /// Tries to convert the provided byte array into Event Key.
    fn try_from((txn_version, event): (u64, ContractEvent)) -> Result<EventView, Error> {
        let event_data = if event.type_tag() == &TypeTag::Struct(received_payment_tag()) {
            let received_event = ReceivedPaymentEvent::try_from(&event)?;
            EventDataView::ReceivedPayment {
                amount: received_event.amount(),
                sender: BytesView::from(received_event.sender().as_ref()),
                metadata: BytesView::from(received_event.metadata()),
            }
        } else if event.type_tag() == &TypeTag::Struct(sent_payment_tag()) {
            let sent_event = SentPaymentEvent::try_from(&event)?;
            EventDataView::SentPayment {
                amount: sent_event.amount(),
                receiver: BytesView::from(sent_event.receiver().as_ref()),
                metadata: BytesView::from(sent_event.metadata()),
            }
        } else {
            EventDataView::SystemEvent
        };
        Ok(EventView {
            key: BytesView::from(event.key().as_bytes()),
            sequence_number: event.sequence_number(),
            transaction_version: txn_version,
            data: event_data,
        })
    }
}

#[derive(Deserialize, Serialize)]
pub struct BlockMetadata {
    pub version: u64,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BytesView(String);

impl From<&[u8]> for BytesView {
    fn from(bytes: &[u8]) -> Self {
        Self(hex::encode(bytes))
    }
}

impl From<&Vec<u8>> for BytesView {
    fn from(bytes: &Vec<u8>) -> Self {
        Self(hex::encode(bytes))
    }
}
