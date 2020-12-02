// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Test infrastructure for modeling Diem accounts.

use crate::{gas_costs, keygen::KeyGen};
use anyhow::{Error, Result};
use diem_crypto::ed25519::*;
use diem_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{
        self, from_currency_code_string, type_tag_for_currency_code, AccountResource,
        BalanceResource, RoleId, XUS_NAME,
    },
    chain_id::ChainId,
    event::EventHandle,
    transaction::{
        authenticator::AuthenticationKey, Module, RawTransaction, Script, SignedTransaction,
        TransactionPayload, WriteSetPayload,
    },
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use move_core_types::{
    identifier::{IdentStr, Identifier},
    language_storage::{ResourceKey, StructTag},
    move_resource::MoveResource,
    value::{MoveStructLayout, MoveTypeLayout},
};
use move_vm_types::values::{Struct, Value};
use std::{collections::BTreeMap, str::FromStr};
use vm_genesis::GENESIS_KEYPAIR;

// TTL is 86400s. Initial time was set to 0.
pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;

pub fn xus_currency_code() -> Identifier {
    from_currency_code_string(XUS_NAME).unwrap()
}

pub fn currency_code(code: &str) -> Identifier {
    from_currency_code_string(code).unwrap()
}

/// Details about a Diem account.
///
/// Tests will typically create a set of `Account` instances to run transactions on. This type
/// encodes the logic to operate on and verify operations on any Diem account.
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
    /// not automatically get added to the Diem store. To add an account to the store, use
    /// [`AccountData`] instances with
    /// [`FakeExecutor::add_account_data`][crate::executor::FakeExecutor::add_account_data].
    /// This function returns distinct values upon every call.
    pub fn new() -> Self {
        let (privkey, pubkey) = KeyGen::from_os_rng().generate_keypair();
        Self::with_keypair(privkey, pubkey)
    }

    /// Creates a new account in memory given a random seed.
    pub fn new_from_seed(seed: &mut KeyGen) -> Self {
        let (privkey, pubkey) = seed.generate_keypair();
        Self::with_keypair(privkey, pubkey)
    }

    /// Creates a new account with the given keypair.
    ///
    /// Like with [`Account::new`], the account returned by this constructor is a purely logical
    /// entity.
    pub fn with_keypair(privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) -> Self {
        let addr = diem_types::account_address::from_public_key(&pubkey);
        Account {
            addr,
            privkey,
            pubkey,
        }
    }

    /// Creates a new account with the given addr and key pair
    ///
    /// Like with [`Account::new`], the account returned by this constructor is a purely logical
    /// entity.
    pub fn new_validator(
        addr: AccountAddress,
        privkey: Ed25519PrivateKey,
        pubkey: Ed25519PublicKey,
    ) -> Self {
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

    /// Creates a new account representing the diem root account in memory.
    ///
    /// The address will be [`diem_root_address`][account_config::diem_root_address], and
    /// the account will use [`GENESIS_KEYPAIR`][struct@GENESIS_KEYPAIR] as its keypair.
    pub fn new_diem_root() -> Self {
        Self::new_genesis_account(account_config::diem_root_address())
    }

    /// Creates a new account representing treasury compliance in memory.
    /// The account will use [`GENESIS_KEYPAIR`][struct@GENESIS_KEYPAIR] as its keypair.
    pub fn new_blessed_tc() -> Self {
        Self::new_genesis_account(account_config::treasury_compliance_account_address())
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
        self.make_access_path(AccountResource::struct_tag())
    }

    /// Returns the AccessPath that describes the EventHandleGenerator resource instance.
    ///
    /// Use this to retrieve or publish the EventHandleGenerator blob.
    pub fn make_event_generator_access_path(&self) -> AccessPath {
        self.make_access_path(account_config::event_handle_generator_struct_tag())
    }

    /// Returns the AccessPath that describes the Account balance resource instance.
    ///
    /// Use this to retrieve or publish the Account balance blob.
    pub fn make_balance_access_path(&self, balance_currency_code: Identifier) -> AccessPath {
        let type_tag = type_tag_for_currency_code(balance_currency_code);
        // TODO/XXX: Convert this to BalanceResource::struct_tag once that takes type args
        self.make_access_path(BalanceResource::struct_tag_for_currency(type_tag))
    }

    // TODO: plug in the account type
    pub fn make_access_path(&self, tag: StructTag) -> AccessPath {
        // TODO: we need a way to get the type (FatStructType) of the Account in place
        let resource_tag = ResourceKey::new(self.addr, tag);
        AccessPath::resource_access_path(resource_tag)
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

    pub fn transaction(&self) -> TransactionBuilder {
        TransactionBuilder::new(self.clone())
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TransactionBuilder {
    pub sender: Account,
    pub sequence_number: Option<u64>,
    pub program: Option<TransactionPayload>,
    pub max_gas_amount: Option<u64>,
    pub gas_unit_price: Option<u64>,
    pub gas_currency_code: Option<String>,
    pub chain_id: Option<ChainId>,
    pub ttl: Option<u64>,
}

impl TransactionBuilder {
    pub fn new(sender: Account) -> Self {
        Self {
            sender,
            sequence_number: None,
            program: None,
            max_gas_amount: None,
            gas_unit_price: None,
            gas_currency_code: None,
            chain_id: None,
            ttl: None,
        }
    }

    pub fn sequence_number(mut self, sequence_number: u64) -> Self {
        self.sequence_number = Some(sequence_number);
        self
    }

    pub fn chain_id(mut self, id: ChainId) -> Self {
        self.chain_id = Some(id);
        self
    }

    pub fn script(mut self, s: Script) -> Self {
        self.program = Some(TransactionPayload::Script(s));
        self
    }

    pub fn module(mut self, m: Module) -> Self {
        self.program = Some(TransactionPayload::Module(m));
        self
    }

    pub fn write_set(mut self, w: WriteSetPayload) -> Self {
        self.program = Some(TransactionPayload::WriteSet(w));
        self
    }

    pub fn max_gas_amount(mut self, max_gas_amount: u64) -> Self {
        self.max_gas_amount = Some(max_gas_amount);
        self
    }

    pub fn gas_unit_price(mut self, gas_unit_price: u64) -> Self {
        self.gas_unit_price = Some(gas_unit_price);
        self
    }

    pub fn gas_currency_code(mut self, gas_currency_code: &str) -> Self {
        self.gas_currency_code = Some(gas_currency_code.to_string());
        self
    }

    pub fn ttl(mut self, ttl: u64) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn raw(self) -> RawTransaction {
        RawTransaction::new(
            *self.sender.address(),
            self.sequence_number.expect("sequence number not set"),
            self.program.expect("transaction payload not set"),
            self.max_gas_amount.unwrap_or(gas_costs::TXN_RESERVED),
            self.gas_unit_price.unwrap_or(0),
            self.gas_currency_code
                .unwrap_or_else(|| XUS_NAME.to_owned()),
            self.ttl.unwrap_or(DEFAULT_EXPIRATION_TIME),
            ChainId::test(),
        )
    }

    pub fn sign(self) -> SignedTransaction {
        RawTransaction::new(
            *self.sender.address(),
            self.sequence_number.expect("sequence number not set"),
            self.program.expect("transaction payload not set"),
            self.max_gas_amount.unwrap_or(gas_costs::TXN_RESERVED),
            self.gas_unit_price.unwrap_or(0),
            self.gas_currency_code
                .unwrap_or_else(|| XUS_NAME.to_owned()),
            self.ttl.unwrap_or(DEFAULT_EXPIRATION_TIME),
            self.chain_id.unwrap_or_else(ChainId::test),
        )
        .sign(&self.sender.privkey, self.sender.pubkey)
        .unwrap()
        .into_inner()
    }
}

//---------------------------------------------------------------------------
// Balance resource represenation
//---------------------------------------------------------------------------

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
        Value::struct_(Struct::pack(vec![Value::u64(self.coin)], true))
    }

    /// Returns the value layout for the account balance
    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::U64])
    }
}

//---------------------------------------------------------------------------
// Account type represenation
//---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountRoleSpecifier {
    DiemRoot,
    TreasuryCompliance,
    DesignatedDealer,
    Validator,
    ValidatorOperator,
    ParentVASP,
    ChildVASP,
}

impl AccountRoleSpecifier {
    pub fn id(&self) -> u64 {
        match self {
            Self::DiemRoot => 0,
            Self::TreasuryCompliance => 1,
            Self::DesignatedDealer => 2,
            Self::Validator => 3,
            Self::ValidatorOperator => 4,
            Self::ParentVASP => 5,
            Self::ChildVASP => 6,
        }
    }

    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::U64])
    }

    pub fn to_value(&self) -> Value {
        Value::struct_(Struct::pack(vec![Value::u64(self.id())], true))
    }

    pub fn role_id_struct_tag() -> StructTag {
        StructTag {
            address: account_config::CORE_CODE_ADDRESS,
            module: RoleId::module_identifier(),
            name: RoleId::struct_identifier(),
            type_params: vec![],
        }
    }
}

impl FromStr for AccountRoleSpecifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "vasp" => Ok(AccountRoleSpecifier::ParentVASP), // TODO: rename from vasp
            "validator" => Ok(AccountRoleSpecifier::Validator),
            "validator_operator" => Ok(AccountRoleSpecifier::ValidatorOperator),
            other => Err(Error::msg(format!(
                "Unrecognized account type specifier {} found.",
                other
            ))),
        }
    }
}

impl Default for AccountRoleSpecifier {
    fn default() -> Self {
        AccountRoleSpecifier::ParentVASP
    }
}

//---------------------------------------------------------------------------
// Account type resource represenation
//---------------------------------------------------------------------------

/// Struct that represents an account type for testing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountRole {
    self_address: AccountAddress,
    account_specifier: AccountRoleSpecifier,
}

impl AccountRole {
    /// Create a new AccountRole testing account.
    pub fn new(self_address: AccountAddress, account_specifier: AccountRoleSpecifier) -> Self {
        Self {
            self_address,
            account_specifier,
        }
    }

    pub fn account_specifier(&self) -> AccountRoleSpecifier {
        self.account_specifier
    }
}

//---------------------------------------------------------------------------
// Event generator resource represenation
//---------------------------------------------------------------------------

/// Struct that represents the event generator resource stored under accounts

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventHandleGenerator {
    counter: u64,
    addr: AccountAddress,
}

impl EventHandleGenerator {
    pub fn new(addr: AccountAddress) -> Self {
        Self { addr, counter: 0 }
    }

    pub fn new_with_event_count(addr: AccountAddress, counter: u64) -> Self {
        Self { addr, counter }
    }

    pub fn to_value(&self) -> Value {
        Value::struct_(Struct::pack(
            vec![Value::u64(self.counter), Value::address(self.addr)],
            true,
        ))
    }
    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::U64, MoveTypeLayout::Address])
    }
}

/// Represents an account along with initial state about it.
///
/// `AccountData` captures the initial state needed to create accounts for tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountData {
    account: Account,
    withdrawal_capability: Option<WithdrawCapability>,
    key_rotation_capability: Option<KeyRotationCapability>,
    sequence_number: u64,
    sent_events: EventHandle,
    received_events: EventHandle,

    balances: BTreeMap<Identifier, Balance>,
    event_generator: EventHandleGenerator,
    account_role: AccountRole,
}

fn new_event_handle(count: u64, address: AccountAddress) -> EventHandle {
    EventHandle::new_from_address(&address, count)
}

impl AccountData {
    /// Creates a new `AccountData` with a new account.
    ///
    /// This constructor is non-deterministic and should not be used against golden file.
    pub fn new(balance: u64, sequence_number: u64) -> Self {
        Self::with_account(
            Account::new(),
            balance,
            xus_currency_code(),
            sequence_number,
            AccountRoleSpecifier::ParentVASP,
        )
    }

    /// Creates a new `AccountData` with a new account.
    ///
    /// Most tests will want to use this constructor.
    pub fn new_from_seed(seed: &mut KeyGen, balance: u64, sequence_number: u64) -> Self {
        Self::with_account(
            Account::new_from_seed(seed),
            balance,
            xus_currency_code(),
            sequence_number,
            AccountRoleSpecifier::ParentVASP,
        )
    }

    /// Creates a new `AccountData` with the provided account.
    pub fn with_account(
        account: Account,
        balance: u64,
        balance_currency_code: Identifier,
        sequence_number: u64,
        account_specifier: AccountRoleSpecifier,
    ) -> Self {
        Self::with_account_and_event_counts(
            account,
            balance,
            balance_currency_code,
            sequence_number,
            0,
            0,
            account_specifier,
        )
    }

    /// Creates a new `AccountData` with the provided account.
    pub fn with_keypair(
        privkey: Ed25519PrivateKey,
        pubkey: Ed25519PublicKey,
        balance: u64,
        balance_currency_code: Identifier,
        sequence_number: u64,
        account_specifier: AccountRoleSpecifier,
    ) -> Self {
        let account = Account::with_keypair(privkey, pubkey);
        Self::with_account(
            account,
            balance,
            balance_currency_code,
            sequence_number,
            account_specifier,
        )
    }

    /// Creates a new `AccountData` with custom parameters.
    pub fn with_account_and_event_counts(
        account: Account,
        balance: u64,
        balance_currency_code: Identifier,
        sequence_number: u64,
        sent_events_count: u64,
        received_events_count: u64,
        account_specifier: AccountRoleSpecifier,
    ) -> Self {
        let mut balances = BTreeMap::new();
        balances.insert(balance_currency_code, Balance::new(balance));
        let addr = *account.address();
        Self {
            account_role: AccountRole::new(*account.address(), account_specifier),
            event_generator: EventHandleGenerator::new_with_event_count(addr, 2),
            withdrawal_capability: Some(WithdrawCapability::new(addr)),
            key_rotation_capability: Some(KeyRotationCapability::new(addr)),
            account,
            balances,
            sequence_number,
            sent_events: new_event_handle(sent_events_count, addr),
            received_events: new_event_handle(received_events_count, addr),
        }
    }

    /// Adds the balance held by this account to the one represented as balance_currency_code
    pub fn add_balance_currency(&mut self, balance_currency_code: Identifier) {
        self.balances.insert(balance_currency_code, Balance::new(0));
    }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) {
        self.account.rotate_key(privkey, pubkey)
    }

    pub fn sent_payment_event_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Address,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
        ])
    }

    pub fn received_payment_event_type() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Address,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
        ])
    }

    pub fn event_handle_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
        ])
    }

    /// Returns the (Move value) layout of the DiemAccount::DiemAccount struct
    pub fn layout() -> MoveStructLayout {
        use MoveStructLayout as S;
        use MoveTypeLayout as T;

        S::new(vec![
            T::Vector(Box::new(T::U8)),
            T::Vector(Box::new(T::Struct(WithdrawCapability::layout()))),
            T::Vector(Box::new(T::Struct(KeyRotationCapability::layout()))),
            T::Struct(Self::event_handle_layout()),
            T::Struct(Self::event_handle_layout()),
            T::U64,
        ])
    }

    /// Returns the account role specifier
    pub fn account_role(&self) -> AccountRoleSpecifier {
        self.account_role.account_specifier()
    }

    /// Creates and returns the top-level resources to be published under the account
    pub fn to_value(&self) -> (Value, Vec<(Identifier, Value)>, Value, Value) {
        // TODO: publish some concept of Account
        let balances: Vec<_> = self
            .balances
            .iter()
            .map(|(code, balance)| (code.clone(), balance.to_value()))
            .collect();
        let event_generator = self.event_generator.to_value();
        let role_id = self.account_role.account_specifier.to_value();
        let account = Value::struct_(Struct::pack(
            vec![
                // TODO: this needs to compute the auth key instead
                Value::vector_u8(AuthenticationKey::ed25519(&self.account.pubkey).to_vec()),
                self.withdrawal_capability.as_ref().unwrap().value(),
                self.key_rotation_capability.as_ref().unwrap().value(),
                Value::struct_(Struct::pack(
                    vec![
                        Value::u64(self.received_events.count()),
                        Value::vector_u8(self.received_events.key().to_vec()),
                    ],
                    true,
                )),
                Value::struct_(Struct::pack(
                    vec![
                        Value::u64(self.sent_events.count()),
                        Value::vector_u8(self.sent_events.key().to_vec()),
                    ],
                    true,
                )),
                Value::u64(self.sequence_number),
            ],
            true,
        ));
        (account, balances, event_generator, role_id)
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
    pub fn make_balance_access_path(&self, code: Identifier) -> AccessPath {
        self.account.make_balance_access_path(code)
    }

    /// Returns the AccessPath that describes the EventHandleGenerator resource instance.
    ///
    /// Use this to retrieve or publish the EventHandleGenerator blob.
    pub fn make_event_generator_access_path(&self) -> AccessPath {
        self.account.make_event_generator_access_path()
    }

    /// Creates a writeset that contains the account data and can be patched to the storage
    /// directly.
    pub fn to_writeset(&self) -> WriteSet {
        let (account_blob, balance_blobs, event_generator_blob, role_id_blob) = self.to_value();
        let mut write_set = Vec::new();
        let account = account_blob
            .value_as::<Struct>()
            .unwrap()
            .simple_serialize(&AccountData::layout())
            .unwrap();
        write_set.push((self.make_account_access_path(), WriteOp::Value(account)));
        for (code, balance_blob) in balance_blobs.into_iter() {
            let balance = balance_blob
                .value_as::<Struct>()
                .unwrap()
                .simple_serialize(&Balance::layout())
                .unwrap();
            write_set.push((self.make_balance_access_path(code), WriteOp::Value(balance)));
        }

        let freezing_bit = FreezingBit::value()
            .value_as::<Struct>()
            .unwrap()
            .simple_serialize(&FreezingBit::layout())
            .unwrap();
        write_set.push((
            self.account
                .make_access_path(account_config::FreezingBit::struct_tag()),
            WriteOp::Value(freezing_bit),
        ));

        let role_id = role_id_blob
            .value_as::<Struct>()
            .unwrap()
            .simple_serialize(&AccountRoleSpecifier::layout())
            .unwrap();
        write_set.push((
            self.account
                .make_access_path(AccountRoleSpecifier::role_id_struct_tag()),
            WriteOp::Value(role_id),
        ));

        let event_generator = event_generator_blob
            .value_as::<Struct>()
            .unwrap()
            .simple_serialize(&EventHandleGenerator::layout())
            .unwrap();
        write_set.push((
            self.make_event_generator_access_path(),
            WriteOp::Value(event_generator),
        ));
        WriteSetMut::new(write_set).freeze().unwrap()
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
    pub fn balance(&self, currency_code: &IdentStr) -> u64 {
        self.balances.get(currency_code).unwrap().coin()
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawCapability {
    account_address: AccountAddress,
}
impl WithdrawCapability {
    pub fn new(account_address: AccountAddress) -> Self {
        Self { account_address }
    }

    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Address])
    }

    pub fn value(&self) -> Value {
        Value::vector_resource_for_testing_only(vec![Value::struct_(Struct::pack(
            vec![Value::address(self.account_address)],
            true,
        ))])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyRotationCapability {
    account_address: AccountAddress,
}
impl KeyRotationCapability {
    pub fn new(account_address: AccountAddress) -> Self {
        Self { account_address }
    }

    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Address])
    }

    pub fn value(&self) -> Value {
        Value::vector_resource_for_testing_only(vec![Value::struct_(Struct::pack(
            vec![Value::address(self.account_address)],
            true,
        ))])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreezingBit {
    is_frozen: bool,
}

impl FreezingBit {
    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Bool])
    }

    pub fn value() -> Value {
        Value::struct_(Struct::pack(vec![Value::bool(false)], true))
    }
}
