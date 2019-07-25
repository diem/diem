// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for running the VM to execute and verify transactions.

use crate::{
    account::{Account, AccountData},
    data_store::{FakeDataStore, GENESIS_WRITE_SET},
};
use config::config::{NodeConfig, NodeConfigHelpers, VMPublishingOption};
use state_view::StateView;
use types::{
    access_path::AccessPath,
    language_storage::ModuleId,
    transaction::{SignedTransaction, TransactionOutput},
    vm_error::VMStatus,
    write_set::WriteSet,
};
use vm::CompiledModule;
use vm_runtime::{
    loaded_data::{struct_def::StructDef, types::Type},
    value::Value,
    MoveVM, VMExecutor, VMVerifier,
};

/// Provides an environment to run a VM instance.
///
/// This struct is a mock in-memory implementation of the Libra executor.
#[derive(Debug)]
pub struct FakeExecutor {
    config: NodeConfig,
    data_store: FakeDataStore,
}

impl FakeExecutor {
    /// Creates an executor from a genesis [`WriteSet`].
    pub fn from_genesis(
        write_set: &WriteSet,
        publishing_options: Option<VMPublishingOption>,
    ) -> Self {
        let mut executor = FakeExecutor {
            config: NodeConfigHelpers::get_single_node_test_config_publish_options(
                false,
                publishing_options,
            ),
            data_store: FakeDataStore::default(),
        };
        executor.apply_write_set(write_set);
        executor
    }

    /// Creates an executor from the genesis file GENESIS_FILE_LOCATION
    pub fn from_genesis_file() -> Self {
        Self::from_genesis(&GENESIS_WRITE_SET, None)
    }

    /// Creates an executor from the genesis file GENESIS_FILE_LOCATION with script/module
    /// publishing options given by `publishing_options`. These can only be either `Open` or
    /// `CustomScript`.
    pub fn from_genesis_with_options(publishing_options: VMPublishingOption) -> Self {
        if let VMPublishingOption::Locked(_) = publishing_options {
            panic!("Whitelisted transactions are not supported as a publishing option")
        }
        Self::from_genesis(&GENESIS_WRITE_SET, Some(publishing_options))
    }

    /// Creates an executor in which no genesis state has been applied yet.
    pub fn no_genesis() -> Self {
        FakeExecutor {
            config: NodeConfigHelpers::get_single_node_test_config(false),
            data_store: FakeDataStore::default(),
        }
    }

    /// Creates a number of [`Account`] instances all with the same balance and sequence number,
    /// and publishes them to this executor's data store.
    pub fn create_accounts(&mut self, size: usize, balance: u64, seq_num: u64) -> Vec<Account> {
        let mut accounts: Vec<Account> = Vec::with_capacity(size);
        for _i in 0..size {
            let account_data = AccountData::new(balance, seq_num);
            self.add_account_data(&account_data);
            accounts.push(account_data.into_account());
        }
        accounts
    }

    /// Applies a [`WriteSet`] to this executor's data store.
    pub fn apply_write_set(&mut self, write_set: &WriteSet) {
        self.data_store.add_write_set(write_set);
    }

    /// Adds an account to this executor's data store.
    pub fn add_account_data(&mut self, account_data: &AccountData) {
        self.data_store.add_account_data(account_data)
    }

    /// Adds a module to this executor's data store.
    ///
    /// Does not do any sort of verification on the module.
    pub fn add_module(&mut self, module_id: &ModuleId, module: &CompiledModule) {
        self.data_store.add_module(module_id, module)
    }

    /// Reads the resource [`Value`] for an account from this executor's data store.
    pub fn read_account_resource(&self, account: &Account) -> Option<Value> {
        let ap = account.make_access_path();
        let data_blob = StateView::get(&self.data_store, &ap)
            .expect("account must exist in data store")
            .expect("data must exist in data store");
        let account_type = Self::get_account_struct_def();
        Account::read_account_resource(&data_blob, account_type)
    }

    /// Executes the given block of transactions.
    ///
    /// Typical tests will call this method and check that the output matches what was expected.
    /// However, this doesn't apply the results of successful transactions to the data store.
    pub fn execute_block(&self, txn_block: Vec<SignedTransaction>) -> Vec<TransactionOutput> {
        MoveVM::execute_block(txn_block, &self.config.vm_config, &self.data_store)
    }

    pub fn execute_transaction(&self, txn: SignedTransaction) -> TransactionOutput {
        let txn_block = vec![txn];
        let mut outputs = self.execute_block(txn_block);
        outputs
            .pop()
            .expect("A block with one transaction should have one output")
    }

    /// Get the blob for the associated AccessPath
    pub fn read_from_access_path(&self, path: &AccessPath) -> Option<Vec<u8>> {
        StateView::get(&self.data_store, path).unwrap()
    }

    /// Verifies the given transaction by running it through the VM verifier.
    pub fn verify_transaction(&self, txn: SignedTransaction) -> Option<VMStatus> {
        let vm = MoveVM::new(&self.config.vm_config);
        vm.validate_transaction(txn, &self.data_store)
    }

    /// TODO: This is a hack and likely to break soon. THe Account type is replicated here with no
    /// checks that is the right now. Fix it!
    fn get_account_struct_def() -> StructDef {
        // STRUCT DEF StructDef(StructDefInner { field_definitions: [ByteArray,
        // Struct(StructDef(StructDefInner { field_definitions: [U64] })), U64, U64,
        // U64] }) let coin = StructDef(StructDefInner { field_definitions:
        // [Type::U64] })
        let int_type = Type::U64;
        let byte_array_type = Type::ByteArray;
        let coin = Type::Struct(StructDef::new(vec![int_type.clone()]));
        StructDef::new(vec![
            byte_array_type,
            coin,
            Type::Bool,
            int_type.clone(),
            int_type.clone(),
            int_type.clone(),
        ])
    }
}
