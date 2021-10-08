// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Address, EventKey, HashValue, HexEncodedBytes, MoveModuleBytecode, MoveModuleId, MoveResource,
    MoveResourceType, MoveScriptBytecode, MoveType, MoveValue, U64,
};

use diem_crypto::{
    ed25519::{self, Ed25519PublicKey},
    multi_ed25519::{self, MultiEd25519PublicKey},
    validatable::Validatable,
};
use diem_types::{
    account_address::AccountAddress,
    block_metadata::BlockMetadata,
    contract_event::ContractEvent,
    transaction::{
        authenticator::{AccountAuthenticator, TransactionAuthenticator},
        default_protocol::TransactionWithProof,
        Script, SignedTransaction, TransactionInfoTrait,
    },
};
use move_core_types::identifier::Identifier;
use resource_viewer::AnnotatedMoveValue;

use serde::Serialize;
use std::{
    boxed::Box,
    convert::{From, Into, TryFrom, TryInto},
    fmt,
    str::FromStr,
};

#[derive(Clone, Debug)]
pub enum TransactionData<T: TransactionInfoTrait> {
    OnChain(TransactionOnChainData<T>),
    Pending(Box<SignedTransaction>),
}

impl<T: TransactionInfoTrait> From<TransactionOnChainData<T>> for TransactionData<T> {
    fn from(txn: TransactionOnChainData<T>) -> Self {
        Self::OnChain(txn)
    }
}

impl<T: TransactionInfoTrait> From<SignedTransaction> for TransactionData<T> {
    fn from(txn: SignedTransaction) -> Self {
        Self::Pending(Box::new(txn))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionOnChainData<T: TransactionInfoTrait> {
    pub version: u64,
    pub transaction: diem_types::transaction::Transaction,
    pub info: T,
    pub events: Vec<ContractEvent>,
}

impl From<TransactionWithProof>
    for TransactionOnChainData<diem_types::transaction::TransactionInfo>
{
    fn from(txn: TransactionWithProof) -> Self {
        Self {
            version: txn.version,
            transaction: txn.transaction,
            info: txn.proof.transaction_info,
            events: txn.events.unwrap_or_default(),
        }
    }
}

impl<T: TransactionInfoTrait>
    From<(
        u64,
        diem_types::transaction::Transaction,
        T,
        Vec<ContractEvent>,
    )> for TransactionOnChainData<T>
{
    fn from(
        (version, transaction, info, events): (
            u64,
            diem_types::transaction::Transaction,
            T,
            Vec<ContractEvent>,
        ),
    ) -> Self {
        Self {
            version,
            transaction,
            info,
            events,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Transaction {
    PendingTransaction(PendingTransaction),
    UserTransaction(Box<UserTransaction>),
    GenesisTransaction(GenesisTransaction),
    BlockMetadataTransaction(BlockMetadataTransaction),
}

impl From<(SignedTransaction, TransactionPayload)> for Transaction {
    fn from((txn, payload): (SignedTransaction, TransactionPayload)) -> Self {
        Transaction::PendingTransaction(PendingTransaction {
            sender: txn.sender().into(),
            sequence_number: txn.sequence_number().into(),
            max_gas_amount: txn.max_gas_amount().into(),
            gas_unit_price: txn.gas_unit_price().into(),
            gas_currency_code: txn.gas_currency_code().to_owned(),
            expiration_timestamp_secs: txn.expiration_timestamp_secs().into(),
            signature: txn.authenticator().into(),
            hash: txn.committed_hash().into(),
            payload,
        })
    }
}

impl<T: TransactionInfoTrait> From<(u64, &SignedTransaction, &T, TransactionPayload, Vec<Event>)>
    for Transaction
{
    fn from(
        (version, txn, info, payload, events): (
            u64,
            &SignedTransaction,
            &T,
            TransactionPayload,
            Vec<Event>,
        ),
    ) -> Self {
        Transaction::UserTransaction(Box::new(UserTransaction {
            info: (version, info).into(),
            sender: txn.sender().into(),
            sequence_number: txn.sequence_number().into(),
            max_gas_amount: txn.max_gas_amount().into(),
            gas_unit_price: txn.gas_unit_price().into(),
            gas_currency_code: txn.gas_currency_code().to_owned(),
            expiration_timestamp_secs: txn.expiration_timestamp_secs().into(),
            signature: txn.authenticator().into(),
            payload,
            events,
        }))
    }
}

impl<T: TransactionInfoTrait> From<(u64, &T, WriteSetPayload, Vec<Event>)> for Transaction {
    fn from((version, info, payload, events): (u64, &T, WriteSetPayload, Vec<Event>)) -> Self {
        Transaction::GenesisTransaction(GenesisTransaction {
            info: (version, info).into(),
            payload: GenesisPayload::WriteSetPayload(payload),
            events,
        })
    }
}

impl<T: TransactionInfoTrait> From<(u64, &BlockMetadata, &T)> for Transaction {
    fn from((version, txn, info): (u64, &BlockMetadata, &T)) -> Self {
        Transaction::BlockMetadataTransaction(BlockMetadataTransaction {
            info: (version, info).into(),
            id: txn.id().into(),
            round: txn.round().into(),
            previous_block_votes: txn
                .previous_block_votes()
                .clone()
                .iter()
                .map(|a| (*a).into())
                .collect(),
            proposer: txn.proposer().into(),
            timestamp: txn.timestamp_usec().into(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TransactionInfo {
    pub version: U64,
    pub hash: HashValue,
    pub state_root_hash: HashValue,
    pub event_root_hash: HashValue,
    pub gas_used: U64,
    pub success: bool,
}

impl<T: TransactionInfoTrait> From<(u64, &T)> for TransactionInfo {
    fn from((version, info): (u64, &T)) -> Self {
        Self {
            version: version.into(),
            hash: info.transaction_hash().into(),
            state_root_hash: info.state_root_hash().into(),
            event_root_hash: info.event_root_hash().into(),
            gas_used: info.gas_used().into(),
            success: info.status().is_success(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PendingTransaction {
    pub hash: HashValue,
    pub sender: Address,
    pub sequence_number: U64,
    pub max_gas_amount: U64,
    pub gas_unit_price: U64,
    pub gas_currency_code: String,
    pub expiration_timestamp_secs: U64,
    pub payload: TransactionPayload,
    pub signature: TransactionSignature,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UserTransaction {
    #[serde(flatten)]
    pub info: TransactionInfo,
    pub sender: Address,
    pub sequence_number: U64,
    pub max_gas_amount: U64,
    pub gas_unit_price: U64,
    pub gas_currency_code: String,
    pub expiration_timestamp_secs: U64,
    pub payload: TransactionPayload,
    pub signature: TransactionSignature,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GenesisTransaction {
    #[serde(flatten)]
    pub info: TransactionInfo,
    pub payload: GenesisPayload,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BlockMetadataTransaction {
    #[serde(flatten)]
    pub info: TransactionInfo,
    pub id: HashValue,
    pub round: U64,
    pub previous_block_votes: Vec<Address>,
    pub proposer: Address,
    pub timestamp: U64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Event {
    pub key: EventKey,
    pub sequence_number: U64,
    #[serde(rename = "type")]
    pub typ: MoveType,
    pub data: MoveValue,
}

impl From<(&ContractEvent, AnnotatedMoveValue)> for Event {
    fn from((event, data): (&ContractEvent, AnnotatedMoveValue)) -> Self {
        match event {
            ContractEvent::V0(v0) => Self {
                key: (*v0.key()).into(),
                sequence_number: v0.sequence_number().into(),
                typ: v0.type_tag().clone().into(),
                data: data.into(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GenesisPayload {
    WriteSetPayload(WriteSetPayload),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransactionPayload {
    ScriptFunctionPayload {
        module: MoveModuleId,
        function: Identifier,
        type_arguments: Vec<MoveType>,
        arguments: Vec<MoveValue>,
    },
    ScriptPayload(ScriptPayload),
    ModulePayload(MoveModuleBytecode),
    WriteSetPayload(WriteSetPayload),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ScriptPayload {
    pub code: MoveScriptBytecode,
    pub type_arguments: Vec<MoveType>,
    pub arguments: Vec<MoveValue>,
}

impl TryFrom<&Script> for ScriptPayload {
    type Error = anyhow::Error;

    fn try_from(script: &Script) -> anyhow::Result<Self> {
        Ok(Self {
            code: script.code().try_into()?,
            type_arguments: script
                .ty_args()
                .iter()
                .map(|arg| arg.clone().into())
                .collect(),
            arguments: script.args().iter().map(|arg| arg.clone().into()).collect(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WriteSetPayload {
    ScriptWriteSet {
        execute_as: Address,
        script: ScriptPayload,
    },
    DirectWriteSet {
        changes: Vec<WriteSetChange>,
        events: Vec<Event>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WriteSetChange {
    DeleteModule {
        address: Address,
        module: MoveModuleId,
    },
    DeleteResource {
        address: Address,
        resource: MoveResourceType,
    },
    WriteModule {
        address: Address,
        data: MoveModuleBytecode,
    },
    WriteResource {
        address: Address,
        data: MoveResource,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransactionSignature {
    Ed25519Signature(Ed25519Signature),
    MultiEd25519Signature(MultiEd25519Signature),
    MultiAgentSignature(MultiAgentSignature),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Ed25519Signature {
    public_key: HexEncodedBytes,
    signature: HexEncodedBytes,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MultiEd25519Signature {
    signatures: Vec<Ed25519Signature>,
    threshold: u8,
    bitmap: HexEncodedBytes,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountSignature {
    Ed25519Signature(Ed25519Signature),
    MultiEd25519Signature(MultiEd25519Signature),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MultiAgentSignature {
    sender: AccountSignature,
    secondary_signer_addresses: Vec<Address>,
    secondary_signers: Vec<AccountSignature>,
}

impl From<(&Validatable<Ed25519PublicKey>, &ed25519::Ed25519Signature)> for Ed25519Signature {
    fn from((pk, sig): (&Validatable<Ed25519PublicKey>, &ed25519::Ed25519Signature)) -> Self {
        Self {
            public_key: pk.unvalidated().to_bytes().to_vec().into(),
            signature: sig.to_bytes().to_vec().into(),
        }
    }
}

impl From<(&Ed25519PublicKey, &ed25519::Ed25519Signature)> for Ed25519Signature {
    fn from((pk, sig): (&Ed25519PublicKey, &ed25519::Ed25519Signature)) -> Self {
        Self {
            public_key: pk.to_bytes().to_vec().into(),
            signature: sig.to_bytes().to_vec().into(),
        }
    }
}

impl
    From<(
        &MultiEd25519PublicKey,
        &multi_ed25519::MultiEd25519Signature,
    )> for MultiEd25519Signature
{
    fn from(
        (pk, sig): (
            &MultiEd25519PublicKey,
            &multi_ed25519::MultiEd25519Signature,
        ),
    ) -> Self {
        Self {
            signatures: pk
                .public_keys()
                .iter()
                .zip(sig.signatures().iter())
                .map(|(k, s)| (k, s).into())
                .collect(),
            threshold: *pk.threshold(),
            bitmap: sig.bitmap().to_vec().into(),
        }
    }
}

impl From<&AccountAuthenticator> for AccountSignature {
    fn from(auth: &AccountAuthenticator) -> Self {
        use AccountAuthenticator::*;
        match auth {
            Ed25519 {
                public_key,
                signature,
            } => Self::Ed25519Signature((public_key, signature).into()),
            MultiEd25519 {
                public_key,
                signature,
            } => Self::MultiEd25519Signature((public_key, signature).into()),
        }
    }
}

impl
    From<(
        &AccountAuthenticator,
        &Vec<AccountAddress>,
        &Vec<AccountAuthenticator>,
    )> for MultiAgentSignature
{
    fn from(
        (sender, addresses, signers): (
            &AccountAuthenticator,
            &Vec<AccountAddress>,
            &Vec<AccountAuthenticator>,
        ),
    ) -> Self {
        Self {
            sender: sender.into(),
            secondary_signer_addresses: addresses.iter().map(|address| (*address).into()).collect(),
            secondary_signers: signers.iter().map(|s| s.into()).collect(),
        }
    }
}

impl From<TransactionAuthenticator> for TransactionSignature {
    fn from(auth: TransactionAuthenticator) -> Self {
        use TransactionAuthenticator::*;
        match &auth {
            Ed25519 {
                public_key,
                signature,
            } => Self::Ed25519Signature((public_key, signature).into()),
            MultiEd25519 {
                public_key,
                signature,
            } => Self::MultiEd25519Signature((public_key, signature).into()),
            MultiAgent {
                sender,
                secondary_signer_addresses,
                secondary_signers,
            } => Self::MultiAgentSignature(
                (sender, secondary_signer_addresses, secondary_signers).into(),
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TransactionId {
    Hash(HashValue),
    Version(u64),
}

impl FromStr for TransactionId {
    type Err = anyhow::Error;

    fn from_str(hash_or_version: &str) -> Result<Self, anyhow::Error> {
        let id = match hash_or_version.parse::<u64>() {
            Ok(version) => TransactionId::Version(version),
            Err(_) => TransactionId::Hash(hash_or_version.parse()?),
        };
        Ok(id)
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Hash(h) => write!(f, "hash({})", h),
            Self::Version(v) => write!(f, "version({})", v),
        }
    }
}
