// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod genesis_gas_schedule;

use crate::genesis_gas_schedule::initial_gas_schedule;
use failure::prelude::*;
use lazy_static::lazy_static;
use libra_crypto::{ed25519::*, traits::ValidKey};
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    byte_array::ByteArray,
    identifier::Identifier,
    transaction::{
        ChangeSet, RawTransaction, Script, SignatureCheckedTransaction, TransactionArgument,
    },
    validator_set::ValidatorSet,
};
use rand::{rngs::StdRng, SeedableRng};
use std::time::Duration;
use stdlib::stdlib_modules;
use vm::{
    access::ModuleAccess, gas_schedule::CostTable, transaction_metadata::TransactionMetadata,
};
use vm_cache_map::Arena;
use vm_runtime::{
    code_cache::{
        module_adapter::FakeFetcher,
        module_cache::{BlockModuleCache, VMModuleCache},
    },
    data_cache::BlockDataCache,
    txn_executor::{
        TransactionExecutor, ACCOUNT_MODULE, COIN_MODULE, GAS_SCHEDULE_MODULE, LIBRA_SYSTEM_MODULE,
        VALIDATOR_CONFIG_MODULE,
    },
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
    static ref INITIALIZE_VALIDATOR: Identifier =
        Identifier::new("initialize_validator_set").unwrap();
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

pub fn encode_genesis_transaction(
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
) -> SignatureCheckedTransaction {
    encode_genesis_transaction_with_validator(private_key, public_key, ValidatorSet::new(vec![]))
}

pub fn encode_genesis_transaction_with_validator(
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    validator_set: ValidatorSet,
) -> SignatureCheckedTransaction {
    const INIT_BALANCE: u64 = 1_000_000_000;

    // Compile the needed stdlib modules.
    let modules = stdlib_modules();
    let arena = Arena::new();
    let state_view = FakeStateView;
    let vm_cache = VMModuleCache::new(&arena);
    let genesis_addr = account_config::association_address();
    let genesis_auth_key = ByteArray::new(AccountAddress::from_public_key(&public_key).to_vec());
    let gas_schedule = CostTable::zero();

    let genesis_write_set = {
        let fake_fetcher = FakeFetcher::new(modules.iter().map(|m| m.as_inner().clone()).collect());
        let data_cache = BlockDataCache::new(&state_view);
        let block_cache = BlockModuleCache::new(&vm_cache, fake_fetcher);
        let initial_gas_schedule = initial_gas_schedule(&block_cache, &data_cache);
        {
            let mut txn_data = TransactionMetadata::default();
            txn_data.sender = genesis_addr;

            let mut txn_executor =
                TransactionExecutor::new(&block_cache, &gas_schedule, &data_cache, txn_data);
            txn_executor.create_account(genesis_addr).unwrap();
            txn_executor
                .create_account(account_config::transaction_fee_address())
                .unwrap();
            txn_executor
                .create_account(account_config::core_code_address())
                .unwrap();
            txn_executor
                .execute_function(&COIN_MODULE, &INITIALIZE, vec![])
                .unwrap();
            txn_executor
                .execute_function(&LIBRA_SYSTEM_MODULE, &INITIALIZE_BLOCK, vec![])
                .unwrap();
            txn_executor
                .execute_function_with_sender_FOR_GENESIS_ONLY(
                    account_config::association_address(),
                    &GAS_SCHEDULE_MODULE,
                    &INITIALIZE,
                    vec![initial_gas_schedule],
                )
                .unwrap();

            txn_executor
                .execute_function(
                    &ACCOUNT_MODULE,
                    &MINT_TO_ADDRESS,
                    vec![Value::address(genesis_addr), Value::u64(INIT_BALANCE)],
                )
                .unwrap();

            txn_executor
                .execute_function(
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
                .execute_function(&ACCOUNT_MODULE, &EPILOGUE, vec![])
                .unwrap();

            // Create the transaction fees resource under the fees account
            txn_executor
                .execute_function_with_sender_FOR_GENESIS_ONLY(
                    account_config::transaction_fee_address(),
                    &LIBRA_SYSTEM_MODULE,
                    &INITIALIZE_TXN_FEES,
                    vec![],
                )
                .unwrap();

            // Initialize the validator set.
            txn_executor
                .create_account(account_config::validator_set_address())
                .unwrap();
            txn_executor
                .execute_function_with_sender_FOR_GENESIS_ONLY(
                    account_config::validator_set_address(),
                    &LIBRA_SYSTEM_MODULE,
                    &INITIALIZE_VALIDATOR,
                    vec![],
                )
                .unwrap();
            for validator_keys in validator_set.payload().iter().rev() {
                // First, add a ValidatorConfig resource under each account
                let validator_address = *validator_keys.account_address();
                txn_executor.create_account(validator_address).unwrap();
                txn_executor
                    .execute_function_with_sender_FOR_GENESIS_ONLY(
                        validator_address,
                        &VALIDATOR_CONFIG_MODULE,
                        &REGISTER_CANDIDATE_VALIDATOR,
                        vec![
                            Value::byte_array(ByteArray::new(
                                validator_keys
                                    .network_signing_public_key()
                                    .to_bytes()
                                    .to_vec(),
                            )),
                            Value::byte_array(ByteArray::new(
                                validator_keys
                                    .network_identity_public_key()
                                    .to_bytes()
                                    .to_vec(),
                            )),
                            Value::byte_array(ByteArray::new(
                                validator_keys.consensus_public_key().to_bytes().to_vec(),
                            )),
                        ],
                    )
                    .unwrap();
                // Then, add the account to the validator set
                txn_executor
                    .execute_function(
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
            // (1) The genesis tx should emit 3 events: a pair of payment sent/received events for
            // minting to the genesis address, and a ValidatorSet.ChangeEvent
            assert_eq!(
                txn_output.events().len(),
                3,
                "Genesis transaction should emit three events, but found {} events: {:?}",
                txn_output.events().len(),
                txn_output.events()
            );
            // (2) The last event should be the validator set change event
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

            ChangeSet::new(txn_output.write_set().clone(), txn_output.events().to_vec())
        }
    };
    let transaction = RawTransaction::new_change_set(genesis_addr, 0, genesis_write_set);
    transaction.sign(private_key, public_key).unwrap()
}
