// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for running the VM to execute and verify transactions.

use crate::{
    account::{Account, AccountData},
    data_store::{FakeDataStore, GENESIS_WRITE_SET},
};
use bytecode_verifier::VerifiedModule;
use libra_config::{
    config::{VMConfig, VMPublishingOption},
    generator,
};
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_config::AccountResource,
    crypto_proxies::ValidatorSet,
    discovery_set::mock::mock_discovery_set,
    language_storage::ModuleId,
    transaction::{
        SignedTransaction, Transaction, TransactionOutput, TransactionPayload, TransactionStatus,
    },
    vm_error::{StatusCode, VMStatus},
    write_set::WriteSet,
};
use libra_vm::{LibraVM, VMExecutor, VMVerifier};
use stdlib::{stdlib_modules, StdLibOptions};
use vm::CompiledModule;
use vm_genesis::GENESIS_KEYPAIR;

/// Provides an environment to run a VM instance.
///
/// This struct is a mock in-memory implementation of the Libra executor.
#[derive(Debug)]
pub struct FakeExecutor {
    config: VMConfig,
    data_store: FakeDataStore,
}

pub fn test_all_genesis_impl<T, F>(
    publishing_options: Option<VMPublishingOption>,
    test_fn: F,
) -> Result<(), T>
where
    F: Fn(FakeExecutor) -> Result<(), T>,
{
    let genesis: Vec<&WriteSet> = vec![&GENESIS_WRITE_SET];
    genesis
        .iter()
        .map(|ws| test_fn(FakeExecutor::from_genesis(ws, publishing_options.clone())))
        .collect()
}

pub fn test_all_genesis_default(test_fn: fn(FakeExecutor) -> ()) {
    let result: Result<(), ()> = test_all_genesis_impl(None, |executor| {
        test_fn(executor);
        Ok(())
    });
    result.unwrap()
}

pub fn test_all_genesis(
    publishing_options: Option<VMPublishingOption>,
    test_fn: fn(FakeExecutor) -> (),
) {
    let result: Result<(), ()> = test_all_genesis_impl(publishing_options, |executor| {
        test_fn(executor);
        Ok(())
    });
    result.unwrap()
}

impl FakeExecutor {
    /// Creates an executor from a genesis [`WriteSet`].
    pub fn from_genesis(
        write_set: &WriteSet,
        publishing_options: Option<VMPublishingOption>,
    ) -> Self {
        let mut config = VMConfig::default();
        if let Some(vm_publishing_options) = publishing_options {
            config.publishing_options = vm_publishing_options;
        }

        let mut executor = FakeExecutor {
            config,
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
            config: VMConfig::default(),
            data_store: FakeDataStore::default(),
        }
    }

    /// Creates fresh genesis from the stdlib modules passed in. If none are passed in the staged
    /// genesis write set is used.
    pub fn custom_genesis(
        genesis_modules: Option<Vec<VerifiedModule>>,
        validator_set: Option<ValidatorSet>,
        publishing_options: VMPublishingOption,
    ) -> Self {
        let genesis_write_set = if genesis_modules.is_none() && validator_set.is_none() {
            GENESIS_WRITE_SET.clone()
        } else {
            let validator_set = validator_set
                .unwrap_or_else(|| generator::validator_swarm_for_testing(10).validator_set);
            let discovery_set = mock_discovery_set(&validator_set);
            let stdlib_modules =
                genesis_modules.unwrap_or_else(|| stdlib_modules(StdLibOptions::Staged).to_vec());
            match vm_genesis::encode_genesis_transaction_with_validator_and_modules(
                &GENESIS_KEYPAIR.0,
                GENESIS_KEYPAIR.1.clone(),
                validator_set,
                discovery_set,
                &stdlib_modules,
            )
            .payload()
            {
                TransactionPayload::WriteSet(ws) => ws.write_set().clone(),
                _ => panic!("Expected writeset txn in genesis txn"),
            }
        };
        Self::from_genesis(&genesis_write_set, Some(publishing_options))
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
        let ap = account.make_access_path();
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
            &self.config,
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
        LibraVM::execute_block(txn_block, &self.config, &self.data_store)
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
    pub fn verify_transaction(&self, txn: SignedTransaction) -> Option<VMStatus> {
        let mut vm = LibraVM::new(&self.config);
        vm.load_configs(self.get_state_view());
        vm.validate_transaction(txn, &self.data_store)
    }

    pub fn get_state_view(&self) -> &FakeDataStore {
        &self.data_store
    }
    pub fn config(&self) -> &VMConfig {
        &self.config
    }
}
