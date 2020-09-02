// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod genesis_context;
mod genesis_gas_schedule;

use crate::{genesis_context::GenesisStateView, genesis_gas_schedule::INITIAL_GAS_SCHEDULE};
use compiled_stdlib::{stdlib_modules, transaction_scripts::StdlibScript, StdLibOptions};
use libra_config::config::{NodeConfig, HANDSHAKE_VERSION};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    PrivateKey, Uniform,
};
use libra_network_address::encrypted::{
    TEST_SHARED_VAL_NETADDR_KEY, TEST_SHARED_VAL_NETADDR_KEY_VERSION,
};
use libra_types::{
    account_address, account_config,
    chain_id::ChainId,
    contract_event::ContractEvent,
    on_chain_config::{new_epoch_event_key, VMPublishingOption},
    transaction::{
        authenticator::AuthenticationKey, ChangeSet, Script, Transaction, TransactionArgument,
        WriteSetPayload,
    },
};
use libra_vm::{data_cache::StateViewCache, txn_effects_to_writeset_and_events};
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{CostTable, GasAlgebra, GasUnits},
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
};
use move_vm_runtime::{data_cache::TransactionEffects, move_vm::MoveVM, session::Session};
use move_vm_types::{
    gas_schedule::{zero_cost_schedule, CostStrategy},
    values::Value,
};
use once_cell::sync::Lazy;
use rand::prelude::*;
use std::collections::btree_map::BTreeMap;
use transaction_builder::encode_create_designated_dealer_script;
use vm::{file_format::SignatureToken, CompiledModule};

// The seed is arbitrarily picked to produce a consistent key. XXX make this more formal?
const GENESIS_SEED: [u8; 32] = [42; 32];

const GENESIS_MODULE_NAME: &str = "Genesis";

pub static GENESIS_KEYPAIR: Lazy<(Ed25519PrivateKey, Ed25519PublicKey)> = Lazy::new(|| {
    let mut rng = StdRng::from_seed(GENESIS_SEED);
    let private_key = Ed25519PrivateKey::generate(&mut rng);
    let public_key = private_key.public_key();
    (private_key, public_key)
});

pub static ZERO_COST_SCHEDULE: Lazy<CostTable> = Lazy::new(zero_cost_schedule);

const ZERO_AUTH_KEY: [u8; 32] = [0; 32];

pub type Name = Vec<u8>;
// Defines a validator owner and maps that to an operator
pub type OperatorAssignment = (Option<Ed25519PublicKey>, Name, Script);

// Defines a validator operator and maps that to a validator (config)
pub type OperatorRegistration = (Ed25519PublicKey, Name, Script);

pub fn encode_genesis_transaction(
    libra_root_key: Ed25519PublicKey,
    treasury_compliance_key: Ed25519PublicKey,
    operator_assignments: &[OperatorAssignment],
    operator_registrations: &[OperatorRegistration],
    vm_publishing_option: Option<VMPublishingOption>,
    chain_id: ChainId,
) -> Transaction {
    Transaction::GenesisTransaction(WriteSetPayload::Direct(
        encode_genesis_change_set(
            &libra_root_key,
            &treasury_compliance_key,
            operator_assignments,
            operator_registrations,
            stdlib_modules(StdLibOptions::Compiled), // Must use compiled stdlib,
            vm_publishing_option
                .unwrap_or_else(|| VMPublishingOption::locked(StdlibScript::allowlist())),
            chain_id,
        )
        .0,
    ))
}

fn merge_txn_effects(
    mut effects_1: TransactionEffects,
    effects_2: TransactionEffects,
) -> TransactionEffects {
    effects_1.resources.extend(effects_2.resources);
    effects_1.modules.extend(effects_2.modules);
    effects_1.events.extend(effects_2.events);
    effects_1
}

pub fn encode_genesis_change_set(
    libra_root_key: &Ed25519PublicKey,
    treasury_compliance_key: &Ed25519PublicKey,
    operator_assignments: &[OperatorAssignment],
    operator_registrations: &[OperatorRegistration],
    stdlib_modules: &[CompiledModule],
    vm_publishing_option: VMPublishingOption,
    chain_id: ChainId,
) -> (ChangeSet, BTreeMap<Vec<u8>, StructTag>) {
    // create a data view for move_vm
    let mut state_view = GenesisStateView::new();
    for module in stdlib_modules {
        let module_id = module.self_id();
        state_view.add_module(&module_id, &module);
    }
    let data_cache = StateViewCache::new(&state_view);

    let move_vm = MoveVM::new();
    let mut session = move_vm.new_session(&data_cache);

    let lbr_ty = TypeTag::Struct(StructTag {
        address: *account_config::LBR_MODULE.address(),
        module: account_config::LBR_MODULE.name().to_owned(),
        name: account_config::LBR_STRUCT_NAME.to_owned(),
        type_params: vec![],
    });

    create_and_initialize_main_accounts(
        &mut session,
        &libra_root_key,
        &treasury_compliance_key,
        vm_publishing_option,
        &lbr_ty,
        chain_id,
    );
    // generate the genesis WriteSet
    create_and_initialize_owners_operators(
        &mut session,
        &operator_assignments,
        &operator_registrations,
    );
    reconfigure(&mut session);

    // XXX/TODO: for testnet only
    create_and_initialize_testnet_minting(&mut session, &treasury_compliance_key);

    let effects_1 = session.finish().unwrap();

    let state_view = GenesisStateView::new();
    let data_cache = StateViewCache::new(&state_view);
    let mut session = move_vm.new_session(&data_cache);
    publish_stdlib(&mut session, stdlib_modules);
    let effects_2 = session.finish().unwrap();

    let effects = merge_txn_effects(effects_1, effects_2);

    // REVIEW: Performance & caching.
    let type_mapping = effects
        .resources
        .iter()
        .flat_map(|(_adddr, resources)| {
            resources.iter().map(|(ty_tag, _)| match ty_tag {
                TypeTag::Struct(struct_tag) => (struct_tag.access_vector(), struct_tag.clone()),
                _ => panic!("not a struct"),
            })
        })
        .collect();
    let (write_set, events) = txn_effects_to_writeset_and_events(effects).unwrap();

    assert!(!write_set.iter().any(|(_, op)| op.is_deletion()));
    verify_genesis_write_set(&events);
    (ChangeSet::new(write_set, events), type_mapping)
}

/// Convert the transaction arguments into Move values.
fn convert_txn_args(args: &[TransactionArgument]) -> Vec<Value> {
    args.iter()
        .map(|arg| match arg {
            TransactionArgument::U8(i) => Value::u8(*i),
            TransactionArgument::U64(i) => Value::u64(*i),
            TransactionArgument::U128(i) => Value::u128(*i),
            TransactionArgument::Address(a) => Value::address(*a),
            TransactionArgument::Bool(b) => Value::bool(*b),
            TransactionArgument::U8Vector(v) => Value::vector_u8(v.clone()),
        })
        .collect()
}

fn exec_function(
    session: &mut Session<StateViewCache>,
    sender: AccountAddress,
    module_name: &str,
    function_name: &str,
    ty_args: Vec<TypeTag>,
    args: Vec<Value>,
) {
    session
        .execute_function(
            &ModuleId::new(
                account_config::CORE_CODE_ADDRESS,
                Identifier::new(module_name).unwrap(),
            ),
            &Identifier::new(function_name).unwrap(),
            ty_args,
            args,
            sender,
            &mut CostStrategy::system(&ZERO_COST_SCHEDULE, GasUnits::new(100_000_000)),
            |e| e,
        )
        .unwrap_or_else(|e| panic!("Error calling {}.{}: {}", module_name, function_name, e))
}

fn exec_script(session: &mut Session<StateViewCache>, sender: AccountAddress, script: &Script) {
    session
        .execute_script(
            script.code().to_vec(),
            script.ty_args().to_vec(),
            convert_txn_args(script.args()),
            vec![sender],
            &mut CostStrategy::system(&ZERO_COST_SCHEDULE, GasUnits::new(100_000_000)),
        )
        .unwrap()
}

/// Create and initialize Association and Core Code accounts.
fn create_and_initialize_main_accounts(
    session: &mut Session<StateViewCache>,
    libra_root_key: &Ed25519PublicKey,
    treasury_compliance_key: &Ed25519PublicKey,
    publishing_option: VMPublishingOption,
    lbr_ty: &TypeTag,
    chain_id: ChainId,
) {
    let libra_root_auth_key = AuthenticationKey::ed25519(libra_root_key);
    let treasury_compliance_auth_key = AuthenticationKey::ed25519(treasury_compliance_key);

    let root_libra_root_address = account_config::libra_root_address();
    let tc_account_address = account_config::treasury_compliance_account_address();

    let initial_allow_list = Value::constant_vector_generic(
        publishing_option
            .script_allow_list
            .into_iter()
            .map(|hash| Value::vector_u8(hash.to_vec().into_iter())),
        &Box::new(SignatureToken::Vector(Box::new(SignatureToken::U8))),
    )
    .unwrap();

    exec_function(
        session,
        root_libra_root_address,
        GENESIS_MODULE_NAME,
        "initialize",
        vec![],
        vec![
            Value::transaction_argument_signer_reference(root_libra_root_address),
            Value::transaction_argument_signer_reference(tc_account_address),
            Value::vector_u8(libra_root_auth_key.to_vec()),
            Value::address(tc_account_address),
            Value::vector_u8(treasury_compliance_auth_key.to_vec()),
            initial_allow_list,
            Value::bool(publishing_option.is_open_module),
            Value::vector_u8(INITIAL_GAS_SCHEDULE.0.clone()),
            Value::vector_u8(INITIAL_GAS_SCHEDULE.1.clone()),
            Value::u8(chain_id.id()),
        ],
    );

    // Bump the sequence number for the Association account. If we don't do this and a
    // subsequent transaction (e.g., minting) is sent from the Assocation account, a problem
    // arises: both the genesis transaction and the subsequent transaction have sequence
    // number 0
    exec_function(
        session,
        root_libra_root_address,
        "LibraAccount",
        "success_epilogue",
        vec![lbr_ty.clone()],
        vec![
            Value::transaction_argument_signer_reference(root_libra_root_address),
            Value::u64(/* txn_sequence_number */ 0),
            Value::u64(/* txn_gas_price */ 0),
            Value::u64(/* txn_max_gas_units */ 0),
            Value::u64(/* gas_units_remaining */ 0),
        ],
    );
}

fn create_and_initialize_testnet_minting(
    session: &mut Session<StateViewCache>,
    public_key: &Ed25519PublicKey,
) {
    let genesis_auth_key = AuthenticationKey::ed25519(public_key);
    let coin1_tag = account_config::type_tag_for_currency_code(
        account_config::from_currency_code_string("Coin1").unwrap(),
    );
    let coin2_tag = account_config::type_tag_for_currency_code(
        account_config::from_currency_code_string("Coin2").unwrap(),
    );
    let create_dd_script = encode_create_designated_dealer_script(
        coin1_tag.clone(),
        0,
        account_config::testnet_dd_account_address(),
        genesis_auth_key.prefix().to_vec(),
        b"moneybags".to_vec(), // name
        true,                  // add_all_currencies
    );

    let mint_max_coin1 = transaction_builder::encode_tiered_mint_script(
        coin1_tag,
        0,
        account_config::testnet_dd_account_address(),
        std::u64::MAX / 2,
        3,
    );

    let mint_max_coin2 = transaction_builder::encode_tiered_mint_script(
        coin2_tag,
        0,
        account_config::testnet_dd_account_address(),
        std::u64::MAX / 2,
        3,
    );

    // Create the DD account
    exec_script(
        session,
        account_config::treasury_compliance_account_address(),
        &create_dd_script,
    );
    exec_function(
        session,
        account_config::treasury_compliance_account_address(),
        "DesignatedDealer",
        "update_tier",
        vec![account_config::coin1_tag()],
        vec![
            Value::transaction_argument_signer_reference(
                account_config::treasury_compliance_account_address(),
            ),
            Value::address(account_config::testnet_dd_account_address()),
            Value::u64(3),
            Value::u64(std::u64::MAX),
        ],
    );

    exec_function(
        session,
        account_config::treasury_compliance_account_address(),
        "DesignatedDealer",
        "update_tier",
        vec![account_config::type_tag_for_currency_code(
            account_config::from_currency_code_string(account_config::COIN2_NAME).unwrap(),
        )],
        vec![
            Value::transaction_argument_signer_reference(
                account_config::treasury_compliance_account_address(),
            ),
            Value::address(account_config::testnet_dd_account_address()),
            Value::u64(3),
            Value::u64(std::u64::MAX),
        ],
    );

    // mint the coins, and mint LBR
    let treasury_compliance_account_address = account_config::treasury_compliance_account_address();
    exec_script(
        session,
        treasury_compliance_account_address,
        &mint_max_coin1,
    );
    exec_script(
        session,
        treasury_compliance_account_address,
        &mint_max_coin2,
    );

    let testnet_dd_account_address = account_config::testnet_dd_account_address();
    exec_script(
        session,
        testnet_dd_account_address,
        &transaction_builder::encode_mint_lbr_script(std::u64::MAX / 2),
    );
    exec_script(
        session,
        testnet_dd_account_address,
        &transaction_builder::encode_rotate_authentication_key_script(genesis_auth_key.to_vec()),
    );
}

/// Creates and initializes each validator owner and validator operator. This method creates all
/// the required accounts, sets the validator operators for each validator owner, and sets the
/// validator config on-chain.
fn create_and_initialize_owners_operators(
    session: &mut Session<StateViewCache>,
    operator_assignments: &[OperatorAssignment],
    operator_registrations: &[OperatorRegistration],
) {
    let libra_root_address = account_config::libra_root_address();

    // Create accounts for each validator owner. The inputs for creating an account are the auth
    // key prefix and account address. Internally move then computes the auth key as auth key
    // prefix || address. Because of this, the initial auth key will be invalid as we produce the
    // account address from the name and not the public key.
    for (owner_key, owner_name, _op_assignment) in operator_assignments {
        let staged_owner_auth_key =
            libra_config::utils::default_validator_owner_auth_key_from_name(owner_name);
        let owner_address = staged_owner_auth_key.derived_address();
        let create_owner_script = transaction_builder::encode_create_validator_account_script(
            0,
            owner_address,
            staged_owner_auth_key.prefix().to_vec(),
            owner_name.clone(),
        );
        exec_script(session, libra_root_address, &create_owner_script);

        // If there is a key, make it the auth key, otherwise use a zero auth key.
        let real_owner_auth_key = if let Some(owner_key) = owner_key {
            AuthenticationKey::ed25519(owner_key).to_vec()
        } else {
            ZERO_AUTH_KEY.to_vec()
        };

        exec_script(
            session,
            owner_address,
            &transaction_builder::encode_rotate_authentication_key_script(real_owner_auth_key),
        );
    }

    // Create accounts for each validator operator
    for (operator_key, operator_name, _) in operator_registrations {
        let operator_auth_key = AuthenticationKey::ed25519(&operator_key);
        let operator_account = account_address::from_public_key(operator_key);
        let create_operator_script =
            transaction_builder::encode_create_validator_operator_account_script(
                0,
                operator_account,
                operator_auth_key.prefix().to_vec(),
                operator_name.clone(),
            );
        exec_script(session, libra_root_address, &create_operator_script);
    }

    // Set the validator operator for each validator owner
    for (_owner_key, owner_name, op_assignment) in operator_assignments {
        let owner_address = libra_config::utils::validator_owner_account_from_name(owner_name);
        exec_script(session, owner_address, op_assignment);
    }

    // Set the validator config for each validator
    for (operator_key, _, registration) in operator_registrations {
        let operator_account = account_address::from_public_key(operator_key);
        exec_script(session, operator_account, registration);
    }

    // Add each validator to the validator set
    for (_owner_key, owner_name, _op_assignment) in operator_assignments {
        let owner_address = libra_config::utils::validator_owner_account_from_name(owner_name);
        exec_function(
            session,
            libra_root_address,
            "LibraSystem",
            "add_validator",
            vec![],
            vec![
                Value::transaction_argument_signer_reference(libra_root_address),
                Value::address(owner_address),
            ],
        );
    }
}

fn remove_genesis(stdlib_modules: &[CompiledModule]) -> impl Iterator<Item = &CompiledModule> {
    stdlib_modules
        .iter()
        .filter(|module| module.self_id().name().as_str() != GENESIS_MODULE_NAME)
}

/// Publish the standard library.
fn publish_stdlib(session: &mut Session<StateViewCache>, stdlib: &[CompiledModule]) {
    for module in remove_genesis(stdlib) {
        assert!(module.self_id().name().as_str() != GENESIS_MODULE_NAME);
        let mut module_vec = vec![];
        module.serialize(&mut module_vec).unwrap();
        session
            .publish_module(
                module_vec,
                *module.self_id().address(),
                &mut CostStrategy::system(&ZERO_COST_SCHEDULE, GasUnits::new(100_000_000)),
            )
            .unwrap_or_else(|e| {
                panic!("Failure publishing module {:?}, {:?}", module.self_id(), e)
            });
    }
}

/// Trigger a reconfiguration. This emits an event that will be passed along to the storage layer.
fn reconfigure(session: &mut Session<StateViewCache>) {
    exec_function(
        session,
        account_config::libra_root_address(),
        "LibraConfig",
        "emit_genesis_reconfiguration_event",
        vec![],
        vec![],
    );
}

/// Verify the consistency of the genesis `WriteSet`
fn verify_genesis_write_set(events: &[ContractEvent]) {
    // Sanity checks on emitted events:
    // (1) The genesis tx should emit 1 event: a NewEpochEvent.
    assert_eq!(
        events.len(),
        //1, // This is the proper number of events for mainnet. Once we have a good layering
        // strategy for mainnet/testnet genesis writesets uncomment this and remove the line
        // below.
        10, // XXX/TODO(tzakian). For testnet only!
        "Genesis transaction should emit one event, but found {} events: {:?}",
        events.len(),
        events,
    );

    // (2) The first event should be the new epoch event
    let new_epoch_event = &events[0];
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
        &GENESIS_KEYPAIR.1,
        &operator_assignments(&swarm.nodes),
        &operator_registrations(&swarm.nodes),
        stdlib_modules,
        VMPublishingOption::open(),
        ChainId::test(),
    )
    .0
}

/// Generate an artificial genesis `ChangeSet` for testing
pub fn generate_genesis_type_mapping() -> BTreeMap<Vec<u8>, StructTag> {
    let stdlib_modules = stdlib_modules(StdLibOptions::Compiled);
    let swarm = libra_config::generator::validator_swarm_for_testing(10);

    encode_genesis_change_set(
        &GENESIS_KEYPAIR.1,
        &GENESIS_KEYPAIR.1,
        &operator_assignments(&swarm.nodes),
        &operator_registrations(&swarm.nodes),
        stdlib_modules,
        VMPublishingOption::open(),
        ChainId::test(),
    )
    .1
}

/// Generates an artificial set of OperatorAssignments using the given node configurations for
/// testing.
pub fn operator_assignments(node_configs: &[NodeConfig]) -> Vec<OperatorAssignment> {
    node_configs
        .iter()
        .enumerate()
        .map(|(idx, n)| {
            let test_config = n.test.as_ref().unwrap();
            let owner_key = test_config.owner_key.as_ref().unwrap().public_key();
            let operator_key = test_config.operator_key.as_ref().unwrap().public_key();
            let operator_account = account_address::from_public_key(&operator_key);
            let name = idx.to_string().as_bytes().to_vec();
            let set_operator_script = transaction_builder::encode_set_validator_operator_script(
                name.clone(),
                operator_account,
            );

            (Some(owner_key), name, set_operator_script)
        })
        .collect::<Vec<_>>()
}

/// Generates an artificial set of OperatorRegistrations using the given node configurations for
/// testing.
pub fn operator_registrations(node_configs: &[NodeConfig]) -> Vec<OperatorRegistration> {
    node_configs
        .iter()
        .enumerate()
        .map(|(idx, n)| {
            let test_config = n.test.as_ref().unwrap();
            let name = idx.to_string().as_bytes().to_vec();
            let owner_account = libra_config::utils::validator_owner_account_from_name(&name);
            let operator_key = test_config.operator_key.as_ref().unwrap().public_key();

            let sr_test = n.consensus.safety_rules.test.as_ref().unwrap();
            let consensus_key = sr_test.consensus_key.as_ref().unwrap().public_key();

            let network = n.validator_network.as_ref().unwrap();
            let identity_key = network.identity_key().public_key();

            let addr = network
                .discovery_method
                .advertised_address()
                .append_prod_protos(identity_key, HANDSHAKE_VERSION);

            let seq_num = 0;
            let addr_idx = 0;
            let enc_addr = addr.clone().encrypt(
                &TEST_SHARED_VAL_NETADDR_KEY,
                TEST_SHARED_VAL_NETADDR_KEY_VERSION,
                &owner_account,
                seq_num,
                addr_idx,
            );

            // TODO(philiphayes): do something with n.full_node_networks instead
            // of ignoring them?

            let script = transaction_builder::encode_register_validator_config_script(
                owner_account,
                consensus_key.to_bytes().to_vec(),
                lcs::to_bytes(&vec![enc_addr.unwrap()]).unwrap(),
                lcs::to_bytes(&vec![addr]).unwrap(),
            );
            (operator_key, name, script)
        })
        .collect::<Vec<_>>()
}
