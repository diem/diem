// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for running the VM to execute and verify transactions.

use serde::Serialize;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    account::{Account, AccountData},
    data_store::{FakeDataStore, GENESIS_CHANGE_SET, GENESIS_CHANGE_SET_FRESH},
    golden_outputs::GoldenOutputs,
    keygen::KeyGen,
};
use diem_crypto::HashValue;
use diem_framework_releases::{
    current_module_blobs, current_modules, legacy::transaction_scripts::LegacyStdlibScript,
};
use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath,
    account_config::{AccountResource, BalanceResource, CORE_CODE_ADDRESS},
    block_metadata::{new_block_event_key, BlockMetadata, NewBlockEvent},
    on_chain_config::{DiemVersion, OnChainConfig, VMPublishingOption, ValidatorSet},
    transaction::{
        ChangeSet, SignedTransaction, Transaction, TransactionOutput, TransactionStatus,
        VMValidatorResult,
    },
    vm_status::{KeptVMStatus, VMStatus},
    write_set::WriteSet,
};
use diem_vm::{
    convert_changeset_and_events, data_cache::RemoteStorage, DiemVM, DiemVMValidator, VMExecutor,
    VMValidator,
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use move_vm_runtime::{logging::NoContextLog, move_vm::MoveVM};
use move_vm_types::gas_schedule::GasStatus;

static RNG_SEED: [u8; 32] = [9u8; 32];

pub const RELEASE_1_1_GENESIS: &[u8] =
    include_bytes!("../genesis-release-1-1/release-1-1-genesis.blob");
pub const RELEASE_1_1_GENESIS_PRIVKEY: &[u8] =
    include_bytes!("../genesis-release-1-1/release-1-1-privkey.blob");
pub const RELEASE_1_1_GENESIS_PUBKEY: &[u8] =
    include_bytes!("../genesis-release-1-1/release-1-1-pubkey.blob");

const ENV_TRACE_DIR: &str = "TRACE";

/// Directory structure of the trace dir
pub const TRACE_FILE_NAME: &str = "name";
pub const TRACE_FILE_ERROR: &str = "error";
pub const TRACE_DIR_META: &str = "meta";
pub const TRACE_DIR_DATA: &str = "data";
pub const TRACE_DIR_INPUT: &str = "input";
pub const TRACE_DIR_OUTPUT: &str = "output";

/// Maps block number N to the index of the input and output transactions
pub type TraceSeqMapping = (usize, Vec<usize>, Vec<usize>);

/// Provides an environment to run a VM instance.
///
/// This struct is a mock in-memory implementation of the Diem executor.
#[derive(Debug)]
pub struct FakeExecutor {
    data_store: FakeDataStore,
    block_time: u64,
    executed_output: Option<GoldenOutputs>,
    trace_dir: Option<PathBuf>,
    rng: KeyGen,
}

impl FakeExecutor {
    /// Creates an executor from a genesis [`WriteSet`].
    pub fn from_genesis(write_set: &WriteSet) -> Self {
        let mut executor = FakeExecutor {
            data_store: FakeDataStore::default(),
            block_time: 0,
            executed_output: None,
            trace_dir: None,
            rng: KeyGen::from_seed(RNG_SEED),
        };
        executor.apply_write_set(write_set);
        executor
    }

    /// Create an executor from a saved genesis blob
    pub fn from_saved_genesis(saved_genesis_blob: &[u8]) -> Self {
        let change_set = bcs::from_bytes::<ChangeSet>(saved_genesis_blob).unwrap();
        Self::from_genesis(change_set.write_set())
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
            current_module_blobs(),
            None,
            VMPublishingOption::locked(LegacyStdlibScript::allowlist()),
        )
    }

    /// Creates an executor from the genesis file GENESIS_FILE_LOCATION with script/module
    /// publishing options given by `publishing_options`. These can only be either `Open` or
    /// `CustomScript`.
    pub fn from_genesis_with_options(publishing_options: VMPublishingOption) -> Self {
        if !publishing_options.is_open_script() {
            panic!("Allowlisted transactions are not supported as a publishing option")
        }

        Self::custom_genesis(current_module_blobs(), None, publishing_options)
    }

    /// Creates an executor in which no genesis state has been applied yet.
    pub fn no_genesis() -> Self {
        FakeExecutor {
            data_store: FakeDataStore::default(),
            block_time: 0,
            executed_output: None,
            trace_dir: None,
            rng: KeyGen::from_seed(RNG_SEED),
        }
    }

    pub fn set_golden_file(&mut self, test_name: &str) {
        // 'test_name' includes ':' in the names, lets re-write these to be '_'s so that these
        // files can persist on windows machines.
        let file_name = test_name.replace(':', "_");
        self.executed_output = Some(GoldenOutputs::new(&file_name));

        // NOTE: tracing is only available when
        //  - the e2e test outputs a golden file, and
        //  - the environment variable is properly set
        if let Some(env_trace_dir) = env::var_os(ENV_TRACE_DIR) {
            let diem_version = DiemVersion::fetch_config(&self.data_store).map_or(0, |v| v.major);

            let trace_dir = Path::new(&env_trace_dir).join(file_name);
            if trace_dir.exists() {
                fs::remove_dir_all(&trace_dir).expect("Failed to clean up the trace directory");
            }
            fs::create_dir_all(&trace_dir).expect("Failed to create the trace directory");
            let mut name_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(trace_dir.join(TRACE_FILE_NAME))
                .unwrap();
            write!(name_file, "{}::{}", test_name, diem_version).unwrap();
            for sub_dir in &[
                TRACE_DIR_META,
                TRACE_DIR_DATA,
                TRACE_DIR_INPUT,
                TRACE_DIR_OUTPUT,
            ] {
                fs::create_dir(trace_dir.join(sub_dir)).unwrap_or_else(|err| {
                    panic!("Failed to create <trace>/{} directory: {}", sub_dir, err)
                });
            }
            self.trace_dir = Some(trace_dir);
        }
    }

    /// Creates an executor with only the standard library Move modules published and not other
    /// initialization done.
    pub fn stdlib_only_genesis() -> Self {
        let mut genesis = Self::no_genesis();
        let blobs = current_module_blobs();
        let modules = current_modules();
        assert!(blobs.len() == modules.len());
        for (module, bytes) in modules.iter().zip(blobs) {
            let id = module.self_id();
            genesis.add_module(&id, bytes.to_vec());
        }
        genesis
    }

    /// Creates fresh genesis from the stdlib modules passed in.
    pub fn custom_genesis(
        genesis_modules: &[Vec<u8>],
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

    /// Create one instance of [`AccountData`] with XDX balance without saving it to data store.
    pub fn create_xdx_raw_account_data(&mut self, balance: u64, seq_num: u64) -> AccountData {
        AccountData::new_xdx_from_seed(&mut self.rng, balance, seq_num)
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
    pub fn add_module(&mut self, module_id: &ModuleId, module_blob: Vec<u8>) {
        self.data_store.add_module(module_id, module_blob)
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
        let mut trace_map = TraceSeqMapping::default();

        // dump serialized transaction details before execution, if tracing
        if let Some(trace_dir) = &self.trace_dir {
            let trace_data_dir = trace_dir.join(TRACE_DIR_DATA);
            trace_map.0 = Self::trace(trace_data_dir.as_path(), self.get_state_view());
            let trace_input_dir = trace_dir.join(TRACE_DIR_INPUT);
            for txn in &txn_block {
                let input_seq = Self::trace(trace_input_dir.as_path(), txn);
                trace_map.1.push(input_seq);
            }
        }

        let output = DiemVM::execute_block(txn_block, &self.data_store);
        if let Some(logger) = &self.executed_output {
            logger.log(format!("{:?}\n", output).as_str());
        }

        // dump serialized transaction output after execution, if tracing
        if let Some(trace_dir) = &self.trace_dir {
            match &output {
                Ok(results) => {
                    let trace_output_dir = trace_dir.join(TRACE_DIR_OUTPUT);
                    for res in results {
                        let output_seq = Self::trace(trace_output_dir.as_path(), res);
                        trace_map.2.push(output_seq);
                    }
                }
                Err(e) => {
                    let mut error_file = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(trace_dir.join(TRACE_FILE_ERROR))
                        .unwrap();
                    error_file.write_all(e.to_string().as_bytes()).unwrap();
                }
            }
            let trace_meta_dir = trace_dir.join(TRACE_DIR_META);
            Self::trace(trace_meta_dir.as_path(), &trace_map);
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

    fn trace<P: AsRef<Path>, T: Serialize>(dir: P, item: &T) -> usize {
        let dir = dir.as_ref();
        let seq = fs::read_dir(dir).expect("Unable to read trace dir").count();
        let bytes = bcs::to_bytes(item)
            .unwrap_or_else(|err| panic!("Failed to serialize the trace item: {:?}", err));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(seq.to_string()))
            .expect("Unable to create a trace file");
        file.write_all(&bytes)
            .expect("Failed to write to the trace file");
        seq
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
        args: Vec<Vec<u8>>,
    ) {
        let write_set = {
            let mut gas_status = GasStatus::new_unmetered();
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
                    &mut gas_status,
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
            let (changeset, events) = session.finish().expect("Failed to generate txn effects");
            let (writeset, _events) = convert_changeset_and_events(changeset, events)
                .expect("Failed to generate writeset");
            writeset
        };
        self.data_store.add_write_set(&write_set);
    }

    pub fn try_exec(
        &mut self,
        module_name: &str,
        function_name: &str,
        type_params: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Result<WriteSet, VMStatus> {
        let mut gas_status = GasStatus::new_unmetered();
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
                &mut gas_status,
                &log_context,
            )
            .map_err(|e| e.into_vm_status())?;
        let (changeset, events) = session.finish().expect("Failed to generate txn effects");
        let (writeset, _events) =
            convert_changeset_and_events(changeset, events).expect("Failed to generate writeset");
        Ok(writeset)
    }
}
