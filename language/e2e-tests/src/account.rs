// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Test infrastructure for modeling Libra accounts.

use crate::{gas_costs, keygen::KeyGen};
use libra_crypto::ed25519::*;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    event::EventHandle,
    language_storage::{StructTag, TypeTag},
    transaction::{
        authenticator::AuthenticationKey, RawTransaction, Script, SignedTransaction,
        TransactionArgument, TransactionPayload,
    },
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use move_vm_types::{
    identifier::create_access_path,
    loaded_data::types::{StructType, Type},
    values::{Struct, Value},
};
use std::time::Duration;
use vm_genesis::GENESIS_KEYPAIR;

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
        let (privkey, pubkey) = KeyGen::from_os_rng().generate_keypair();
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
    pub fn make_account_access_path(&self) -> AccessPath {
        self.make_access_path(account_config::account_struct_tag())
    }

    /// Returns the AccessPath that describes the Account balance resource instance.
    ///
    /// Use this to retrieve or publish the Account balance blob.
    pub fn make_balance_access_path(&self) -> AccessPath {
        self.make_access_path(account_config::account_balance_struct_tag())
    }

    // TODO: plug in the account type
    fn make_access_path(&self, tag: StructTag) -> AccessPath {
        // TODO: we need a way to get the type (StructType) of the Account in place
        create_access_path(self.addr, tag)
    }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) {
        self.privkey = privkey;
        self.pubkey = pubkey;
    }

    /// Computes the authentication key for this account, as stored on the chain.
    ///
    /// This is the same as the account's address if the keys have never been rotated.
    pub fn auth_key(&self) -> Vec<u8> {
        AuthenticationKey::ed25519(&self.pubkey).to_vec()
    }

    /// Return the first 16 bytes of the account's auth key
    pub fn auth_key_prefix(&self) -> Vec<u8> {
        AuthenticationKey::ed25519(&self.pubkey).prefix().to_vec()
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
        gas_specifier: TypeTag,
    ) -> SignedTransaction {
        Self::create_raw_user_txn(
            *self.address(),
            payload,
            sequence_number,
            max_gas_amount,
            gas_unit_price,
            gas_specifier,
        )
        .sign(&self.privkey, self.pubkey.clone())
        .unwrap()
        .into_inner()
    }

    pub fn create_raw_user_txn(
        address: AccountAddress,
        payload: TransactionPayload,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_specifier: TypeTag,
    ) -> RawTransaction {
        match payload {
            TransactionPayload::Program => RawTransaction::new(
                address,
                sequence_number,
                TransactionPayload::Program,
                max_gas_amount,
                gas_unit_price,
                gas_specifier,
                Duration::from_secs(DEFAULT_EXPIRATION_TIME),
            ),
            TransactionPayload::WriteSet(writeset) => {
                RawTransaction::new_change_set(address, sequence_number, writeset)
            }
            TransactionPayload::Module(module) => RawTransaction::new_module(
                address,
                sequence_number,
                module,
                max_gas_amount,
                gas_unit_price,
                gas_specifier,
                Duration::from_secs(DEFAULT_EXPIRATION_TIME),
            ),
            TransactionPayload::Script(script) => RawTransaction::new_script(
                address,
                sequence_number,
                script,
                max_gas_amount,
                gas_unit_price,
                gas_specifier,
                Duration::from_secs(DEFAULT_EXPIRATION_TIME),
            ),
        }
    }

    /// Returns a [`SignedTransaction`] with the arguments defined in `args` and this account as
    /// the sender.
    pub fn create_signed_txn_with_args(
        &self,
        program: Vec<u8>,
        ty_args: Vec<TypeTag>,
        args: Vec<TransactionArgument>,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_specifier: TypeTag,
    ) -> SignedTransaction {
        self.create_signed_txn_impl(
            *self.address(),
            TransactionPayload::Script(Script::new(program, ty_args, args)),
            sequence_number,
            max_gas_amount,
            gas_unit_price,
            gas_specifier,
        )
    }

    pub fn create_raw_txn_with_args(
        address: AccountAddress,
        program: Vec<u8>,
        ty_args: Vec<TypeTag>,
        args: Vec<TransactionArgument>,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_specifier: TypeTag,
    ) -> RawTransaction {
        Self::create_raw_txn_impl(
            address,
            TransactionPayload::Script(Script::new(program, ty_args, args)),
            sequence_number,
            max_gas_amount,
            gas_unit_price,
            gas_specifier,
        )
    }

    /// Returns a [`SignedTransaction`] with the arguments defined in `args` and a custom sender.
    ///
    /// The transaction is signed with the key corresponding to this account, not the custom sender.
    pub fn create_signed_txn_with_args_and_sender(
        &self,
        sender: AccountAddress,
        program: Vec<u8>,
        ty_args: Vec<TypeTag>,
        args: Vec<TransactionArgument>,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_specifier: TypeTag,
    ) -> SignedTransaction {
        self.create_signed_txn_impl(
            sender,
            TransactionPayload::Script(Script::new(program, ty_args, args)),
            sequence_number,
            max_gas_amount,
            gas_unit_price,
            gas_specifier,
        )
    }

    pub fn create_raw_txn_with_args_and_sender(
        sender: AccountAddress,
        program: Vec<u8>,
        ty_args: Vec<TypeTag>,
        args: Vec<TransactionArgument>,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_specifier: TypeTag,
    ) -> RawTransaction {
        Self::create_raw_txn_impl(
            sender,
            TransactionPayload::Script(Script::new(program, ty_args, args)),
            sequence_number,
            max_gas_amount,
            gas_unit_price,
            gas_specifier,
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
        gas_specifier: TypeTag,
    ) -> SignedTransaction {
        Self::create_raw_txn_impl(
            sender,
            program,
            sequence_number,
            max_gas_amount,
            gas_unit_price,
            gas_specifier,
        )
        .sign(&self.privkey, self.pubkey.clone())
        .unwrap()
        .into_inner()
    }

    /// Create a transaction containing `script` signed by `sender` with default values for gas
    /// cost, gas price, expiration time, and currency type.
    pub fn signed_script_txn(&self, script: Script, sequence_number: u64) -> SignedTransaction {
        self.create_signed_txn_impl(
            *self.address(),
            TransactionPayload::Script(script),
            sequence_number,
            gas_costs::TXN_RESERVED,
            0, // gas price
            account_config::lbr_type_tag(),
        )
    }

    pub fn create_raw_txn_impl(
        sender: AccountAddress,
        program: TransactionPayload,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_specifier: TypeTag,
    ) -> RawTransaction {
        RawTransaction::new(
            sender,
            sequence_number,
            program,
            max_gas_amount,
            gas_unit_price,
            gas_specifier,
            // TTL is 86400s. Initial time was set to 0.
            Duration::from_secs(DEFAULT_EXPIRATION_TIME),
        )
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

/// Struct that represents an account balance resource for tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Balance {
    coin: u64,
}

impl Balance {
    /// Create a new balance with amount `balance`
    pub fn new(coin: u64) -> Self {
        Self { coin }
    }

    /// Retrieve the balance inside of this
    pub fn coin(&self) -> u64 {
        self.coin
    }

    /// Returns the Move Value for the account balance
    pub fn to_value(&self) -> Value {
        Value::struct_(Struct::pack(vec![Value::u64(self.coin)]))
    }

    /// Returns the value layout for the account balance
    pub fn type_() -> StructType {
        StructType {
            address: account_config::CORE_CODE_ADDRESS,
            module: account_config::account_module_name().to_owned(),
            name: account_config::account_balance_struct_name().to_owned(),
            is_resource: true,
            ty_args: vec![],
            layout: vec![Type::U64],
        }
    }
}

/// Represents an account along with initial state about it.
///
/// `AccountData` captures the initial state needed to create accounts for tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountData {
    account: Account,
    balance: Balance,
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
            balance: Balance::new(balance),
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

    pub fn sent_payment_event_type() -> StructType {
        StructType {
            address: account_config::CORE_CODE_ADDRESS,
            module: account_config::account_module_name().to_owned(),
            name: account_config::sent_event_name().to_owned(),
            is_resource: false,
            ty_args: vec![],
            layout: vec![Type::U64, Type::Address, Type::Vector(Box::new(Type::U8))],
        }
    }

    pub fn received_payment_event_type() -> StructType {
        StructType {
            address: account_config::CORE_CODE_ADDRESS,
            module: account_config::account_module_name().to_owned(),
            name: account_config::received_event_name().to_owned(),
            is_resource: false,
            ty_args: vec![],
            layout: vec![Type::U64, Type::Address, Type::Vector(Box::new(Type::U8))],
        }
    }

    pub fn event_handle_type(ty: Type) -> StructType {
        StructType {
            address: account_config::CORE_CODE_ADDRESS,
            module: account_config::account_module_name().to_owned(),
            name: account_config::account_event_handle_struct_name().to_owned(),
            is_resource: true,
            ty_args: vec![ty],
            layout: vec![Type::U64, Type::Vector(Box::new(Type::U8))],
        }
    }

    pub fn event_handle_generator_type() -> StructType {
        StructType {
            address: account_config::CORE_CODE_ADDRESS,
            module: account_config::account_module_name().to_owned(),
            name: account_config::account_event_handle_generator_struct_name().to_owned(),
            is_resource: true,
            ty_args: vec![],
            layout: vec![Type::U64],
        }
    }

    /// Returns the (Move value) layout of the LibraAccount::T struct
    pub fn account_type() -> StructType {
        StructType {
            address: account_config::CORE_CODE_ADDRESS,
            module: account_config::account_module_name().to_owned(),
            name: account_config::account_struct_name().to_owned(),
            is_resource: true,
            ty_args: vec![],
            layout: vec![
                Type::Vector(Box::new(Type::U8)),
                Type::Bool,
                Type::Bool,
                Type::Struct(Box::new(Self::event_handle_type(Type::Struct(Box::new(
                    Self::sent_payment_event_type(),
                ))))),
                Type::Struct(Box::new(Self::event_handle_type(Type::Struct(Box::new(
                    Self::received_payment_event_type(),
                ))))),
                Type::U64,
                Type::Struct(Box::new(Self::event_handle_generator_type())),
            ],
        }
    }

    /// Returns the layout for the LibraAccount::Balance struct
    pub fn balance_type() -> StructType {
        Balance::type_()
    }

    /// Creates and returns a resource [`Value`] for this data.
    pub fn to_account(&self) -> (Value, Value) {
        // TODO: publish some concept of Account
        let balance = self.balance.to_value();
        let account = Value::struct_(Struct::pack(vec![
            // TODO: this needs to compute the auth key instead
            Value::vector_u8(AccountAddress::authentication_key(&self.account.pubkey).to_vec()),
            Value::bool(self.delegated_key_rotation_capability),
            Value::bool(self.delegated_withdrawal_capability),
            Value::struct_(Struct::pack(vec![
                Value::u64(self.received_events.count()),
                Value::vector_u8(self.received_events.key().to_vec()),
            ])),
            Value::struct_(Struct::pack(vec![
                Value::u64(self.sent_events.count()),
                Value::vector_u8(self.sent_events.key().to_vec()),
            ])),
            Value::u64(self.sequence_number),
            Value::struct_(Struct::pack(vec![Value::u64(self.event_generator)])),
        ]));
        (account, balance)
    }

    /// Returns the AccessPath that describes the Account resource instance.
    ///
    /// Use this to retrieve or publish the Account blob.
    pub fn make_account_access_path(&self) -> AccessPath {
        self.account.make_account_access_path()
    }

    /// Returns the AccessPath that describes the Account balance resource instance.
    ///
    /// Use this to retrieve or publish the Account blob.
    pub fn make_balance_access_path(&self) -> AccessPath {
        self.account.make_balance_access_path()
    }

    pub fn to_writeset(&self) -> WriteSet {
        let (account_blob, balance_blob) = self.to_account();
        let account = account_blob
            .value_as::<Struct>()
            .unwrap()
            .simple_serialize(&AccountData::account_type())
            .unwrap();
        let balance = balance_blob
            .value_as::<Struct>()
            .unwrap()
            .simple_serialize(&AccountData::balance_type())
            .unwrap();
        WriteSetMut::new(vec![
            (self.make_account_access_path(), WriteOp::Value(account)),
            (self.make_balance_access_path(), WriteOp::Value(balance)),
        ])
        .freeze()
        .unwrap()
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
        self.balance.coin()
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
