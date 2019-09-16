// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use config::config::{VMConfig, VMPublishingOption};
use crypto::{ed25519::*, HashValue};
use failure::prelude::*;
use ir_to_bytecode::{compiler::compile_program, parser::ast};
use lazy_static::lazy_static;
use rand::{rngs::StdRng, SeedableRng};
use state_view::StateView;
use std::{collections::HashSet, iter::FromIterator, time::Duration};
use stdlib::{
    stdlib_modules,
    transaction_scripts::{
        CREATE_ACCOUNT_TXN_BODY, MINT_TXN_BODY, PEER_TO_PEER_TRANSFER_TXN_BODY,
        ROTATE_AUTHENTICATION_KEY_TXN_BODY,
    },
};
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    byte_array::ByteArray,
    identifier::Identifier,
    transaction::{
        RawTransaction, Script, SignatureCheckedTransaction, TransactionArgument,
        SCRIPT_HASH_LENGTH,
    },
    validator_public_keys::ValidatorPublicKeys,
};
use vm::{access::ModuleAccess, transaction_metadata::TransactionMetadata};
use vm_cache_map::Arena;
use vm_runtime::{
    code_cache::{
        module_adapter::FakeFetcher,
        module_cache::{BlockModuleCache, VMModuleCache},
    },
    data_cache::BlockDataCache,
    txn_executor::{
        TransactionExecutor, ACCOUNT_MODULE, BLOCK_MODULE, COIN_MODULE, VALIDATOR_SET_MODULE,
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
}

pub fn sign_genesis_transaction(raw_txn: RawTransaction) -> Result<SignatureCheckedTransaction> {
    let (private_key, public_key) = &*GENESIS_KEYPAIR;
    raw_txn.sign(private_key, public_key.clone())
}

// Identifiers for well-known functions.
lazy_static! {
    static ref ADD_VALIDATOR: Identifier = Identifier::new("add_validator").unwrap();
    static ref INITIALIZE: Identifier = Identifier::new("initialize").unwrap();
    static ref MINT_TO_ADDRESS: Identifier = Identifier::new("mint_to_address").unwrap();
    static ref REGISTER_CANDIDATE_VALIDATOR: Identifier =
        Identifier::new("register_candidate_validator").unwrap();
    static ref ROTATE_AUTHENTICATION_KEY: Identifier =
        { Identifier::new("rotate_authentication_key").unwrap() };
    static ref EPILOGUE: Identifier = Identifier::new("epilogue").unwrap();
}

#[derive(Debug)]
#[cfg_attr(any(test, feature = "testing"), derive(Clone))]
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

lazy_static! {
    static ref PEER_TO_PEER_TXN: Vec<u8> = { compile_script(&PEER_TO_PEER_TRANSFER_TXN_BODY) };
    static ref CREATE_ACCOUNT_TXN: Vec<u8> = { compile_script(&CREATE_ACCOUNT_TXN_BODY) };
    static ref ROTATE_AUTHENTICATION_KEY_TXN: Vec<u8> =
        { compile_script(&ROTATE_AUTHENTICATION_KEY_TXN_BODY) };
    static ref MINT_TXN: Vec<u8> = { compile_script(&MINT_TXN_BODY) };
    static ref GENESIS_ACCOUNT: Accounts = {
        let mut account = Accounts::empty();
        account.new_account();
        account
    };
}

fn compile_script(body: &ast::Program) -> Vec<u8> {
    let compiled_program =
        compile_program(AccountAddress::default(), body.clone(), stdlib_modules()).unwrap();
    let mut script_bytes = vec![];
    compiled_program
        .script
        .serialize(&mut script_bytes)
        .unwrap();
    script_bytes
}

/// Encode a program transferring `amount` coins from `sender` to `recipient`. Fails if there is no
/// account at the recipient address or if the sender's balance is lower than `amount`.
pub fn encode_transfer_script(recipient: &AccountAddress, amount: u64) -> Script {
    Script::new(
        PEER_TO_PEER_TXN.clone(),
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U64(amount),
        ],
    )
}

/// Encode a program creating a fresh account at `account_address` with `initial_balance` coins
/// transferred from the sender's account balance. Fails if there is already an account at
/// `account_address` or if the sender's balance is lower than `initial_balance`.
pub fn encode_create_account_script(
    account_address: &AccountAddress,
    initial_balance: u64,
) -> Script {
    Script::new(
        CREATE_ACCOUNT_TXN.clone(),
        vec![
            TransactionArgument::Address(*account_address),
            TransactionArgument::U64(initial_balance),
        ],
    )
}

/// Encode a program that rotates the sender's authentication key to `new_key`.
pub fn rotate_authentication_key_script(new_key: AccountAddress) -> Script {
    Script::new(
        ROTATE_AUTHENTICATION_KEY_TXN.clone(),
        vec![TransactionArgument::ByteArray(ByteArray::new(
            new_key.as_ref().to_vec(),
        ))],
    )
}

// TODO: this should go away once we are no longer using it in tests
/// Encode a program creating `amount` coins for sender
pub fn encode_mint_script(sender: &AccountAddress, amount: u64) -> Script {
    Script::new(
        MINT_TXN.clone(),
        vec![
            TransactionArgument::Address(*sender),
            TransactionArgument::U64(amount),
        ],
    )
}

/// Returns a user friendly mnemonic for the transaction type if the transaction is
/// for a known, white listed, transaction.
pub fn get_transaction_name(code: &[u8]) -> String {
    if code == &PEER_TO_PEER_TXN[..] {
        return "peer_to_peer_transaction".to_string();
    } else if code == &CREATE_ACCOUNT_TXN[..] {
        return "create_account_transaction".to_string();
    } else if code == &MINT_TXN[..] {
        return "mint_transaction".to_string();
    } else if code == &ROTATE_AUTHENTICATION_KEY_TXN[..] {
        return "rotate_authentication_key_transaction".to_string();
    }
    "<unknown transaction>".to_string()
}

pub fn allowing_script_hashes() -> Vec<[u8; SCRIPT_HASH_LENGTH]> {
    vec![
        MINT_TXN.clone(),
        PEER_TO_PEER_TXN.clone(),
        ROTATE_AUTHENTICATION_KEY_TXN.clone(),
        CREATE_ACCOUNT_TXN.clone(),
    ]
    .into_iter()
    .map(|s| *HashValue::from_sha3_256(&s).as_ref())
    .collect()
}

pub fn default_config() -> VMConfig {
    VMConfig {
        publishing_options: VMPublishingOption::Locked(HashSet::from_iter(
            allowing_script_hashes().into_iter(),
        )),
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
    encode_genesis_transaction_with_validator(private_key, public_key, vec![])
}

pub fn encode_genesis_transaction_with_validator(
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    _validator_set: Vec<ValidatorPublicKeys>,
) -> SignatureCheckedTransaction {
    const INIT_BALANCE: u64 = 1_000_000_000;

    // Compile the needed stdlib modules.
    let modules = stdlib_modules();
    let arena = Arena::new();
    let state_view = FakeStateView;
    let vm_cache = VMModuleCache::new(&arena);
    let genesis_addr = account_config::association_address();
    let genesis_auth_key = ByteArray::new(AccountAddress::from_public_key(&public_key).to_vec());

    let genesis_write_set = {
        let fake_fetcher = FakeFetcher::new(modules.iter().map(|m| m.as_inner().clone()).collect());
        let data_cache = BlockDataCache::new(&state_view);
        let block_cache = BlockModuleCache::new(&vm_cache, fake_fetcher);
        {
            let mut txn_data = TransactionMetadata::default();
            txn_data.sender = genesis_addr;

            let mut txn_executor = TransactionExecutor::new(&block_cache, &data_cache, txn_data);
            txn_executor.create_account(genesis_addr).unwrap();
            txn_executor
                .create_account(account_config::core_code_address())
                .unwrap();
            txn_executor
                .execute_function(&BLOCK_MODULE, &INITIALIZE, vec![])
                .unwrap();
            txn_executor
                .execute_function(&COIN_MODULE, &INITIALIZE, vec![])
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

            // Initialize the validator set.
            txn_executor
                .create_account(account_config::validator_set_address())
                .unwrap();
            txn_executor
                .execute_function_with_sender_FOR_GENESIS_ONLY(
                    account_config::validator_set_address(),
                    &VALIDATOR_SET_MODULE,
                    &INITIALIZE,
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

            txn_executor
                .make_write_set(stdlib_modules, Ok(()))
                .unwrap()
                .write_set()
                .clone()
                .into_mut()
        }
    };
    let transaction =
        RawTransaction::new_write_set(genesis_addr, 0, genesis_write_set.freeze().unwrap());
    transaction.sign(private_key, public_key).unwrap()
}
