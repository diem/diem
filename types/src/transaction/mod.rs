// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    block_metadata::BlockMetadata,
    contract_event::ContractEvent,
    ledger_info::LedgerInfo,
    proof::{accumulator::InMemoryAccumulator, TransactionListProof, TransactionProof},
    transaction::authenticator::TransactionAuthenticator,
    vm_error::{StatusCode, StatusType, VMStatus},
    write_set::WriteSet,
};
use anyhow::{ensure, format_err, Error, Result};
use libra_crypto::{
    ed25519::*,
    hash::{CryptoHash, CryptoHasher, EventAccumulatorHasher},
    multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature},
    traits::SigningKey,
    HashValue,
};
use libra_crypto_derive::CryptoHasher;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{de, ser, Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    fmt,
    fmt::{Display, Formatter},
    time::Duration,
};

pub mod authenticator;
mod change_set;
pub mod helpers;
mod module;
mod script;
mod transaction_argument;

pub use change_set::ChangeSet;
pub use module::Module;
pub use script::{Script, SCRIPT_HASH_LENGTH};

use std::ops::Deref;
pub use transaction_argument::{parse_as_transaction_argument, TransactionArgument};

pub type Version = u64; // Height - also used for MVCC in StateDB

// In StateDB, things readable by the genesis transaction are under this version.
pub const PRE_GENESIS_VERSION: Version = u64::max_value();

pub const MAX_TRANSACTION_SIZE_IN_BYTES: usize = 4096;

/// RawTransaction is the portion of a transaction that a client signs
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
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
    #[serde(serialize_with = "serialize_duration")]
    #[serde(deserialize_with = "deserialize_duration")]
    expiration_time: Duration,
}

// TODO(#1307)
fn serialize_duration<S>(d: &Duration, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    serializer.serialize_u64(d.as_secs())
}

fn deserialize_duration<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct DurationVisitor;
    impl<'de> de::Visitor<'de> for DurationVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("Duration as u64")
        }

        fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Duration::from_secs(v))
        }
    }

    deserializer.deserialize_u64(DurationVisitor)
}

impl RawTransaction {
    /// Create a new `RawTransaction` with a payload.
    ///
    /// It can be either to publish a module, to execute a script, or to issue a writeset
    /// transaction.
    pub fn new(
        sender: AccountAddress,
        sequence_number: u64,
        payload: TransactionPayload,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_time: Duration,
    ) -> Self {
        RawTransaction {
            sender,
            sequence_number,
            payload,
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
            payload: TransactionPayload::WriteSet(ChangeSet::new(write_set, vec![])),
            // Since write-set transactions bypass the VM, these fields aren't relevant.
            max_gas_amount: 0,
            gas_unit_price: 0,
            // Write-set transactions are special and important and shouldn't expire.
            expiration_time: Duration::new(u64::max_value(), 0),
        }
    }

    pub fn new_change_set(
        sender: AccountAddress,
        sequence_number: u64,
        change_set: ChangeSet,
    ) -> Self {
        RawTransaction {
            sender,
            sequence_number,
            payload: TransactionPayload::WriteSet(change_set),
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
        let signature = private_key.sign_message(&self.hash());
        Ok(SignatureCheckedTransaction(SignedTransaction::new(
            self, public_key, signature,
        )))
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn multi_sign_for_testing(
        self,
        private_key: &Ed25519PrivateKey,
        public_key: Ed25519PublicKey,
    ) -> Result<SignatureCheckedTransaction> {
        let signature = private_key.sign_message(&self.hash());
        Ok(SignatureCheckedTransaction(
            SignedTransaction::new_multisig(self, public_key.into(), signature.into()),
        ))
    }

    pub fn into_payload(self) -> TransactionPayload {
        self.payload
    }

    pub fn format_for_client(&self, get_transaction_name: impl Fn(&[u8]) -> String) -> String {
        let empty_vec = vec![];
        let (code, args) = match &self.payload {
            TransactionPayload::Program => {
                return "Deprecated".to_string();
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

impl CryptoHash for RawTransaction {
    type Hasher = RawTransactionHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            lcs::to_bytes(self)
                .expect("Failed to serialize RawTransaction")
                .as_slice(),
        );
        state.finish()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionPayload {
    /// Deprecated. See https://developers.libra.org/blog/2019/10/22/simplifying-payloads for more
    /// details.
    Program,
    WriteSet(ChangeSet),
    /// A transaction that executes code.
    Script(Script),
    /// A transaction that publishes code.
    Module(Module),
}

/// A transaction that has been signed.
///
/// A `SignedTransaction` is a single transaction that can be atomically executed. Clients submit
/// these to validator nodes, and the validator and executor submits these to the VM.
///
/// **IMPORTANT:** The signature of a `SignedTransaction` is not guaranteed to be verified. For a
/// transaction whose signature is statically guaranteed to be verified, see
/// [`SignatureCheckedTransaction`].
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, CryptoHasher)]
pub struct SignedTransaction {
    /// The raw transaction
    raw_txn: RawTransaction,

    /// Public key and signature to authenticate
    authenticator: TransactionAuthenticator,
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
             authenticator: {:#?}, \n \
             }} \n \
             }}",
            self.raw_txn, self.authenticator
        )
    }
}

impl SignedTransaction {
    pub fn new(
        raw_txn: RawTransaction,
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    ) -> SignedTransaction {
        let authenticator = TransactionAuthenticator::ed25519(public_key, signature);
        SignedTransaction {
            raw_txn,
            authenticator,
        }
    }

    pub fn new_multisig(
        raw_txn: RawTransaction,
        public_key: MultiEd25519PublicKey,
        signature: MultiEd25519Signature,
    ) -> SignedTransaction {
        let authenticator = TransactionAuthenticator::multi_ed25519(public_key, signature);
        SignedTransaction {
            raw_txn,
            authenticator,
        }
    }

    pub fn authenticator(&self) -> TransactionAuthenticator {
        self.authenticator.clone()
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
        lcs::to_bytes(&self.raw_txn)
            .expect("Unable to serialize RawTransaction")
            .len()
    }

    /// Checks that the signature of given transaction. Returns `Ok(SignatureCheckedTransaction)` if
    /// the signature is valid.
    pub fn check_signature(self) -> Result<SignatureCheckedTransaction> {
        self.authenticator.verify_signature(&self.raw_txn.hash())?;
        Ok(SignatureCheckedTransaction(self))
    }

    pub fn format_for_client(&self, get_transaction_name: impl Fn(&[u8]) -> String) -> String {
        format!(
            "SignedTransaction {{ \n \
             raw_txn: {}, \n \
             authenticator: {:#?}, \n \
             }}",
            self.raw_txn.format_for_client(get_transaction_name),
            self.authenticator
        )
    }
}

impl TryFrom<crate::proto::types::SignedTransaction> for SignedTransaction {
    type Error = Error;

    fn try_from(txn: crate::proto::types::SignedTransaction) -> Result<Self> {
        lcs::from_bytes(&txn.txn_bytes).map_err(Into::into)
    }
}

impl From<SignedTransaction> for crate::proto::types::SignedTransaction {
    fn from(txn: SignedTransaction) -> Self {
        let txn_bytes = lcs::to_bytes(&txn).expect("Unable to serialize SignedTransaction");
        Self { txn_bytes }
    }
}

impl From<SignatureCheckedTransaction> for crate::proto::types::SignedTransaction {
    fn from(txn: SignatureCheckedTransaction) -> Self {
        txn.0.into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct TransactionWithProof {
    pub version: Version,
    pub transaction: Transaction,
    pub events: Option<Vec<ContractEvent>>,
    pub proof: TransactionProof,
}

impl TransactionWithProof {
    /// Verifies the transaction with the proof, both carried by `self`.
    ///
    /// A few things are ensured if no error is raised:
    ///   1. This transaction exists in the ledger represented by `ledger_info`.
    ///   2. This transaction is a `UserTransaction`.
    ///   3. And this user transaction has the same `version`, `sender`, and `sequence_number` as
    ///      indicated by the parameter list. If any of these parameter is unknown to the call site
    ///      that is supposed to be informed via this struct, get it from the struct itself, such
    ///      as version and sender.
    pub fn verify_user_txn(
        &self,
        ledger_info: &LedgerInfo,
        version: Version,
        sender: AccountAddress,
        sequence_number: u64,
    ) -> Result<()> {
        let signed_transaction = self.transaction.as_signed_user_txn()?;

        ensure!(
            self.version == version,
            "Version ({}) is not expected ({}).",
            self.version,
            version,
        );
        ensure!(
            signed_transaction.sender() == sender,
            "Sender ({}) not expected ({}).",
            signed_transaction.sender(),
            sender,
        );
        ensure!(
            signed_transaction.sequence_number() == sequence_number,
            "Sequence number ({}) not expected ({}).",
            signed_transaction.sequence_number(),
            sequence_number,
        );

        let events_root_hash = self.events.as_ref().map(|events| {
            let event_hashes: Vec<_> = events.iter().map(ContractEvent::hash).collect();
            InMemoryAccumulator::<EventAccumulatorHasher>::from_leaves(&event_hashes).root_hash()
        });
        self.proof.verify(
            ledger_info,
            self.transaction.hash(),
            events_root_hash,
            version,
        )
    }
}

impl TryFrom<crate::proto::types::TransactionWithProof> for TransactionWithProof {
    type Error = Error;

    fn try_from(mut proto: crate::proto::types::TransactionWithProof) -> Result<Self> {
        let version = proto.version;
        let transaction = proto
            .transaction
            .ok_or_else(|| format_err!("Missing transaction"))?
            .try_into()?;
        let proof = proto
            .proof
            .ok_or_else(|| format_err!("Missing proof"))?
            .try_into()?;
        let events = proto
            .events
            .take()
            .map(|list| {
                list.events
                    .into_iter()
                    .map(ContractEvent::try_from)
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;

        Ok(Self {
            version,
            transaction,
            proof,
            events,
        })
    }
}

impl From<TransactionWithProof> for crate::proto::types::TransactionWithProof {
    fn from(mut txn: TransactionWithProof) -> Self {
        Self {
            version: txn.version,
            transaction: Some(txn.transaction.into()),
            proof: Some(txn.proof.into()),
            events: txn
                .events
                .take()
                .map(|list| crate::proto::types::EventsList {
                    events: list.into_iter().map(ContractEvent::into).collect(),
                }),
        }
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

    /// Retry the transaction because it is after a ValidatorSetChange txn
    Retry,
}

impl TransactionStatus {
    pub fn vm_status(&self) -> VMStatus {
        match self {
            TransactionStatus::Discard(vm_status) | TransactionStatus::Keep(vm_status) => {
                vm_status.clone()
            }
            TransactionStatus::Retry => VMStatus::new(StatusCode::UNKNOWN_VALIDATION_STATUS),
        }
    }

    pub fn is_discarded(&self) -> bool {
        match self {
            TransactionStatus::Discard(_) => true,
            TransactionStatus::Keep(_) => false,
            TransactionStatus::Retry => true,
        }
    }
}

impl From<VMStatus> for TransactionStatus {
    fn from(vm_status: VMStatus) -> Self {
        let should_discard = match vm_status.status_type() {
            // Any unknown error should be discarded
            StatusType::Unknown => true,
            // Any error that is a validation status (i.e. an error arising from the prologue)
            // causes the transaction to not be included.
            StatusType::Validation => true,
            // If the VM encountered an invalid internal state, we should discard the transaction.
            StatusType::InvariantViolation => true,
            // A transaction that publishes code that cannot be verified will be charged.
            StatusType::Verification => false,
            // Even if we are unable to decode the transaction, there should be a charge made to
            // that user's account for the gas fees related to decoding, running the prologue etc.
            StatusType::Deserialization => false,
            // Any error encountered during the execution of the transaction will charge gas.
            StatusType::Execution => false,
        };

        if should_discard {
            TransactionStatus::Discard(vm_status)
        } else {
            TransactionStatus::Keep(vm_status)
        }
    }
}

/// The result of running the transaction through the VM validator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VMValidatorResult {
    status: Option<VMStatus>,
    score: u64,
    is_governance_txn: bool,
}

impl VMValidatorResult {
    pub fn new(status: Option<VMStatus>, score: u64, is_governance_txn: bool) -> Self {
        Self {
            status,
            score,
            is_governance_txn,
        }
    }

    pub fn status(&self) -> Option<VMStatus> {
        self.status.clone()
    }

    pub fn score(&self) -> u64 {
        self.score
    }

    pub fn is_governance_txn(&self) -> bool {
        self.is_governance_txn
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

impl TryFrom<crate::proto::types::TransactionInfo> for TransactionInfo {
    type Error = Error;

    fn try_from(proto_txn_info: crate::proto::types::TransactionInfo) -> Result<Self> {
        let transaction_hash = HashValue::from_slice(&proto_txn_info.transaction_hash)?;
        let state_root_hash = HashValue::from_slice(&proto_txn_info.state_root_hash)?;
        let event_root_hash = HashValue::from_slice(&proto_txn_info.event_root_hash)?;
        let gas_used = proto_txn_info.gas_used;
        let major_status =
            StatusCode::try_from(proto_txn_info.major_status).unwrap_or(StatusCode::UNKNOWN_STATUS);
        Ok(TransactionInfo::new(
            transaction_hash,
            state_root_hash,
            event_root_hash,
            gas_used,
            major_status,
        ))
    }
}

impl From<TransactionInfo> for crate::proto::types::TransactionInfo {
    fn from(txn_info: TransactionInfo) -> Self {
        Self {
            transaction_hash: txn_info.transaction_hash.to_vec(),
            state_root_hash: txn_info.state_root_hash.to_vec(),
            event_root_hash: txn_info.event_root_hash.to_vec(),
            gas_used: txn_info.gas_used,
            major_status: txn_info.major_status.into(),
        }
    }
}

/// `TransactionInfo` is the object we store in the transaction accumulator. It consists of the
/// transaction as well as the execution result of this transaction.
#[derive(Clone, CryptoHasher, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct TransactionInfo {
    /// The hash of this transaction.
    transaction_hash: HashValue,

    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    state_root_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    event_root_hash: HashValue,

    /// The amount of gas used.
    gas_used: u64,

    /// The major status. This will provide the general error class. Note that this is not
    /// particularly high fidelity in the presence of sub statuses but, the major status does
    /// determine whether or not the transaction is applied to the global state or not.
    major_status: StatusCode,
}

impl TransactionInfo {
    /// Constructs a new `TransactionInfo` object using transaction hash, state root hash and event
    /// root hash.
    pub fn new(
        transaction_hash: HashValue,
        state_root_hash: HashValue,
        event_root_hash: HashValue,
        gas_used: u64,
        major_status: StatusCode,
    ) -> TransactionInfo {
        TransactionInfo {
            transaction_hash,
            state_root_hash,
            event_root_hash,
            gas_used,
            major_status,
        }
    }

    /// Returns the hash of this transaction.
    pub fn transaction_hash(&self) -> HashValue {
        self.transaction_hash
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

    pub fn major_status(&self) -> StatusCode {
        self.major_status
    }
}

impl CryptoHash for TransactionInfo {
    type Hasher = TransactionInfoHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&lcs::to_bytes(self).expect("Serialization should work."));
        state.finish()
    }
}

impl Display for TransactionInfo {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "TransactionInfo: [txn_hash: {}, state_root_hash: {}, event_root_hash: {}, gas_used: {}, major_status: {:?}]",
            self.transaction_hash(), self.state_root_hash(), self.event_root_hash(), self.gas_used(), self.major_status(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TransactionToCommit {
    transaction: Transaction,
    account_states: HashMap<AccountAddress, AccountStateBlob>,
    events: Vec<ContractEvent>,
    gas_used: u64,
    major_status: StatusCode,
}

impl TransactionToCommit {
    pub fn new(
        transaction: Transaction,
        account_states: HashMap<AccountAddress, AccountStateBlob>,
        events: Vec<ContractEvent>,
        gas_used: u64,
        major_status: StatusCode,
    ) -> Self {
        TransactionToCommit {
            transaction,
            account_states,
            events,
            gas_used,
            major_status,
        }
    }

    pub fn transaction(&self) -> &Transaction {
        &self.transaction
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

    pub fn major_status(&self) -> StatusCode {
        self.major_status
    }
}

impl TryFrom<crate::proto::types::TransactionToCommit> for TransactionToCommit {
    type Error = Error;

    fn try_from(proto: crate::proto::types::TransactionToCommit) -> Result<Self> {
        let transaction = proto
            .transaction
            .ok_or_else(|| format_err!("Missing signed_transaction"))?
            .try_into()?;
        let num_account_states = proto.account_states.len();
        let account_states = proto
            .account_states
            .into_iter()
            .map(|x| {
                Ok((
                    AccountAddress::try_from(x.address)?,
                    AccountStateBlob::from(x.blob),
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        ensure!(
            account_states.len() == num_account_states,
            "account_states should have no duplication."
        );
        let events = proto
            .events
            .into_iter()
            .map(ContractEvent::try_from)
            .collect::<Result<Vec<_>>>()?;
        let gas_used = proto.gas_used;
        let major_status =
            StatusCode::try_from(proto.major_status).unwrap_or(StatusCode::UNKNOWN_STATUS);

        Ok(TransactionToCommit {
            transaction,
            account_states,
            events,
            gas_used,
            major_status,
        })
    }
}

impl From<TransactionToCommit> for crate::proto::types::TransactionToCommit {
    fn from(txn: TransactionToCommit) -> Self {
        Self {
            transaction: Some(txn.transaction.into()),
            account_states: txn
                .account_states
                .into_iter()
                .map(|(address, blob)| crate::proto::types::AccountState {
                    address: address.as_ref().to_vec(),
                    blob: blob.into(),
                })
                .collect(),
            events: txn.events.into_iter().map(Into::into).collect(),
            gas_used: txn.gas_used,
            major_status: txn.major_status.into(),
        }
    }
}

/// The list may have three states:
/// 1. The list is empty. Both proofs must be `None`.
/// 2. The list has only 1 transaction/transaction_info. Then `proof_of_first_transaction`
/// must exist and `proof_of_last_transaction` must be `None`.
/// 3. The list has 2+ transactions/transaction_infos. The both proofs must exist.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TransactionListWithProof {
    pub transactions: Vec<Transaction>,
    pub events: Option<Vec<Vec<ContractEvent>>>,
    pub first_transaction_version: Option<Version>,
    pub proof: TransactionListProof,
}

impl TransactionListWithProof {
    /// Constructor.
    pub fn new(
        transactions: Vec<Transaction>,
        events: Option<Vec<Vec<ContractEvent>>>,
        first_transaction_version: Option<Version>,
        proof: TransactionListProof,
    ) -> Self {
        Self {
            transactions,
            events,
            first_transaction_version,
            proof,
        }
    }

    /// Creates an empty transaction list.
    pub fn new_empty() -> Self {
        Self::new(vec![], None, None, TransactionListProof::new_empty())
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

        let txn_hashes: Vec<_> = self.transactions.iter().map(CryptoHash::hash).collect();
        self.proof
            .verify(ledger_info, self.first_transaction_version, &txn_hashes)?;

        // Verify the events if they exist.
        if let Some(event_lists) = &self.events {
            ensure!(
                event_lists.len() == self.transactions.len(),
                "The length of event_lists ({}) does not match the number of transactions ({}).",
                event_lists.len(),
                self.transactions.len(),
            );
            itertools::zip_eq(event_lists, self.proof.transaction_infos())
                .map(|(events, txn_info)| {
                    let event_hashes: Vec<_> = events.iter().map(ContractEvent::hash).collect();
                    let event_root_hash =
                        InMemoryAccumulator::<EventAccumulatorHasher>::from_leaves(&event_hashes)
                            .root_hash();
                    ensure!(
                        event_root_hash == txn_info.event_root_hash(),
                        "Some event root hash calculated doesn't match that carried on the \
                         transaction info.",
                    );
                    Ok(())
                })
                .collect::<Result<Vec<_>>>()?;
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    fn display_option_version(version: Option<Version>) -> String {
        match version {
            Some(v) => format!("{}", v),
            None => String::from("absent"),
        }
    }
}

impl TryFrom<crate::proto::types::TransactionListWithProof> for TransactionListWithProof {
    type Error = Error;

    fn try_from(mut proto: crate::proto::types::TransactionListWithProof) -> Result<Self> {
        let transactions = proto
            .transactions
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;

        let events = proto
            .events_for_versions
            .take() // Option<EventsForVersions>
            .map(|events_for_versions| {
                // EventsForVersion
                events_for_versions
                    .events_for_version
                    .into_iter()
                    .map(|events_for_version| {
                        events_for_version
                            .events
                            .into_iter()
                            .map(ContractEvent::try_from)
                            .collect::<Result<Vec<_>>>()
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;

        let first_transaction_version = proto.first_transaction_version;

        let proof = proto
            .proof
            .ok_or_else(|| format_err!("Missing proof."))?
            .try_into()?;

        Ok(Self::new(
            transactions,
            events,
            first_transaction_version,
            proof,
        ))
    }
}

impl From<TransactionListWithProof> for crate::proto::types::TransactionListWithProof {
    fn from(txn: TransactionListWithProof) -> Self {
        let transactions = txn.transactions.into_iter().map(Into::into).collect();

        let events_for_versions =
            txn.events
                .map(|all_events| crate::proto::types::EventsForVersions {
                    events_for_version: all_events
                        .into_iter()
                        .map(|events_for_version| crate::proto::types::EventsList {
                            events: events_for_version
                                .into_iter()
                                .map(ContractEvent::into)
                                .collect::<Vec<_>>(),
                        })
                        .collect::<Vec<_>>(),
                });

        Self {
            transactions,
            events_for_versions,
            first_transaction_version: txn.first_transaction_version,
            proof: Some(txn.proof.into()),
        }
    }
}

/// `Transaction` will be the transaction type used internally in the libra node to represent the
/// transaction to be processed and persisted.
///
/// We suppress the clippy warning here as we would expect most of the transaction to be user
/// transaction.
#[allow(clippy::large_enum_variant)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
pub enum Transaction {
    /// Transaction submitted by the user. e.g: P2P payment transaction, publishing module
    /// transaction, etc.
    /// TODO: We need to rename SignedTransaction to SignedUserTransaction, as well as all the other
    ///       transaction types we had in our codebase.
    UserTransaction(SignedTransaction),

    /// Transaction that applies a WriteSet to the current storage. This should be used for ONLY for
    /// genesis right now.
    WaypointWriteSet(ChangeSet),

    /// Transaction to update the block metadata resource at the beginning of a block.
    BlockMetadata(BlockMetadata),
}

impl Transaction {
    pub fn as_signed_user_txn(&self) -> Result<&SignedTransaction> {
        match self {
            Transaction::UserTransaction(txn) => Ok(txn),
            _ => Err(format_err!("Not a user transaction.")),
        }
    }

    pub fn format_for_client(&self, get_transaction_name: impl Fn(&[u8]) -> String) -> String {
        match self {
            Transaction::UserTransaction(user_txn) => {
                user_txn.format_for_client(get_transaction_name)
            }
            // TODO: display proper information for client
            Transaction::WaypointWriteSet(_write_set) => String::from("genesis"),
            // TODO: display proper information for client
            Transaction::BlockMetadata(_block_metadata) => String::from("block_metadata"),
        }
    }
}

impl CryptoHash for Transaction {
    type Hasher = TransactionHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&lcs::to_bytes(self).expect("Failed to serialize Transaction."));
        state.finish()
    }
}

impl TryFrom<crate::proto::types::Transaction> for Transaction {
    type Error = Error;

    fn try_from(proto: crate::proto::types::Transaction) -> Result<Self> {
        lcs::from_bytes(&proto.transaction).map_err(Into::into)
    }
}

impl From<Transaction> for crate::proto::types::Transaction {
    fn from(txn: Transaction) -> Self {
        let bytes = lcs::to_bytes(&txn).expect("Serialization should not fail.");
        Self { transaction: bytes }
    }
}

impl TryFrom<Transaction> for SignedTransaction {
    type Error = Error;

    fn try_from(txn: Transaction) -> Result<Self> {
        match txn {
            Transaction::UserTransaction(txn) => Ok(txn),
            _ => Err(format_err!("Not a user transaction.")),
        }
    }
}
