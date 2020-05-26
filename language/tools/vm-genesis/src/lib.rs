// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod genesis_context;
mod genesis_gas_schedule;

use crate::{
    genesis_context::{GenesisContext, GenesisStateView},
    genesis_gas_schedule::INITIAL_GAS_SCHEDULE,
};
use bytecode_verifier::VerifiedModule;
use libra_config::config::NodeConfig;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    PrivateKey, Uniform, ValidCryptoMaterial,
};
use libra_network_address::RawNetworkAddress;
use libra_types::{
    account_config,
    contract_event::ContractEvent,
    on_chain_config::{config_address, new_epoch_event_key, VMPublishingOption},
    transaction::{authenticator::AuthenticationKey, ChangeSet, Script, Transaction},
};
use libra_vm::data_cache::StateViewCache;
use move_core_types::language_storage::{StructTag, TypeTag};
use move_vm_types::{data_store::DataStore, loaded_data::types::FatStructType, values::Value};
use once_cell::sync::Lazy;
use rand::prelude::*;
use std::{collections::btree_map::BTreeMap, convert::TryFrom};
use stdlib::{stdlib_modules, transaction_scripts::StdlibScript, StdLibOptions};
use vm::access::ModuleAccess;

// The seed is arbitrarily picked to produce a consistent key. XXX make this more formal?
const GENESIS_SEED: [u8; 32] = [42; 32];

const GENESIS_MODULE_NAME: &str = "Genesis";

// TODO(philiphayes): probably not the right place to put this...
const HANDSHAKE_VERSION: u8 = 0;

pub static GENESIS_KEYPAIR: Lazy<(Ed25519PrivateKey, Ed25519PublicKey)> = Lazy::new(|| {
    let mut rng = StdRng::from_seed(GENESIS_SEED);
    let private_key = Ed25519PrivateKey::generate(&mut rng);
    let public_key = private_key.public_key();
    (private_key, public_key)
});

pub type ValidatorRegistration = (Ed25519PublicKey, Script);

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
) -> (ChangeSet, BTreeMap<Vec<u8>, FatStructType>) {
    // create a data view for move_vm
    let mut state_view = GenesisStateView::new();
    for module in stdlib_modules {
        let module_id = module.self_id();
        state_view.add_module(&module_id, &module);
    }
    let data_cache = StateViewCache::new(&state_view);

    let mut genesis_context = GenesisContext::new(&data_cache, stdlib_modules);

    let lbr_ty = TypeTag::Struct(StructTag {
        address: *account_config::LBR_MODULE.address(),
        module: account_config::LBR_MODULE.name().to_owned(),
        name: account_config::LBR_STRUCT_NAME.to_owned(),
        type_params: vec![],
    });

    // generate the genesis WriteSet
    create_and_initialize_main_accounts(&mut genesis_context, &public_key, &lbr_ty);
    initialize_validators(&mut genesis_context, &validators, &lbr_ty);
    setup_vm_config(&mut genesis_context, vm_publishing_option);
    reconfigure(&mut genesis_context);

    let mut interpreter_context = genesis_context.into_interpreter_context();
    publish_stdlib(&mut interpreter_context, stdlib_modules);

    verify_genesis_write_set(interpreter_context.events());
    (
        ChangeSet::new(
            interpreter_context
                .make_write_set()
                .expect("Genesis WriteSet failure"),
            interpreter_context.events().to_vec(),
        ),
        interpreter_context.get_type_map(),
    )
}

pub fn encode_genesis_transaction(
    public_key: Ed25519PublicKey,
    validators: &[ValidatorRegistration],
    stdlib_modules: &[VerifiedModule],
    vm_publishing_option: VMPublishingOption,
) -> Transaction {
    Transaction::WaypointWriteSet(
        encode_genesis_change_set(
            &public_key,
            validators,
            stdlib_modules,
            vm_publishing_option,
        )
        .0,
    )
}

/// Create an initialize Association, Transaction Fee and Core Code accounts.
fn create_and_initialize_main_accounts(
    context: &mut GenesisContext,
    public_key: &Ed25519PublicKey,
    lbr_ty: &TypeTag,
) {
    let genesis_auth_key = AuthenticationKey::ed25519(public_key);

    let root_association_address = account_config::association_address();
    let tc_account_address = account_config::treasury_compliance_account_address();
    let fee_account_address = account_config::transaction_fee_address();

    context.set_sender(root_association_address);
    context.exec(
        GENESIS_MODULE_NAME,
        "initialize",
        vec![],
        vec![
            Value::transaction_argument_signer_reference(root_association_address),
            Value::transaction_argument_signer_reference(config_address()),
            Value::address(tc_account_address),
            Value::vector_u8(AuthenticationKey::prefix(&genesis_auth_key).to_vec()), // TODO: different key here?
            Value::vector_u8(genesis_auth_key.to_vec()),
        ],
    );

    // TODO: use signer to eliminate this
    context.set_sender(config_address());
    context.exec(
        "LibraAccount",
        "rotate_authentication_key",
        vec![],
        vec![Value::vector_u8(genesis_auth_key.to_vec())],
    );

    // TODO: use signer to eliminate this
    context.set_sender(account_config::treasury_compliance_account_address());
    context.exec(
        "LibraAccount",
        "rotate_authentication_key",
        vec![],
        vec![Value::vector_u8(genesis_auth_key.to_vec())],
    );

    // TODO: use signer to eliminate this
    context.set_sender(fee_account_address);
    context.exec(
        GENESIS_MODULE_NAME,
        "initialize_txn_fee_account",
        vec![],
        vec![
            Value::transaction_argument_signer_reference(fee_account_address),
            Value::vector_u8(genesis_auth_key.to_vec()),
        ],
    );

    context.set_sender(root_association_address);
    // Bump the sequence number for the Association account. If we don't do this and a
    // subsequent transaction (e.g., minting) is sent from the Assocation account, a problem
    // arises: both the genesis transaction and the subsequent transaction have sequence
    // number 0
    context.exec(
        "LibraAccount",
        "epilogue",
        vec![lbr_ty.clone()],
        vec![
            Value::u64(/* txn_sequence_number */ 0),
            Value::u64(/* txn_gas_price */ 0),
            Value::u64(/* txn_max_gas_units */ 0),
            Value::u64(/* gas_units_remaining */ 0),
        ],
    );
}

/// Initialize each validator.
fn initialize_validators(
    context: &mut GenesisContext,
    validators: &[ValidatorRegistration],
    lbr_ty: &TypeTag,
) {
    for (account_key, registration) in validators {
        context.set_sender(account_config::association_address());
        let auth_key = AuthenticationKey::ed25519(&account_key);
        let account = auth_key.derived_address();

        // Create an account

        context.exec(
            "LibraAccount",
            "create_testnet_account",
            vec![lbr_ty.clone()],
            vec![
                Value::address(account),
                Value::vector_u8(auth_key.prefix().to_vec()),
            ],
        );

        context.set_sender(account);
        context.exec_script(registration);

        context.set_sender(account_config::association_address());
        // Finally, add the account to the validator set
        context.exec(
            "LibraSystem",
            "add_validator",
            vec![],
            vec![Value::address(account)],
        );
    }
}

fn setup_vm_config(context: &mut GenesisContext, publishing_option: VMPublishingOption) {
    context.set_sender(config_address());

    let option_bytes =
        lcs::to_bytes(&publishing_option).expect("Cannot serialize publishing option");
    context.exec(
        "LibraVMConfig",
        "initialize",
        vec![],
        vec![
            Value::vector_u8(option_bytes),
            Value::vector_u8(INITIAL_GAS_SCHEDULE.0.clone()),
            Value::vector_u8(INITIAL_GAS_SCHEDULE.1.clone()),
            Value::transaction_argument_signer_reference(config_address()),
        ],
    );
}

fn remove_genesis(stdlib_modules: &[VerifiedModule]) -> impl Iterator<Item = &VerifiedModule> {
    stdlib_modules
        .iter()
        .filter(|module| module.self_id().name().as_str() != GENESIS_MODULE_NAME)
}

/// Publish the standard library.
fn publish_stdlib(interpreter_context: &mut dyn DataStore, stdlib: &[VerifiedModule]) {
    for module in remove_genesis(stdlib) {
        assert!(module.self_id().name().as_str() != GENESIS_MODULE_NAME);
        let mut module_vec = vec![];
        module.serialize(&mut module_vec).unwrap();
        interpreter_context
            .publish_module(module.self_id(), module_vec)
            .unwrap_or_else(|_| panic!("Failure publishing module {:?}", module.self_id()));
    }
}

/// Trigger a reconfiguration. This emits an event that will be passed along to the storage layer.
fn reconfigure(context: &mut GenesisContext) {
    context.set_sender(account_config::association_address());
    context.exec("LibraConfig", "emit_reconfiguration_event", vec![], vec![]);
}

/// Verify the consistency of the genesis `WriteSet`
fn verify_genesis_write_set(events: &[ContractEvent]) {
    // Sanity checks on emitted events:
    // (1) The genesis tx should emit 1 event: a NewEpochEvent.
    assert_eq!(
        events.len(),
        1,
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
        &validator_registrations(&swarm.nodes),
        stdlib_modules,
        VMPublishingOption::Open,
    )
    .0
}

/// Generate an artificial genesis `ChangeSet` for testing
pub fn generate_genesis_type_mapping() -> BTreeMap<Vec<u8>, FatStructType> {
    let stdlib_modules = stdlib_modules(StdLibOptions::Staged);
    let swarm = libra_config::generator::validator_swarm_for_testing(10);

    encode_genesis_change_set(
        &GENESIS_KEYPAIR.1,
        &validator_registrations(&swarm.nodes),
        stdlib_modules,
        VMPublishingOption::Open,
    )
    .1
}

pub fn validator_registrations(node_configs: &[NodeConfig]) -> Vec<ValidatorRegistration> {
    node_configs
        .iter()
        .map(|n| {
            let test = n.test.as_ref().unwrap();
            let account_key = test.operator_keypair.as_ref().unwrap().public_key();
            let consensus_key = test.consensus_keypair.as_ref().unwrap().public_key();
            let network = n.validator_network.as_ref().unwrap();
            let network_keypairs = network.network_keypairs.as_ref().unwrap();
            let signing_key = network_keypairs.signing_keypair.public_key();
            let identity_key = network_keypairs.identity_keypair.public_key();

            let advertised_address = network
                .advertised_address
                .clone()
                .append_prod_protos(identity_key, HANDSHAKE_VERSION);
            let raw_advertised_address = RawNetworkAddress::try_from(&advertised_address).unwrap();

            // TODO(philiphayes): do something with n.full_node_networks instead
            // of ignoring them?

            let script = transaction_builder::encode_register_validator_script(
                consensus_key.to_bytes().to_vec(),
                signing_key.to_bytes().to_vec(),
                identity_key.to_bytes(),
                raw_advertised_address.clone().into(),
                identity_key.to_bytes(),
                raw_advertised_address.into(),
            );
            (account_key, script)
        })
        .collect::<Vec<_>>()
}
