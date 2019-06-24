// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use ::compiler::{compiler, parser::ast};
use config::config::{VMConfig, VMPublishingOption};
use crypto::{signing, PrivateKey, PublicKey};
use failure::prelude::*;
use lazy_static::lazy_static;
use rand::{rngs::StdRng, SeedableRng};
use state_view::StateView;
use std::{collections::HashSet, iter::FromIterator, time::Duration};
use stdlib::{
    stdlib::*,
    transaction_scripts::{
        CREATE_ACCOUNT_TXN_BODY, MINT_TXN_BODY, PEER_TO_PEER_TRANSFER_TXN_BODY,
        ROTATE_AUTHENTICATION_KEY_TXN_BODY,
    },
};
use tiny_keccak::Keccak;
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    byte_array::ByteArray,
    language_storage::ModuleId,
    transaction::{
        Program, RawTransaction, SignatureCheckedTransaction, TransactionArgument,
        SCRIPT_HASH_LENGTH,
    },
    validator_public_keys::ValidatorPublicKeys,
};
use vm::{file_format::CompiledModule, transaction_metadata::TransactionMetadata};
use vm_cache_map::Arena;
use vm_runtime::{
    code_cache::{
        module_adapter::FakeFetcher,
        module_cache::{BlockModuleCache, VMModuleCache},
    },
    data_cache::BlockDataCache,
    txn_executor::{TransactionExecutor, ACCOUNT_MODULE, COIN_MODULE},
    value::Local,
};

#[cfg(test)]
mod tests;

// The seed is arbitrarily picked to produce a consistent key. XXX make this more formal?
const GENESIS_SEED: [u8; 32] = [42; 32];
// Max size of the validator set
const VALIDATOR_SIZE_LIMIT: usize = 10;

lazy_static! {
    pub static ref GENESIS_KEYPAIR: (PrivateKey, PublicKey) = {
        let mut rng: StdRng = SeedableRng::from_seed(GENESIS_SEED);
        signing::generate_keypair_for_testing(&mut rng)
    };
}

pub fn sign_genesis_transaction(raw_txn: RawTransaction) -> Result<SignatureCheckedTransaction> {
    let (private_key, public_key) = &*GENESIS_KEYPAIR;
    raw_txn.sign(private_key, *public_key)
}

#[derive(Debug, Clone)]
pub struct Account {
    pub addr: AccountAddress,
    pub privkey: PrivateKey,
    pub pubkey: PublicKey,
}

impl Account {
    pub fn new(rng: &mut StdRng) -> Self {
        let (privkey, pubkey) = crypto::signing::generate_keypair_for_testing(rng);
        let addr = pubkey.into();
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

    pub fn get_account(&self, account: usize) -> Account {
        self.accounts[account].clone()
    }

    pub fn get_public_key(&self, account: usize) -> PublicKey {
        self.accounts[account].pubkey
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
        RawTransaction::new(
            sender,
            sequence_number,
            Program::new(program, vec![], args),
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
    pub static ref STDLIB_ADDRESS: AccountAddress = { account_config::core_code_address() };
    pub static ref STDLIB_MODULES: Vec<CompiledModule> = {
        let mut modules: Vec<CompiledModule> = vec![];
        let stdlib = vec![coin_module(), native_hash_module(), account_module(), signature_module(), validator_set_module()];
        for m in stdlib.iter() {
            let (compiled_module, verification_errors) =
                compiler::compile_and_verify_module(&STDLIB_ADDRESS, m, &modules).unwrap();

            // Fail if the module doesn't verify
            for e in &verification_errors {
                println!("{:?}", e);
            }
            assert!(verification_errors.is_empty());

            modules.push(compiled_module);
        }
        modules
    };
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
        compiler::compile_program(&AccountAddress::default(), body, &STDLIB_MODULES.clone())
            .unwrap();
    let mut script_bytes = vec![];
    compiled_program
        .script
        .serialize(&mut script_bytes)
        .unwrap();
    script_bytes
}

/// Encode a program transferring `amount` coins from `sender` to `recipient`. Fails if there is no
/// account at the recipient address or if the sender's balance is lower than `amount`.
pub fn encode_transfer_program(recipient: &AccountAddress, amount: u64) -> Program {
    Program::new(
        PEER_TO_PEER_TXN.clone(),
        vec![],
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U64(amount),
        ],
    )
}

/// Encode a program creating a fresh account at `account_address` with `initial_balance` coins
/// transferred from the sender's account balance. Fails if there is already an account at
/// `account_address` or if the sender's balance is lower than `initial_balance`.
pub fn encode_create_account_program(
    account_address: &AccountAddress,
    initial_balance: u64,
) -> Program {
    Program::new(
        CREATE_ACCOUNT_TXN.clone(),
        vec![],
        vec![
            TransactionArgument::Address(*account_address),
            TransactionArgument::U64(initial_balance),
        ],
    )
}

/// Encode a program that rotates the sender's authentication key to `new_key`.
pub fn rotate_authentication_key_program(new_key: AccountAddress) -> Program {
    Program::new(
        ROTATE_AUTHENTICATION_KEY_TXN.clone(),
        vec![],
        vec![TransactionArgument::ByteArray(ByteArray::new(
            new_key.as_ref().to_vec(),
        ))],
    )
}

// TODO: this should go away once we are no longer using it in tests
/// Encode a program creating `amount` coins for sender
pub fn encode_mint_program(sender: &AccountAddress, amount: u64) -> Program {
    Program::new(
        MINT_TXN.clone(),
        vec![],
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
    .map(|s| {
        let mut hash = [0u8; SCRIPT_HASH_LENGTH];
        let mut keccak = Keccak::new_sha3_256();

        keccak.update(&s);
        keccak.finalize(&mut hash);
        hash
    })
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
    private_key: &PrivateKey,
    public_key: PublicKey,
) -> SignatureCheckedTransaction {
    encode_genesis_transaction_with_validator(private_key, public_key, vec![])
}

pub fn encode_genesis_transaction_with_validator(
    private_key: &PrivateKey,
    public_key: PublicKey,
    validator_set: Vec<ValidatorPublicKeys>,
) -> SignatureCheckedTransaction {
    assert!(validator_set.len() <= VALIDATOR_SIZE_LIMIT);
    const INIT_BALANCE: u64 = 1_000_000_000;

    // Compile the needed stdlib modules.
    let modules = STDLIB_MODULES.clone();
    let arena = Arena::new();
    let state_view = FakeStateView;
    let vm_cache = VMModuleCache::new(&arena);
    let genesis_addr = account_config::association_address();
    let genesis_auth_key = ByteArray::new(AccountAddress::from(public_key).to_vec());

    let genesis_write_set = {
        let fake_fetcher = FakeFetcher::new(modules.clone());
        let data_cache = BlockDataCache::new(&state_view);
        let block_cache = BlockModuleCache::new(&vm_cache, fake_fetcher);
        {
            let mut txn_data = TransactionMetadata::default();
            txn_data.sender = genesis_addr;
            let validator_set_key = ModuleId::new(
                account_config::core_code_address(),
                "ValidatorSet".to_string(),
            );

            let mut txn_executor = TransactionExecutor::new(&block_cache, &data_cache, txn_data);
            txn_executor.create_account(genesis_addr).unwrap().unwrap();
            txn_executor
                .execute_function(&COIN_MODULE, "grant_mint_capability", vec![])
                .unwrap()
                .unwrap();

            txn_executor
                .execute_function(
                    &ACCOUNT_MODULE,
                    "mint_to_address",
                    vec![Local::address(genesis_addr), Local::u64(INIT_BALANCE)],
                )
                .unwrap()
                .unwrap();

            txn_executor
                .execute_function(
                    &ACCOUNT_MODULE,
                    "rotate_authentication_key",
                    vec![Local::bytearray(genesis_auth_key)],
                )
                .unwrap()
                .unwrap();

            let mut validator_args = vec![Local::u64(validator_set.len() as u64)];
            for key in validator_set.iter() {
                txn_executor
                    .execute_function(
                        &validator_set_key,
                        "make_new_validator_key",
                        vec![
                            Local::address(*key.account_address()),
                            Local::bytearray(ByteArray::new(
                                key.consensus_public_key().to_slice().to_vec(),
                            )),
                            Local::bytearray(ByteArray::new(
                                key.network_signing_public_key().to_slice().to_vec(),
                            )),
                            Local::bytearray(ByteArray::new(
                                key.network_identity_public_key().to_slice().to_vec(),
                            )),
                        ],
                    )
                    .unwrap()
                    .unwrap();
                validator_args.push(txn_executor.pop_stack().unwrap());
            }
            let placeholder = {
                txn_executor
                    .execute_function(
                        &validator_set_key,
                        "make_new_validator_key",
                        vec![
                            Local::address(AccountAddress::default()),
                            Local::bytearray(ByteArray::new(vec![])),
                            Local::bytearray(ByteArray::new(vec![])),
                            Local::bytearray(ByteArray::new(vec![])),
                        ],
                    )
                    .unwrap()
                    .unwrap();
                txn_executor.pop_stack().unwrap()
            };
            validator_args.resize(VALIDATOR_SIZE_LIMIT + 1, placeholder);

            txn_executor
                .execute_function(&validator_set_key, "publish_validator_set", validator_args)
                .unwrap()
                .unwrap();

            let stdlib_modules = modules
                .into_iter()
                .map(|m| {
                    let mut module_vec = vec![];
                    m.serialize(&mut module_vec).unwrap();
                    (m.self_id(), module_vec)
                })
                .collect();

            txn_executor
                .make_write_set(stdlib_modules, Ok(Ok(())))
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
