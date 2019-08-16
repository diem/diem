// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]

use crate::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    contract_event::ContractEvent,
    ledger_info::LedgerInfo,
    proof::{
        get_accumulator_root_hash, verify_signed_transaction, verify_transaction_list,
        AccumulatorProof, SignedTransactionProof,
    },
    proto::events::{EventsForVersions, EventsList},
    vm_error::VMStatus,
    write_set::WriteSet,
};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
    SimpleSerializer,
};
use crypto::{
    ed25519::*,
    hash::{
        CryptoHash, CryptoHasher, EventAccumulatorHasher, RawTransactionHasher,
        SignedTransactionHasher, TransactionInfoHasher,
    },
    traits::*,
    HashValue,
};
use failure::prelude::*;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto, IntoProtoBytes};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom, fmt, time::Duration};

mod module;
mod program;
mod script;
mod transaction_argument;

#[cfg(test)]
mod unit_tests;

pub use module::Module;
pub use program::{Program, SCRIPT_HASH_LENGTH};
use protobuf::well_known_types::UInt64Value;
pub use script::Script;
use std::ops::Deref;
pub use transaction_argument::{parse_as_transaction_argument, TransactionArgument};

pub type Version = u64; // Height - also used for MVCC in StateDB

pub const MAX_TRANSACTION_SIZE_IN_BYTES: usize = 4096;

/// RawTransaction is the portion of a transaction that a client signs
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RawTransaction {
    /// Sender's address.
    sender: AccountAddress,
    // Sequence number of this transaction corresponding to sender's account.
    sequence_number: u64,
    // The transaction script to execute.
    payload: TransactionPayload,

    // Maximal total gas specified by wallet to spend for this transaction.
    max_gas_amount: u64,
    // Maximal price can be paid per gas.
    gas_unit_price: u64,
    // Expiration time for this transaction.  If storage is queried and
    // the time returned is greater than or equal to this time and this
    // transaction has not been included, you can be certain that it will
    // never be included.
    // A transaction that doesn't expire is represented by a very large value like
    // u64::max_value().
    expiration_time: Duration,
}

impl RawTransaction {
    /// Create a new `RawTransaction` with a program.
    ///
    /// Almost all transactions are program transactions. See `new_write_set` for write-set
    /// transactions.
    pub fn new(
        sender: AccountAddress,
        sequence_number: u64,
        program: Program,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_time: Duration,
    ) -> Self {
        RawTransaction {
            sender,
            sequence_number,
            payload: TransactionPayload::Program(program),
            max_gas_amount,
            gas_unit_price,
            expiration_time,
        }
    }

    /// Create a new `RawTransaction` with a script.
    ///
    /// A script transaction contains only code to execute. No publishing is allowed in scripts.
    pub fn new_script(
        sender: AccountAddress,
        sequence_number: u64,
        script: Script,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_time: Duration,
    ) -> Self {
        RawTransaction {
            sender,
            sequence_number,
            payload: TransactionPayload::Script(script),
            max_gas_amount,
            gas_unit_price,
            expiration_time,
        }
    }

    /// Create a new `RawTransaction` with a module to publish.
    ///
    /// A module transaction is the only way to publish code. Only one module per transaction
    /// can be published.
    pub fn new_module(
        sender: AccountAddress,
        sequence_number: u64,
        module: Module,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_time: Duration,
    ) -> Self {
        RawTransaction {
            sender,
            sequence_number,
            payload: TransactionPayload::Module(module),
            max_gas_amount,
            gas_unit_price,
            expiration_time,
        }
    }

    pub fn new_write_set(
        sender: AccountAddress,
        sequence_number: u64,
        write_set: WriteSet,
    ) -> Self {
        RawTransaction {
            sender,
            sequence_number,
            payload: TransactionPayload::WriteSet(write_set),
            // Since write-set transactions bypass the VM, these fields aren't relevant.
            max_gas_amount: 0,
            gas_unit_price: 0,
            // Write-set transactions are special and important and shouldn't expire.
            expiration_time: Duration::new(u64::max_value(), 0),
        }
    }

    /// Signs the given `RawTransaction`. Note that this consumes the `RawTransaction` and turns it
    /// into a `SignatureCheckedTransaction`.
    ///
    /// For a transaction that has just been signed, its signature is expected to be valid.
    pub fn sign(
        self,
        private_key: &Ed25519PrivateKey,
        public_key: Ed25519PublicKey,
    ) -> Result<SignatureCheckedTransaction> {
        let raw_txn_bytes = SimpleSerializer::<Vec<u8>>::serialize(&self)?;
        let hash = RawTransactionBytes(&raw_txn_bytes).hash();
        let signature = private_key.sign_message(&hash);
        Ok(SignatureCheckedTransaction(SignedTransaction {
            // Will be cleaned up after eliminating raw_txn_bytes below
            raw_txn: self.clone(),
            public_key,
            signature,
            raw_txn_bytes: self.into_proto_bytes()?,
        }))
    }

    pub fn into_payload(self) -> TransactionPayload {
        self.payload
    }

    pub fn format_for_client(&self, get_transaction_name: impl Fn(&[u8]) -> String) -> String {
        let empty_vec = vec![];
        let (code, args) = match &self.payload {
            TransactionPayload::Program(program) => {
                (get_transaction_name(program.code()), program.args())
            }
            TransactionPayload::WriteSet(_) => ("genesis".to_string(), &empty_vec[..]),
            TransactionPayload::Script(script) => {
                (get_transaction_name(script.code()), script.args())
            }
            TransactionPayload::Module(_) => ("module publishing".to_string(), &empty_vec[..]),
        };
        let mut f_args: String = "".to_string();
        for arg in args {
            f_args = format!("{}\n\t\t\t{:#?},", f_args, arg);
        }
        format!(
            "RawTransaction {{ \n\
             \tsender: {}, \n\
             \tsequence_number: {}, \n\
             \tpayload: {{, \n\
             \t\ttransaction: {}, \n\
             \t\targs: [ {} \n\
             \t\t]\n\
             \t}}, \n\
             \tmax_gas_amount: {}, \n\
             \tgas_unit_price: {}, \n\
             \texpiration_time: {:#?}, \n\
             }}",
            self.sender,
            self.sequence_number,
            code,
            f_args,
            self.max_gas_amount,
            self.gas_unit_price,
            self.expiration_time,
        )
    }
    /// Return the sender of this transaction.
    pub fn sender(&self) -> AccountAddress {
        self.sender
    }
}

pub struct RawTransactionBytes<'a>(pub &'a [u8]);

impl<'a> CryptoHash for RawTransactionBytes<'a> {
    type Hasher = RawTransactionHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(self.0);
        state.finish()
    }
}

impl FromProto for RawTransaction {
    type ProtoType = crate::proto::transaction::RawTransaction;

    fn from_proto(mut txn: Self::ProtoType) -> Result<Self> {
        Ok(RawTransaction {
            sender: AccountAddress::try_from(txn.get_sender_account())?,
            sequence_number: txn.sequence_number,
            payload: if txn.has_program() {
                TransactionPayload::Program(Program::from_proto(txn.take_program())?)
            } else if txn.has_write_set() {
                TransactionPayload::WriteSet(WriteSet::from_proto(txn.take_write_set())?)
            } else if txn.has_module() {
                TransactionPayload::Module(Module::from_proto(txn.take_module())?)
            } else if txn.has_script() {
                TransactionPayload::Script(Script::from_proto(txn.take_script())?)
            } else {
                bail!("RawTransaction payload missing");
            },
            max_gas_amount: txn.max_gas_amount,
            gas_unit_price: txn.gas_unit_price,
            expiration_time: Duration::from_secs(txn.expiration_time),
        })
    }
}

impl IntoProto for RawTransaction {
    type ProtoType = crate::proto::transaction::RawTransaction;

    fn into_proto(self) -> Self::ProtoType {
        let mut transaction = Self::ProtoType::new();
        transaction.set_sender_account(self.sender.as_ref().to_vec());
        transaction.set_sequence_number(self.sequence_number);
        match self.payload {
            TransactionPayload::Program(program) => transaction.set_program(program.into_proto()),
            TransactionPayload::WriteSet(write_set) => {
                transaction.set_write_set(write_set.into_proto())
            }
            TransactionPayload::Script(script) => transaction.set_script(script.into_proto()),
            TransactionPayload::Module(module) => transaction.set_module(module.into_proto()),
        }
        transaction.set_gas_unit_price(self.gas_unit_price);
        transaction.set_max_gas_amount(self.max_gas_amount);
        transaction.set_expiration_time(self.expiration_time.as_secs());
        transaction
    }
}

impl CanonicalSerialize for RawTransaction {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_struct(&self.sender)?;
        serializer.encode_u64(self.sequence_number)?;
        serializer.encode_struct(&self.payload)?;
        serializer.encode_u64(self.max_gas_amount)?;
        serializer.encode_u64(self.gas_unit_price)?;
        serializer.encode_u64(self.expiration_time.as_secs())?;
        Ok(())
    }
}

impl CanonicalDeserialize for RawTransaction {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let sender = deserializer.decode_struct()?;
        let sequence_number = deserializer.decode_u64()?;
        let payload = deserializer.decode_struct()?;
        let max_gas_amount = deserializer.decode_u64()?;
        let gas_unit_price = deserializer.decode_u64()?;
        let expiration_time = Duration::from_secs(deserializer.decode_u64()?);

        Ok(RawTransaction {
            sender,
            sequence_number,
            payload,
            max_gas_amount,
            gas_unit_price,
            expiration_time,
        })
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionPayload {
    /// A regular programmatic transaction that is executed by the VM.
    Program(Program),
    WriteSet(WriteSet),
    /// A transaction that publishes code.
    Module(Module),
    /// A transaction that executes code.
    Script(Script),
}

impl CanonicalSerialize for TransactionPayload {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        match self {
            TransactionPayload::Program(program) => {
                serializer.encode_u32(TransactionPayloadType::Program as u32)?;
                serializer.encode_struct(program)?;
            }
            TransactionPayload::WriteSet(write_set) => {
                serializer.encode_u32(TransactionPayloadType::WriteSet as u32)?;
                serializer.encode_struct(write_set)?;
            }
            TransactionPayload::Script(script) => {
                serializer.encode_u32(TransactionPayloadType::Script as u32)?;
                serializer.encode_struct(script)?;
            }
            TransactionPayload::Module(module) => {
                serializer.encode_u32(TransactionPayloadType::Module as u32)?;
                serializer.encode_struct(module)?;
            }
        };
        Ok(())
    }
}

impl CanonicalDeserialize for TransactionPayload {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let decoded_payload_type = deserializer.decode_u32()?;
        let payload_type = TransactionPayloadType::from_u32(decoded_payload_type);
        match payload_type {
            Some(TransactionPayloadType::Program) => {
                Ok(TransactionPayload::Program(deserializer.decode_struct()?))
            }
            Some(TransactionPayloadType::WriteSet) => {
                Ok(TransactionPayload::WriteSet(deserializer.decode_struct()?))
            }
            Some(TransactionPayloadType::Script) => {
                Ok(TransactionPayload::Script(deserializer.decode_struct()?))
            }
            Some(TransactionPayloadType::Module) => {
                Ok(TransactionPayload::Module(deserializer.decode_struct()?))
            }
            None => Err(format_err!(
                "ParseError: Unable to decode TransactionPayloadType, found {}",
                decoded_payload_type
            )),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum TransactionPayloadType {
    Program = 0,
    WriteSet = 1,
    Script = 2,
    Module = 3,
}

impl TransactionPayloadType {
    fn from_u32(value: u32) -> Option<TransactionPayloadType> {
        match value {
            0 => Some(TransactionPayloadType::Program),
            1 => Some(TransactionPayloadType::WriteSet),
            2 => Some(TransactionPayloadType::Script),
            3 => Some(TransactionPayloadType::Module),
            _ => None,
        }
    }
}

impl ::std::marker::Copy for TransactionPayloadType {}

/// A transaction that has been signed.
///
/// A `SignedTransaction` is a single transaction that can be atomically executed. Clients submit
/// these to validator nodes, and the validator and executor submits these to the VM.
///
/// **IMPORTANT:** The signature of a `SignedTransaction` is not guaranteed to be verified. For a
/// transaction whose signature is statically guaranteed to be verified, see
/// [`SignatureCheckedTransaction`].
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SignedTransaction {
    /// The raw transaction
    raw_txn: RawTransaction,

    /// Sender's public key. When checking the signature, we first need to check whether this key
    /// is indeed the pre-image of the pubkey hash stored under sender's account.
    public_key: Ed25519PublicKey,

    /// Signature of the transaction that correspond to the public key
    signature: Ed25519Signature,

    // The original raw bytes from the protobuf are also stored here so that we use
    // these bytes when generating the canonical serialization of the SignedTransaction struct
    // rather than re-serializing them again to avoid risk of non-determinism in the process

    // the raw transaction bytes generated from the wallet
    raw_txn_bytes: Vec<u8>,
}

/// A transaction for which the signature has been verified. Created by
/// [`SignedTransaction::check_signature`] and [`RawTransaction::sign`].
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SignatureCheckedTransaction(SignedTransaction);

impl SignatureCheckedTransaction {
    /// Returns the `SignedTransaction` within.
    pub fn into_inner(self) -> SignedTransaction {
        self.0
    }

    /// Returns the `RawTransaction` within.
    pub fn into_raw_transaction(self) -> RawTransaction {
        self.0.into_raw_transaction()
    }
}

impl Deref for SignatureCheckedTransaction {
    type Target = SignedTransaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for SignedTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SignedTransaction {{ \n \
             {{ raw_txn: {:#?}, \n \
             public_key: {:#?}, \n \
             signature: {:#?}, \n \
             }} \n \
             }}",
            self.raw_txn, self.public_key, self.signature,
        )
    }
}

impl SignedTransaction {
    pub fn craft_signed_transaction_for_client(
        raw_txn: RawTransaction,
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    ) -> SignedTransaction {
        SignedTransaction {
            raw_txn: raw_txn.clone(),
            public_key,
            signature,
            raw_txn_bytes: raw_txn
                .into_proto_bytes()
                .expect("Unable to serialize transaction"),
        }
    }

    pub fn public_key(&self) -> Ed25519PublicKey {
        self.public_key.clone()
    }

    pub fn signature(&self) -> Ed25519Signature {
        self.signature.clone()
    }

    pub fn sender(&self) -> AccountAddress {
        self.raw_txn.sender
    }

    pub fn into_raw_transaction(self) -> RawTransaction {
        self.raw_txn
    }

    pub fn sequence_number(&self) -> u64 {
        self.raw_txn.sequence_number
    }

    pub fn payload(&self) -> &TransactionPayload {
        &self.raw_txn.payload
    }

    pub fn max_gas_amount(&self) -> u64 {
        self.raw_txn.max_gas_amount
    }

    pub fn gas_unit_price(&self) -> u64 {
        self.raw_txn.gas_unit_price
    }

    pub fn expiration_time(&self) -> Duration {
        self.raw_txn.expiration_time
    }

    pub fn raw_txn_bytes_len(&self) -> usize {
        self.raw_txn_bytes.len()
    }

    /// Checks that the signature of given transaction. Returns `Ok(SignatureCheckedTransaction)` if
    /// the signature is valid.
    pub fn check_signature(self) -> Result<SignatureCheckedTransaction> {
        let raw_txn_bytes = SimpleSerializer::<Vec<u8>>::serialize(&self.raw_txn)?;
        let hash = RawTransactionBytes(&raw_txn_bytes).hash();
        self.public_key.verify_signature(&hash, &self.signature)?;
        Ok(SignatureCheckedTransaction(self))
    }

    pub fn format_for_client(&self, get_transaction_name: impl Fn(&[u8]) -> String) -> String {
        format!(
            "SignedTransaction {{ \n \
             raw_txn: {}, \n \
             public_key: {:#?}, \n \
             signature: {:#?}, \n \
             }}",
            self.raw_txn.format_for_client(get_transaction_name),
            self.public_key,
            self.signature,
        )
    }
}

impl CryptoHash for SignedTransaction {
    type Hasher = SignedTransactionHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&SimpleSerializer::<Vec<u8>>::serialize(self).expect("serialization failed"));
        state.finish()
    }
}

impl FromProto for SignedTransaction {
    type ProtoType = crate::proto::transaction::SignedTransaction;

    fn from_proto(txn: Self::ProtoType) -> Result<Self> {
        let proto_raw_transaction = protobuf::parse_from_bytes::<
            crate::proto::transaction::RawTransaction,
        >(txn.raw_txn_bytes.as_ref())?;

        let raw_txn = RawTransaction::from_proto(proto_raw_transaction)?;

        let t = SignedTransaction {
            raw_txn,
            public_key: Ed25519PublicKey::try_from(txn.get_sender_public_key())?,
            signature: Ed25519Signature::try_from(txn.get_sender_signature())?,
            raw_txn_bytes: txn.raw_txn_bytes,
        };

        // Signature checking is encoded in `SignatureCheckedTransaction`.

        Ok(t)
    }
}

impl IntoProto for SignedTransaction {
    type ProtoType = crate::proto::transaction::SignedTransaction;

    fn into_proto(self) -> Self::ProtoType {
        let mut transaction = Self::ProtoType::new();
        transaction.set_raw_txn_bytes(
            self.raw_txn
                .into_proto_bytes()
                .expect("serialization failed"),
        );
        transaction.set_sender_public_key(self.public_key.to_bytes().to_vec());
        transaction.set_sender_signature(self.signature.to_bytes().to_vec());
        transaction
    }
}

impl IntoProto for SignatureCheckedTransaction {
    type ProtoType = crate::proto::transaction::SignedTransaction;

    fn into_proto(self) -> Self::ProtoType {
        self.0.into_proto()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct SignedTransactionWithProof {
    pub version: Version,
    pub signed_transaction: SignedTransaction,
    pub events: Option<Vec<ContractEvent>>,
    pub proof: SignedTransactionProof,
}

impl SignedTransactionWithProof {
    /// Verifies the signed transaction with the proof, both carried by `self`.
    ///
    /// Two things are ensured if no error is raised:
    ///   1. This signed transaction exists in the ledger represented by `ledger_info`.
    ///   2. And this signed transaction has the same `version`, `sender`, and `sequence_number` as
    /// indicated by the parameter list. If any of these parameter is unknown to the call site that
    /// is supposed to be informed via this struct, get it from the struct itself, such as:
    /// `signed_txn_with_proof.version`, `signed_txn_with_proof.signed_transaction.sender()`, etc.
    pub fn verify(
        &self,
        ledger_info: &LedgerInfo,
        version: Version,
        sender: AccountAddress,
        sequence_number: u64,
    ) -> Result<()> {
        ensure!(
            self.version == version,
            "Version ({}) is not expected ({}).",
            self.version,
            version,
        );
        ensure!(
            self.signed_transaction.sender() == sender,
            "Sender ({}) not expected ({}).",
            self.signed_transaction.sender(),
            sender,
        );
        ensure!(
            self.signed_transaction.sequence_number() == sequence_number,
            "Sequence number ({}) not expected ({}).",
            self.signed_transaction.sequence_number(),
            sequence_number,
        );

        let events_root_hash = self.events.as_ref().map(|events| {
            let event_hashes: Vec<_> = events.iter().map(ContractEvent::hash).collect();
            get_accumulator_root_hash::<EventAccumulatorHasher>(&event_hashes)
        });
        verify_signed_transaction(
            ledger_info,
            self.signed_transaction.hash(),
            events_root_hash,
            version,
            &self.proof,
        )
    }
}

impl FromProto for SignedTransactionWithProof {
    type ProtoType = crate::proto::transaction::SignedTransactionWithProof;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        let version = object.get_version();
        let signed_transaction = SignedTransaction::from_proto(object.take_signed_transaction())?;
        let proof = SignedTransactionProof::from_proto(object.take_proof())?;
        let events = object
            .events
            .take()
            .map(|mut list| {
                list.take_events()
                    .into_iter()
                    .map(ContractEvent::from_proto)
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;

        Ok(Self {
            version,
            signed_transaction,
            proof,
            events,
        })
    }
}

impl IntoProto for SignedTransactionWithProof {
    type ProtoType = crate::proto::transaction::SignedTransactionWithProof;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_version(self.version);
        proto.set_signed_transaction(self.signed_transaction.into_proto());
        proto.set_proof(self.proof.into_proto());
        if let Some(events) = self.events {
            let mut events_list = EventsList::new();
            events_list.set_events(protobuf::RepeatedField::from_vec(
                events.into_iter().map(ContractEvent::into_proto).collect(),
            ));
            proto.set_events(events_list);
        }

        proto
    }
}

impl CanonicalSerialize for SignedTransaction {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_struct(&self.raw_txn)?
            .encode_bytes(&self.public_key.to_bytes())?
            .encode_bytes(&self.signature.to_bytes())?;
        Ok(())
    }
}

impl CanonicalDeserialize for SignedTransaction {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        let raw_txn: RawTransaction = deserializer.decode_struct()?;
        let public_key_bytes = deserializer.decode_bytes()?;
        let signature_bytes = deserializer.decode_bytes()?;
        let raw_txn_bytes = raw_txn.clone().into_proto_bytes()?;

        Ok(SignedTransaction {
            raw_txn,
            public_key: Ed25519PublicKey::try_from(&public_key_bytes[..]).unwrap(),
            signature: Ed25519Signature::try_from(&signature_bytes[..]).unwrap(),
            raw_txn_bytes,
        })
    }
}

/// The status of executing a transaction. The VM decides whether or not we should `Keep` the
/// transaction output or `Discard` it based upon the execution of the transaction. We wrap these
/// decisions around a `VMStatus` that provides more detail on the final execution state of the VM.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionStatus {
    /// Discard the transaction output
    Discard(VMStatus),

    /// Keep the transaction output
    Keep(VMStatus),
}

impl From<VMStatus> for TransactionStatus {
    fn from(vm_status: VMStatus) -> Self {
        let should_discard = match vm_status {
            // Any error that is a validation status (i.e. an error arising from the prologue)
            // causes the transaction to not be included.
            VMStatus::Validation(_) => true,
            // If the VM encountered an invalid internal state, we should discard the transaction.
            VMStatus::InvariantViolation(_) => true,
            // A transaction that publishes code that cannot be verified is currently not charged.
            // Therefore the transaction can be excluded.
            //
            // The original plan was to charge for verification, but the code didn't implement it
            // properly. The decision of whether to charge or not will be made based on data (if
            // verification checks are too expensive then yes, otherwise no).
            VMStatus::Verification(_) => true,
            // Even if we are unable to decode the transaction, there should be a charge made to
            // that user's account for the gas fees related to decoding, running the prologue etc.
            VMStatus::Deserialization(_) => false,
            // Any error encountered during the execution of the transaction will charge gas.
            VMStatus::Execution(_) => false,
        };

        if should_discard {
            TransactionStatus::Discard(vm_status)
        } else {
            TransactionStatus::Keep(vm_status)
        }
    }
}

/// The output of executing a transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionOutput {
    /// The list of writes this transaction intends to do.
    write_set: WriteSet,

    /// The list of events emitted during this transaction.
    events: Vec<ContractEvent>,

    /// The amount of gas used during execution.
    gas_used: u64,

    /// The execution status.
    status: TransactionStatus,
}

impl TransactionOutput {
    pub fn new(
        write_set: WriteSet,
        events: Vec<ContractEvent>,
        gas_used: u64,
        status: TransactionStatus,
    ) -> Self {
        TransactionOutput {
            write_set,
            events,
            gas_used,
            status,
        }
    }

    pub fn write_set(&self) -> &WriteSet {
        &self.write_set
    }

    pub fn events(&self) -> &[ContractEvent] {
        &self.events
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn status(&self) -> &TransactionStatus {
        &self.status
    }
}

/// `TransactionInfo` is the object we store in the transaction accumulator. It consists of the
/// transaction as well as the execution result of this transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FromProto, IntoProto)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[ProtoType(crate::proto::transaction_info::TransactionInfo)]
pub struct TransactionInfo {
    /// The hash of this transaction.
    signed_transaction_hash: HashValue,

    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    state_root_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    event_root_hash: HashValue,

    /// The amount of gas used.
    gas_used: u64,
}

impl TransactionInfo {
    /// Constructs a new `TransactionInfo` object using signed transaction hash, state root hash
    /// and event root hash.
    pub fn new(
        signed_transaction_hash: HashValue,
        state_root_hash: HashValue,
        event_root_hash: HashValue,
        gas_used: u64,
    ) -> TransactionInfo {
        TransactionInfo {
            signed_transaction_hash,
            state_root_hash,
            event_root_hash,
            gas_used,
        }
    }

    /// Returns the hash of this transaction.
    pub fn signed_transaction_hash(&self) -> HashValue {
        self.signed_transaction_hash
    }

    /// Returns root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub fn state_root_hash(&self) -> HashValue {
        self.state_root_hash
    }

    /// Returns the root hash of Merkle Accumulator storing all events emitted during this
    /// transaction.
    pub fn event_root_hash(&self) -> HashValue {
        self.event_root_hash
    }

    /// Returns the amount of gas used by this transaction.
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }
}

impl CanonicalSerialize for TransactionInfo {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_bytes(self.signed_transaction_hash.as_ref())?
            .encode_bytes(self.state_root_hash.as_ref())?
            .encode_bytes(self.event_root_hash.as_ref())?
            .encode_u64(self.gas_used)?;
        Ok(())
    }
}

impl CryptoHash for TransactionInfo {
    type Hasher = TransactionInfoHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            &SimpleSerializer::<Vec<u8>>::serialize(self).expect("Serialization should work."),
        );
        state.finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionToCommit {
    signed_txn: SignedTransaction,
    account_states: HashMap<AccountAddress, AccountStateBlob>,
    events: Vec<ContractEvent>,
    gas_used: u64,
}

impl TransactionToCommit {
    pub fn new(
        signed_txn: SignedTransaction,
        account_states: HashMap<AccountAddress, AccountStateBlob>,
        events: Vec<ContractEvent>,
        gas_used: u64,
    ) -> Self {
        TransactionToCommit {
            signed_txn,
            account_states,
            events,
            gas_used,
        }
    }

    pub fn signed_txn(&self) -> &SignedTransaction {
        &self.signed_txn
    }

    pub fn account_states(&self) -> &HashMap<AccountAddress, AccountStateBlob> {
        &self.account_states
    }

    pub fn events(&self) -> &[ContractEvent] {
        &self.events
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }
}

impl FromProto for TransactionToCommit {
    type ProtoType = crate::proto::transaction::TransactionToCommit;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        let signed_txn = SignedTransaction::from_proto(object.take_signed_txn())?;
        let account_states_proto = object.take_account_states();
        let num_account_states = account_states_proto.len();
        let account_states = account_states_proto
            .into_iter()
            .map(|mut x| {
                Ok((
                    AccountAddress::from_proto(x.take_address())?,
                    AccountStateBlob::from(x.take_blob()),
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        ensure!(
            account_states.len() == num_account_states,
            "account_states should have no duplication."
        );
        let events = object
            .take_events()
            .into_iter()
            .map(ContractEvent::from_proto)
            .collect::<Result<Vec<_>>>()?;
        let gas_used = object.get_gas_used();

        Ok(TransactionToCommit {
            signed_txn,
            account_states,
            events,
            gas_used,
        })
    }
}

impl IntoProto for TransactionToCommit {
    type ProtoType = crate::proto::transaction::TransactionToCommit;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_signed_txn(self.signed_txn.into_proto());
        proto.set_account_states(protobuf::RepeatedField::from_vec(
            self.account_states
                .into_iter()
                .map(|(address, blob)| {
                    let mut account_state = crate::proto::transaction::AccountState::new();
                    account_state.set_address(address.as_ref().to_vec());
                    account_state.set_blob(blob.into());
                    account_state
                })
                .collect::<Vec<_>>(),
        ));
        proto.set_events(protobuf::RepeatedField::from_vec(
            self.events
                .into_iter()
                .map(ContractEvent::into_proto)
                .collect::<Vec<_>>(),
        ));
        proto.set_gas_used(self.gas_used);
        proto
    }
}

/// The list may have three states:
/// 1. The list is empty. Both proofs must be `None`.
/// 2. The list has only 1 transaction/transaction_info. Then `proof_of_first_transaction`
/// must exist and `proof_of_last_transaction` must be `None`.
/// 3. The list has 2+ transactions/transaction_infos. The both proofs must exist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionListWithProof {
    pub transaction_and_infos: Vec<(SignedTransaction, TransactionInfo)>,
    pub events: Option<Vec<Vec<ContractEvent>>>,
    pub first_transaction_version: Option<Version>,
    pub proof_of_first_transaction: Option<AccumulatorProof>,
    pub proof_of_last_transaction: Option<AccumulatorProof>,
}

impl TransactionListWithProof {
    /// Constructor.
    pub fn new(
        transaction_and_infos: Vec<(SignedTransaction, TransactionInfo)>,
        events: Option<Vec<Vec<ContractEvent>>>,
        first_transaction_version: Option<Version>,
        proof_of_first_transaction: Option<AccumulatorProof>,
        proof_of_last_transaction: Option<AccumulatorProof>,
    ) -> Self {
        Self {
            transaction_and_infos,
            events,
            first_transaction_version,
            proof_of_first_transaction,
            proof_of_last_transaction,
        }
    }

    /// Creates an empty transaction list.
    pub fn new_empty() -> Self {
        Self::new(Vec::new(), None, None, None, None)
    }

    /// Verifies the transaction list with the proofs, both carried on `self`.
    ///
    /// Two things are ensured if no error is raised:
    ///   1. All the transactions exist on the ledger represented by `ledger_info`.
    ///   2. And the transactions in the list has consecutive versions starting from
    /// `first_transaction_version`. When `first_transaction_version` is None, ensures the list is
    /// empty.
    pub fn verify(
        &self,
        ledger_info: &LedgerInfo,
        first_transaction_version: Option<Version>,
    ) -> Result<()> {
        ensure!(
            self.first_transaction_version == first_transaction_version,
            "First transaction version ({}) not expected ({}).",
            Self::display_option_version(self.first_transaction_version),
            Self::display_option_version(first_transaction_version),
        );

        verify_transaction_list(ledger_info, self)
    }

    fn display_option_version(version: Option<Version>) -> String {
        match version {
            Some(v) => format!("{}", v),
            None => String::from("absent"),
        }
    }
}

impl FromProto for TransactionListWithProof {
    type ProtoType = crate::proto::transaction::TransactionListWithProof;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        let num_txns = object.get_transactions().len();
        let num_infos = object.get_infos().len();
        ensure!(
            num_txns == num_infos,
            "Number of transactions ({}) does not match the number of transaction infos ({}).",
            num_txns,
            num_infos
        );
        let (has_first, has_last, has_first_version) = (
            object.has_proof_of_first_transaction(),
            object.has_proof_of_last_transaction(),
            object.has_first_transaction_version(),
        );
        match num_txns {
            0 => ensure!(
                !has_first && !has_last && !has_first_version,
                "Some proof exists with 0 transactions"
            ),
            1 => ensure!(
                has_first && !has_last && has_first_version,
                "Proof of last transaction exists with 1 transaction"
            ),
            _ => ensure!(
                has_first && has_last && has_first_version,
                "Both proofs of first and last transactions must exist with 2+ transactions"
            ),
        }

        let events = object
            .events_for_versions
            .take() // Option<EventsForVersions>
            .map(|mut events_for_versions| {
                // EventsForVersion
                events_for_versions
                    .take_events_for_version()
                    .into_iter()
                    .map(|mut events_for_version| {
                        events_for_version
                            .take_events()
                            .into_iter()
                            .map(ContractEvent::from_proto)
                            .collect::<Result<Vec<_>>>()
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;

        let transaction_and_infos = itertools::zip_eq(
            object.take_transactions().into_iter(),
            object.take_infos().into_iter(),
        )
        .map(|(txn, info)| {
            Ok((
                SignedTransaction::from_proto(txn)?,
                TransactionInfo::from_proto(info)?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;

        Ok(TransactionListWithProof {
            transaction_and_infos,
            events,
            proof_of_first_transaction: object
                .proof_of_first_transaction
                .take()
                .map(AccumulatorProof::from_proto)
                .transpose()?,
            proof_of_last_transaction: object
                .proof_of_last_transaction
                .take()
                .map(AccumulatorProof::from_proto)
                .transpose()?,
            first_transaction_version: object
                .first_transaction_version
                .take()
                .map(|v| v.get_value()),
        })
    }
}

impl IntoProto for TransactionListWithProof {
    type ProtoType = crate::proto::transaction::TransactionListWithProof;

    fn into_proto(self) -> Self::ProtoType {
        let (transactions, infos) = self
            .transaction_and_infos
            .into_iter()
            .map(|(txn, info)| (txn.into_proto(), info.into_proto()))
            .unzip();

        let mut out = Self::ProtoType::new();
        out.set_transactions(protobuf::RepeatedField::from_vec(transactions));
        out.set_infos(protobuf::RepeatedField::from_vec(infos));

        if let Some(all_events) = self.events {
            let mut events_for_versions = EventsForVersions::new();
            for events_for_version in all_events {
                let mut events_this_version = EventsList::new();
                events_this_version.set_events(protobuf::RepeatedField::from_vec(
                    events_for_version
                        .into_iter()
                        .map(ContractEvent::into_proto)
                        .collect(),
                ));
                events_for_versions
                    .events_for_version
                    .push(events_this_version);
            }
            out.set_events_for_versions(events_for_versions);
        }

        if let Some(first_transaction_version) = self.first_transaction_version {
            let mut ver = UInt64Value::new();
            ver.set_value(first_transaction_version);
            out.set_first_transaction_version(ver);
        }
        if let Some(proof_of_first_transaction) = self.proof_of_first_transaction {
            out.set_proof_of_first_transaction(proof_of_first_transaction.into_proto());
        }
        if let Some(proof_of_last_transaction) = self.proof_of_last_transaction {
            out.set_proof_of_last_transaction(proof_of_last_transaction.into_proto());
        }
        out
    }
}
