// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod genesis_gas_schedule;

use crate::genesis_gas_schedule::INITIAL_GAS_SCHEDULE;
use anyhow::Result;
use bytecode_verifier::VerifiedModule;
use libra_config::config::NodeConfig;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    PrivateKey, Uniform, ValidCryptoMaterial,
};
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    contract_event::ContractEvent,
    language_storage::{ModuleId, StructTag, TypeTag},
    on_chain_config::{new_epoch_event_key, VMPublishingOption},
    transaction::{
        authenticator::AuthenticationKey, ChangeSet, Script, Transaction, TransactionArgument,
    },
};
use libra_vm::{self, system_module_names::*};
use move_core_types::{
    gas_schedule::{CostTable, GasAlgebra, GasUnits},
    identifier::Identifier,
};
use move_vm_runtime::MoveVM;
use move_vm_state::{
    data_cache::BlockDataCache,
    execution_context::{ExecutionContext, TransactionExecutionContext},
};
use move_vm_types::{chain_state::ChainState, values::Value};
use once_cell::sync::Lazy;
use rand::prelude::*;
use std::collections::HashMap;
use stdlib::{stdlib_modules, transaction_scripts::StdlibScript, StdLibOptions};
use vm::{
    access::ModuleAccess, gas_schedule::zero_cost_schedule,
    transaction_metadata::TransactionMetadata,
};

// The seed is arbitrarily picked to produce a consistent key. XXX make this more formal?
const GENESIS_SEED: [u8; 32] = [42; 32];

/// The initial balance of the association account.
pub const ASSOCIATION_INIT_BALANCE: u64 = 1_000_000_000_000_000;

pub static GENESIS_KEYPAIR: Lazy<(Ed25519PrivateKey, Ed25519PublicKey)> = Lazy::new(|| {
    let mut rng = StdRng::from_seed(GENESIS_SEED);
    let private_key = Ed25519PrivateKey::generate(&mut rng);
    let public_key = private_key.public_key();
    (private_key, public_key)
});

pub type ValidatorRegistration = (Ed25519PublicKey, Script);

// Identifiers for well-known functions.
static ADD_VALIDATOR: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("add_validator_no_discovery_event").unwrap());
static INITIALIZE: Lazy<Identifier> = Lazy::new(|| Identifier::new("initialize").unwrap());
static INITIALIZE_BLOCK: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_block_metadata").unwrap());
static INITIALIZE_CONFIG: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_configuration").unwrap());
static INITIALIZE_VALIDATOR_SET: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_validator_set").unwrap());
static INITIALIZE_DISCOVERY_SET: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_discovery_set").unwrap());
static MINT_TO_ADDRESS: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("mint_to_address").unwrap());
static RECONFIGURE: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("emit_reconfiguration_event").unwrap());
static EMIT_DISCOVERY_SET: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("emit_discovery_set_change").unwrap());
static ROTATE_AUTHENTICATION_KEY: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("rotate_authentication_key").unwrap());
static EPILOGUE: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());

static VM_CONFIG_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new("LibraVMConfig").unwrap(),
    )
});

static LIBRA_VERSION_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new("LibraVersion").unwrap(),
    )
});

static LIBRA_TIME_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new("LibraTimestamp").unwrap(),
    )
});

pub fn module(name: &str) -> ModuleId {
    ModuleId::new(
        account_config::CORE_CODE_ADDRESS,
        Identifier::new(name).unwrap(),
    )
}

pub fn name(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

pub fn encode_genesis_transaction_with_validator(
    public_key: Ed25519PublicKey,
    validators: &[ValidatorRegistration],
    vm_publishing_option: Option<VMPublishingOption>,
) -> Transaction {
    encode_genesis_transaction(
        public_key,
        validators,
        stdlib_modules(StdLibOptions::Staged), // Must use staged stdlib
        vm_publishing_option
            .unwrap_or_else(|| VMPublishingOption::Locked(StdlibScript::whitelist())),
    )
}

pub fn encode_genesis_change_set(
    public_key: &Ed25519PublicKey,
    validators: &[ValidatorRegistration],
    stdlib_modules: &[VerifiedModule],
    vm_publishing_option: VMPublishingOption,
) -> ChangeSet {
    let move_vm = MoveVM::new();

    // create a data view for move_vm
    let mut state_view = GenesisStateView::new();
    let gas_schedule = zero_cost_schedule();
    for module in stdlib_modules {
        let module_id = module.self_id();
        state_view.add_module(&module_id, &module);
    }
    let data_cache = BlockDataCache::new(&state_view);

    // create an execution context for the move_vm.
    // It will contain the genesis WriteSet after execution
    let mut interpreter_context =
        TransactionExecutionContext::new(GasUnits::new(100_000_000), &data_cache);

    // initialize the VM with stdlib modules.
    // This step is needed because we are creating the main accounts and we are calling
    // code to create those. However, code lives under an account but we have none.
    // So we are pushing code into the VM blindly in order to create the main accounts.
    for module in stdlib_modules {
        move_vm
            .cache_module(module.clone(), &mut interpreter_context)
            .expect("Failure loading stdlib");
    }

    let lbr_ty = TypeTag::Struct(StructTag {
        address: *account_config::LBR_MODULE.address(),
        module: account_config::LBR_MODULE.name().to_owned(),
        name: account_config::LBR_STRUCT_NAME.to_owned(),
        type_params: vec![],
    });

    // generate the genesis WriteSet
    create_and_initialize_main_accounts(
        &move_vm,
        &gas_schedule,
        &mut interpreter_context,
        &public_key,
        &lbr_ty,
    );
    create_and_initialize_validator_and_discovery_set(
        &move_vm,
        &gas_schedule,
        &mut interpreter_context,
        &validators,
        &lbr_ty,
    );
    setup_libra_version(&move_vm, &gas_schedule, &mut interpreter_context);
    setup_vm_config(
        &move_vm,
        &gas_schedule,
        &mut interpreter_context,
        vm_publishing_option,
    );
    reconfigure(&move_vm, &gas_schedule, &mut interpreter_context);
    publish_stdlib(&mut interpreter_context, stdlib_modules);

    verify_genesis_write_set(interpreter_context.events());
    ChangeSet::new(
        interpreter_context
            .make_write_set()
            .expect("Genesis WriteSet failure"),
        interpreter_context.events().to_vec(),
    )
}

pub fn encode_genesis_transaction(
    public_key: Ed25519PublicKey,
    validators: &[ValidatorRegistration],
    stdlib_modules: &[VerifiedModule],
    vm_publishing_option: VMPublishingOption,
) -> Transaction {
    Transaction::WaypointWriteSet(encode_genesis_change_set(
        &public_key,
        validators,
        stdlib_modules,
        vm_publishing_option,
    ))
}

/// Create an initialize Association, Transaction Fee and Core Code accounts.
fn create_and_initialize_main_accounts(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    public_key: &Ed25519PublicKey,
    lbr_ty: &TypeTag,
) {
    let association_addr = account_config::association_address();
    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = association_addr;

    let burn_account_address = account_config::burn_account_address();
    let mut burn_txn_data = TransactionMetadata::default();
    burn_txn_data.sender = burn_account_address;

    {
        // Association module setup
        move_vm
            .execute_function(
                &module("Association"),
                &name("initialize"),
                gas_schedule,
                interpreter_context,
                &txn_data,
                vec![],
                vec![],
            )
            .expect("Unable to register root association account in genesis");

        let add_currency_priv_ty = TypeTag::Struct(StructTag {
            address: account_config::CORE_CODE_ADDRESS,
            module: name("Libra"),
            name: name("AddCurrency"),
            type_params: vec![],
        });

        move_vm
            .execute_function(
                &module("Association"),
                &name("apply_for_privilege"),
                gas_schedule,
                interpreter_context,
                &txn_data,
                vec![add_currency_priv_ty.clone()],
                vec![],
            )
            .expect("Unable to apply for currency privilege in genesis");

        move_vm
            .execute_function(
                &module("Association"),
                &name("grant_privilege"),
                gas_schedule,
                interpreter_context,
                &txn_data,
                vec![add_currency_priv_ty],
                vec![Value::address(association_addr)],
            )
            .expect("Unable to grant currency privilege in genesis");
    }

    {
        // Initialize burn account

        move_vm
            .execute_function(
                &module("Association"),
                &name("apply_for_association"),
                gas_schedule,
                interpreter_context,
                &burn_txn_data,
                vec![],
                vec![],
            )
            .expect("Unable to create burn account - apply_for_association failed in genesis");

        move_vm
            .execute_function(
                &module("Association"),
                &name("grant_association_address"),
                gas_schedule,
                interpreter_context,
                &txn_data,
                vec![],
                vec![Value::address(burn_account_address)],
            )
            .expect("Unable to create burn account - grant_association_address failed in genesis");
    }

    {
        // Initialize currencies
        move_vm
            .execute_function(
                &module("Event"),
                &name("initialize"),
                &gas_schedule,
                interpreter_context,
                &txn_data,
                vec![],
                vec![],
            )
            .expect("Failure initializing Event module");
        move_vm
            .execute_function(
                &module("Libra"),
                &name("initialize"),
                &gas_schedule,
                interpreter_context,
                &txn_data,
                vec![],
                vec![],
            )
            .expect("Failure initializing Libra module");

        // NB: Order matters
        let currencies = vec!["Coin1", "Coin2", "LBR"];
        for currency in currencies.into_iter() {
            move_vm
                .execute_function(
                    &module(currency),
                    &name("initialize"),
                    &gas_schedule,
                    interpreter_context,
                    &txn_data,
                    vec![],
                    vec![],
                )
                .unwrap_or_else(|e| panic!("Failure initializing currency {}: {}", currency, e));

            let currency_type = TypeTag::Struct(StructTag {
                address: account_config::CORE_CODE_ADDRESS,
                module: name(currency),
                name: name("T"),
                type_params: vec![],
            });

            move_vm
                .execute_function(
                    &module("Libra"),
                    &name("grant_burn_capability_for_sender"),
                    gas_schedule,
                    interpreter_context,
                    &burn_txn_data,
                    vec![currency_type],
                    vec![],
                )
                .expect(
                    "Unable to create burn account - grant_burn_capability_for_sender failed in genesis",
                );
        }
    }

    {
        // Account type initialization
        let unhosted_type = TypeTag::Struct(StructTag {
            address: account_config::CORE_CODE_ADDRESS,
            module: name("Unhosted"),
            name: name("T"),
            type_params: vec![],
        });
        let empty_type = TypeTag::Struct(StructTag {
            address: account_config::CORE_CODE_ADDRESS,
            module: name("Empty"),
            name: name("T"),
            type_params: vec![],
        });

        let account_types = vec![unhosted_type, empty_type];
        for account_type in account_types.into_iter() {
            move_vm
                .execute_function(
                    &module("AccountType"),
                    &name("register"),
                    &gas_schedule,
                    interpreter_context,
                    &txn_data,
                    vec![account_type.clone()],
                    vec![],
                )
                .unwrap_or_else(|_| {
                    panic!("Failure initializing account type {:#?}", account_type)
                });
        }
        move_vm
            .execute_function(
                &module("VASP"),
                &name("initialize"),
                &gas_schedule,
                interpreter_context,
                &txn_data,
                vec![],
                vec![],
            )
            .expect("Failure initializing VASP module");
    }

    {
        // Account module setup
        move_vm
            .execute_function(
                &module("AccountTrack"),
                &name("initialize"),
                &gas_schedule,
                interpreter_context,
                &txn_data,
                vec![],
                vec![],
            )
            .expect("Failure initializing account tracking module");

        move_vm
            .execute_function(
                &module("LibraAccount"),
                &name("initialize"),
                &gas_schedule,
                interpreter_context,
                &txn_data,
                vec![],
                vec![],
            )
            .expect("Failure initializing Libra account module");

        move_vm
            .execute_function(
                &module("Unhosted"),
                &name("publish_global_limits_definition"),
                &gas_schedule,
                interpreter_context,
                &txn_data,
                vec![],
                vec![],
            )
            .expect("Failure publishing limits for unhosted accounts");
    }

    // create the association account
    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &CREATE_ACCOUNT_NAME,
            gas_schedule,
            interpreter_context,
            &txn_data,
            vec![lbr_ty.clone()],
            vec![
                Value::address(association_addr),
                Value::vector_u8(association_addr.to_vec()),
            ],
        )
        .unwrap_or_else(|e| {
            panic!(
                "Failure creating association account {:?}: {}",
                association_addr, e
            )
        });

    // create burn account
    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &CREATE_ACCOUNT_NAME,
            gas_schedule,
            interpreter_context,
            &txn_data,
            vec![lbr_ty.clone()],
            vec![
                Value::address(burn_account_address),
                Value::vector_u8(burn_account_address.to_vec()),
            ],
        )
        .unwrap_or_else(|e| {
            panic!(
                "Failure creating burn account {:?}: {}",
                burn_account_address, e
            )
        });

    {
        // transaction fee addresses setup
        move_vm
            .execute_function(
                &module("TransactionFeeAccounts"),
                &name("initialize"),
                gas_schedule,
                interpreter_context,
                &txn_data,
                vec![],
                vec![],
            )
            .unwrap();

        move_vm
            .execute_function(
                &module("Association"),
                &name("grant_privilege"),
                gas_schedule,
                interpreter_context,
                &txn_data,
                vec![TypeTag::Struct(StructTag {
                    address: account_config::CORE_CODE_ADDRESS,
                    module: name("TransactionFeeAccounts"),
                    name: name("TransactionFeeAccountRegistration"),
                    type_params: vec![],
                })],
                vec![Value::address(association_addr)],
            )
            .expect("Unable to grant transaction fee account registration privilege");
    }

    {
        // Transaction fee setup
        let prev_sender = txn_data.sender;
        // create the transaction fee accounts and addresses
        let accounts = vec![("Coin1", "0xFEE0"), ("Coin2", "0xFEE1"), ("LBR", "0xFEE2")]
            .into_iter()
            .map(|(code, addr)| {
                (
                    account_config::type_tag_for_currency_code(
                        account_config::from_currency_code_string(code).unwrap(),
                    ),
                    AccountAddress::from_hex_literal(addr).unwrap(),
                )
            });
        for (type_tag, addr) in accounts {
            // create the account
            move_vm
                .execute_function(
                    &account_config::ACCOUNT_MODULE,
                    &CREATE_ACCOUNT_NAME,
                    gas_schedule,
                    interpreter_context,
                    &txn_data,
                    vec![type_tag.clone()],
                    vec![Value::address(addr), Value::vector_u8(addr.to_vec())],
                )
                .unwrap_or_else(|e| {
                    panic!("Failure creating transaction fee account {:?}: {}", addr, e)
                });

            // Now register that address as a transaction fee address
            move_vm
                .execute_function(
                    &module("TransactionFeeAccounts"),
                    &name("add_transaction_fee_account"),
                    &gas_schedule,
                    interpreter_context,
                    &txn_data,
                    vec![type_tag],
                    vec![Value::address(addr)],
                )
                .expect("Failure initializing transaction fee");

            txn_data.sender = addr;
            // Initialize the account as a transaction fee account
            move_vm
                .execute_function(
                    &TRANSACTION_FEE_MODULE,
                    &name("initialize_transaction_fees"),
                    &gas_schedule,
                    interpreter_context,
                    &txn_data,
                    vec![],
                    vec![],
                )
                .expect("Failure initializing transaction fee");
            txn_data.sender = prev_sender;
        }
    }

    move_vm
        .execute_function(
            &LIBRA_TRANSACTION_TIMEOUT,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure initializing LibraTransactionTimeout");

    move_vm
        .execute_function(
            &LIBRA_BLOCK_MODULE,
            &INITIALIZE_BLOCK,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure initializing block metadata");

    move_vm
        .execute_function(
            &LIBRA_WRITESET_MANAGER_MODULE,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure initializing LibraWriteSetManager");

    move_vm
        .execute_function(
            &LIBRA_CONFIG_MODULE,
            &INITIALIZE_CONFIG,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure initializing block metadata");

    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &MINT_TO_ADDRESS,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![lbr_ty.clone()],
            vec![
                Value::address(association_addr),
                Value::u64(ASSOCIATION_INIT_BALANCE),
            ],
        )
        .expect("Failure minting to association");

    let genesis_auth_key = AuthenticationKey::ed25519(public_key).to_vec();
    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &ROTATE_AUTHENTICATION_KEY,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![Value::vector_u8(genesis_auth_key)],
        )
        .expect("Failure rotating association key");

    let genesis_auth_key = AuthenticationKey::ed25519(public_key).to_vec();
    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &ROTATE_AUTHENTICATION_KEY,
            &gas_schedule,
            interpreter_context,
            &burn_txn_data,
            vec![],
            vec![Value::vector_u8(genesis_auth_key)],
        )
        .expect("Failure rotating burn account key");

    // Bump the sequence number for the Association account. If we don't do this and a
    // subsequent transaction (e.g., minting) is sent from the Assocation account, a problem
    // arises: both the genesis transaction and the subsequent transaction have sequence
    // number 0
    let lbr_ty = TypeTag::Struct(StructTag {
        address: *account_config::LBR_MODULE.address(),
        module: account_config::LBR_MODULE.name().to_owned(),
        name: account_config::LBR_STRUCT_NAME.to_owned(),
        type_params: vec![],
    });
    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &EPILOGUE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![lbr_ty],
            vec![
                Value::u64(/* txn_sequence_number */ 0),
                Value::u64(/* txn_gas_price */ 0),
                Value::u64(/* txn_max_gas_units */ 0),
                Value::u64(/* gas_units_remaining */ 0),
            ],
        )
        .expect("Failure running epilogue for association account");
}

/// Create and initialize validator and discovery set.
fn create_and_initialize_validator_and_discovery_set(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    validators: &[ValidatorRegistration],
    lbr_ty: &TypeTag,
) {
    create_and_initialize_validator_set(move_vm, gas_schedule, interpreter_context, lbr_ty);
    create_and_initialize_discovery_set(move_vm, gas_schedule, interpreter_context, lbr_ty);
    initialize_validators(
        move_vm,
        gas_schedule,
        interpreter_context,
        &validators,
        lbr_ty,
    );
}

/// Create and initialize the validator set.
fn create_and_initialize_validator_set(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    lbr_ty: &TypeTag,
) {
    let mut txn_data = TransactionMetadata::default();
    let validator_set_address = account_config::validator_set_address();
    txn_data.sender = validator_set_address;

    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &CREATE_ACCOUNT_NAME,
            gas_schedule,
            interpreter_context,
            &txn_data,
            vec![lbr_ty.clone()],
            vec![
                Value::address(validator_set_address),
                Value::vector_u8(validator_set_address.to_vec()),
            ],
        )
        .unwrap_or_else(|e| {
            panic!(
                "Failure creating validator set account {:?}: {}",
                validator_set_address, e
            )
        });

    move_vm
        .execute_function(
            &LIBRA_SYSTEM_MODULE,
            &INITIALIZE_VALIDATOR_SET,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure initializing validator set");
}

/// Create and initialize the discovery set.
fn create_and_initialize_discovery_set(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    lbr_ty: &TypeTag,
) {
    let mut txn_data = TransactionMetadata::default();
    let discovery_set_address = account_config::discovery_set_address();
    txn_data.sender = discovery_set_address;

    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &CREATE_ACCOUNT_NAME,
            gas_schedule,
            interpreter_context,
            &txn_data,
            vec![lbr_ty.clone()],
            vec![
                Value::address(discovery_set_address),
                Value::vector_u8(discovery_set_address.to_vec()),
            ],
        )
        .unwrap_or_else(|e| {
            panic!(
                "Failure creating discovery set account {:?}: {}",
                discovery_set_address, e
            )
        });

    move_vm
        .execute_function(
            &LIBRA_SYSTEM_MODULE,
            &INITIALIZE_DISCOVERY_SET,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure initializing discovery set");
}

/// Initialize each validator.
fn initialize_validators(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    validators: &[ValidatorRegistration],
    lbr_ty: &TypeTag,
) {
    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = account_config::association_address();

    for (account_key, registration) in validators {
        let auth_key = AuthenticationKey::ed25519(&account_key);
        let account = auth_key.derived_address();

        // Create an account
        move_vm
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                &CREATE_ACCOUNT_NAME,
                gas_schedule,
                interpreter_context,
                &txn_data,
                vec![lbr_ty.clone()],
                vec![
                    Value::address(account),
                    Value::vector_u8(auth_key.prefix().to_vec()),
                ],
            )
            .unwrap_or_else(|e| panic!("Failure creating validator account {:?}: {}", account, e));

        // Add the validator information
        let mut validator_txn_data = TransactionMetadata::default();
        validator_txn_data.sender = account;
        move_vm
            .execute_script(
                registration.code().to_vec(),
                &gas_schedule,
                interpreter_context,
                &validator_txn_data,
                registration.ty_args().to_vec(),
                convert_txn_args(registration.args()),
            )
            .unwrap_or_else(|_| panic!("Failure initializing validator {:?}", account));

        // Finally, add the account to the validator set
        move_vm
            .execute_function(
                &LIBRA_SYSTEM_MODULE,
                &ADD_VALIDATOR,
                &gas_schedule,
                interpreter_context,
                &txn_data,
                vec![],
                vec![Value::address(account)],
            )
            .unwrap_or_else(|_| panic!("Failure adding validator {:?}", account));
    }
}

fn setup_vm_config(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    publishing_option: VMPublishingOption,
) {
    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = account_config::association_address();

    let option_bytes =
        lcs::to_bytes(&publishing_option).expect("Cannot serialize publishing option");
    move_vm
        .execute_function(
            &VM_CONFIG_MODULE,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![
                Value::vector_u8(option_bytes),
                Value::vector_u8(INITIAL_GAS_SCHEDULE.0.clone()),
                Value::vector_u8(INITIAL_GAS_SCHEDULE.1.clone()),
            ],
        )
        .expect("Failure setting up publishing option");
}

fn setup_libra_version(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
) {
    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = account_config::association_address();

    move_vm
        .execute_function(
            &LIBRA_VERSION_MODULE,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure setting up libra version number");
}

/// Publish the standard library.
fn publish_stdlib(interpreter_context: &mut dyn ChainState, stdlib: &[VerifiedModule]) {
    for module in stdlib {
        let mut module_vec = vec![];
        module.serialize(&mut module_vec).unwrap();
        interpreter_context
            .publish_module(module.self_id(), module_vec)
            .unwrap_or_else(|_| panic!("Failure publishing module {:?}", module.self_id()));
    }
}

/// Trigger a reconfiguration. This emits an event that will be passed along to the storage layer.
fn reconfigure(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
) {
    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = account_config::association_address();

    move_vm
        .execute_function(
            &LIBRA_TIME_MODULE,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure reconfiguring the system");

    move_vm
        .execute_function(
            &LIBRA_CONFIG_MODULE,
            &RECONFIGURE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure reconfiguring the system");

    move_vm
        .execute_function(
            &LIBRA_SYSTEM_MODULE,
            &EMIT_DISCOVERY_SET,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure emitting discovery set change");
}

/// Verify the consistency of the genesis `WriteSet`
fn verify_genesis_write_set(events: &[ContractEvent]) {
    // Sanity checks on emitted events:
    // (1) The genesis tx should emit 4 events: a pair of payment sent/received events for
    // minting to the genesis address, a ValidatorSetChangeEvent, and a
    // DiscoverySetChangeEvent.
    assert_eq!(
        events.len(),
        4,
        "Genesis transaction should emit four events, but found {} events: {:?}",
        events.len(),
        events,
    );

    // (2) The third event should be the new epoch event
    let new_epoch_event = &events[2];
    assert_eq!(
        *new_epoch_event.key(),
        new_epoch_event_key(),
        "Key of emitted event {:?} does not match change event key {:?}",
        *new_epoch_event.key(),
        new_epoch_event_key(),
    );
    // (3) This should be the first new_epoch_event
    assert_eq!(
        new_epoch_event.sequence_number(),
        0,
        "Expected sequence number 0 for validator set change event but got {}",
        new_epoch_event.sequence_number()
    );
}

/// Generate an artificial genesis `ChangeSet` for testing
pub fn generate_genesis_change_set_for_testing(stdlib_options: StdLibOptions) -> ChangeSet {
    let stdlib_modules = stdlib_modules(stdlib_options);
    let swarm = libra_config::generator::validator_swarm_for_testing(10);

    encode_genesis_change_set(
        &GENESIS_KEYPAIR.1,
        &validator_registrations(&swarm.nodes),
        stdlib_modules,
        VMPublishingOption::Open,
    )
}

// `StateView` has no data given we are creating the genesis
struct GenesisStateView {
    data: HashMap<AccessPath, Vec<u8>>,
}

impl GenesisStateView {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn add_module(&mut self, module_id: &ModuleId, module: &VerifiedModule) {
        let access_path = AccessPath::from(module_id);
        let mut blob = vec![];
        module
            .serialize(&mut blob)
            .expect("serializing stdlib must work");
        self.data.insert(access_path, blob);
    }
}

impl StateView for GenesisStateView {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Ok(self.data.get(access_path).cloned())
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        true
    }
}

pub fn validator_registrations(node_configs: &[NodeConfig]) -> Vec<ValidatorRegistration> {
    node_configs
        .iter()
        .map(|n| {
            let test = n.test.as_ref().unwrap();
            let account_key = test.account_keypair.as_ref().unwrap().public_key();
            let consensus_key = test.consensus_keypair.as_ref().unwrap().public_key();
            let network = n.validator_network.as_ref().unwrap();
            let network_keypairs = network.network_keypairs.as_ref().unwrap();
            let signing_key = network_keypairs.signing_keypair.public_key();

            let script = transaction_builder::encode_register_validator_script(
                consensus_key.to_bytes().to_vec(),
                signing_key.to_bytes().to_vec(),
                network_keypairs.identity_keypair.public_key().to_bytes(),
                network.advertised_address.to_vec(),
                network_keypairs.identity_keypair.public_key().to_bytes(),
                network.advertised_address.to_vec(),
            );
            (account_key, script)
        })
        .collect::<Vec<_>>()
}

/// Convert the transaction arguments into move values.
fn convert_txn_args(args: &[TransactionArgument]) -> Vec<Value> {
    args.iter()
        .map(|arg| match arg {
            TransactionArgument::U64(i) => Value::u64(*i),
            TransactionArgument::Address(a) => Value::address(*a),
            TransactionArgument::Bool(b) => Value::bool(*b),
            TransactionArgument::U8Vector(v) => Value::vector_u8(v.clone()),
        })
        .collect()
}
