// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for running the VM to execute and verify transactions.

use crate::{
    account::{Account, AccountData},
    data_store::{FakeDataStore, GENESIS_CHANGE_SET, GENESIS_CHANGE_SET_FRESH},
    golden_outputs::GoldenOutputs,
    keygen::KeyGen,
};
use compiled_stdlib::{stdlib_modules, transaction_scripts::StdlibScript, StdLibOptions};
use diem_crypto::HashValue;
use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath,
    account_config::{AccountResource, BalanceResource, CORE_CODE_ADDRESS},
    block_metadata::{new_block_event_key, BlockMetadata, NewBlockEvent},
    on_chain_config::{OnChainConfig, VMPublishingOption, ValidatorSet},
    transaction::{
        SignedTransaction, Transaction, TransactionOutput, TransactionStatus, VMValidatorResult,
    },
    vm_status::{KeptVMStatus, VMStatus},
    write_set::WriteSet,
};
use diem_vm::{
    data_cache::RemoteStorage, txn_effects_to_writeset_and_events, DiemVM, DiemVMValidator,
    VMExecutor, VMValidator,
};
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use move_vm_runtime::{logging::NoContextLog, move_vm::MoveVM};
use move_vm_types::{
    gas_schedule::{zero_cost_schedule, CostStrategy},
    values::Value,
};
use vm::CompiledModule;

static RNG_SEED: [u8; 32] = [9u8; 32];

/// Provides an environment to run a VM instance.
///
/// This struct is a mock in-memory implementation of the Diem executor.
#[derive(Debug)]
pub struct FakeExecutor {
    data_store: FakeDataStore,
    block_time: u64,
    executed_output: Option<GoldenOutputs>,
    rng: KeyGen,
}

impl FakeExecutor {
    /// Creates an executor from a genesis [`WriteSet`].
    pub fn from_genesis(write_set: &WriteSet) -> Self {
        let mut executor = FakeExecutor {
            data_store: FakeDataStore::default(),
            block_time: 0,
            executed_output: None,
            rng: KeyGen::from_seed(RNG_SEED),
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

    pub fn allowlist_genesis() -> Self {
        Self::custom_genesis(
            stdlib_modules(StdLibOptions::Compiled).to_vec(),
            None,
            VMPublishingOption::locked(StdlibScript::allowlist()),
        )
    }

    /// Creates an executor from the genesis file GENESIS_FILE_LOCATION with script/module
    /// publishing options given by `publishing_options`. These can only be either `Open` or
    /// `CustomScript`.
    pub fn from_genesis_with_options(publishing_options: VMPublishingOption) -> Self {
        if !publishing_options.is_open_script() {
            panic!("Allowlisted transactions are not supported as a publishing option")
        }

        Self::custom_genesis(
            stdlib_modules(StdLibOptions::Compiled).to_vec(),
            None,
            publishing_options,
        )
    }

    /// Creates an executor in which no genesis state has been applied yet.
    pub fn no_genesis() -> Self {
        FakeExecutor {
            data_store: FakeDataStore::default(),
            block_time: 0,
            executed_output: None,
            rng: KeyGen::from_seed(RNG_SEED),
        }
    }

    pub fn set_golden_file(&mut self, test_name: &str) {
        self.executed_output = Some(GoldenOutputs::new(test_name));
    }

    /// Creates an executor with only the standard library Move modules published and not other
    /// initialization done.
    pub fn stdlib_only_genesis() -> Self {
        let mut genesis = Self::no_genesis();
        for module in stdlib_modules(StdLibOptions::Compiled) {
            let id = module.self_id();
            genesis.add_module(&id, module);
        }
        genesis
    }

    /// Creates fresh genesis from the stdlib modules passed in.
    pub fn custom_genesis(
        genesis_modules: Vec<CompiledModule>,
        validator_accounts: Option<usize>,
        publishing_options: VMPublishingOption,
    ) -> Self {
        let genesis = vm_genesis::generate_test_genesis(
            &genesis_modules,
            publishing_options,
            validator_accounts,
        );
        Self::from_genesis(genesis.0.write_set())
    }

    /// Create one instance of [`AccountData`] without saving it to data store.
    pub fn create_raw_account(&mut self) -> Account {
        Account::new_from_seed(&mut self.rng)
    }

    /// Create one instance of [`AccountData`] without saving it to data store.
    pub fn create_raw_account_data(&mut self, balance: u64, seq_num: u64) -> AccountData {
        AccountData::new_from_seed(&mut self.rng, balance, seq_num)
    }

    /// Creates a number of [`Account`] instances all with the same balance and sequence number,
    /// and publishes them to this executor's data store.
    pub fn create_accounts(&mut self, size: usize, balance: u64, seq_num: u64) -> Vec<Account> {
        let mut accounts: Vec<Account> = Vec::with_capacity(size);
        for _i in 0..size {
            let account_data = AccountData::new_from_seed(&mut self.rng, balance, seq_num);
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
            .unwrap_or_else(|| panic!("Can't fetch account resource for {}", account.address()));
        bcs::from_bytes(data_blob.as_slice()).ok()
    }

    /// Reads the balance resource value for an account from this executor's data store with the
    /// given balance currency_code.
    pub fn read_balance_resource(
        &self,
        account: &Account,
        balance_currency_code: Identifier,
    ) -> Option<BalanceResource> {
        let ap = account.make_balance_access_path(balance_currency_code);
        StateView::get(&self.data_store, &ap)
            .unwrap_or_else(|_| panic!("account {:?} must exist in data store", account.address()))
            .map(|data_blob| {
                bcs::from_bytes(data_blob.as_slice()).expect("Failure decoding balance resource")
            })
    }

    /// Executes the given block of transactions.
    ///
    /// Typical tests will call this method and check that the output matches what was expected.
    /// However, this doesn't apply the results of successful transactions to the data store.
    pub fn execute_block(
        &self,
        txn_block: Vec<SignedTransaction>,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        self.execute_transaction_block(
            txn_block
                .into_iter()
                .map(Transaction::UserTransaction)
                .collect(),
        )
    }

    /// Alternate form of 'execute_block' that keeps the vm_status before it goes into the
    /// `TransactionOutput`
    pub fn execute_block_and_keep_vm_status(
        &self,
        txn_block: Vec<SignedTransaction>,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
        DiemVM::execute_block_and_keep_vm_status(
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
                    status == &KeptVMStatus::Executed,
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
        let output = DiemVM::execute_block(txn_block, &self.data_store);
        if let Some(logger) = &self.executed_output {
            logger.log(format!("{:?}\n", output).as_str());
        }
        output
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
        let vm = DiemVMValidator::new(self.get_state_view());
        vm.validate_transaction(txn, &self.data_store)
    }

    pub fn get_state_view(&self) -> &FakeDataStore {
        &self.data_store
    }

    pub fn new_block(&mut self) {
        self.new_block_with_timestamp(self.block_time + 1);
    }

    pub fn new_block_with_timestamp(&mut self, time_stamp: u64) {
        let validator_set = ValidatorSet::fetch_config(&self.data_store)
            .expect("Unable to retrieve the validator set from storage");
        self.block_time = time_stamp;
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
        assert_eq!(event.key(), &new_block_event_key());
        assert!(bcs::from_bytes::<NewBlockEvent>(event.event_data()).is_ok());
        self.apply_write_set(output.write_set());
    }

    fn module(name: &str) -> ModuleId {
        ModuleId::new(CORE_CODE_ADDRESS, Identifier::new(name).unwrap())
    }

    fn name(name: &str) -> Identifier {
        Identifier::new(name).unwrap()
    }

    pub fn set_block_time(&mut self, new_block_time: u64) {
        self.block_time = new_block_time;
    }

    pub fn get_block_time(&mut self) -> u64 {
        self.block_time
    }

    pub fn exec(
        &mut self,
        module_name: &str,
        function_name: &str,
        type_params: Vec<TypeTag>,
        args: Vec<Value>,
        sender: &AccountAddress,
    ) {
        let write_set = {
            let cost_table = zero_cost_schedule();
            let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(100_000_000));
            let vm = MoveVM::new();
            let remote_view = RemoteStorage::new(&self.data_store);
            let mut session = vm.new_session(&remote_view);
            let log_context = NoContextLog::new();
            session
                .execute_function(
                    &Self::module(module_name),
                    &Self::name(function_name),
                    type_params,
                    args,
                    *sender,
                    &mut cost_strategy,
                    &log_context,
                )
                .unwrap_or_else(|e| {
                    panic!(
                        "Error calling {}.{}: {}",
                        module_name,
                        function_name,
                        e.into_vm_status()
                    )
                });
            let effects = session.finish().expect("Failed to generate txn effects");
            let (writeset, _events) =
                txn_effects_to_writeset_and_events(effects).expect("Failed to generate writeset");
            writeset
        };
        self.data_store.add_write_set(&write_set);
    }

    pub fn try_exec(
        &mut self,
        module_name: &str,
        function_name: &str,
        type_params: Vec<TypeTag>,
        args: Vec<Value>,
        sender: &AccountAddress,
    ) -> Result<WriteSet, VMStatus> {
        let cost_table = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(100_000_000));
        let vm = MoveVM::new();
        let remote_view = RemoteStorage::new(&self.data_store);
        let mut session = vm.new_session(&remote_view);
        let log_context = NoContextLog::new();
        session
            .execute_function(
                &Self::module(module_name),
                &Self::name(function_name),
                type_params,
                args,
                *sender,
                &mut cost_strategy,
                &log_context,
            )
            .map_err(|e| e.into_vm_status())?;
        let effects = session.finish().expect("Failed to generate txn effects");
        let (writeset, _events) =
            txn_effects_to_writeset_and_events(effects).expect("Failed to generate writeset");
        Ok(writeset)
    }
}
