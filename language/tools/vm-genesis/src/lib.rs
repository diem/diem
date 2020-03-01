// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod genesis_gas_schedule;

use crate::genesis_gas_schedule::initial_gas_schedule;
use anyhow::Result;
use bytecode_verifier::VerifiedModule;
use libra_crypto::{
    ed25519::*,
    traits::ValidKey,
    x25519::{X25519StaticPrivateKey, X25519StaticPublicKey},
};
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    contract_event::ContractEvent,
    crypto_proxies::ValidatorSet,
    discovery_info::DiscoveryInfo,
    discovery_set::DiscoverySet,
    transaction::{ChangeSet, RawTransaction, SignatureCheckedTransaction},
};
use libra_vm::system_module_names::*;
use move_core_types::identifier::Identifier;
use move_vm_runtime::MoveVM;
use move_vm_state::{
    data_cache::BlockDataCache,
    execution_context::{ExecutionContext, TransactionExecutionContext},
};
use move_vm_types::{chain_state::ChainState, values::Value};
use once_cell::sync::Lazy;
use parity_multiaddr::Multiaddr;
use rand::{rngs::StdRng, SeedableRng};
use std::str::FromStr;
use stdlib::{stdlib_modules, StdLibOptions};
use vm::{
    access::ModuleAccess,
    gas_schedule::{CostTable, GasAlgebra, GasUnits},
    transaction_metadata::TransactionMetadata,
};

// The seed is arbitrarily picked to produce a consistent key. XXX make this more formal?
const GENESIS_SEED: [u8; 32] = [42; 32];

/// The initial balance of the association account.
pub const ASSOCIATION_INIT_BALANCE: u64 = 1_000_000_000_000_000;

pub static GENESIS_KEYPAIR: Lazy<(Ed25519PrivateKey, Ed25519PublicKey)> = Lazy::new(|| {
    let mut rng = StdRng::from_seed(GENESIS_SEED);
    compat::generate_keypair(&mut rng)
});
// TODO(philiphayes): remove this when we add discovery set to genesis config.
static PLACEHOLDER_PUBKEY: Lazy<X25519StaticPublicKey> = Lazy::new(|| {
    let salt = None;
    let seed = [69u8; 32];
    let app_info = None;
    let (_, pubkey) = X25519StaticPrivateKey::derive_keypair_from_seed(salt, &seed, app_info);
    pubkey
});

// Identifiers for well-known functions.
static ADD_VALIDATOR: Lazy<Identifier> = Lazy::new(|| Identifier::new("add_validator").unwrap());
static INITIALIZE: Lazy<Identifier> = Lazy::new(|| Identifier::new("initialize").unwrap());
static INITIALIZE_BLOCK: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_block_metadata").unwrap());
static INITIALIZE_TXN_FEES: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_transaction_fees").unwrap());
static INITIALIZE_VALIDATOR_SET: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_validator_set").unwrap());
static INITIALIZE_DISCOVERY_SET: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_discovery_set").unwrap());
static MINT_TO_ADDRESS: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("mint_to_address").unwrap());
static RECONFIGURE: Lazy<Identifier> = Lazy::new(|| Identifier::new("reconfigure").unwrap());
static REGISTER_CANDIDATE_VALIDATOR: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("register_candidate_validator").unwrap());
static ROTATE_AUTHENTICATION_KEY: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("rotate_authentication_key").unwrap());
static EPILOGUE: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());

// TODO(philiphayes): remove this after integrating on-chain discovery with config.
/// Make a placeholder `DiscoverySet` from the `ValidatorSet`.
pub fn make_placeholder_discovery_set(validator_set: &ValidatorSet) -> DiscoverySet {
    let mock_addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();
    let discovery_set = validator_set
        .iter()
        .map(|validator_pubkeys| DiscoveryInfo {
            account_address: *validator_pubkeys.account_address(),
            validator_network_identity_pubkey: validator_pubkeys
                .network_identity_public_key()
                .clone(),
            validator_network_address: mock_addr.clone(),
            fullnodes_network_identity_pubkey: PLACEHOLDER_PUBKEY.clone(),
            fullnodes_network_address: mock_addr.clone(),
        })
        .collect::<Vec<_>>();
    DiscoverySet::new(discovery_set)
}

pub fn encode_genesis_transaction_with_validator(
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    validator_set: ValidatorSet,
    discovery_set: DiscoverySet,
) -> SignatureCheckedTransaction {
    encode_genesis_transaction_with_validator_and_modules(
        private_key,
        public_key,
        validator_set,
        discovery_set,
        stdlib_modules(StdLibOptions::Staged), // Must use staged stdlib
    )
}

pub fn encode_genesis_transaction_with_validator_and_modules(
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    validator_set: ValidatorSet,
    discovery_set: DiscoverySet,
    stdlib_modules: &[VerifiedModule],
) -> SignatureCheckedTransaction {
    // create a MoveVM
    let move_vm = MoveVM::new();

    // create a data view for move_vm
    let state_view = GenesisStateView;
    let gas_schedule = CostTable::zero();
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
        move_vm.cache_module(module.clone());
    }

    // generate the genesis WriteSet
    let genesis_write_set = {
        {
            create_and_initialize_main_accounts(
                &move_vm,
                &gas_schedule,
                &mut interpreter_context,
                &public_key,
                initial_gas_schedule(&move_vm, &data_cache),
            );
            create_and_initialize_validator_and_discovery_set(
                &move_vm,
                &gas_schedule,
                &mut interpreter_context,
                &validator_set,
                &discovery_set,
            );
            reconfigure(&move_vm, &gas_schedule, &mut interpreter_context);
            publish_stdlib(&mut interpreter_context, stdlib_modules);

            verify_genesis_write_set(interpreter_context.events(), &validator_set, &discovery_set);
            ChangeSet::new(
                interpreter_context
                    .make_write_set()
                    .expect("Genesis WriteSet failure"),
                interpreter_context.events().to_vec(),
            )
        }
    };
    let transaction =
        RawTransaction::new_change_set(account_config::association_address(), 0, genesis_write_set);
    transaction.sign(private_key, public_key).unwrap()
}

/// Create an initialize Association, Transaction Fee and Core Code accounts.
fn create_and_initialize_main_accounts(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    public_key: &Ed25519PublicKey,
    initial_gas_schedule: Value,
) {
    let association_addr = account_config::association_address();
    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = association_addr;

    // create the association account
    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &CREATE_ACCOUNT_NAME,
            gas_schedule,
            interpreter_context,
            &txn_data,
            vec![Value::address(association_addr)],
        )
        .unwrap_or_else(|_| {
            panic!(
                "Failure creating association account {:?}",
                association_addr
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
            vec![Value::address(transaction_fee_address)],
        )
        .unwrap_or_else(|_| {
            panic!(
                "Failure creating transaction fee account {:?}",
                transaction_fee_address
            )
        });

    move_vm
        .execute_function(
            &account_config::COIN_MODULE,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
        )
        .expect("Failure initializing LibraCoin");

    move_vm
        .execute_function(
            &LIBRA_TRANSACTION_TIMEOUT,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
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
        )
        .expect("Failure initializing block metadata");

    move_vm
        .execute_function(
            &GAS_SCHEDULE_MODULE,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![initial_gas_schedule],
        )
        .expect("Failure initializing gas module");

    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &MINT_TO_ADDRESS,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![
                Value::address(association_addr),
                Value::u64(ASSOCIATION_INIT_BALANCE),
            ],
        )
        .expect("Failure minting to association");

    let genesis_auth_key = AccountAddress::from_public_key(public_key).to_vec();
    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &ROTATE_AUTHENTICATION_KEY,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![Value::vector_u8(genesis_auth_key)],
        )
        .expect("Failure rotating association key");

    // Bump the sequence number for the Association account. If we don't do this and a
    // subsequent transaction (e.g., minting) is sent from the Assocation account, a problem
    // arises: both the genesis transaction and the subsequent transaction have sequence
    // number 0
    move_vm
        .execute_function(
            &account_config::ACCOUNT_MODULE,
            &EPILOGUE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
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
        )
        .expect("Failure initializing transaction fee");
}

/// Create and initialize validator and discovery set.
fn create_and_initialize_validator_and_discovery_set(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    validator_set: &ValidatorSet,
    discovery_set: &DiscoverySet,
) {
    create_and_initialize_validator_set(move_vm, gas_schedule, interpreter_context);
    create_and_initialize_discovery_set(move_vm, gas_schedule, interpreter_context);
    initialize_validators(
        move_vm,
        gas_schedule,
        interpreter_context,
        validator_set,
        discovery_set,
    );
}

/// Create and initialize the validator set.
fn create_and_initialize_validator_set(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
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
            vec![Value::address(validator_set_address)],
        )
        .unwrap_or_else(|_| {
            panic!(
                "Failure creating validator set account {:?}",
                validator_set_address
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
        )
        .expect("Failure initializing validator set");
}

/// Create and initialize the discovery set.
fn create_and_initialize_discovery_set(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
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
            vec![Value::address(discovery_set_address)],
        )
        .unwrap_or_else(|_| {
            panic!(
                "Failure creating discovery set account {:?}",
                discovery_set_address
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
        )
        .expect("Failure initializing discovery set");
}

/// Initialize each validator.
fn initialize_validators(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    validator_set: &ValidatorSet,
    discovery_set: &DiscoverySet,
) {
    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = account_config::association_address();

    for (validator_keys, discovery_info) in validator_set.iter().zip(discovery_set.iter()).rev() {
        // First, add a ValidatorConfig resource under each account
        let validator_address = *validator_keys.account_address();
        move_vm
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                &CREATE_ACCOUNT_NAME,
                gas_schedule,
                interpreter_context,
                &txn_data,
                vec![Value::address(validator_address)],
            )
            .unwrap_or_else(|_| {
                panic!("Failure creating validator account {:?}", validator_address)
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
                vec![Value::address(validator_address)],
            )
            .unwrap_or_else(|_| panic!("Failure adding validator {:?}", validator_address));
    }
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
    let txn_data = TransactionMetadata::default();

    // TODO: Direct write set transactions cannot specify emitted events, so this currently
    // will not work.
    move_vm
        .execute_function(
            &LIBRA_SYSTEM_MODULE,
            &RECONFIGURE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
        )
        .expect("Failure reconfiguring the system");
}

/// Verify the consistency of the genesis `WriteSet`
fn verify_genesis_write_set(
    events: &[ContractEvent],
    validator_set: &ValidatorSet,
    discovery_set: &DiscoverySet,
) {
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

    // (2) The third event should be the validator set change event
    let validator_set_change_event = &events[2];
    assert_eq!(
        *validator_set_change_event.key(),
        ValidatorSet::change_event_key(),
        "Key of emitted event {:?} does not match change event key {:?}",
        *validator_set_change_event.key(),
        ValidatorSet::change_event_key()
    );
    // (3) This should be the first validator set change event
    assert_eq!(
        validator_set_change_event.sequence_number(),
        0,
        "Expected sequence number 0 for validator set change event but got {}",
        validator_set_change_event.sequence_number()
    );
    // (4) It should emit the validator set we fed into the genesis tx
    assert_eq!(
        &ValidatorSet::from_bytes(validator_set_change_event.event_data()).unwrap(),
        validator_set,
        "Validator set in emitted event does not match validator set fed into genesis transaction"
    );

    // (5) The fourth event should be the discovery set change event
    let discovery_set_change_event = &events[3];
    assert_eq!(
        *discovery_set_change_event.key(),
        DiscoverySet::change_event_key(),
        "Key of emitted event {:?} does not match change event key {:?}",
        *discovery_set_change_event.key(),
        DiscoverySet::change_event_key()
    );
    // (6) This should be the first discovery set change event
    assert_eq!(
        discovery_set_change_event.sequence_number(),
        0,
        "Expected sequence number 0 for discovery set change event but got {}",
        discovery_set_change_event.sequence_number()
    );
    // (7) It should emit the discovery set we fed into the genesis tx
    assert_eq!(
        &DiscoverySet::from_bytes(discovery_set_change_event.event_data()).unwrap(),
        discovery_set,
        "Discovery set in emitted event does not match discovery set fed into genesis transaction",
    );
}

// `StateView` has no data given we are creating the genesis
struct GenesisStateView;

impl StateView for GenesisStateView {
    fn get(&self, _access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        true
    }
}
