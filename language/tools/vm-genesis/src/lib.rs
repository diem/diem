// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod genesis_gas_schedule;

use crate::genesis_gas_schedule::INITIAL_GAS_SCHEDULE;
use anyhow::Result;
use bytecode_verifier::VerifiedModule;
use libra_config::{config::NodeConfig, generator};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    traits::ValidKey,
    PrivateKey, Uniform,
};
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    contract_event::ContractEvent,
    discovery_set::DiscoverySet,
    language_storage::{ModuleId, StructTag, TypeTag},
    on_chain_config::{new_epoch_event_key, VMPublishingOption, ValidatorSet},
    transaction::{authenticator::AuthenticationKey, ChangeSet, Transaction},
};
use libra_vm::system_module_names::*;
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

// Identifiers for well-known functions.
static ADD_VALIDATOR: Lazy<Identifier> = Lazy::new(|| Identifier::new("add_validator_").unwrap());
static INITIALIZE: Lazy<Identifier> = Lazy::new(|| Identifier::new("initialize").unwrap());
static INITIALIZE_BLOCK: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_block_metadata").unwrap());
static INITIALIZE_CONFIG: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_configuration").unwrap());
static INITIALIZE_TXN_FEES: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_transaction_fees").unwrap());
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
static REGISTER_CANDIDATE_VALIDATOR: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("register_candidate_validator").unwrap());
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
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    nodes: &[NodeConfig],
    validator_set: ValidatorSet,
    discovery_set: DiscoverySet,
    vm_publishing_option: Option<VMPublishingOption>,
) -> Transaction {
    encode_genesis_transaction(
        private_key,
        public_key,
        nodes,
        validator_set,
        discovery_set,
        stdlib_modules(StdLibOptions::Staged), // Must use staged stdlib
        vm_publishing_option
            .unwrap_or_else(|| VMPublishingOption::Locked(StdlibScript::whitelist())),
    )
}

pub fn encode_genesis_change_set(
    public_key: &Ed25519PublicKey,
    nodes: &[NodeConfig],
    validator_set: ValidatorSet,
    discovery_set: DiscoverySet,
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
        &nodes,
        &validator_set,
        &discovery_set,
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

    verify_genesis_write_set(interpreter_context.events(), &discovery_set);
    ChangeSet::new(
        interpreter_context
            .make_write_set()
            .expect("Genesis WriteSet failure"),
        interpreter_context.events().to_vec(),
    )
}

pub fn encode_genesis_transaction(
    _private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    nodes: &[NodeConfig],
    validator_set: ValidatorSet,
    discovery_set: DiscoverySet,
    stdlib_modules: &[VerifiedModule],
    vm_publishing_option: VMPublishingOption,
) -> Transaction {
    Transaction::WaypointWriteSet(encode_genesis_change_set(
        &public_key,
        nodes,
        validator_set,
        discovery_set,
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
        // Initialize currencies
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
            .expect("Failure initializing currency module");

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
                .unwrap_or_else(|_| panic!("Failure initializing currency {}", currency));
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
            .expect("Failure initializing currency module");
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

    // create the transaction fee account
    let transaction_fee_address = account_config::transaction_fee_address();
    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &CREATE_ACCOUNT_NAME,
            gas_schedule,
            interpreter_context,
            &txn_data,
            vec![lbr_ty.clone()],
            vec![
                Value::address(transaction_fee_address),
                Value::vector_u8(transaction_fee_address.to_vec()),
            ],
        )
        .unwrap_or_else(|e| {
            panic!(
                "Failure creating transaction fee account {:?}: {}",
                transaction_fee_address, e
            )
        });

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

    txn_data.sender = account_config::transaction_fee_address();
    move_vm
        .execute_function(
            &TRANSACTION_FEE_MODULE,
            &INITIALIZE_TXN_FEES,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure initializing transaction fee");
}

/// Create and initialize validator and discovery set.
fn create_and_initialize_validator_and_discovery_set(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    nodes: &[NodeConfig],
    validator_set: &ValidatorSet,
    discovery_set: &DiscoverySet,
    lbr_ty: &TypeTag,
) {
    create_and_initialize_validator_set(move_vm, gas_schedule, interpreter_context, lbr_ty);
    create_and_initialize_discovery_set(move_vm, gas_schedule, interpreter_context, lbr_ty);
    initialize_validators(
        move_vm,
        gas_schedule,
        interpreter_context,
        nodes,
        validator_set,
        discovery_set,
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

// TODO: refactor ValidatorSet / NodeConfig to make it possible to  do this resolution without a
// linear search through nodes
/// Return an authentication key in `nodes` whose derived address is equal to `address`
fn get_validator_authentication_key(
    nodes: &[NodeConfig],
    address: &AccountAddress,
) -> Option<AuthenticationKey> {
    for node in nodes {
        let public_key = node
            .test
            .as_ref()
            .unwrap()
            .account_keypair
            .as_ref()
            .unwrap()
            .public_key();
        let validator_authentication_key = AuthenticationKey::ed25519(&public_key);
        let derived_address = validator_authentication_key.derived_address();
        if derived_address == *address {
            return Some(validator_authentication_key);
        }
    }
    None
}

/// Initialize each validator.
fn initialize_validators(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    nodes: &[NodeConfig],
    validator_set: &ValidatorSet,
    discovery_set: &DiscoverySet,
    lbr_ty: &TypeTag,
) {
    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = account_config::association_address();

    for (validator_keys, discovery_info) in validator_set.payload().iter().zip(discovery_set.iter())
    {
        // First, add a ValidatorConfig resource under each account
        let validator_address = *validator_keys.account_address();
        let validator_authentication_key =
            get_validator_authentication_key(nodes, &validator_address).unwrap_or_else(|| {
                panic!(
                    "Couldn't find authentication key for validator address {}",
                    validator_address
                )
            });
        move_vm
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                &CREATE_ACCOUNT_NAME,
                gas_schedule,
                interpreter_context,
                &txn_data,
                vec![lbr_ty.clone()],
                vec![
                    Value::address(validator_address),
                    Value::vector_u8(validator_authentication_key.prefix().to_vec()),
                ],
            )
            .unwrap_or_else(|e| {
                panic!(
                    "Failure creating validator account {:?}: {}",
                    validator_address, e
                )
            });

        let mut validator_txn_data = TransactionMetadata::default();
        validator_txn_data.sender = validator_address;
        move_vm
            .execute_function(
                &VALIDATOR_CONFIG_MODULE,
                &REGISTER_CANDIDATE_VALIDATOR,
                &gas_schedule,
                interpreter_context,
                &validator_txn_data,
                vec![],
                vec![
                    // consensus_pubkey
                    Value::vector_u8(validator_keys.consensus_public_key().to_bytes().to_vec()),
                    // network_signing_pubkey
                    Value::vector_u8(
                        validator_keys
                            .network_signing_public_key()
                            .to_bytes()
                            .to_vec(),
                    ),
                    // validator_network_identity_pubkey
                    Value::vector_u8(discovery_info.validator_network_identity_pubkey.to_bytes()),
                    // validator_network_address
                    Value::vector_u8(discovery_info.validator_network_address.to_vec()),
                    // fullnodes_network_identity_pubkey
                    Value::vector_u8(discovery_info.fullnodes_network_identity_pubkey.to_bytes()),
                    // fullnodes_network_address
                    Value::vector_u8(discovery_info.fullnodes_network_address.to_vec()),
                ],
            )
            .unwrap_or_else(|_| panic!("Failure initializing validator {:?}", validator_address));
        // Then, add the account to the validator set
        move_vm
            .execute_function(
                &LIBRA_SYSTEM_MODULE,
                &ADD_VALIDATOR,
                &gas_schedule,
                interpreter_context,
                &txn_data,
                vec![],
                vec![Value::address(validator_address)],
            )
            .unwrap_or_else(|_| panic!("Failure adding validator {:?}", validator_address));
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
fn verify_genesis_write_set(events: &[ContractEvent], discovery_set: &DiscoverySet) {
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

    // (4) The fourth event should be the discovery set change event
    let discovery_set_change_event = &events[3];
    assert_eq!(
        *discovery_set_change_event.key(),
        DiscoverySet::change_event_key(),
        "Key of emitted event {:?} does not match change event key {:?}",
        *discovery_set_change_event.key(),
        DiscoverySet::change_event_key()
    );
    // (5) This should be the first discovery set change event
    assert_eq!(
        discovery_set_change_event.sequence_number(),
        0,
        "Expected sequence number 0 for discovery set change event but got {}",
        discovery_set_change_event.sequence_number()
    );
    // (6) It should emit the discovery set we fed into the genesis tx
    assert_eq!(
        &DiscoverySet::from_bytes(discovery_set_change_event.event_data()).unwrap(),
        discovery_set,
        "Discovery set in emitted event does not match discovery set fed into genesis transaction",
    );
}

/// Generate an artificial genesis `ChangeSet` for testing
pub fn generate_genesis_change_set_for_testing(stdlib_options: StdLibOptions) -> ChangeSet {
    let stdlib_modules = stdlib_modules(stdlib_options);
    let swarm = generator::validator_swarm_for_testing(10);

    encode_genesis_change_set(
        &GENESIS_KEYPAIR.1,
        &swarm.nodes,
        swarm.validator_set,
        swarm.discovery_set,
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
