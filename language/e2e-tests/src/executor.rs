// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for running the VM to execute and verify transactions.

use crate::{
    account::{Account, AccountData},
    data_store::{FakeDataStore, GENESIS_CHANGE_SET, GENESIS_CHANGE_SET_FRESH},
};
use bytecode_verifier::VerifiedModule;
use libra_config::generator;
use libra_crypto::HashValue;
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_config::{AccountResource, BalanceResource},
    block_metadata::{new_block_event_key, BlockMetadata, NewBlockEvent},
    on_chain_config::{OnChainConfig, VMPublishingOption, ValidatorSet},
    transaction::{
        SignedTransaction, Transaction, TransactionOutput, TransactionStatus, VMValidatorResult,
    },
    vm_error::{StatusCode, VMStatus},
    write_set::WriteSet,
};
use libra_vm::{LibraVM, VMExecutor, VMValidator};
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use stdlib::{stdlib_modules, transaction_scripts::StdlibScript, StdLibOptions};
use vm::CompiledModule;
use vm_genesis::GENESIS_KEYPAIR;

/// Provides an environment to run a VM instance.
///
/// This struct is a mock in-memory implementation of the Libra executor.
#[derive(Debug)]
pub struct FakeExecutor {
    data_store: FakeDataStore,
    block_time: u64,
}

impl FakeExecutor {
    /// Creates an executor from a genesis [`WriteSet`].
    pub fn from_genesis(write_set: &WriteSet) -> Self {
        let mut executor = FakeExecutor {
            data_store: FakeDataStore::default(),
            block_time: 0,
        };
        executor.apply_write_set(write_set);
        executor
    }

    /// Creates an executor from the genesis file GENESIS_FILE_LOCATION
    pub fn from_genesis_file() -> Self {
        Self::from_genesis(GENESIS_CHANGE_SET.clone().write_set())
    }

    /// Creates an executor using the standard genesis.
    pub fn from_fresh_genesis() -> Self {
        Self::from_genesis(GENESIS_CHANGE_SET_FRESH.clone().write_set())
    }

    pub fn whitelist_genesis() -> Self {
        Self::custom_genesis(
            stdlib_modules(StdLibOptions::Staged).to_vec(),
            None,
            VMPublishingOption::Locked(StdlibScript::whitelist()),
        )
    }

    /// Creates an executor from the genesis file GENESIS_FILE_LOCATION with script/module
    /// publishing options given by `publishing_options`. These can only be either `Open` or
    /// `CustomScript`.
    pub fn from_genesis_with_options(publishing_options: VMPublishingOption) -> Self {
        if let VMPublishingOption::Locked(_) = publishing_options {
            panic!("Whitelisted transactions are not supported as a publishing option")
        }

        Self::custom_genesis(
            stdlib_modules(StdLibOptions::Staged).to_vec(),
            None,
            publishing_options,
        )
    }

    /// Creates an executor in which no genesis state has been applied yet.
    pub fn no_genesis() -> Self {
        FakeExecutor {
            data_store: FakeDataStore::default(),
            block_time: 0,
        }
    }

    /// Creates fresh genesis from the stdlib modules passed in.
    pub fn custom_genesis(
        genesis_modules: Vec<VerifiedModule>,
        validator_accounts: Option<usize>,
        publishing_options: VMPublishingOption,
    ) -> Self {
        let genesis_change_set = {
            let validator_count = validator_accounts.map_or(10, |s| s);
            let swarm = generator::validator_swarm_for_testing(validator_count);

            vm_genesis::encode_genesis_change_set(
                &GENESIS_KEYPAIR.1,
                &vm_genesis::validator_registrations(&swarm.nodes),
                &genesis_modules,
                publishing_options,
            )
            .0
        };
        Self::from_genesis(genesis_change_set.write_set())
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
    pub fn read_account_resource(&self, account: &Account) -> Option<AccountResource> {
        let ap = account.make_account_access_path();
        let data_blob = StateView::get(&self.data_store, &ap)
            .expect("account must exist in data store")
            .expect("data must exist in data store");
        lcs::from_bytes(data_blob.as_slice()).ok()
    }

    /// Reads the balance resource value for an account from this executor's data store with the
    /// given balance currency_code.
    pub fn read_balance_resource(
        &self,
        account: &Account,
        balance_currency_code: Identifier,
    ) -> Option<BalanceResource> {
        let ap = account.make_balance_access_path(balance_currency_code);
        let data_blob = StateView::get(&self.data_store, &ap)
            .expect("account must exist in data store")
            .expect("data must exist in data store");
        lcs::from_bytes(data_blob.as_slice()).ok()
    }

    /// Executes the given block of transactions.
    ///
    /// Typical tests will call this method and check that the output matches what was expected.
    /// However, this doesn't apply the results of successful transactions to the data store.
    pub fn execute_block(
        &self,
        txn_block: Vec<SignedTransaction>,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        LibraVM::execute_block(
            txn_block
                .into_iter()
                .map(Transaction::UserTransaction)
                .collect(),
            &self.data_store,
        )
    }

    /// Executes the transaction as a singleton block and applies the resulting write set to the
    /// data store. Panics if execution fails
    pub fn execute_and_apply(&mut self, transaction: SignedTransaction) -> TransactionOutput {
        let mut outputs = self.execute_block(vec![transaction]).unwrap();
        assert!(outputs.len() == 1, "transaction outputs size mismatch");
        let output = outputs.pop().unwrap();
        match output.status() {
            TransactionStatus::Keep(status) => {
                self.apply_write_set(output.write_set());
                assert!(
                    status.major_status == StatusCode::EXECUTED,
                    "transaction failed with {:?}",
                    status
                );
                output
            }
            TransactionStatus::Discard(status) => panic!("transaction discarded with {:?}", status),
            TransactionStatus::Retry => panic!("transaction status is retry"),
        }
    }

    pub fn execute_transaction_block(
        &self,
        txn_block: Vec<Transaction>,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        LibraVM::execute_block(txn_block, &self.data_store)
    }

    pub fn execute_transaction(&self, txn: SignedTransaction) -> TransactionOutput {
        let txn_block = vec![txn];
        let mut outputs = self
            .execute_block(txn_block)
            .expect("The VM should not fail to startup");
        outputs
            .pop()
            .expect("A block with one transaction should have one output")
    }

    /// Get the blob for the associated AccessPath
    pub fn read_from_access_path(&self, path: &AccessPath) -> Option<Vec<u8>> {
        StateView::get(&self.data_store, path).unwrap()
    }

    /// Verifies the given transaction by running it through the VM verifier.
    pub fn verify_transaction(&self, txn: SignedTransaction) -> VMValidatorResult {
        let mut vm = LibraVM::new();
        vm.load_configs(self.get_state_view());
        vm.validate_transaction(txn, &self.data_store)
    }

    pub fn get_state_view(&self) -> &FakeDataStore {
        &self.data_store
    }

    pub fn new_block(&mut self) {
        let validator_set = ValidatorSet::fetch_config(&self.data_store)
            .expect("Unable to retrieve the validator set from storage");
        self.block_time += 1;
        let new_block = BlockMetadata::new(
            HashValue::zero(),
            0,
            self.block_time,
            vec![],
            *validator_set.payload()[0].account_address(),
        );
        let output = self
            .execute_transaction_block(vec![Transaction::BlockMetadata(new_block)])
            .expect("Executing block prologue should succeed")
            .pop()
            .expect("Failed to get the execution result for Block Prologue");
        // check if we emit the expected event, there might be more events for transaction fees
        let event = output.events()[0].clone();
        assert!(event.key() == &new_block_event_key());
        assert!(lcs::from_bytes::<NewBlockEvent>(event.event_data()).is_ok());
        self.apply_write_set(output.write_set());
    }
}
