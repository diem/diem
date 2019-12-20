// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod genesis_gas_schedule;

use crate::genesis_gas_schedule::initial_gas_schedule;
use anyhow::Result;
use lazy_static::lazy_static;
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
    byte_array::ByteArray,
    crypto_proxies::ValidatorSet,
    discovery_info::DiscoveryInfo,
    discovery_set::DiscoverySet,
    identifier::Identifier,
    transaction::{
        ChangeSet, RawTransaction, Script, SignatureCheckedTransaction, TransactionArgument,
    },
};
use parity_multiaddr::Multiaddr;
use rand::{rngs::StdRng, SeedableRng};
use std::{str::FromStr, time::Duration};
use stdlib::stdlib_modules;
use vm::{
    access::ModuleAccess, gas_schedule::CostTable, transaction_metadata::TransactionMetadata,
};
use vm_cache_map::Arena;
use vm_runtime::{
    data_cache::BlockDataCache, runtime::VMRuntime, system_module_names::*,
    txn_executor::TransactionExecutor,
};
use vm_runtime_types::value::Value;

// The seed is arbitrarily picked to produce a consistent key. XXX make this more formal?
const GENESIS_SEED: [u8; 32] = [42; 32];

lazy_static! {
    pub static ref GENESIS_KEYPAIR: (Ed25519PrivateKey, Ed25519PublicKey) = {
        let mut rng = StdRng::from_seed(GENESIS_SEED);
        compat::generate_keypair(&mut rng)
    };
    static ref GENESIS_ACCOUNT: Accounts = {
        let mut account = Accounts::empty();
        account.new_account();
        account
    };
    // TODO(philiphayes): remove this when we add discovery set to genesis config.
    static ref PLACEHOLDER_PUBKEY: X25519StaticPublicKey = {
        let salt = None;
        let seed = [69u8; 32];
        let app_info = None;
        let (_, pubkey) = X25519StaticPrivateKey::derive_keypair_from_seed(salt, &seed, app_info);
        pubkey
    };
}

pub fn sign_genesis_transaction(raw_txn: RawTransaction) -> Result<SignatureCheckedTransaction> {
    let (private_key, public_key) = &*GENESIS_KEYPAIR;
    raw_txn.sign(private_key, public_key.clone())
}

// Identifiers for well-known functions.
lazy_static! {
    static ref ADD_VALIDATOR: Identifier = Identifier::new("add_validator").unwrap();
    static ref INITIALIZE: Identifier = Identifier::new("initialize").unwrap();
    static ref INITIALIZE_BLOCK: Identifier = Identifier::new("initialize_block_metadata").unwrap();
    static ref INITIALIZE_TXN_FEES: Identifier =
        Identifier::new("initialize_transaction_fees").unwrap();
    static ref INITIALIZE_VALIDATOR_SET: Identifier =
        Identifier::new("initialize_validator_set").unwrap();
    static ref INITIALIZE_DISCOVERY_SET: Identifier =
        Identifier::new("initialize_discovery_set").unwrap();
    static ref MINT_TO_ADDRESS: Identifier = Identifier::new("mint_to_address").unwrap();
    static ref RECONFIGURE: Identifier = Identifier::new("reconfigure").unwrap();
    static ref REGISTER_CANDIDATE_VALIDATOR: Identifier =
        Identifier::new("register_candidate_validator").unwrap();
    static ref ROTATE_AUTHENTICATION_KEY: Identifier =
        { Identifier::new("rotate_authentication_key").unwrap() };
    static ref EPILOGUE: Identifier = Identifier::new("epilogue").unwrap();
}

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct Account {
    pub addr: AccountAddress,
    pub privkey: Ed25519PrivateKey,
    pub pubkey: Ed25519PublicKey,
}

impl Account {
    pub fn new(rng: &mut StdRng) -> Self {
        let (privkey, pubkey) = compat::generate_keypair(&mut *rng);
        let addr = AccountAddress::from_public_key(&pubkey);
        Account {
            addr,
            privkey,
            pubkey,
        }
    }
}

pub struct Accounts {
    accounts: Vec<Account>,
    pub randomness_source: StdRng,
}

impl Default for Accounts {
    fn default() -> Self {
        let mut accounts = Accounts::empty();
        for _i in 0..Self::NUMBER_OF_ACCOUNTS {
            accounts.new_account();
        }
        accounts
    }
}

impl Accounts {
    const NUMBER_OF_ACCOUNTS: i64 = 10;

    pub fn empty() -> Self {
        let mut seed = [0u8; 32];
        seed[..4].copy_from_slice(&[1, 2, 3, 4]);
        let rng: StdRng = StdRng::from_seed(seed);
        Accounts {
            accounts: vec![],
            randomness_source: rng,
        }
    }

    pub fn fresh_account(&mut self) -> Account {
        Account::new(&mut self.randomness_source)
    }

    pub fn new_account(&mut self) -> usize {
        self.accounts
            .push(Account::new(&mut self.randomness_source));
        self.accounts.len() - 1
    }

    pub fn get_address(&self, account: usize) -> AccountAddress {
        self.accounts[account].addr
    }

    pub fn get_account(&self, account: usize) -> &Account {
        &self.accounts[account]
    }

    pub fn get_public_key(&self, account: usize) -> Ed25519PublicKey {
        self.accounts[account].pubkey.clone()
    }

    pub fn create_txn_with_args(
        &self,
        program: Vec<u8>,
        args: Vec<TransactionArgument>,
        sender: AccountAddress,
        sender_account: Account,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
    ) -> SignatureCheckedTransaction {
        RawTransaction::new_script(
            sender,
            sequence_number,
            Script::new(program, args),
            max_gas_amount,
            gas_unit_price,
            Duration::from_secs(u64::max_value()),
        )
        .sign(&sender_account.privkey, sender_account.pubkey)
        .unwrap()
    }

    pub fn get_addresses(&self) -> Vec<AccountAddress> {
        self.accounts.iter().map(|account| account.addr).collect()
    }

    pub fn accounts(&self) -> &[Account] {
        &self.accounts
    }
}

struct FakeStateView;

impl StateView for FakeStateView {
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

// TODO(philiphayes): remove this after integrating on-chain discovery with config.
/// Make a placeholder `DiscoverySet` from the `ValidatorSet`.
pub fn make_placeholder_discovery_set(validator_set: &ValidatorSet) -> DiscoverySet {
    let discovery_set = validator_set
        .iter()
        .map(|validator_pubkeys| {
            DiscoveryInfo::new(
                *validator_pubkeys.account_address(),
                // validator_network_identity_pubkey
                validator_pubkeys.network_identity_public_key().clone(),
                // validator_network_address PLACEHOLDER
                Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234").unwrap(),
                // fullnodes_network_identity_pubkey PLACEHOLDER
                PLACEHOLDER_PUBKEY.clone(),
                // fullnodes_network_address PLACEHOLDER
                Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234").unwrap(),
            )
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
    const INIT_BALANCE: u64 = 1_000_000_000;

    // Compile the needed stdlib modules.
    let modules = stdlib_modules();

    let arena = Arena::new();
    let mut runtime = VMRuntime::new(&arena);
    for module in modules {
        runtime.cache_module(module.clone());
    }

    let state_view = FakeStateView;
    let genesis_addr = account_config::association_address();
    let genesis_auth_key = ByteArray::new(AccountAddress::from_public_key(&public_key).to_vec());
    let gas_schedule = CostTable::zero();
    let data_cache = BlockDataCache::new(&state_view);
    let initial_gas_schedule = initial_gas_schedule(&runtime, &data_cache);

    let genesis_write_set = {
        {
            let mut txn_data = TransactionMetadata::default();
            txn_data.sender = genesis_addr;

            let mut txn_executor = TransactionExecutor::new(&gas_schedule, &data_cache, txn_data);
            txn_executor
                .create_account(&runtime, &state_view, genesis_addr)
                .unwrap();
            txn_executor
                .create_account(
                    &runtime,
                    &state_view,
                    account_config::transaction_fee_address(),
                )
                .unwrap();
            txn_executor
                .create_account(&runtime, &state_view, account_config::core_code_address())
                .unwrap();
            txn_executor
                .execute_function(&runtime, &state_view, &COIN_MODULE, &INITIALIZE, vec![])
                .unwrap();
            txn_executor
                .execute_function(
                    &runtime,
                    &state_view,
                    &LIBRA_SYSTEM_MODULE,
                    &INITIALIZE_BLOCK,
                    vec![],
                )
                .unwrap();
            txn_executor
                .execute_function_with_sender_FOR_GENESIS_ONLY(
                    &runtime,
                    &state_view,
                    account_config::association_address(),
                    &GAS_SCHEDULE_MODULE,
                    &INITIALIZE,
                    vec![initial_gas_schedule],
                )
                .unwrap();

            txn_executor
                .execute_function(
                    &runtime,
                    &state_view,
                    &ACCOUNT_MODULE,
                    &MINT_TO_ADDRESS,
                    vec![Value::address(genesis_addr), Value::u64(INIT_BALANCE)],
                )
                .unwrap();

            txn_executor
                .execute_function(
                    &runtime,
                    &state_view,
                    &ACCOUNT_MODULE,
                    &ROTATE_AUTHENTICATION_KEY,
                    vec![Value::byte_array(genesis_auth_key)],
                )
                .unwrap();

            // Bump the sequence number for the Association account. If we don't do this and a
            // subsequent transaction (e.g., minting) is sent from the Assocation account, a problem
            // arises: both the genesis transaction and the subsequent transaction have sequence
            // number 0
            txn_executor
                .execute_function(
                    &runtime,
                    &state_view,
                    &ACCOUNT_MODULE,
                    &EPILOGUE,
                    vec![
                        Value::u64(/* txn_sequence_number */ 0),
                        Value::u64(/* txn_gas_price */ 0),
                        Value::u64(/* txn_max_gas_units */ 0),
                        Value::u64(/* gas_units_remaining */ 0),
                    ],
                )
                .unwrap();

            // Create the transaction fees resource under the fees account
            txn_executor
                .execute_function_with_sender_FOR_GENESIS_ONLY(
                    &runtime,
                    &state_view,
                    account_config::transaction_fee_address(),
                    &LIBRA_SYSTEM_MODULE,
                    &INITIALIZE_TXN_FEES,
                    vec![],
                )
                .unwrap();

            // Initialize the validator set.
            txn_executor
                .create_account(
                    &runtime,
                    &state_view,
                    account_config::validator_set_address(),
                )
                .unwrap();
            txn_executor
                .execute_function_with_sender_FOR_GENESIS_ONLY(
                    &runtime,
                    &state_view,
                    account_config::validator_set_address(),
                    &LIBRA_SYSTEM_MODULE,
                    &INITIALIZE_VALIDATOR_SET,
                    vec![],
                )
                .unwrap();

            // Initialize the discovery set.
            txn_executor
                .create_account(
                    &runtime,
                    &state_view,
                    account_config::discovery_set_address(),
                )
                .unwrap();
            txn_executor
                .execute_function_with_sender_FOR_GENESIS_ONLY(
                    &runtime,
                    &state_view,
                    account_config::discovery_set_address(),
                    &LIBRA_SYSTEM_MODULE,
                    &INITIALIZE_DISCOVERY_SET,
                    vec![],
                )
                .unwrap();

            // Initialize each validator.
            for (validator_keys, discovery_info) in
                validator_set.iter().zip(discovery_set.iter()).rev()
            {
                // First, add a ValidatorConfig resource under each account
                let validator_address = *validator_keys.account_address();
                txn_executor
                    .create_account(&runtime, &state_view, validator_address)
                    .unwrap();
                txn_executor
                    .execute_function_with_sender_FOR_GENESIS_ONLY(
                        &runtime,
                        &state_view,
                        validator_address,
                        &VALIDATOR_CONFIG_MODULE,
                        &REGISTER_CANDIDATE_VALIDATOR,
                        vec![
                            // consensus_pubkey
                            Value::byte_array(ByteArray::new(
                                validator_keys.consensus_public_key().to_bytes().to_vec(),
                            )),
                            // network_signing_pubkey
                            Value::byte_array(ByteArray::new(
                                validator_keys
                                    .network_signing_public_key()
                                    .to_bytes()
                                    .to_vec(),
                            )),
                            // validator_network_identity_pubkey
                            Value::byte_array(ByteArray::new(
                                discovery_info
                                    .validator_network_identity_pubkey()
                                    .to_bytes()
                                    .to_vec(),
                            )),
                            // validator_network_address placeholder
                            Value::byte_array(ByteArray::new(
                                discovery_info.validator_network_address().to_vec(),
                            )),
                            // fullnodes_network_identity_pubkey placeholder
                            Value::byte_array(ByteArray::new(
                                discovery_info
                                    .fullnodes_network_identity_pubkey()
                                    .to_bytes()
                                    .to_vec(),
                            )),
                            // fullnodes_network_address placeholder
                            Value::byte_array(ByteArray::new(
                                discovery_info.fullnodes_network_address().to_vec(),
                            )),
                        ],
                    )
                    .unwrap();
                // Then, add the account to the validator set
                txn_executor
                    .execute_function(
                        &runtime,
                        &state_view,
                        &LIBRA_SYSTEM_MODULE,
                        &ADD_VALIDATOR,
                        vec![Value::address(validator_address)],
                    )
                    .unwrap()
            }
            // Finally, trigger a reconfiguration. This emits an event that will be passed along
            // to the storage layer.
            // TODO: Direct write set transactions cannot specify emitted events, so this currently
            // will not work.
            txn_executor
                .execute_function_with_sender_FOR_GENESIS_ONLY(
                    &runtime,
                    &state_view,
                    account_config::validator_set_address(),
                    &LIBRA_SYSTEM_MODULE,
                    &RECONFIGURE,
                    vec![],
                )
                .unwrap();

            let stdlib_modules = modules
                .iter()
                .map(|m| {
                    let mut module_vec = vec![];
                    m.serialize(&mut module_vec).unwrap();
                    (m.self_id(), module_vec)
                })
                .collect();

            let txn_output = txn_executor.make_write_set(stdlib_modules, Ok(())).unwrap();

            // Sanity checks on emitted events:
            // (1) The genesis tx should emit 4 events: a pair of payment sent/received events for
            // minting to the genesis address, a ValidatorSetChangeEvent, and a
            // DiscoverySetChangeEvent.
            assert_eq!(
                txn_output.events().len(),
                4,
                "Genesis transaction should emit four events, but found {} events: {:?}",
                txn_output.events().len(),
                txn_output.events()
            );

            // (2) The third event should be the validator set change event
            let validator_set_change_event = &txn_output.events()[2];
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
                ValidatorSet::from_bytes(validator_set_change_event.event_data()).unwrap(),
                validator_set,
                "Validator set in emitted event does not match validator set fed into genesis transaction"
            );

            // (5) The fourth event should be the discovery set change event
            let discovery_set_change_event = &txn_output.events()[3];
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
                DiscoverySet::from_bytes(discovery_set_change_event.event_data()).unwrap(),
                discovery_set,
                "Discovery set in emitted event does not match discovery set fed into genesis transaction",
            );

            ChangeSet::new(txn_output.write_set().clone(), txn_output.events().to_vec())
        }
    };
    let transaction = RawTransaction::new_change_set(genesis_addr, 0, genesis_write_set);
    transaction.sign(private_key, public_key).unwrap()
}
