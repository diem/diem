// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error, Result};
use diem_crypto::HashValue;
use diem_types::{
    account_config::{
        AccountResource, AccountRole, AdminTransactionEvent, BalanceResource, BaseUrlRotationEvent,
        BurnEvent, CancelBurnEvent, ComplianceKeyRotationEvent, CreateAccountEvent,
        CurrencyInfoResource, DesignatedDealerPreburns, FreezingBit, MintEvent, NewBlockEvent,
        NewEpochEvent, PreburnEvent, ReceivedMintEvent, ReceivedPaymentEvent, SentPaymentEvent,
        ToXDXExchangeRateUpdateEvent,
    },
    account_state_blob::AccountStateWithProof,
    contract_event::ContractEvent,
    epoch_change::EpochChangeProof,
    event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
    proof::{AccountStateProof, AccumulatorConsistencyProof},
};
use hex::FromHex;
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::TypeTag,
    move_resource::MoveResource,
};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

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
#[serde(tag = "type")]
pub enum AccountRoleView {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "child_vasp")]
    ChildVASP { parent_vasp_address: AccountAddress },
    #[serde(rename = "parent_vasp")]
    ParentVASP {
        human_name: String,
        base_url: String,
        expiration_time: u64,
        compliance_key: BytesView,
        num_children: u64,
        compliance_key_rotation_events_key: EventKey,
        base_url_rotation_events_key: EventKey,
    },
    #[serde(rename = "designated_dealer")]
    DesignatedDealer {
        human_name: String,
        base_url: String,
        expiration_time: u64,
        compliance_key: BytesView,
        preburn_balances: Vec<AmountView>,
        received_mint_events_key: EventKey,
        compliance_key_rotation_events_key: EventKey,
        base_url_rotation_events_key: EventKey,
        preburn_queues: Option<Vec<PreburnQueueView>>,
    },
}

impl AccountRoleView {
    pub(crate) fn convert_preburn_balances(
        preburn_balances: DesignatedDealerPreburns,
    ) -> (Vec<AmountView>, Option<Vec<PreburnQueueView>>) {
        match preburn_balances {
            DesignatedDealerPreburns::Preburn(preburn_balances) => {
                let preburn_balances: Vec<_> = preburn_balances
                    .iter()
                    .map(|(currency_code, balance)| {
                        AmountView::new(balance.coin(), &currency_code.as_str())
                    })
                    .collect();
                let preburn_queues = preburn_balances
                    .iter()
                    .cloned()
                    .map(|amt_view| {
                        PreburnQueueView::new(
                            amt_view.currency.clone(),
                            vec![PreburnWithMetadataView {
                                preburn: amt_view,
                                metadata: None,
                            }],
                        )
                    })
                    .collect();
                (preburn_balances, Some(preburn_queues))
            }
            DesignatedDealerPreburns::PreburnQueue(preburn_queues) => {
                let preburn_balances = preburn_queues
                    .iter()
                    .map(|(currency_code, preburns)| {
                        let total_balance =
                            preburns.preburns().iter().fold(0, |acc: u64, preburn| {
                                acc.checked_add(preburn.preburn().coin()).unwrap()
                            });
                        AmountView::new(total_balance, &currency_code.as_str())
                    })
                    .collect();
                let preburn_queues = preburn_queues
                    .into_iter()
                    .map(|(currency_code, preburns)| {
                        PreburnQueueView::new(
                            currency_code.to_string(),
                            preburns
                                .preburns()
                                .iter()
                                .map(|preburn| PreburnWithMetadataView {
                                    preburn: AmountView::new(
                                        preburn.preburn().coin(),
                                        &currency_code.as_str(),
                                    ),
                                    metadata: Some(BytesView::new(preburn.metadata())),
                                })
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect();
                (preburn_balances, Some(preburn_queues))
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct AccountView {
    pub address: AccountAddress,
    pub balances: Vec<AmountView>,
    pub sequence_number: u64,
    pub authentication_key: BytesView,
    pub sent_events_key: EventKey,
    pub received_events_key: EventKey,
    pub delegated_key_rotation_capability: bool,
    pub delegated_withdrawal_capability: bool,
    pub is_frozen: bool,
    pub role: AccountRoleView,
    // the transaction version of the account data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u64>,
}

impl AccountView {
    pub fn new(
        address: AccountAddress,
        account: &AccountResource,
        balances: BTreeMap<Identifier, BalanceResource>,
        account_role: AccountRole,
        freezing_bit: FreezingBit,
        version: u64,
    ) -> Self {
        Self {
            address,
            balances: balances
                .iter()
                .map(|(currency_code, balance)| {
                    AmountView::new(balance.coin(), &currency_code.as_str())
                })
                .collect(),
            sequence_number: account.sequence_number(),
            authentication_key: BytesView::from(account.authentication_key()),
            sent_events_key: *account.sent_events().key(),
            received_events_key: *account.received_events().key(),
            delegated_key_rotation_capability: account.has_delegated_key_rotation_capability(),
            delegated_withdrawal_capability: account.has_delegated_withdrawal_capability(),
            is_frozen: freezing_bit.is_frozen(),
            role: AccountRoleView::from(account_role),
            version: Some(version),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct PreburnQueueView {
    pub currency: String,
    pub preburns: Vec<PreburnWithMetadataView>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct PreburnWithMetadataView {
    pub preburn: AmountView,
    pub metadata: Option<BytesView>,
}

impl PreburnQueueView {
    pub fn new(currency: String, preburns: Vec<PreburnWithMetadataView>) -> Self {
        Self { currency, preburns }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventView {
    pub key: EventKey,
    pub sequence_number: u64,
    pub transaction_version: u64,
    pub data: EventDataView,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventWithProofView {
    pub event_with_proof: BytesView,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum EventDataView {
    #[serde(rename = "burn")]
    Burn {
        amount: AmountView,
        preburn_address: AccountAddress,
    },
    #[serde(rename = "cancelburn")]
    CancelBurn {
        amount: AmountView,
        preburn_address: AccountAddress,
    },
    #[serde(rename = "mint")]
    Mint { amount: AmountView },
    #[serde(rename = "to_xdx_exchange_rate_update")]
    ToXDXExchangeRateUpdate {
        currency_code: String,
        new_to_xdx_exchange_rate: f32,
    },
    #[serde(rename = "preburn")]
    Preburn {
        amount: AmountView,
        preburn_address: AccountAddress,
    },
    #[serde(rename = "receivedpayment")]
    ReceivedPayment {
        amount: AmountView,
        sender: AccountAddress,
        receiver: AccountAddress,
        metadata: BytesView,
    },
    #[serde(rename = "sentpayment")]
    SentPayment {
        amount: AmountView,
        receiver: AccountAddress,
        sender: AccountAddress,
        metadata: BytesView,
    },
    #[serde(rename = "admintransaction")]
    AdminTransaction { committed_timestamp_secs: u64 },
    #[serde(rename = "newepoch")]
    NewEpoch { epoch: u64 },
    #[serde(rename = "newblock")]
    NewBlock {
        round: u64,
        proposer: AccountAddress,
        proposed_time: u64,
    },
    #[serde(rename = "receivedmint")]
    ReceivedMint {
        amount: AmountView,
        destination_address: AccountAddress,
    },
    #[serde(rename = "compliancekeyrotation")]
    ComplianceKeyRotation {
        new_compliance_public_key: BytesView,
        time_rotated_seconds: u64,
    },
    #[serde(rename = "baseurlrotation")]
    BaseUrlRotation {
        new_base_url: String,
        time_rotated_seconds: u64,
    },
    #[serde(rename = "createaccount")]
    CreateAccount {
        created_address: AccountAddress,
        role_id: u64,
    },
    #[serde(rename = "unknown")]
    Unknown {},
}

impl TryFrom<ContractEvent> for EventDataView {
    type Error = Error;

    fn try_from(event: ContractEvent) -> Result<Self> {
        let data = if event.type_tag() == &TypeTag::Struct(ReceivedPaymentEvent::struct_tag()) {
            let received_event = ReceivedPaymentEvent::try_from(&event)?;
            let amount_view = AmountView::new(
                received_event.amount(),
                received_event.currency_code().as_str(),
            );
            EventDataView::ReceivedPayment {
                amount: amount_view,
                sender: received_event.sender(),
                receiver: event.key().get_creator_address(),
                metadata: BytesView::from(received_event.metadata()),
            }
        } else if event.type_tag() == &TypeTag::Struct(SentPaymentEvent::struct_tag()) {
            let sent_event = SentPaymentEvent::try_from(&event)?;
            let amount_view =
                AmountView::new(sent_event.amount(), sent_event.currency_code().as_str());
            EventDataView::SentPayment {
                amount: amount_view,
                receiver: sent_event.receiver(),
                sender: event.key().get_creator_address(),
                metadata: BytesView::from(sent_event.metadata()),
            }
        } else if event.type_tag() == &TypeTag::Struct(PreburnEvent::struct_tag()) {
            let preburn_event = PreburnEvent::try_from(&event)?;
            let amount_view = AmountView::new(
                preburn_event.amount(),
                preburn_event.currency_code().as_str(),
            );
            EventDataView::Preburn {
                amount: amount_view,
                preburn_address: preburn_event.preburn_address(),
            }
        } else if event.type_tag() == &TypeTag::Struct(BurnEvent::struct_tag()) {
            let burn_event = BurnEvent::try_from(&event)?;
            let amount_view =
                AmountView::new(burn_event.amount(), burn_event.currency_code().as_str());
            EventDataView::Burn {
                amount: amount_view,
                preburn_address: burn_event.preburn_address(),
            }
        } else if event.type_tag() == &TypeTag::Struct(CancelBurnEvent::struct_tag()) {
            let cancel_burn_event = CancelBurnEvent::try_from(&event)?;
            let amount_view = AmountView::new(
                cancel_burn_event.amount(),
                cancel_burn_event.currency_code().as_str(),
            );
            EventDataView::CancelBurn {
                amount: amount_view,
                preburn_address: cancel_burn_event.preburn_address(),
            }
        } else if event.type_tag() == &TypeTag::Struct(ToXDXExchangeRateUpdateEvent::struct_tag()) {
            let update_event = ToXDXExchangeRateUpdateEvent::try_from(&event)?;
            EventDataView::ToXDXExchangeRateUpdate {
                currency_code: update_event.currency_code().to_string(),
                new_to_xdx_exchange_rate: update_event.new_to_xdx_exchange_rate(),
            }
        } else if event.type_tag() == &TypeTag::Struct(MintEvent::struct_tag()) {
            let mint_event = MintEvent::try_from(&event)?;
            let amount_view =
                AmountView::new(mint_event.amount(), mint_event.currency_code().as_str());
            EventDataView::Mint {
                amount: amount_view,
            }
        } else if event.type_tag() == &TypeTag::Struct(ReceivedMintEvent::struct_tag()) {
            let received_mint_event = ReceivedMintEvent::try_from(&event)?;
            let amount_view = AmountView::new(
                received_mint_event.amount(),
                received_mint_event.currency_code().as_str(),
            );
            EventDataView::ReceivedMint {
                amount: amount_view,
                destination_address: received_mint_event.destination_address(),
            }
        } else if event.type_tag() == &TypeTag::Struct(ComplianceKeyRotationEvent::struct_tag()) {
            let rotation_event = ComplianceKeyRotationEvent::try_from(&event)?;
            EventDataView::ComplianceKeyRotation {
                new_compliance_public_key: rotation_event.new_compliance_public_key().into(),
                time_rotated_seconds: rotation_event.time_rotated_seconds(),
            }
        } else if event.type_tag() == &TypeTag::Struct(BaseUrlRotationEvent::struct_tag()) {
            let rotation_event = BaseUrlRotationEvent::try_from(&event)?;
            String::from_utf8(rotation_event.new_base_url().to_vec())
                .map(|new_base_url| EventDataView::BaseUrlRotation {
                    new_base_url,
                    time_rotated_seconds: rotation_event.time_rotated_seconds(),
                })
                .map_err(|_| format_err!("Unable to parse BaseUrlRotationEvent"))?
        } else if event.type_tag() == &TypeTag::Struct(NewBlockEvent::struct_tag()) {
            let new_block_event = NewBlockEvent::try_from(&event)?;
            EventDataView::NewBlock {
                proposer: new_block_event.proposer(),
                round: new_block_event.round(),
                proposed_time: new_block_event.proposed_time(),
            }
        } else if event.type_tag() == &TypeTag::Struct(NewEpochEvent::struct_tag()) {
            let new_epoch_event = NewEpochEvent::try_from(&event)?;
            EventDataView::NewEpoch {
                epoch: new_epoch_event.epoch(),
            }
        } else if event.type_tag() == &TypeTag::Struct(CreateAccountEvent::struct_tag()) {
            let create_account_event = CreateAccountEvent::try_from(&event)?;
            let created_address = create_account_event.created();
            let role_id = create_account_event.role_id();
            EventDataView::CreateAccount {
                created_address,
                role_id,
            }
        } else if event.type_tag() == &TypeTag::Struct(AdminTransactionEvent::struct_tag()) {
            let admin_transaction_event = AdminTransactionEvent::try_from(&event)?;
            EventDataView::AdminTransaction {
                committed_timestamp_secs: admin_transaction_event.committed_timestamp_secs(),
            }
        } else {
            EventDataView::Unknown {}
        };

        Ok(data)
    }
}

impl TryFrom<(u64, ContractEvent)> for EventView {
    type Error = Error;

    fn try_from((txn_version, event): (u64, ContractEvent)) -> Result<Self> {
        Ok(EventView {
            key: *event.key(),
            sequence_number: event.sequence_number(),
            transaction_version: txn_version,
            data: event.try_into()?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MetadataView {
    pub version: u64,
    pub accumulator_root_hash: HashValue,
    pub timestamp: u64,
    pub chain_id: u8,
    pub script_hash_allow_list: Option<Vec<HashValue>>,
    pub module_publishing_allowed: Option<bool>,
    pub diem_version: Option<u64>,
    pub dual_attestation_limit: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BytesView(Box<[u8]>);

impl BytesView {
    pub fn new<T: Into<Box<[u8]>>>(bytes: T) -> Self {
        Self(bytes.into())
    }

    pub fn into_inner(self) -> Box<[u8]> {
        self.0
    }

    pub fn inner(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Deref for BytesView {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::convert::AsRef<[u8]> for BytesView {
    fn as_ref(&self) -> &[u8] {
        self.inner()
    }
}

impl std::fmt::Display for BytesView {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte in self.inner() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl From<&[u8]> for BytesView {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.into())
    }
}

impl From<Vec<u8>> for BytesView {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes.into_boxed_slice())
    }
}

impl<'de> Deserialize<'de> for BytesView {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        <Vec<u8>>::from_hex(s)
            .map_err(D::Error::custom)
            .map(Into::into)
    }
}

impl Serialize for BytesView {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        hex::encode(self).serialize(serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MoveAbortExplanationView {
    pub category: String,
    pub category_description: String,
    pub reason: String,
    pub reason_description: String,
}

impl std::fmt::Display for MoveAbortExplanationView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error Category: {}", self.category)?;
        writeln!(f, "\tCategory Description: {}", self.category_description)?;
        writeln!(f, "Error Reason: {}", self.reason)?;
        writeln!(f, "\tReason Description: {}", self.reason_description)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum VMStatusView {
    Executed,
    OutOfGas,
    MoveAbort {
        location: String,
        abort_code: u64,
        explanation: Option<MoveAbortExplanationView>,
    },
    ExecutionFailure {
        location: String,
        function_index: u16,
        code_offset: u16,
    },
    MiscellaneousError,
    VerificationError,
    DeserializationError,
    PublishingFailure,
}

impl std::fmt::Display for VMStatusView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMStatusView::Executed => write!(f, "Executed"),
            VMStatusView::OutOfGas => write!(f, "Out of Gas"),
            VMStatusView::MoveAbort {
                location,
                abort_code,
                explanation,
            } => {
                write!(f, "Move Abort: {} at {}", abort_code, location)?;
                if let Some(explanation) = explanation {
                    write!(f, "\nExplanation:\n{}", explanation)?
                }
                Ok(())
            }
            VMStatusView::ExecutionFailure {
                location,
                function_index,
                code_offset,
            } => write!(
                f,
                "Execution failure: {} {} {}",
                location, function_index, code_offset
            ),
            VMStatusView::MiscellaneousError => write!(f, "Miscellaneous Error"),
            VMStatusView::VerificationError => write!(f, "Verification Error"),
            VMStatusView::DeserializationError => write!(f, "Deserialization Error"),
            VMStatusView::PublishingFailure => write!(f, "Publishing Failure"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TransactionView {
    pub version: u64,
    pub transaction: TransactionDataView,
    pub hash: HashValue,
    pub bytes: BytesView,
    pub events: Vec<EventView>,
    pub vm_status: VMStatusView,
    pub gas_used: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TransactionsWithProofsView {
    pub serialized_transactions: Vec<BytesView>,
    pub proofs: TransactionsProofsView,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TransactionsProofsView {
    pub ledger_info_to_transaction_infos_proof: BytesView,
    pub transaction_infos: BytesView,
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
        sender: AccountAddress,
        signature_scheme: String,
        signature: BytesView,
        public_key: BytesView,
        sequence_number: u64,
        chain_id: u8,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_currency: String,
        expiration_timestamp_secs: u64,
        script_hash: HashValue,
        script_bytes: BytesView,
        script: ScriptView,
    },
    #[serde(rename = "unknown")]
    UnknownTransaction {},
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct ScriptView {
    // script name / type
    pub r#type: String,

    // script code bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<BytesView>,
    // script arguments, converted into string with type information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,
    // script function arguments, converted into hex encoded BCS bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments_bcs: Option<Vec<BytesView>>,
    // script type arguments, converted into string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_arguments: Option<Vec<String>>,

    // the following fields are legacy fields: maybe removed in the future
    // please move to use above fields

    // peer_to_peer_transaction, other known script name or unknown
    // because of a bug, we never rendered mint_transaction
    // this is deprecated, please switch to use field `name` which is script name

    // peer_to_peer_transaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<AccountAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BytesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_signature: Option<BytesView>,

    // Script functions
    // The address that the module is published under
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_address: Option<AccountAddress>,
    // The name of the module that the called function is defined in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_name: Option<String>,
    // The (unqualified) name of the function being called.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_name: Option<String>,
}

impl ScriptView {
    pub fn unknown() -> Self {
        ScriptView {
            r#type: "unknown".to_string(),
            ..Default::default()
        }
    }
}

impl From<AccountRole> for AccountRoleView {
    fn from(role: AccountRole) -> Self {
        match role {
            AccountRole::Unknown => AccountRoleView::Unknown,
            AccountRole::ChildVASP(child_vasp) => AccountRoleView::ChildVASP {
                parent_vasp_address: child_vasp.parent_vasp_addr(),
            },
            AccountRole::ParentVASP { vasp, credential } => AccountRoleView::ParentVASP {
                human_name: credential.human_name().to_string(),
                base_url: credential.base_url().to_string(),
                expiration_time: credential.expiration_date(),
                compliance_key: BytesView::from(credential.compliance_public_key()),
                num_children: vasp.num_children(),
                compliance_key_rotation_events_key: *credential
                    .compliance_key_rotation_events()
                    .key(),
                base_url_rotation_events_key: *credential.base_url_rotation_events().key(),
            },
            AccountRole::DesignatedDealer {
                dd_credential,
                preburn_balances,
                designated_dealer,
            } => {
                let (preburn_balances, preburn_queues) =
                    AccountRoleView::convert_preburn_balances(preburn_balances);
                AccountRoleView::DesignatedDealer {
                    human_name: dd_credential.human_name().to_string(),
                    base_url: dd_credential.base_url().to_string(),
                    expiration_time: dd_credential.expiration_date(),
                    compliance_key: BytesView::from(dd_credential.compliance_public_key()),
                    preburn_balances,
                    received_mint_events_key: *designated_dealer.received_mint_events().key(),
                    compliance_key_rotation_events_key: *dd_credential
                        .compliance_key_rotation_events()
                        .key(),
                    base_url_rotation_events_key: *dd_credential.base_url_rotation_events().key(),
                    preburn_queues,
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CurrencyInfoView {
    pub code: String,
    pub scaling_factor: u64,
    pub fractional_part: u64,
    pub to_xdx_exchange_rate: f32,
    pub mint_events_key: EventKey,
    pub burn_events_key: EventKey,
    pub preburn_events_key: EventKey,
    pub cancel_burn_events_key: EventKey,
    pub exchange_rate_update_events_key: EventKey,
}

impl From<&CurrencyInfoResource> for CurrencyInfoView {
    fn from(info: &CurrencyInfoResource) -> CurrencyInfoView {
        CurrencyInfoView {
            code: info.currency_code().to_string(),
            scaling_factor: info.scaling_factor(),
            fractional_part: info.fractional_part(),
            to_xdx_exchange_rate: info.exchange_rate(),
            mint_events_key: *info.mint_events().key(),
            burn_events_key: *info.burn_events().key(),
            preburn_events_key: *info.preburn_events().key(),
            cancel_burn_events_key: *info.cancel_burn_events().key(),
            exchange_rate_update_events_key: *info.exchange_rate_update_events().key(),
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
            ledger_info_with_signatures: BytesView::new(bcs::to_bytes(
                &ledger_info_with_signatures,
            )?),
            epoch_change_proof: BytesView::new(bcs::to_bytes(&epoch_change_proof)?),
            ledger_consistency_proof: BytesView::new(bcs::to_bytes(&ledger_consistency_proof)?),
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
            Some(BytesView::new(bcs::to_bytes(&account_blob)?))
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
            ledger_info_to_transaction_info_proof: BytesView::new(bcs::to_bytes(
                account_state_proof
                    .transaction_info_with_proof()
                    .ledger_info_to_transaction_info_proof(),
            )?),
            transaction_info: BytesView::new(bcs::to_bytes(
                account_state_proof
                    .transaction_info_with_proof()
                    .transaction_info(),
            )?),
            transaction_info_to_account_proof: BytesView::new(bcs::to_bytes(
                account_state_proof.transaction_info_to_account_proof(),
            )?),
        })
    }
}
