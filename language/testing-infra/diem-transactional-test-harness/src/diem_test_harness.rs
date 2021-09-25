// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    ValidCryptoMaterialStringExt,
};
use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath,
    account_config::{
        self, type_tag_for_currency_code, AccountResource, BalanceResource, XUS_NAME,
    },
    chain_id::ChainId,
    transaction::{
        Module as TransactionModule, RawTransaction, Script as TransactionScript,
        ScriptFunction as TransactionScriptFunction, SignedTransaction, Transaction,
        TransactionStatus,
    },
    vm_status::KeptVMStatus,
};
use diem_vm::DiemVM;
use language_e2e_tests::data_store::{FakeDataStore, GENESIS_CHANGE_SET_FRESH};
use move_binary_format::file_format::{CompiledModule, CompiledScript};
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasConstants},
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, ResourceKey, TypeTag},
    move_resource::MoveStructType,
    transaction_argument::{convert_txn_args, TransactionArgument},
};
use move_lang::{shared::verify_and_create_named_address_mapping, FullyCompiledProgram};
use move_transactional_test_runner::{
    framework::{run_test_impl, CompiledState, MoveTestAdapter},
    tasks::{EmptyCommand, InitCommand, SyntaxChoice, TaskInput},
    vm_test_harness::view_resource_in_move_storage,
};
use once_cell::sync::Lazy;
use std::{
    collections::{BTreeMap, BTreeMap},
    path::Path,
};
use structopt::StructOpt;

/// The Diem transaction test adapter.
///
/// This differs from the SimpleVMTestAdapter in a few ways to ensure that our tests mimics
/// production settings:
///   - It uses a Diem StateView as its storage backend
///   - It executes transactions through DiemVM, instead of MoveVM directly
struct DiemTestAdapter<'a> {
    compiled_state: CompiledState<'a>,
    storage: FakeDataStore,
    default_syntax: SyntaxChoice,
}

/// Parameters *required* to create a Diem transaction.
struct TransactionParameters {
    pub sequence_number: u64,
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub gas_currency_code: String,
    pub expiration_timestamp_secs: u64,
}

fn parse_ed25519_private_key(s: &str) -> Result<Ed25519PrivateKey> {
    Ok(Ed25519PrivateKey::from_encoded_string(s)?)
}

/// Diem-specific arguments for the publish command.
#[derive(StructOpt, Debug)]
struct DiemPublishArgs {
    #[structopt(short = "k", long = "private-key", parse(try_from_str = parse_ed25519_private_key))]
    privkey: Ed25519PrivateKey,
}

/// Diem-specifc arguments for the run command,
#[derive(StructOpt, Debug)]
struct DiemRunArgs {
    #[structopt(short = "k", long = "private-key", parse(try_from_str = parse_ed25519_private_key))]
    privkey: Ed25519PrivateKey,
}

impl<'a> DiemTestAdapter<'a> {
    /// Obtain a Rust representation of the account resource from storage, which is used to derive
    /// a few default transaction parameters.
    fn fetch_account_resource(&self, signer_addr: &AccountAddress) -> Result<AccountResource> {
        let account_access_path = AccessPath::resource_access_path(ResourceKey::new(
            *signer_addr,
            AccountResource::struct_tag(),
        ));
        let account_blob = self
            .storage
            .get(&account_access_path)
            .unwrap()
            .ok_or_else(|| {
                format_err!(
                "Failed to fetch account resource under address {}. Has the account been created?",
                signer_addr
            )
            })?;
        Ok(bcs::from_bytes(&account_blob).unwrap())
    }

    /// Obtain a Rust representation of the balance resource from storage, which is used to derive
    /// a few default transaction parameters.
    fn fetch_balance_resource(
        &self,
        signer_addr: &AccountAddress,
        balance_currency_code: Identifier,
    ) -> Result<BalanceResource> {
        let currency_code_tag = type_tag_for_currency_code(balance_currency_code);
        let balance_resource_tag = BalanceResource::struct_tag_for_currency(currency_code_tag);
        let balance_access_path =
            AccessPath::resource_access_path(ResourceKey::new(*signer_addr, balance_resource_tag));

        let balance_blob = self
            .storage
            .get(&balance_access_path)
            .unwrap()
            .ok_or_else(|| {
                format_err!(
                    "Failed to fetch balance resource under address {}.",
                    signer_addr
                )
            })?;

        Ok(bcs::from_bytes(&balance_blob).unwrap())
    }

    /// Derive the default transaction parameters from the account and balance resources fetched
    /// from storage. In the future, we are planning to allow the user to override these using
    /// command arguments.
    fn fetch_default_transaction_parameters(
        &self,
        signer_addr: &AccountAddress,
    ) -> Result<TransactionParameters> {
        let account_resource = self.fetch_account_resource(signer_addr)?;

        let sequence_number = account_resource.sequence_number();
        let gas_currency_code = XUS_NAME.to_string();
        let max_number_of_gas_units = GasConstants::default().maximum_number_of_gas_units;
        let gas_unit_price = 0;
        let max_gas_amount = if gas_unit_price == 0 {
            max_number_of_gas_units.get()
        } else {
            let account_balance = self.fetch_balance_resource(
                signer_addr,
                account_config::from_currency_code_string(&gas_currency_code).unwrap(),
            )?;
            std::cmp::min(
                max_number_of_gas_units.get(),
                account_balance.coin() / gas_unit_price,
            )
        };

        Ok(TransactionParameters {
            sequence_number,
            gas_currency_code,
            gas_unit_price,
            max_gas_amount,
            expiration_timestamp_secs: 40000,
        })
    }

    /// Perform a single Diem transaction.
    ///
    /// Should error if the transaction ends up being discarded, or having a status other than
    /// EXECUTED.
    fn run_transaction(&mut self, txn: SignedTransaction) -> Result<()> {
        let mut outputs = DiemVM::execute_block_and_keep_vm_status(
            vec![Transaction::UserTransaction(txn)],
            &self.storage,
        )
        .unwrap();

        assert_eq!(outputs.len(), 1);

        let (status, output) = outputs.pop().unwrap();
        match output.status() {
            TransactionStatus::Keep(kept_vm_status) => match kept_vm_status {
                KeptVMStatus::Executed => {
                    self.storage.add_write_set(output.write_set());
                }
                _ => {
                    bail!("Failed to execute transaction. VMStatus: {}", status)
                }
            },
            TransactionStatus::Discard(_) => {
                bail!("Transaction discarded. VMStatus: {}", status)
            }
            TransactionStatus::Retry => panic!(),
        }

        Ok(())
    }
}

impl<'a> MoveTestAdapter<'a> for DiemTestAdapter<'a> {
    type ExtraInitArgs = EmptyCommand;
    type ExtraPublishArgs = DiemPublishArgs;
    type ExtraRunArgs = DiemRunArgs;
    type Subcommand = EmptyCommand;

    fn compiled_state(&mut self) -> &mut CompiledState<'a> {
        &mut self.compiled_state
    }

    fn default_syntax(&self) -> SyntaxChoice {
        self.default_syntax
    }

    fn init(
        default_syntax: SyntaxChoice,
        pre_compiled_deps: Option<&'a FullyCompiledProgram>,
        task_opt: Option<TaskInput<(InitCommand, Self::ExtraInitArgs)>>,
    ) -> Self {
        let additional_mapping = match task_opt.map(|t| t.command) {
            Some((InitCommand { named_addresses }, _)) => {
                verify_and_create_named_address_mapping(named_addresses).unwrap()
            }
            None => BTreeMap::new(),
        };

        let mut named_address_mapping = diem_framework::diem_framework_named_addresses();
        for (name, addr) in additional_mapping {
            if named_address_mapping.contains_key(&name) {
                panic!(
                    "Invalid init. The named address '{}' is reserved by either the move-stdlib or diem-framework",
                    name
                )
            }
            named_address_mapping.insert(name, addr);
        }

        // TODO: rework vm-genesis and try not to compile the genesis modules twice.
        let mut storage = FakeDataStore::new(BTreeMap::new());
        storage.add_write_set(GENESIS_CHANGE_SET_FRESH.write_set());

        Self {
            compiled_state: CompiledState::new(named_address_mapping, pre_compiled_deps),
            default_syntax,
            storage,
        }
    }

    fn publish_module(
        &mut self,
        module: CompiledModule,
        gas_budget: Option<u64>,
        extra_args: Self::ExtraPublishArgs,
    ) -> Result<()> {
        let module_id = module.self_id();
        let signer = module_id.address();
        let params = self.fetch_default_transaction_parameters(signer)?;

        let mut module_blob = vec![];
        module.serialize(&mut module_blob).unwrap();

        let txn = RawTransaction::new_module(
            *signer,
            params.sequence_number,
            TransactionModule::new(module_blob),
            gas_budget.unwrap_or(params.max_gas_amount),
            params.gas_unit_price,
            params.gas_currency_code,
            params.expiration_timestamp_secs,
            ChainId::test(),
        )
        .sign(
            &extra_args.privkey,
            Ed25519PublicKey::from(&extra_args.privkey),
        )?
        .into_inner();

        self.run_transaction(txn)?;

        Ok(())
    }

    fn execute_script(
        &mut self,
        script: CompiledScript,
        type_args: Vec<TypeTag>,
        signers: Vec<AccountAddress>,
        txn_args: Vec<TransactionArgument>,
        gas_budget: Option<u64>,
        extra_args: Self::ExtraRunArgs,
    ) -> Result<()> {
        assert!(!signers.is_empty());

        let mut script_blob = vec![];
        script.serialize(&mut script_blob)?;

        let params = self.fetch_default_transaction_parameters(&signers[0])?;

        let txn = RawTransaction::new_script(
            signers[0],
            params.sequence_number,
            TransactionScript::new(script_blob, type_args, txn_args),
            gas_budget.unwrap_or(params.max_gas_amount),
            params.gas_unit_price,
            params.gas_currency_code,
            params.expiration_timestamp_secs,
            ChainId::test(),
        )
        .sign_multi_agent(&extra_args.privkey, vec![], vec![])
        .unwrap()
        .into_inner();

        self.run_transaction(txn)?;

        Ok(())
    }

    fn call_function(
        &mut self,
        module: &ModuleId,
        function: &IdentStr,
        type_args: Vec<TypeTag>,
        signers: Vec<AccountAddress>,
        txn_args: Vec<TransactionArgument>,
        gas_budget: Option<u64>,
        extra_args: Self::ExtraRunArgs,
    ) -> Result<()> {
        assert!(!signers.is_empty());

        let params = self.fetch_default_transaction_parameters(&signers[0])?;
        let txn = RawTransaction::new_script_function(
            signers[0],
            params.sequence_number,
            TransactionScriptFunction::new(
                module.clone(),
                function.to_owned(),
                type_args,
                convert_txn_args(&txn_args),
            ),
            gas_budget.unwrap_or(params.max_gas_amount),
            params.gas_unit_price,
            params.gas_currency_code,
            params.expiration_timestamp_secs,
            ChainId::test(),
        )
        .sign(
            &extra_args.privkey,
            Ed25519PublicKey::from(&extra_args.privkey),
        )?
        .into_inner();

        self.run_transaction(txn)?;

        Ok(())
    }

    fn view_data(
        &mut self,
        address: AccountAddress,
        module: &ModuleId,
        resource: &IdentStr,
        type_args: Vec<TypeTag>,
    ) -> Result<String> {
        view_resource_in_move_storage(&self.storage, address, module, resource, type_args)
    }

    fn handle_subcommand(
        &mut self,
        _subcommand: TaskInput<Self::Subcommand>,
    ) -> Result<Option<String>> {
        unreachable!()
    }
}

static PRECOMPILED_DIEM_FRAMEWORK: Lazy<FullyCompiledProgram> = Lazy::new(|| {
    let program_res = move_lang::construct_pre_compiled_lib(
        &diem_framework::diem_stdlib_files(),
        None,
        move_lang::Flags::empty().set_sources_shadow_deps(false),
        diem_framework::diem_framework_named_addresses(),
    )
    .unwrap();
    match program_res {
        Ok(df) => df,
        Err((files, errors)) => {
            eprintln!("!!!Diem Framework failed to compile!!!");
            move_lang::diagnostics::report_diagnostics(&files, errors)
        }
    }
});

/// Run the Diem transactional test flow, using the given file as input.
pub fn run_test(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_test_impl::<DiemTestAdapter>(path, Some(&*PRECOMPILED_DIEM_FRAMEWORK))
}
