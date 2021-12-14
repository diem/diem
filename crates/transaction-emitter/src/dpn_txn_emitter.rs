// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    execute_and_wait_transactions, gen_random_accounts, query_sequence_numbers,
    retrieve_account_balance, EmitJobRequest, Mode, TxnEmitter, MAX_TXNS, MAX_TXN_BATCH_SIZE,
    SEND_AMOUNT,
};
use anyhow::{format_err, Context, Result};
use async_trait::async_trait;
use diem_crypto::ed25519::Ed25519PrivateKey;
use diem_logger::*;
use diem_rest_client::Client as RestClient;
use diem_sdk::{
    move_types::account_address::AccountAddress,
    transaction_builder::{Currency, TransactionFactory},
    types::{
        account_config::XUS_NAME,
        transaction::{authenticator::AuthenticationKey, SignedTransaction},
        AccountKey, LocalAccount,
    },
};
use futures::future::try_join_all;
use rand::{rngs::StdRng, seq::SliceRandom, Rng};
use rand_core::SeedableRng;
use std::{cmp::min, time::Duration};

const MAX_CHILD_VASP_NUM: usize = 65536;
const MAX_VASP_ACCOUNT_NUM: usize = 16;

/// Create `num_new_accounts` by transferring diem from `source_account`. Return Vec of created
/// accounts
async fn create_new_accounts<R>(
    mut source_account: LocalAccount,
    num_new_accounts: usize,
    diem_per_new_account: u64,
    max_num_accounts_per_batch: u64,
    client: RestClient,
    txn_factory: &TransactionFactory,
    reuse_account: bool,
    mut rng: R,
) -> Result<Vec<LocalAccount>>
where
    R: ::rand_core::RngCore + ::rand_core::CryptoRng,
{
    let mut i = 0;
    let mut accounts = vec![];
    while i < num_new_accounts {
        let batch_size = min(
            max_num_accounts_per_batch as usize,
            min(MAX_TXN_BATCH_SIZE, num_new_accounts - i),
        );
        let mut batch = if reuse_account {
            println!("loading {} accounts if they exist", batch_size);
            gen_reusable_accounts(&client, batch_size, &mut rng).await?
        } else {
            let batch = gen_random_accounts(batch_size, &mut rng);
            let requests = batch
                .as_slice()
                .iter()
                .map(|account| {
                    source_account.sign_with_transaction_builder(
                        txn_factory.create_child_vasp_account(
                            Currency::XUS,
                            account.authentication_key(),
                            false,
                            diem_per_new_account,
                        ),
                    )
                })
                .collect();
            execute_and_wait_transactions(&client, &mut source_account, requests).await?;
            batch
        };

        i += batch.len();
        accounts.append(&mut batch);
    }
    Ok(accounts)
}

/// Mint `diem_per_new_account` from `minting_account` to each account in `accounts`.
async fn mint_to_new_accounts<R>(
    minting_account: &mut LocalAccount,
    accounts: &[LocalAccount],
    diem_per_new_account: u64,
    max_num_accounts_per_batch: u64,
    client: RestClient,
    txn_factory: &TransactionFactory,
    mut rng: R,
) -> Result<()>
where
    R: ::rand_core::RngCore + ::rand_core::CryptoRng,
{
    let mut left = accounts;
    let mut i = 0;
    let num_accounts = accounts.len();
    while !left.is_empty() {
        let batch_size = rng.gen::<usize>()
            % min(
                max_num_accounts_per_batch as usize,
                min(MAX_TXN_BATCH_SIZE, num_accounts - i),
            );
        let (to_batch, rest) = left.split_at(batch_size + 1);
        let mint_requests = to_batch
            .iter()
            .map(|account| {
                gen_transfer_txn_request(
                    minting_account,
                    &account.address(),
                    diem_per_new_account,
                    txn_factory,
                    0,
                )
            })
            .collect();
        execute_and_wait_transactions(&client, minting_account, mint_requests).await?;
        i += to_batch.len();
        left = rest;
    }
    Ok(())
}

fn create_parent_vasp_request(
    creation_account: &mut LocalAccount,
    account_auth_key: AuthenticationKey,
    txn_factory: &TransactionFactory,
) -> SignedTransaction {
    creation_account.sign_with_transaction_builder(txn_factory.create_parent_vasp_account(
        Currency::XUS,
        0,
        account_auth_key,
        "",
        false,
    ))
}

pub(crate) fn gen_transfer_txn_request(
    sender: &mut LocalAccount,
    receiver: &AccountAddress,
    num_coins: u64,
    txn_factory: &TransactionFactory,
    gas_price: u64,
) -> SignedTransaction {
    sender.sign_with_transaction_builder(
        txn_factory
            .peer_to_peer(Currency::XUS, *receiver, num_coins)
            .gas_unit_price(gas_price),
    )
}

#[async_trait]
impl TxnEmitter for DPNTxnEmitter {
    fn accounts(&mut self) -> &mut Vec<LocalAccount> {
        &mut self.accounts
    }

    fn transaction_factory(&self) -> &TransactionFactory {
        &self.txn_factory
    }

    fn client(&self) -> &RestClient {
        &self.client
    }

    fn rng(&mut self) -> &mut ::rand::rngs::StdRng {
        &mut self.rng
    }

    /// workflow of mint accounts:
    /// 1. mint faucet account as the money source
    /// 2. load tc account to create seed accounts(parent VASP), one seed account for each endpoint
    /// 3. mint coins from faucet to new created seed accounts
    /// 4. split number of requested accounts(child VASP) into equally size of groups
    /// 5. each seed account take responsibility to create one size of group requested accounts and mint coins to them
    /// example:
    /// requested totally 100 new accounts with 10 endpoints
    /// will create 10 seed accounts(parent VASP), each seed account create 10 new accounts
    async fn mint_accounts(
        &mut self,
        req: &EmitJobRequest,
        total_requested_accounts: usize,
    ) -> Result<()> {
        if self.accounts.len() >= total_requested_accounts {
            info!("Already have enough accounts exist, do not need to mint more");
            return Ok(());
        }
        let expected_num_seed_accounts =
            if total_requested_accounts / req.rest_clients.len() > MAX_CHILD_VASP_NUM {
                total_requested_accounts / MAX_CHILD_VASP_NUM + 1
            } else {
                req.rest_clients.len()
            };
        let num_accounts = total_requested_accounts - self.accounts.len(); // Only minting extra accounts
        let coins_per_account = SEND_AMOUNT * MAX_TXNS * 10; // extra coins for secure to pay none zero gas price
        let coins_total = coins_per_account * num_accounts as u64;
        let txn_factory = self.txn_factory.clone();
        let client = self.pick_mint_client(&req.rest_clients);

        // Create seed accounts with which we can create actual accounts concurrently
        let vasp = match &req.mode {
            Mode::NFT => unreachable!(),
            Mode::DPN { vasp, .. } => *vasp,
        };
        let seed_accounts = self
            .get_seed_accounts(&req.rest_clients, expected_num_seed_accounts, vasp)
            .await?;
        let rng = self.from_rng();
        let faucet_account = self.get_money_source(coins_total).await?;
        let actual_num_seed_accounts = seed_accounts.len();
        let num_new_child_accounts =
            (num_accounts + actual_num_seed_accounts - 1) / actual_num_seed_accounts;
        let coins_per_seed_account = coins_per_account * num_new_child_accounts as u64;
        mint_to_new_accounts(
            faucet_account,
            &seed_accounts,
            coins_per_seed_account as u64,
            100,
            client.clone(),
            &txn_factory,
            rng,
        )
        .await
        .map_err(|e| format_err!("Failed to mint seed_accounts: {}", e))?;
        println!("Completed minting seed accounts");
        println!("Minting additional {} accounts", num_accounts);
        tokio::time::sleep(Duration::from_secs(10)).await;

        let seed_rngs = gen_rng_for_reusable_account(actual_num_seed_accounts);
        // For each seed account, create a future and transfer diem from that seed account to new accounts
        let account_futures = seed_accounts
            .into_iter()
            .enumerate()
            .map(|(i, seed_account)| {
                // Spawn new threads
                let index = i % req.rest_clients.len();
                let cur_client = req.rest_clients[index].clone();
                create_new_accounts(
                    seed_account,
                    num_new_child_accounts,
                    coins_per_account,
                    20,
                    cur_client,
                    &txn_factory,
                    vasp,
                    if vasp {
                        seed_rngs[i].clone()
                    } else {
                        self.from_rng()
                    },
                )
            });
        let mut minted_accounts = try_join_all(account_futures)
            .await
            .context("Failed to mint accounts")?
            .into_iter()
            .flatten()
            .collect();

        self.accounts.append(&mut minted_accounts);
        assert!(
            self.accounts.len() >= num_accounts,
            "Something wrong in mint_account, wanted to mint {}, only have {}",
            total_requested_accounts,
            self.accounts.len()
        );
        println!("Mint is done");
        tokio::time::sleep(Duration::from_secs(60)).await;
        Ok(())
    }
}

#[derive(Debug)]
pub struct DPNTxnEmitter {
    accounts: Vec<LocalAccount>,
    txn_factory: TransactionFactory,
    treasury_compliance_account: LocalAccount,
    designated_dealer_account: LocalAccount,
    client: RestClient,
    rng: ::rand::rngs::StdRng,
}

impl DPNTxnEmitter {
    pub fn new(
        treasury_compliance_account: LocalAccount,
        designated_dealer_account: LocalAccount,
        client: RestClient,
        transaction_factory: TransactionFactory,
        rng: ::rand::rngs::StdRng,
    ) -> Self {
        Self {
            accounts: vec![],
            txn_factory: transaction_factory,
            treasury_compliance_account,
            designated_dealer_account,
            client,
            rng,
        }
    }

    async fn get_money_source(&mut self, coins_total: u64) -> Result<&mut LocalAccount> {
        let client = self.client.clone();
        println!("Creating and minting faucet account");
        let faucet_account = &mut self.designated_dealer_account;
        let balance = retrieve_account_balance(&client, faucet_account.address()).await?;
        for b in balance {
            if b.currency.eq(XUS_NAME) {
                println!(
                    "DD account current balances are {}, requested {} coins",
                    b.amount, coins_total
                );
                break;
            }
        }
        Ok(faucet_account)
    }

    async fn load_vasp_account(&self, client: &RestClient, index: usize) -> Result<LocalAccount> {
        let file = "vasp".to_owned() + index.to_string().as_str() + ".key";
        let mint_key: Ed25519PrivateKey = generate_key::load_key(file);
        let account_key = AccountKey::from_private_key(mint_key);
        let address = account_key.authentication_key().derived_address();
        let sequence_number = query_sequence_numbers(client, &[address])
            .await
            .map_err(|e| {
                format_err!(
                    "query_sequence_numbers on {:?} for dd account failed: {}",
                    client,
                    e
                )
            })?[0];
        Ok(LocalAccount::new(address, account_key, sequence_number))
    }

    async fn get_seed_accounts(
        &mut self,
        rest_clients: &[RestClient],
        seed_account_num: usize,
        vasp: bool,
    ) -> Result<Vec<LocalAccount>> {
        info!("Creating and minting seeds accounts");
        let mut i = 0;
        let mut seed_accounts = vec![];
        // load vasp account created by AOS directly
        if vasp {
            let client = self.pick_mint_client(rest_clients).clone();
            info!("Loading VASP account as seed accounts");
            let load_account_num = min(seed_account_num, MAX_VASP_ACCOUNT_NUM);
            for i in 0..load_account_num {
                let account = self.load_vasp_account(&client, i).await?;
                seed_accounts.push(account);
            }
            info!("Loaded {} VASP accounts", seed_accounts.len());
            return Ok(seed_accounts);
        }
        while i < seed_account_num {
            let client = self.pick_mint_client(rest_clients).clone();
            let batch_size = min(MAX_TXN_BATCH_SIZE, seed_account_num - i);
            let mut batch = gen_random_accounts(batch_size, self.rng());
            let creation_account = &mut self.treasury_compliance_account;
            let txn_factory = &self.txn_factory;
            let create_requests = batch
                .iter()
                .map(|account| {
                    create_parent_vasp_request(
                        creation_account,
                        account.authentication_key(),
                        txn_factory,
                    )
                })
                .collect();
            execute_and_wait_transactions(&client, creation_account, create_requests).await?;
            i += batch_size;
            seed_accounts.append(&mut batch);
        }
        info!("Completed creating seed accounts");

        Ok(seed_accounts)
    }

    fn pick_mint_client<'a>(&mut self, clients: &'a [RestClient]) -> &'a RestClient {
        clients
            .choose(self.rng())
            .expect("json-rpc clients can not be empty")
    }
}

fn gen_rng_for_reusable_account(count: usize) -> Vec<StdRng> {
    // use same seed for reuse account creation and reuse
    let mut seed = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let mut rngs = vec![];
    for i in 0..count {
        seed[31] = i as u8;
        rngs.push(StdRng::from_seed(seed));
    }
    rngs
}

async fn gen_reusable_account<R>(client: &RestClient, rng: &mut R) -> Result<LocalAccount>
where
    R: ::rand_core::RngCore + ::rand_core::CryptoRng,
{
    let account_key = AccountKey::generate(rng);
    let address = account_key.authentication_key().derived_address();
    let sequence_number = match query_sequence_numbers(client, &[address]).await {
        Ok(v) => v[0],
        Err(_) => 0,
    };
    Ok(LocalAccount::new(address, account_key, sequence_number))
}

async fn gen_reusable_accounts<R>(
    client: &RestClient,
    num_accounts: usize,
    rng: &mut R,
) -> Result<Vec<LocalAccount>>
where
    R: ::rand_core::RngCore + ::rand_core::CryptoRng,
{
    let mut vasp_accounts = vec![];
    let mut i = 0;
    while i < num_accounts {
        vasp_accounts.push(gen_reusable_account(client, rng).await?);
        i += 1;
    }
    Ok(vasp_accounts)
}
