// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Test infrastructure for modeling Libra accounts.

use libra_crypto::ed25519::*;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    byte_array::ByteArray,
    event::EventHandle,
    transaction::{
        RawTransaction, Script, SignedTransaction, TransactionArgument, TransactionPayload,
    },
};
use move_vm_types::{
    loaded_data::{struct_def::StructDef, types::Type},
    values::{Struct, Value},
};
use rand::{Rng, SeedableRng};
use std::time::Duration;
use vm_genesis::GENESIS_KEYPAIR;
use vm_runtime::identifier::create_access_path;

// TTL is 86400s. Initial time was set to 0.
pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;

/// Details about a Libra account.
///
/// Tests will typically create a set of `Account` instances to run transactions on. This type
/// encodes the logic to operate on and verify operations on any Libra account.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Account {
    addr: AccountAddress,
    /// The current private key for this account.
    pub privkey: Ed25519PrivateKey,
    /// The current public key for this account.
    pub pubkey: Ed25519PublicKey,
}

impl Account {
    /// Creates a new account in memory.
    ///
    /// The account returned by this constructor is a purely logical entity, meaning that it does
    /// not automatically get added to the Libra store. To add an account to the store, use
    /// [`AccountData`] instances with
    /// [`FakeExecutor::add_account_data`][crate::executor::FakeExecutor::add_account_data].
    /// This function returns distinct values upon every call.
    pub fn new() -> Self {
        let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
        let seed_buf: [u8; 32] = seed_rng.gen();
        let mut rng = rand::rngs::StdRng::from_seed(seed_buf);

        // replace `&mut rng` by None (making the function deterministic) and watch the
        // functional_tests fail!
        let (privkey, pubkey) = compat::generate_keypair(&mut rng);
        Self::with_keypair(privkey, pubkey)
    }

    /// Creates a new account with the given keypair.
    ///
    /// Like with [`Account::new`], the account returned by this constructor is a purely logical
    /// entity.
    pub fn with_keypair(privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) -> Self {
        let addr = AccountAddress::from_public_key(&pubkey);
        Account {
            addr,
            privkey,
            pubkey,
        }
    }

    /// Creates a new account in memory representing an account created in the genesis transaction.
    ///
    /// The address will be [`address`], which should be an address for a genesis account and
    /// the account will use [`GENESIS_KEYPAIR`][struct@GENESIS_KEYPAIR] as its keypair.
    pub fn new_genesis_account(address: AccountAddress) -> Self {
        Account {
            addr: address,
            pubkey: GENESIS_KEYPAIR.1.clone(),
            privkey: GENESIS_KEYPAIR.0.clone(),
        }
    }

    /// Creates a new account representing the association in memory.
    ///
    /// The address will be [`association_address`][account_config::association_address], and
    /// the account will use [`GENESIS_KEYPAIR`][struct@GENESIS_KEYPAIR] as its keypair.
    pub fn new_association() -> Self {
        Self::new_genesis_account(account_config::association_address())
    }

    /// Returns the address of the account. This is a hash of the public key the account was created
    /// with.
    ///
    /// The address does not change if the account's [keys are rotated][Account::rotate_key].
    pub fn address(&self) -> &AccountAddress {
        &self.addr
    }

    /// Returns the AccessPath that describes the Account resource instance.
    ///
    /// Use this to retrieve or publish the Account blob.
    // TODO: plug in the account type
    pub fn make_access_path(&self) -> AccessPath {
        // TODO: we need a way to get the type (StructDef) of the Account in place
        create_access_path(&self.addr, account_config::account_struct_tag())
    }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) {
        self.privkey = privkey;
        self.pubkey = pubkey;
    }

    /// Computes the authentication key for this account, as stored on the chain.
    ///
    /// This is the same as the account's address if the keys have never been rotated.
    pub fn auth_key(&self) -> ByteArray {
        ByteArray::new(AccountAddress::from_public_key(&self.pubkey).to_vec())
    }

    //
    // Helpers to read data from an Account resource
    //

    //
    // Helpers for transaction creation with Account instance as sender
    //

    /// Returns a [`SignedTransaction`] with a payload and this account as the sender.
    ///
    /// This is the most generic way to create a transaction for testing.
    /// Max gas amount and gas unit price are ignored for WriteSet transactions.
    pub fn create_user_txn(
        &self,
        payload: TransactionPayload,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
    ) -> SignedTransaction {
        let raw_txn = match payload {
            TransactionPayload::Program => RawTransaction::new(
                *self.address(),
                sequence_number,
                TransactionPayload::Program,
                max_gas_amount,
                gas_unit_price,
                Duration::from_secs(DEFAULT_EXPIRATION_TIME),
            ),
            TransactionPayload::WriteSet(writeset) => {
                RawTransaction::new_change_set(*self.address(), sequence_number, writeset)
            }
            TransactionPayload::Module(module) => RawTransaction::new_module(
                *self.address(),
                sequence_number,
                module,
                max_gas_amount,
                gas_unit_price,
                Duration::from_secs(DEFAULT_EXPIRATION_TIME),
            ),
            TransactionPayload::Script(script) => RawTransaction::new_script(
                *self.address(),
                sequence_number,
                script,
                max_gas_amount,
                gas_unit_price,
                Duration::from_secs(DEFAULT_EXPIRATION_TIME),
            ),
        };

        raw_txn
            .sign(&self.privkey, self.pubkey.clone())
            .unwrap()
            .into_inner()
    }

    /// Returns a [`SignedTransaction`] with the arguments defined in `args` and this account as
    /// the sender.
    pub fn create_signed_txn_with_args(
        &self,
        program: Vec<u8>,
        args: Vec<TransactionArgument>,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
    ) -> SignedTransaction {
        self.create_signed_txn_impl(
            *self.address(),
            TransactionPayload::Script(Script::new(program, args)),
            sequence_number,
            max_gas_amount,
            gas_unit_price,
        )
    }

    /// Returns a [`SignedTransaction`] with the arguments defined in `args` and a custom sender.
    ///
    /// The transaction is signed with the key corresponding to this account, not the custom sender.
    pub fn create_signed_txn_with_args_and_sender(
        &self,
        sender: AccountAddress,
        program: Vec<u8>,
        args: Vec<TransactionArgument>,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
    ) -> SignedTransaction {
        self.create_signed_txn_impl(
            sender,
            TransactionPayload::Script(Script::new(program, args)),
            sequence_number,
            max_gas_amount,
            gas_unit_price,
        )
    }

    /// Returns a [`SignedTransaction`] with the arguments defined in `args` and a custom sender.
    ///
    /// The transaction is signed with the key corresponding to this account, not the custom sender.
    pub fn create_signed_txn_impl(
        &self,
        sender: AccountAddress,
        program: TransactionPayload,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
    ) -> SignedTransaction {
        RawTransaction::new(
            sender,
            sequence_number,
            program,
            max_gas_amount,
            gas_unit_price,
            // TTL is 86400s. Initial time was set to 0.
            Duration::from_secs(DEFAULT_EXPIRATION_TIME),
        )
        .sign(&self.privkey, self.pubkey.clone())
        .unwrap()
        .into_inner()
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an account along with initial state about it.
///
/// `AccountData` captures the initial state needed to create accounts for tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountData {
    account: Account,
    balance: u64,
    sequence_number: u64,
    delegated_key_rotation_capability: bool,
    delegated_withdrawal_capability: bool,
    sent_events: EventHandle,
    received_events: EventHandle,
    event_generator: u64,
}

fn new_event_handle(count: u64) -> EventHandle {
    EventHandle::random_handle(count)
}

impl AccountData {
    /// Creates a new `AccountData` with a new account.
    ///
    /// Most tests will want to use this constructor.
    pub fn new(balance: u64, sequence_number: u64) -> Self {
        Self::with_account(Account::new(), balance, sequence_number)
    }

    /// Creates a new `AccountData` with the provided account.
    pub fn with_account(account: Account, balance: u64, sequence_number: u64) -> Self {
        Self::with_account_and_event_counts(account, balance, sequence_number, 0, 0, false, false)
    }

    /// Creates a new `AccountData` with the provided account.
    pub fn with_keypair(
        privkey: Ed25519PrivateKey,
        pubkey: Ed25519PublicKey,
        balance: u64,
        sequence_number: u64,
    ) -> Self {
        let account = Account::with_keypair(privkey, pubkey);
        Self::with_account(account, balance, sequence_number)
    }

    /// Creates a new `AccountData` with custom parameters.
    pub fn with_account_and_event_counts(
        account: Account,
        balance: u64,
        sequence_number: u64,
        sent_events_count: u64,
        received_events_count: u64,
        delegated_key_rotation_capability: bool,
        delegated_withdrawal_capability: bool,
    ) -> Self {
        Self {
            account,
            balance,
            sequence_number,
            delegated_key_rotation_capability,
            delegated_withdrawal_capability,
            sent_events: new_event_handle(sent_events_count),
            received_events: new_event_handle(received_events_count),
            event_generator: 2,
        }
    }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) {
        self.account.rotate_key(privkey, pubkey)
    }

    pub fn layout() -> StructDef {
        StructDef::new(vec![
            Type::ByteArray,
            Type::Struct(StructDef::new(vec![Type::U64])),
            Type::Bool,
            Type::Bool,
            Type::Struct(StructDef::new(vec![Type::U64, Type::ByteArray])),
            Type::Struct(StructDef::new(vec![Type::U64, Type::ByteArray])),
            Type::U64,
            Type::Struct(StructDef::new(vec![Type::U64])),
        ])
    }

    /// Creates and returns a resource [`Value`] for this data.
    pub fn to_resource(&self) -> Value {
        // TODO: publish some concept of Account
        let coin = Value::struct_(Struct::pack(vec![Value::u64(self.balance)]));
        Value::struct_(Struct::pack(vec![
            Value::byte_array(ByteArray::new(
                AccountAddress::from_public_key(&self.account.pubkey).to_vec(),
            )),
            coin,
            Value::bool(self.delegated_key_rotation_capability),
            Value::bool(self.delegated_withdrawal_capability),
            Value::struct_(Struct::pack(vec![
                Value::u64(self.received_events.count()),
                Value::byte_array(ByteArray::new(self.received_events.key().to_vec())),
            ])),
            Value::struct_(Struct::pack(vec![
                Value::u64(self.sent_events.count()),
                Value::byte_array(ByteArray::new(self.sent_events.key().to_vec())),
            ])),
            Value::u64(self.sequence_number),
            Value::struct_(Struct::pack(vec![Value::u64(self.event_generator)])),
        ]))
    }

    /// Returns the AccessPath that describes the Account resource instance.
    ///
    /// Use this to retrieve or publish the Account blob.
    // TODO: plug in the account type
    pub fn make_access_path(&self) -> AccessPath {
        self.account.make_access_path()
    }

    /// Returns the address of the account. This is a hash of the public key the account was created
    /// with.
    ///
    /// The address does not change if the account's [keys are rotated][AccountData::rotate_key].
    pub fn address(&self) -> &AccountAddress {
        self.account.address()
    }

    /// Returns the underlying [`Account`] instance.
    pub fn account(&self) -> &Account {
        &self.account
    }

    /// Converts this data into an `Account` instance.
    pub fn into_account(self) -> Account {
        self.account
    }

    /// Returns the initial balance.
    pub fn balance(&self) -> u64 {
        self.balance
    }

    /// Returns the initial sequence number.
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Returns the unique key for this sent events stream.
    pub fn sent_events_key(&self) -> &[u8] {
        self.sent_events.key().as_bytes()
    }

    /// Returns the initial sent events count.
    pub fn sent_events_count(&self) -> u64 {
        self.sent_events.count()
    }

    /// Returns the unique key for this received events stream.
    pub fn received_events_key(&self) -> &[u8] {
        self.received_events.key().as_bytes()
    }

    /// Returns the initial received events count.
    pub fn received_events_count(&self) -> u64 {
        self.received_events.count()
    }
}
