// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error, Result};
use libra_crypto::HashValue;
use libra_types::{
    account_config::{
        AccountResource, AccountRole, BalanceResource, BurnEvent, CancelBurnEvent,
        CurrencyInfoResource, MintEvent, NewBlockEvent, NewEpochEvent, PreburnEvent,
        ReceivedPaymentEvent, SentPaymentEvent, ToLBRExchangeRateUpdateEvent, UpgradeEvent,
    },
    account_state_blob::AccountStateWithProof,
    contract_event::ContractEvent,
    epoch_change::EpochChangeProof,
    ledger_info::LedgerInfoWithSignatures,
    proof::{AccountStateProof, AccumulatorConsistencyProof},
    transaction::{Transaction, TransactionArgument, TransactionPayload},
    vm_error::StatusCode,
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
    move_resource::MoveResource,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, convert::TryFrom};
use transaction_builder::get_transaction_name;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct AmountView {
    pub amount: u64,
    pub currency: String,
}

impl AmountView {
    fn new(amount: u64, currency: &str) -> Self {
        Self {
            amount,
            currency: currency.to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum AccountRoleView {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "unhosted")]
    Unhosted,
    #[serde(rename = "empty")]
    Empty,
    #[serde(rename = "child_vasp")]
    ChildVASP { parent_vasp_address: BytesView },
    #[serde(rename = "parent_vasp")]
    ParentVASP {
        human_name: String,
        base_url: String,
        expiration_time: u64,
        compliance_key: BytesView,
        num_children: u64,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct AccountView {
    pub balances: Vec<AmountView>,
    pub sequence_number: u64,
    pub authentication_key: BytesView,
    pub sent_events_key: BytesView,
    pub received_events_key: BytesView,
    pub delegated_key_rotation_capability: bool,
    pub delegated_withdrawal_capability: bool,
    pub is_frozen: bool,
    pub role: AccountRoleView,
}

impl AccountView {
    pub fn new(
        account: &AccountResource,
        balances: BTreeMap<Identifier, BalanceResource>,
        account_role: AccountRole,
    ) -> Self {
        Self {
            balances: balances
                .iter()
                .map(|(currency_code, balance)| {
                    AmountView::new(balance.coin(), &currency_code.as_str())
                })
                .collect(),
            sequence_number: account.sequence_number(),
            authentication_key: BytesView::from(account.authentication_key()),
            sent_events_key: BytesView::from(account.sent_events().key().as_bytes()),
            received_events_key: BytesView::from(account.received_events().key().as_bytes()),
            delegated_key_rotation_capability: account.has_delegated_key_rotation_capability(),
            delegated_withdrawal_capability: account.has_delegated_withdrawal_capability(),
            is_frozen: account.is_frozen(),
            role: AccountRoleView::from(account_role),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventView {
    pub key: BytesView,
    pub sequence_number: u64,
    pub transaction_version: u64,
    pub data: EventDataView,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum EventDataView {
    #[serde(rename = "burn")]
    Burn {
        amount: AmountView,
        preburn_address: BytesView,
    },
    #[serde(rename = "cancelburn")]
    CancelBurn {
        amount: AmountView,
        preburn_address: BytesView,
    },
    #[serde(rename = "mint")]
    Mint { amount: AmountView },
    #[serde(rename = "to_lbr_exchange_rate_update")]
    ToLBRExchangeRateUpdate {
        currency_code: String,
        new_to_lbr_exchange_rate: f32,
    },
    #[serde(rename = "preburn")]
    Preburn {
        amount: AmountView,
        preburn_address: BytesView,
    },
    #[serde(rename = "receivedpayment")]
    ReceivedPayment {
        amount: AmountView,
        sender: BytesView,
        metadata: BytesView,
    },
    #[serde(rename = "sentpayment")]
    SentPayment {
        amount: AmountView,
        receiver: BytesView,
        metadata: BytesView,
    },
    #[serde(rename = "upgrade")]
    Upgrade { write_set: BytesView },
    #[serde(rename = "newepoch")]
    NewEpoch { epoch: u64 },
    #[serde(rename = "newblock")]
    NewBlock {
        round: u64,
        proposer: BytesView,
        proposed_time: u64,
    },
    #[serde(rename = "unknown")]
    Unknown {},
}

impl From<(u64, ContractEvent)> for EventView {
    /// Tries to convert the provided byte array into Event Key.
    fn from((txn_version, event): (u64, ContractEvent)) -> EventView {
        let event_data = if event.type_tag() == &TypeTag::Struct(ReceivedPaymentEvent::struct_tag())
        {
            if let Ok(received_event) = ReceivedPaymentEvent::try_from(&event) {
                let amount_view = AmountView::new(
                    received_event.amount(),
                    received_event.currency_code().as_str(),
                );
                Ok(EventDataView::ReceivedPayment {
                    amount: amount_view,
                    sender: BytesView::from(received_event.sender().as_ref()),
                    metadata: BytesView::from(received_event.metadata()),
                })
            } else {
                Err(format_err!("Unable to parse ReceivedPaymentEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(SentPaymentEvent::struct_tag()) {
            if let Ok(sent_event) = SentPaymentEvent::try_from(&event) {
                let amount_view =
                    AmountView::new(sent_event.amount(), sent_event.currency_code().as_str());
                Ok(EventDataView::SentPayment {
                    amount: amount_view,
                    receiver: BytesView::from(sent_event.receiver().as_ref()),
                    metadata: BytesView::from(sent_event.metadata()),
                })
            } else {
                Err(format_err!("Unable to parse SentPaymentEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(BurnEvent::struct_tag()) {
            if let Ok(burn_event) = BurnEvent::try_from(&event) {
                let amount_view =
                    AmountView::new(burn_event.amount(), burn_event.currency_code().as_str());
                let preburn_address = BytesView::from(burn_event.preburn_address().as_ref());
                Ok(EventDataView::Burn {
                    amount: amount_view,
                    preburn_address,
                })
            } else {
                Err(format_err!("Unable to parse BurnEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(CancelBurnEvent::struct_tag()) {
            if let Ok(cancel_burn_event) = CancelBurnEvent::try_from(&event) {
                let amount_view = AmountView::new(
                    cancel_burn_event.amount(),
                    cancel_burn_event.currency_code().as_str(),
                );
                let preburn_address = BytesView::from(cancel_burn_event.preburn_address().as_ref());
                Ok(EventDataView::CancelBurn {
                    amount: amount_view,
                    preburn_address,
                })
            } else {
                Err(format_err!("Unable to parse CancelBurnEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(ToLBRExchangeRateUpdateEvent::struct_tag()) {
            if let Ok(update_event) = ToLBRExchangeRateUpdateEvent::try_from(&event) {
                Ok(EventDataView::ToLBRExchangeRateUpdate {
                    currency_code: update_event.currency_code().to_string(),
                    new_to_lbr_exchange_rate: update_event.new_to_lbr_exchange_rate(),
                })
            } else {
                Err(format_err!("Unable to parse ToLBRExchangeRateUpdate"))
            }
        } else if event.type_tag() == &TypeTag::Struct(MintEvent::struct_tag()) {
            if let Ok(mint_event) = MintEvent::try_from(&event) {
                let amount_view =
                    AmountView::new(mint_event.amount(), mint_event.currency_code().as_str());
                Ok(EventDataView::Mint {
                    amount: amount_view,
                })
            } else {
                Err(format_err!("Unable to parse MintEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(PreburnEvent::struct_tag()) {
            if let Ok(preburn_event) = PreburnEvent::try_from(&event) {
                let amount_view = AmountView::new(
                    preburn_event.amount(),
                    preburn_event.currency_code().as_str(),
                );
                let preburn_address = BytesView::from(preburn_event.preburn_address().as_ref());
                Ok(EventDataView::Preburn {
                    amount: amount_view,
                    preburn_address,
                })
            } else {
                Err(format_err!("Unable to parse PreBurnEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(NewBlockEvent::struct_tag()) {
            if let Ok(new_block_event) = NewBlockEvent::try_from(&event) {
                Ok(EventDataView::NewBlock {
                    proposer: BytesView::from(new_block_event.proposer().as_ref()),
                    round: new_block_event.round(),
                    proposed_time: new_block_event.proposed_time(),
                })
            } else {
                Err(format_err!("Unable to parse NewBlockEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(NewEpochEvent::struct_tag()) {
            if let Ok(new_epoch_event) = NewEpochEvent::try_from(&event) {
                Ok(EventDataView::NewEpoch {
                    epoch: new_epoch_event.epoch(),
                })
            } else {
                Err(format_err!("Unable to parse NewEpochEvent"))
            }
        } else if event.type_tag() == &TypeTag::Struct(UpgradeEvent::struct_tag()) {
            if let Ok(upgrade_event) = UpgradeEvent::try_from(&event) {
                Ok(EventDataView::Upgrade {
                    write_set: BytesView::from(upgrade_event.write_set()),
                })
            } else {
                Err(format_err!("Unable to parse UpgradeEvent"))
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BlockMetadata {
    pub version: u64,
    pub timestamp: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TransactionView {
    pub version: u64,
    pub transaction: TransactionDataView,
    pub hash: String,
    pub events: Vec<EventView>,
    pub vm_status: StatusCode,
    pub gas_used: u64,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum TransactionDataView {
    #[serde(rename = "blockmetadata")]
    BlockMetadata { timestamp_usecs: u64 },
    #[serde(rename = "writeset")]
    WriteSet {},
    #[serde(rename = "user")]
    UserTransaction {
        sender: String,
        signature_scheme: String,
        signature: String,
        public_key: String,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_currency: String,
        expiration_time: u64,
        script_hash: String,
        script: ScriptView,
    },
    #[serde(rename = "unknown")]
    UnknownTransaction {},
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
// TODO cover all script types
pub enum ScriptView {
    #[serde(rename = "peer_to_peer_transaction")]
    PeerToPeer {
        receiver: String,
        amount: u64,
        currency: String,
        metadata: BytesView,
        metadata_signature: BytesView,
    },
    #[serde(rename = "mint_transaction")]
    Mint {
        receiver: String,
        currency: String,
        auth_key_prefix: BytesView,
        amount: u64,
    },
    #[serde(rename = "unknown_transaction")]
    Unknown {},
}

impl ScriptView {
    // TODO cover all script types
}

impl From<Transaction> for TransactionDataView {
    fn from(tx: Transaction) -> Self {
        let x = match tx {
            Transaction::BlockMetadata(t) => {
                t.into_inner().map(|x| TransactionDataView::BlockMetadata {
                    timestamp_usecs: x.1,
                })
            }
            Transaction::WaypointWriteSet(_) => Ok(TransactionDataView::WriteSet {}),
            Transaction::UserTransaction(t) => {
                let script_hash = match t.payload() {
                    TransactionPayload::Script(s) => HashValue::sha3_256_of(s.code()),
                    _ => HashValue::zero(),
                }
                .to_hex();

                Ok(TransactionDataView::UserTransaction {
                    sender: t.sender().to_string(),
                    signature_scheme: t.authenticator().scheme().to_string(),
                    signature: hex::encode(t.authenticator().signature_bytes()),
                    public_key: hex::encode(t.authenticator().public_key_bytes()),
                    sequence_number: t.sequence_number(),
                    max_gas_amount: t.max_gas_amount(),
                    gas_unit_price: t.gas_unit_price(),
                    gas_currency: t.gas_currency_code().to_string(),
                    expiration_time: t.expiration_time().as_secs(),
                    script_hash,
                    script: t.into_raw_transaction().into_payload().into(),
                })
            }
        };

        x.unwrap_or(TransactionDataView::UnknownTransaction {})
    }
}

impl From<AccountRole> for AccountRoleView {
    fn from(role: AccountRole) -> Self {
        match role {
            AccountRole::Unhosted => AccountRoleView::Unhosted,
            AccountRole::Unknown => AccountRoleView::Unknown,
            AccountRole::ChildVASP(child_vasp) => AccountRoleView::ChildVASP {
                parent_vasp_address: BytesView::from(&child_vasp.parent_vasp_addr().to_vec()),
            },
            AccountRole::ParentVASP(parent_vasp) => AccountRoleView::ParentVASP {
                human_name: parent_vasp.human_name().to_string(),
                base_url: parent_vasp.base_url().to_string(),
                expiration_time: parent_vasp.expiration_date(),
                compliance_key: BytesView::from(parent_vasp.compliance_public_key()),
                num_children: parent_vasp.num_children(),
            },
        }
    }
}

impl From<TransactionPayload> for ScriptView {
    fn from(value: TransactionPayload) -> Self {
        let empty_vec: Vec<TransactionArgument> = vec![];
        let empty_ty_vec: Vec<String> = vec![];
        let unknown_currency = "unknown_currency".to_string();

        let (code, args, ty_args) = match value {
            TransactionPayload::WriteSet(_) => ("genesis".to_string(), empty_vec, empty_ty_vec),
            TransactionPayload::Script(script) => (
                get_transaction_name(script.code()),
                script.args().to_vec(),
                script
                    .ty_args()
                    .iter()
                    .map(|type_tag| match type_tag {
                        TypeTag::Struct(StructTag { module, .. }) => module.to_string(),
                        tag => format!("{}", tag),
                    })
                    .collect(),
            ),
            TransactionPayload::Module(_) => {
                ("module publishing".to_string(), empty_vec, empty_ty_vec)
            }
        };

        let res = match code.as_str() {
            "peer_to_peer_with_metadata_transaction" => {
                if let [TransactionArgument::Address(receiver), TransactionArgument::U64(amount), TransactionArgument::U8Vector(metadata), TransactionArgument::U8Vector(metadata_signature)] =
                    &args[..]
                {
                    Ok(ScriptView::PeerToPeer {
                        receiver: receiver.to_string(),
                        amount: *amount,
                        currency: ty_args.get(0).unwrap_or(&unknown_currency).to_string(),
                        metadata: BytesView::from(metadata),
                        metadata_signature: BytesView::from(metadata_signature),
                    })
                } else {
                    Err(format_err!("Unable to parse PeerToPeer arguments"))
                }
            }
            "mint" => {
                if let [TransactionArgument::Address(receiver), TransactionArgument::U8Vector(auth_key_prefix), TransactionArgument::U64(amount)] =
                    &args[..]
                {
                    let currency = ty_args.get(0).unwrap_or(&unknown_currency).to_string();
                    Ok(ScriptView::Mint {
                        receiver: receiver.to_string(),
                        auth_key_prefix: BytesView::from(auth_key_prefix),
                        amount: *amount,
                        currency,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CurrencyInfoView {
    pub code: String,
    pub scaling_factor: u64,
    pub fractional_part: u64,
    pub to_lbr_exchange_rate: f32,
}

impl From<CurrencyInfoResource> for CurrencyInfoView {
    fn from(info: CurrencyInfoResource) -> CurrencyInfoView {
        CurrencyInfoView {
            code: info.currency_code().to_string(),
            scaling_factor: info.scaling_factor(),
            fractional_part: info.fractional_part(),
            to_lbr_exchange_rate: info.exchange_rate(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StateProofView {
    pub ledger_info_with_signatures: BytesView,
    pub epoch_change_proof: BytesView,
    pub ledger_consistency_proof: BytesView,
}

impl
    TryFrom<(
        LedgerInfoWithSignatures,
        EpochChangeProof,
        AccumulatorConsistencyProof,
    )> for StateProofView
{
    type Error = Error;

    fn try_from(
        (ledger_info_with_signatures, epoch_change_proof, ledger_consistency_proof): (
            LedgerInfoWithSignatures,
            EpochChangeProof,
            AccumulatorConsistencyProof,
        ),
    ) -> Result<StateProofView, Self::Error> {
        Ok(StateProofView {
            ledger_info_with_signatures: BytesView::from(&lcs::to_bytes(
                &ledger_info_with_signatures,
            )?),
            epoch_change_proof: BytesView::from(&lcs::to_bytes(&epoch_change_proof)?),
            ledger_consistency_proof: BytesView::from(&lcs::to_bytes(&ledger_consistency_proof)?),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountStateWithProofView {
    pub version: u64,
    pub blob: Option<BytesView>,
    pub proof: AccountStateProofView,
}

impl TryFrom<AccountStateWithProof> for AccountStateWithProofView {
    type Error = Error;

    fn try_from(
        account_state_with_proof: AccountStateWithProof,
    ) -> Result<AccountStateWithProofView, Error> {
        let blob = if let Some(account_blob) = account_state_with_proof.blob {
            Some(BytesView::from(&lcs::to_bytes(&account_blob)?))
        } else {
            None
        };
        Ok(AccountStateWithProofView {
            version: account_state_with_proof.version,
            blob,
            proof: AccountStateProofView::try_from(account_state_with_proof.proof)?,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountStateProofView {
    pub ledger_info_to_transaction_info_proof: BytesView,
    pub transaction_info: BytesView,
    pub transaction_info_to_account_proof: BytesView,
}

impl TryFrom<AccountStateProof> for AccountStateProofView {
    type Error = Error;

    fn try_from(account_state_proof: AccountStateProof) -> Result<AccountStateProofView, Error> {
        Ok(AccountStateProofView {
            ledger_info_to_transaction_info_proof: BytesView::from(&lcs::to_bytes(
                account_state_proof
                    .transaction_info_with_proof()
                    .ledger_info_to_transaction_info_proof(),
            )?),
            transaction_info: BytesView::from(&lcs::to_bytes(
                account_state_proof
                    .transaction_info_with_proof()
                    .transaction_info(),
            )?),
            transaction_info_to_account_proof: BytesView::from(&lcs::to_bytes(
                account_state_proof.transaction_info_to_account_proof(),
            )?),
        })
    }
}
