// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Test infrastructure for modeling Libra accounts.

use crypto::{PrivateKey, PublicKey};
use lazy_static::lazy_static;
use std::{convert::TryInto, time::Duration};
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    byte_array::ByteArray,
    transaction::{Program, RawTransaction, SignedTransaction, TransactionArgument},
};
use vm_genesis::GENESIS_KEYPAIR;
use vm_runtime::{
    identifier::create_access_path,
    loaded_data::struct_def::StructDef,
    value::{MutVal, Value},
};

// StdLib account, it is where the code is and needed to make access path to Account resources
lazy_static! {
    static ref STDLIB_ADDRESS: AccountAddress = { account_config::core_code_address() };
}

/// Details about a Libra account.
///
/// Tests will typically create a set of `Account` instances to run transactions on. This type
/// encodes the logic to operate on and verify operations on any Libra account.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Account {
    addr: AccountAddress,
    /// The current private key for this account.
    pub privkey: PrivateKey,
    /// The current public key for this account.
    pub pubkey: PublicKey,
}

impl Account {
    /// Creates a new account in memory.
    ///
    /// The account returned by this constructor is a purely logical entity, meaning that it does
    /// not automatically get added to the Libra store. To add an account to the store, use
    /// [`AccountData`] instances with
    /// [`FakeExecutor::add_account_data`][crate::executor::FakeExecutor::add_account_data].
    pub fn new() -> Self {
        let (privkey, pubkey) = crypto::signing::generate_keypair();
        Self::with_keypair(privkey, pubkey)
    }

    /// Creates a new account with the given keypair.
    ///
    /// Like with [`Account::new`], the account returned by this constructor is a purely logical
    /// entity.
    pub fn with_keypair(privkey: PrivateKey, pubkey: PublicKey) -> Self {
        let addr = pubkey.into();
        Account {
            addr,
            privkey,
            pubkey,
        }
    }

    /// Creates a new account representing the association in memory.
    ///
    /// The address will be [`association_address`][account_config::association_address], and
    /// the account will use [`GENESIS_KEYPAIR`][struct@GENESIS_KEYPAIR] as its keypair.
    pub fn new_association() -> Self {
        Account {
            addr: account_config::association_address(),
            pubkey: GENESIS_KEYPAIR.1,
            privkey: GENESIS_KEYPAIR.0.clone(),
        }
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
    pub fn rotate_key(&mut self, privkey: PrivateKey, pubkey: PublicKey) {
        self.privkey = privkey;
        self.pubkey = pubkey;
    }

    /// Computes the authentication key for this account, as stored on the chain.
    ///
    /// This is the same as the account's address if the keys have never been rotated.
    pub fn auth_key(&self) -> AccountAddress {
        AccountAddress::from(self.pubkey)
    }

    //
    // Helpers to read data from an Account resource
    //

    //
    // Helpers for transaction creation with Account instance as sender
    //

    /// Returns a [`SignedTransaction`] with no arguments and this account as the sender.
    pub fn create_signed_txn(
        &self,
        program: Vec<u8>,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
    ) -> SignedTransaction {
        self.create_signed_txn_with_args(
            program,
            vec![],
            sequence_number,
            max_gas_amount,
            gas_unit_price,
        )
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
            Program::new(program, vec![], args),
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
            Program::new(program, vec![], args),
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
        program: Program,
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
            Duration::from_secs(u64::max_value()),
        )
        .sign(&self.privkey, self.pubkey)
        .unwrap()
        .into_inner()
    }

    /// Given a blob, materializes the VM Value behind it.
    pub(crate) fn read_account_resource(blob: &[u8], account_type: StructDef) -> Option<Value> {
        match Value::simple_deserialize(blob, account_type) {
            Ok(account) => Some(account),
            Err(_) => None,
        }
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
    sent_events_count: u64,
    received_events_count: u64,
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
        Self::with_account_and_event_counts(account, balance, sequence_number, 0, 0)
    }

    /// Creates a new `AccountData` with custom parameters.
    pub fn with_account_and_event_counts(
        account: Account,
        balance: u64,
        sequence_number: u64,
        sent_events_count: u64,
        received_events_count: u64,
    ) -> Self {
        Self {
            account,
            balance,
            sequence_number,
            sent_events_count,
            received_events_count,
        }
    }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: PrivateKey, pubkey: PublicKey) {
        self.account.rotate_key(privkey, pubkey)
    }

    /// Creates and returns a resource [`Value`] for this data.
    pub fn to_resource(&self) -> Value {
        // TODO: publish some concept of Account
        let coin = Value::Struct(vec![MutVal::new(Value::U64(self.balance))]);
        Value::Struct(vec![
            MutVal::new(Value::ByteArray(ByteArray::new(
                AccountAddress::from(self.account.pubkey).to_vec(),
            ))),
            MutVal::new(coin),
            MutVal::new(Value::U64(self.received_events_count)),
            MutVal::new(Value::U64(self.sent_events_count)),
            MutVal::new(Value::U64(self.sequence_number)),
        ])
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

    /// Returns the initial sent events count.
    pub fn sent_events_count(&self) -> u64 {
        self.sent_events_count
    }

    /// Returns the initial received events count.
    pub fn received_events_count(&self) -> u64 {
        self.received_events_count
    }
}

/// Helper methods for dealing with account resources as seen by the Libra VM.
pub enum AccountResource {}

impl AccountResource {
    /// Returns the authentication key read from a [`Value`] representing the account.
    pub fn read_auth_key(account: &Value) -> AccountAddress {
        // The return type is slightly confusing -- the auth key stored on the chain is actually
        // just a hash of the public key from the account. This may change in the future with
        // flexible authentication.
        match account {
            Value::Struct(fields) => {
                let auth_key = fields.get(0).expect("Auth key must be field 0 in Account");
                match &*auth_key.peek() {
                    Value::ByteArray(bytes) => bytes
                        .as_bytes()
                        .try_into()
                        .expect("Auth key must be parsable as an account address"),
                    _ => panic!("auth key must be a ByteArray"),
                }
            }
            _ => panic!("Account must be a Value::Struct"),
        }
    }

    /// Returns the balance read from a [`Value`] representing the account.
    pub fn read_balance(account: &Value) -> u64 {
        match account {
            Value::Struct(fields) => {
                let coin = fields
                    .get(1)
                    .expect("LibraCoin.T must be the second field in Account");
                match &*coin.peek() {
                    Value::Struct(balance) => {
                        let value = balance.get(0).expect("balance field must exist");
                        match &*value.peek() {
                            Value::U64(val) => *val,
                            _ => panic!("balance field must exist"),
                        }
                    }
                    _ => panic!("account must contain LibraCoin.T as second field"),
                }
            }
            _ => panic!("Account must be a Value::Struct"),
        }
    }

    /// Returns the received events count read from a [`Value`] representing the account.
    pub fn read_received_events_count(account: &Value) -> u64 {
        match account {
            Value::Struct(fields) => {
                let received_events_count = fields
                    .get(2)
                    .expect("received_events_count must be field 2 in Account");
                match &*received_events_count.peek() {
                    Value::U64(val) => *val,
                    _ => panic!("sequence number field must exist"),
                }
            }
            _ => panic!("Account must be a Value::Struct"),
        }
    }

    /// Returns the sent events count read from a [`Value`] representing the account.
    pub fn read_sent_events_count(account: &Value) -> u64 {
        match account {
            Value::Struct(fields) => {
                let sent_events_count = fields
                    .get(3)
                    .expect("sent_events_count must be field 3 in Account");
                match &*sent_events_count.peek() {
                    Value::U64(val) => *val,
                    _ => panic!("sequence number field must exist"),
                }
            }
            _ => panic!("Account must be a Value::Struct"),
        }
    }

    /// Returns the sequence number read from a [`Value`] representing the account.
    pub fn read_sequence_number(account: &Value) -> u64 {
        match account {
            Value::Struct(fields) => {
                let sequence_number = fields
                    .get(4)
                    .expect("sequence number must be the fifth field in Account");
                match &*sequence_number.peek() {
                    Value::U64(val) => *val,
                    _ => panic!("sequence number field must exist"),
                }
            }
            _ => panic!("Account must be a Value::Struct"),
        }
    }
}
