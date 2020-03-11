// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error};
use hex;
use libra_types::{
    account_config::{
        received_payment_tag, sent_payment_tag, AccountResource, BalanceResource,
        ReceivedPaymentEvent, SentPaymentEvent,
    },
    contract_event::ContractEvent,
    crypto_proxies::{LedgerInfoWithSignatures, ValidatorChangeProof},
    language_storage::TypeTag,
    proof::AccumulatorConsistencyProof,
    transaction::{Transaction, TransactionArgument, TransactionPayload},
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use transaction_builder::get_transaction_name;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AccountView {
    pub balance: u64,
    pub sequence_number: u64,
    pub authentication_key: BytesView,
    pub sent_events_key: BytesView,
    pub received_events_key: BytesView,
    pub delegated_key_rotation_capability: bool,
    pub delegated_withdrawal_capability: bool,
}

impl AccountView {
    pub fn new(account: &AccountResource, balance: &BalanceResource) -> Self {
        Self {
            balance: balance.coin(),
            sequence_number: account.sequence_number(),
            authentication_key: BytesView::from(account.authentication_key()),
            sent_events_key: BytesView::from(account.sent_events().key().as_bytes()),
            received_events_key: BytesView::from(account.received_events().key().as_bytes()),
            delegated_key_rotation_capability: account.delegated_key_rotation_capability(),
            delegated_withdrawal_capability: account.delegated_withdrawal_capability(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventView {
    pub key: BytesView,
    pub sequence_number: u64,
    pub transaction_version: u64,
    pub data: EventDataView,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventDataView {
    #[serde(rename = "receivedpayment")]
    ReceivedPayment {
        amount: u64,
        sender: BytesView,
        metadata: BytesView,
    },
    #[serde(rename = "sentpayment")]
    SentPayment {
        amount: u64,
        receiver: BytesView,
        metadata: BytesView,
    },
    #[serde(rename = "unknown")]
    Unknown {},
}

impl From<(u64, ContractEvent)> for EventView {
    /// Tries to convert the provided byte array into Event Key.
    fn from((txn_version, event): (u64, ContractEvent)) -> EventView {
        let event_data = if event.type_tag() == &TypeTag::Struct(received_payment_tag()) {
            if let Ok(received_event) = ReceivedPaymentEvent::try_from(&event) {
                Ok(EventDataView::ReceivedPayment {
                    amount: received_event.amount(),
                    sender: BytesView::from(received_event.sender().as_ref()),
                    metadata: BytesView::from(received_event.metadata()),
                })
            } else {
                Err(format_err!("Unable to parse ReceivedPaymentEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(sent_payment_tag()) {
            if let Ok(sent_event) = SentPaymentEvent::try_from(&event) {
                Ok(EventDataView::SentPayment {
                    amount: sent_event.amount(),
                    receiver: BytesView::from(sent_event.receiver().as_ref()),
                    metadata: BytesView::from(sent_event.metadata()),
                })
            } else {
                Err(format_err!("Unable to parse ReceivedPaymentEvent"))
            }
        } else {
            Err(format_err!("Unknown events"))
        };

        EventView {
            key: BytesView::from(event.key().as_bytes()),
            sequence_number: event.sequence_number(),
            transaction_version: txn_version,
            data: event_data.unwrap_or(EventDataView::Unknown {}),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct BlockMetadata {
    pub version: u64,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BytesView(pub String);

impl BytesView {
    pub fn into_bytes(self) -> Result<Vec<u8>, Error> {
        Ok(hex::decode(self.0)?)
    }
}

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

#[derive(Deserialize, Serialize)]
pub struct TransactionView {
    pub version: u64,
    pub transaction: TransactionDataView,
    pub events: Vec<EventView>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum TransactionDataView {
    #[serde(rename = "blockmetadata")]
    BlockMetadata { timestamp_usecs: u64 },
    #[serde(rename = "writeset")]
    WriteSet {},
    #[serde(rename = "user")]
    UserTransaction {
        sender: String,
        signature: String,
        public_key: String,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_time: u64,
        script: ScriptView,
    },
    #[serde(rename = "unknown")]
    UnknownTransaction {},
}

impl TransactionView {
    pub fn get_transaction_name(&self) -> String {
        if let TransactionDataView::UserTransaction { script, .. } = &self.transaction {
            script.get_name()
        } else {
            "".to_string()
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
// TODO cover all script types
pub enum ScriptView {
    #[serde(rename = "peer_to_peer_transaction")]
    PeerToPeer {
        receiver: String,
        arg_auth_key_prefix: BytesView,
        amount: u64,
    },

    #[serde(rename = "unknown_transaction")]
    Unknown {},
}

impl ScriptView {
    // TODO cover all script types
    pub fn get_name(&self) -> String {
        match self {
            ScriptView::PeerToPeer { .. } => "peer to peer transaction".to_string(),
            ScriptView::Unknown {} => "unknown transaction".to_string(),
        }
    }
}

impl From<Transaction> for TransactionDataView {
    fn from(tx: Transaction) -> Self {
        let x = match tx {
            Transaction::BlockMetadata(t) => {
                t.into_inner().map(|x| TransactionDataView::BlockMetadata {
                    timestamp_usecs: x.1,
                })
            }
            Transaction::WriteSet(_) => Ok(TransactionDataView::WriteSet {}),
            Transaction::UserTransaction(t) => Ok(TransactionDataView::UserTransaction {
                sender: t.sender().to_string(),
                signature: t.signature().to_string(),
                public_key: t.public_key().to_string(),
                sequence_number: t.sequence_number(),
                max_gas_amount: t.max_gas_amount(),
                gas_unit_price: t.gas_unit_price(),
                expiration_time: t.expiration_time().as_secs(),
                script: t.into_raw_transaction().into_payload().into(),
            }),
        };

        x.unwrap_or(TransactionDataView::UnknownTransaction {})
    }
}

impl From<TransactionPayload> for ScriptView {
    fn from(value: TransactionPayload) -> Self {
        let empty_vec: Vec<TransactionArgument> = vec![];
        let (code, args) = match value {
            TransactionPayload::Program => ("deprecated".to_string(), empty_vec),
            TransactionPayload::WriteSet(_) => ("genesis".to_string(), empty_vec),
            TransactionPayload::Script(script) => {
                (get_transaction_name(script.code()), script.args().to_vec())
            }
            TransactionPayload::Module(_) => ("module publishing".to_string(), empty_vec),
        };

        let res = match code.as_str() {
            "peer_to_peer_transaction" => {
                if let [TransactionArgument::Address(receiver), TransactionArgument::U8Vector(arg_auth_key_prefix), TransactionArgument::U64(amount)] =
                    &args[..]
                {
                    Ok(ScriptView::PeerToPeer {
                        receiver: receiver.to_string(),
                        arg_auth_key_prefix: BytesView::from(arg_auth_key_prefix),
                        amount: *amount,
                    })
                } else {
                    Err(format_err!("Unable to parse PeerToPeer arguments"))
                }
            }
            _ => Err(format_err!("Unknown scripts")),
        };
        res.unwrap_or(ScriptView::Unknown {})
    }
}

#[derive(Serialize, Deserialize)]
pub struct StateProofView {
    pub ledger_info_with_signatures: BytesView,
    pub validator_change_proof: BytesView,
    pub ledger_consistency_proof: BytesView,
}

impl
    TryFrom<(
        LedgerInfoWithSignatures,
        ValidatorChangeProof,
        AccumulatorConsistencyProof,
    )> for StateProofView
{
    type Error = Error;

    fn try_from(
        (ledger_info_with_signatures, validator_change_proof, ledger_consistency_proof): (
            LedgerInfoWithSignatures,
            ValidatorChangeProof,
            AccumulatorConsistencyProof,
        ),
    ) -> Result<StateProofView, Self::Error> {
        Ok(StateProofView {
            ledger_info_with_signatures: BytesView::from(&lcs::to_bytes(
                &ledger_info_with_signatures,
            )?),
            validator_change_proof: BytesView::from(&lcs::to_bytes(&validator_change_proof)?),
            ledger_consistency_proof: BytesView::from(&lcs::to_bytes(&ledger_consistency_proof)?),
        })
    }
}
