// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{atomic_histogram::*, cluster::Cluster, instance::Instance};
use std::{
    env, fmt, slice,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{format_err, Result};
use itertools::zip;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
    traits::Uniform,
};
use libra_logger::*;
use libra_types::{
    account_address::AccountAddress,
    account_config::{self, testnet_dd_account_address, COIN1_NAME},
    chain_id::ChainId,
    transaction::{
        authenticator::AuthenticationKey, helpers::create_user_txn, Script, TransactionPayload,
    },
};
use rand::{
    prelude::ThreadRng,
    rngs::{OsRng, StdRng},
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};
use tokio::runtime::Handle;

use futures::future::{try_join_all, FutureExt};
use libra_json_rpc_client::JsonRpcAsyncClient;
use libra_types::transaction::SignedTransaction;
use once_cell::sync::Lazy;
use std::{
    cmp::{max, min},
    ops::Sub,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};
use tokio::{task::JoinHandle, time};

const MAX_TXN_BATCH_SIZE: usize = 100; // Max transactions per account in mempool

pub struct TxEmitter {
    accounts: Vec<AccountData>,
    mint_key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    chain_id: ChainId,
}

pub struct EmitJob {
    workers: Vec<Worker>,
    stop: Arc<AtomicBool>,
    stats: Arc<StatsAccumulator>,
}

#[derive(Default)]
struct StatsAccumulator {
    submitted: AtomicU64,
    committed: AtomicU64,
    expired: AtomicU64,
    latency: AtomicU64,
    latencies: Arc<AtomicHistogramAccumulator>,
}

#[derive(Debug, Default)]
pub struct TxStats {
    pub submitted: u64,
    pub committed: u64,
    pub expired: u64,
    pub latency: u64,
    pub latency_buckets: AtomicHistogramSnapshot,
}

#[derive(Debug, Default)]
pub struct TxStatsRate {
    pub submitted: u64,
    pub committed: u64,
    pub expired: u64,
    pub latency: u64,
    pub p99_latency: u64,
}

#[derive(Clone)]
pub struct EmitThreadParams {
    pub wait_millis: u64,
    pub wait_committed: bool,
}

impl Default for EmitThreadParams {
    fn default() -> Self {
        Self {
            wait_millis: 0,
            wait_committed: true,
        }
    }
}

#[derive(Clone)]
pub struct EmitJobRequest {
    pub instances: Vec<Instance>,
    pub accounts_per_client: usize,
    pub workers_per_ac: Option<usize>,
    pub thread_params: EmitThreadParams,
}

pub static GAS_UNIT_PRICE: Lazy<u64> = Lazy::new(|| {
    if let Ok(v) = env::var("GAS_UNIT_PRICE") {
        v.parse().expect("Failed to parse GAS_UNIT_PRICE")
    } else {
        0_u64
    }
});

impl EmitJobRequest {
    pub fn for_instances(
        instances: Vec<Instance>,
        global_emit_job_request: &Option<EmitJobRequest>,
    ) -> Self {
        match global_emit_job_request {
            Some(global_emit_job_request) => EmitJobRequest {
                instances,
                ..global_emit_job_request.clone()
            },
            None => Self {
                instances,
                accounts_per_client: 15,
                workers_per_ac: None,
                thread_params: EmitThreadParams::default(),
            },
        }
    }

    pub fn fixed_tps_params(instance_count: usize, tps: u64) -> (usize, u64) {
        if tps < 1 {
            panic!("Target tps {} can not less than 1", tps)
        }
        let num_workers = tps as usize / instance_count + 1;
        let wait_time = (instance_count * num_workers * 1000_usize / tps as usize) as u64;
        (num_workers, wait_time)
    }

    pub fn fixed_tps(instances: Vec<Instance>, tps: u64) -> Self {
        let (num_workers, wait_time) = EmitJobRequest::fixed_tps_params(instances.len(), tps);
        Self {
            instances,
            accounts_per_client: 1,
            workers_per_ac: Some(num_workers),
            thread_params: EmitThreadParams {
                wait_millis: wait_time,
                wait_committed: true,
            },
        }
    }
}

impl TxEmitter {
    pub fn new(cluster: &Cluster) -> Self {
        Self {
            accounts: vec![],
            mint_key_pair: cluster.mint_key_pair().clone(),
            chain_id: cluster.chain_id,
        }
    }

    pub fn take_account(&mut self) -> AccountData {
        self.accounts.remove(0)
    }

    pub fn clear(&mut self) {
        self.accounts.clear();
    }

    fn pick_mint_instance<'a, 'b>(&'a self, instances: &'b [Instance]) -> &'b Instance {
        let mut rng = ThreadRng::default();
        instances
            .choose(&mut rng)
            .expect("Instances can not be empty")
    }

    fn pick_mint_client(&self, instances: &[Instance]) -> JsonRpcAsyncClient {
        self.pick_mint_instance(instances).json_rpc_client()
    }

    pub async fn submit_single_transaction(
        &self,
        instance: &Instance,
        account: &mut AccountData,
    ) -> Result<Instant> {
        let client = instance.json_rpc_client();
        client
            .submit_transaction(gen_mint_request(account, 10, self.chain_id))
            .await?;
        let deadline = Instant::now() + TXN_MAX_WAIT;
        Ok(deadline)
    }

    pub async fn start_job(&mut self, req: EmitJobRequest) -> Result<EmitJob> {
        let workers_per_ac = match req.workers_per_ac {
            Some(x) => x,
            None => {
                let target_threads = 300;
                // Trying to create somewhere between target_threads/2..target_threads threads
                // We want to have equal numbers of threads for each AC, so that they are equally loaded
                // Otherwise things like flamegrap/perf going to show different numbers depending on which AC is chosen
                // Also limiting number of threads as max 10 per AC for use cases with very small number of nodes or use --peers
                min(10, max(1, target_threads / req.instances.len()))
            }
        };
        let num_clients = req.instances.len() * workers_per_ac;
        info!(
            "Will use {} workers per AC with total {} AC clients",
            workers_per_ac, num_clients
        );
        let num_accounts = req.accounts_per_client * num_clients;
        info!(
            "Will create {} accounts_per_client with total {} accounts",
            req.accounts_per_client, num_accounts
        );
        self.mint_accounts(&req, num_accounts).await?;
        let all_accounts = self.accounts.split_off(self.accounts.len() - num_accounts);
        let mut workers = vec![];
        let all_addresses: Vec<_> = all_accounts.iter().map(|d| d.address).collect();
        let all_addresses = Arc::new(all_addresses);
        let mut all_accounts = all_accounts.into_iter();
        let stop = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(StatsAccumulator::default());
        let tokio_handle = Handle::current();
        for instance in &req.instances {
            for _ in 0..workers_per_ac {
                let client = instance.json_rpc_client();
                let accounts = (&mut all_accounts).take(req.accounts_per_client).collect();
                let all_addresses = all_addresses.clone();
                let stop = stop.clone();
                let params = req.thread_params.clone();
                let stats = Arc::clone(&stats);
                let worker = SubmissionWorker {
                    accounts,
                    client,
                    all_addresses,
                    stop,
                    params,
                    stats,
                    chain_id: self.chain_id,
                };
                let join_handle = tokio_handle.spawn(worker.run().boxed());
                workers.push(Worker { join_handle });
            }
        }
        info!("Tx emitter workers started");
        Ok(EmitJob {
            workers,
            stop,
            stats,
        })
    }

    pub async fn load_faucet_account(&self, instance: &Instance) -> Result<AccountData> {
        let client = instance.json_rpc_client();
        let address = testnet_dd_account_address();
        let sequence_number = query_sequence_numbers(&client, &[address])
            .await
            .map_err(|e| {
                format_err!(
                    "query_sequence_numbers on {:?} for faucet account failed: {}",
                    client,
                    e
                )
            })?[0];
        Ok(AccountData {
            address,
            key_pair: self.mint_key_pair.clone(),
            sequence_number,
        })
    }

    pub async fn load_tc_account(&self, instance: &Instance) -> Result<AccountData> {
        let client = instance.json_rpc_client();
        let address = account_config::treasury_compliance_account_address();
        let sequence_number = query_sequence_numbers(&client, &[address])
            .await
            .map_err(|e| {
                format_err!(
                    "query_sequence_numbers on {:?} for treasury compliance account failed: {}",
                    client,
                    e
                )
            })?[0];
        Ok(AccountData {
            address,
            key_pair: self.mint_key_pair.clone(),
            sequence_number,
        })
    }

    pub async fn mint_accounts(
        &mut self,
        req: &EmitJobRequest,
        requested_accounts: usize,
    ) -> Result<()> {
        if self.accounts.len() >= requested_accounts {
            info!("Not minting accounts");
            return Ok(()); // Early return to skip printing 'Minting ...' logs
        }
        let num_accounts = requested_accounts - self.accounts.len(); // Only minting extra accounts
        info!("Minting additional {} accounts", num_accounts);
        let mut faucet_account = self
            .load_faucet_account(self.pick_mint_instance(&req.instances))
            .await?;
        let mut tc_account = self
            .load_tc_account(self.pick_mint_instance(&req.instances))
            .await?;
        let coins_per_account = (SEND_AMOUNT + *GAS_UNIT_PRICE) * MAX_TXNS;
        info!("Minting additional {} accounts", num_accounts);
        let coins_per_seed_account =
            (coins_per_account * num_accounts as u64) / req.instances.len() as u64;
        let coins_total = coins_per_account * num_accounts as u64;
        let mint_txn = gen_mint_request(&mut faucet_account, coins_total, self.chain_id);
        execute_and_wait_transactions(
            &mut self.pick_mint_client(&req.instances),
            &mut faucet_account,
            vec![mint_txn],
        )
        .await
        .map_err(|e| format_err!("Failed to mint into faucet account: {}", e))?;

        let seed_accounts = create_seed_accounts(
            &mut tc_account,
            req.instances.len(),
            100,
            self.pick_mint_client(&req.instances),
            self.chain_id,
        )
        .await
        .map_err(|e| format_err!("Failed to create seed accounts: {}", e))?;
        info!("Completed creating seed accounts");
        // Create seed accounts with which we can create actual accounts concurrently
        mint_to_new_accounts(
            &mut faucet_account,
            &seed_accounts,
            coins_per_seed_account,
            100,
            self.pick_mint_client(&req.instances),
            self.chain_id,
        )
        .await
        .map_err(|e| format_err!("Failed to mint seed_accounts: {}", e))?;
        info!("Completed minting seed accounts");
        // For each seed account, create a future and transfer libra from that seed account to new accounts
        let account_futures = seed_accounts
            .into_iter()
            .enumerate()
            .map(|(i, seed_account)| {
                // Spawn new threads
                let instance = req.instances[i].clone();
                let num_new_accounts =
                    (num_accounts + req.instances.len() - 1) / req.instances.len();
                let client = instance.json_rpc_client();
                create_new_accounts(
                    seed_account,
                    num_new_accounts,
                    coins_per_account,
                    20,
                    client,
                    self.chain_id,
                )
            });
        let mut minted_accounts = try_join_all(account_futures)
            .await
            .map_err(|e| format_err!("Failed to mint accounts {}", e))?
            .into_iter()
            .flatten()
            .collect();

        self.accounts.append(&mut minted_accounts);
        assert!(
            self.accounts.len() >= num_accounts,
            "Something wrong in mint_account, wanted to mint {}, only have {}",
            requested_accounts,
            self.accounts.len()
        );
        info!("Mint is done");
        Ok(())
    }

    pub fn peek_job_stats(&self, job: &EmitJob) -> TxStats {
        job.stats.accumulate()
    }

    pub async fn stop_job(&mut self, job: EmitJob) -> TxStats {
        job.stop.store(true, Ordering::Relaxed);
        for worker in job.workers {
            let mut accounts = worker
                .join_handle
                .await
                .expect("TxEmitter worker thread failed");
            self.accounts.append(&mut accounts);
        }
        job.stats.accumulate()
    }

    pub async fn emit_txn_for(
        &mut self,
        duration: Duration,
        emit_job_request: EmitJobRequest,
    ) -> Result<TxStats> {
        let job = self.start_job(emit_job_request).await?;
        tokio::time::delay_for(duration).await;
        let stats = self.stop_job(job).await;
        Ok(stats)
    }

    pub async fn query_sequence_numbers(
        &self,
        instance: &Instance,
        address: &AccountAddress,
    ) -> Result<u64> {
        let client = instance.json_rpc_client();
        let resp = client
            .get_accounts(slice::from_ref(address))
            .await
            .map_err(|e| format_err!("[{:?}] get_accounts failed: {:?} ", client, e))?;
        Ok(resp[0]
            .as_ref()
            .ok_or_else(|| format_err!("account does not exist"))?
            .sequence_number)
    }
}

struct Worker {
    join_handle: JoinHandle<Vec<AccountData>>,
}

struct SubmissionWorker {
    accounts: Vec<AccountData>,
    client: JsonRpcAsyncClient,
    all_addresses: Arc<Vec<AccountAddress>>,
    stop: Arc<AtomicBool>,
    params: EmitThreadParams,
    stats: Arc<StatsAccumulator>,
    chain_id: ChainId,
}

impl SubmissionWorker {
    #[allow(clippy::collapsible_if)]
    async fn run(mut self) -> Vec<AccountData> {
        let wait = Duration::from_millis(self.params.wait_millis);
        while !self.stop.load(Ordering::Relaxed) {
            let requests = self.gen_requests();
            let num_requests = requests.len();
            let start_time = Instant::now();
            let wait_util = start_time + wait;
            let mut tx_offset_time = 0u64;
            for request in requests {
                let cur_time = Instant::now();
                tx_offset_time += (cur_time - start_time).as_millis() as u64;
                self.stats.submitted.fetch_add(1, Ordering::Relaxed);
                let resp = self.client.submit_transaction(request).await;
                if let Err(e) = resp {
                    warn!("[{:?}] Failed to submit request: {:?}", self.client, e);
                }
            }
            if self.params.wait_committed {
                if let Err(uncommitted) =
                    wait_for_accounts_sequence(&self.client, &mut self.accounts).await
                {
                    let end_time = (Instant::now() - start_time).as_millis() as u64;
                    let num_committed = (num_requests - uncommitted.len()) as u64;
                    let latency = end_time - tx_offset_time / num_requests as u64;
                    self.stats
                        .committed
                        .fetch_add(num_committed, Ordering::Relaxed);
                    self.stats
                        .expired
                        .fetch_add(uncommitted.len() as u64, Ordering::Relaxed);
                    self.stats.latency.fetch_add(
                        // To avoid negative result caused by uncommitted tx occur
                        // Simplified from:
                        // end_time * num_committed - (tx_offset_time/num_requests) * num_committed
                        // to
                        // (end_time - tx_offset_time / num_requests) * num_committed
                        latency * num_committed as u64,
                        Ordering::Relaxed,
                    );
                    self.stats
                        .latencies
                        .record_data_point(latency, num_committed);
                    info!(
                        "[{:?}] Transactions were not committed before expiration: {:?}",
                        self.client, uncommitted
                    );
                } else {
                    let end_time = (Instant::now() - start_time).as_millis() as u64;
                    let latency = end_time - tx_offset_time / num_requests as u64;
                    self.stats
                        .committed
                        .fetch_add(num_requests as u64, Ordering::Relaxed);
                    self.stats
                        .latency
                        .fetch_add(latency * num_requests as u64, Ordering::Relaxed);
                    self.stats
                        .latencies
                        .record_data_point(latency, num_requests as u64);
                }
            }
            let now = Instant::now();
            if wait_util > now {
                time::delay_for(wait_util - now).await;
            }
        }
        self.accounts
    }

    fn gen_requests(&mut self) -> Vec<SignedTransaction> {
        let mut rng = ThreadRng::default();
        let batch_size = max(MAX_TXN_BATCH_SIZE, self.accounts.len());
        let accounts = self
            .accounts
            .iter_mut()
            .choose_multiple(&mut rng, batch_size);
        let mut requests = Vec::with_capacity(accounts.len());
        for sender in accounts {
            let receiver = self
                .all_addresses
                .choose(&mut rng)
                .expect("all_addresses can't be empty");
            let request = gen_transfer_txn_request(sender, receiver, SEND_AMOUNT, self.chain_id);
            requests.push(request);
        }
        requests
    }
}

async fn wait_for_accounts_sequence(
    client: &JsonRpcAsyncClient,
    accounts: &mut [AccountData],
) -> Result<(), Vec<(AccountAddress, u64)>> {
    let deadline = Instant::now() + TXN_MAX_WAIT;
    let addresses: Vec<_> = accounts.iter().map(|d| d.address).collect();
    loop {
        match query_sequence_numbers(client, &addresses).await {
            Err(e) => {
                info!(
                    "Failed to query ledger info on accounts {:?} for instance {:?} : {:?}",
                    addresses, client, e
                );
                time::delay_for(Duration::from_millis(300)).await;
            }
            Ok(sequence_numbers) => {
                if is_sequence_equal(accounts, &sequence_numbers) {
                    break;
                }
                let mut uncommitted = vec![];
                if Instant::now() > deadline {
                    for (account, sequence_number) in zip(accounts, &sequence_numbers) {
                        if account.sequence_number != *sequence_number {
                            warn!("Wait deadline exceeded for account {}, expected sequence {}, got from server: {}", account.address, account.sequence_number, sequence_number);
                            uncommitted.push((account.address, *sequence_number));
                            account.sequence_number = *sequence_number;
                        }
                    }
                    return Err(uncommitted);
                }
            }
        }
        time::delay_for(Duration::from_millis(100)).await;
    }
    Ok(())
}

fn is_sequence_equal(accounts: &[AccountData], sequence_numbers: &[u64]) -> bool {
    for (account, sequence_number) in zip(accounts, sequence_numbers) {
        if *sequence_number != account.sequence_number {
            return false;
        }
    }
    true
}

async fn query_sequence_numbers(
    client: &JsonRpcAsyncClient,
    addresses: &[AccountAddress],
) -> Result<Vec<u64>> {
    let mut result = vec![];
    for addresses_batch in addresses.chunks(20) {
        let resp = client
            .get_accounts(addresses_batch)
            .await
            .map_err(|e| format_err!("[{:?}] get_accounts failed: {:?} ", client, e))?;

        for item in resp.into_iter() {
            result.push(
                item.ok_or_else(|| format_err!("account does not exist"))?
                    .sequence_number,
            );
        }
    }
    Ok(result)
}

const MAX_GAS_AMOUNT: u64 = 1_000_000;
const GAS_CURRENCY_CODE: &str = COIN1_NAME;
const TXN_EXPIRATION_SECONDS: i64 = 50;
const TXN_MAX_WAIT: Duration = Duration::from_secs(TXN_EXPIRATION_SECONDS as u64 + 30);
const MAX_TXNS: u64 = 1_000_000;
const SEND_AMOUNT: u64 = 1;

fn gen_submit_transaction_request(
    script: Script,
    sender_account: &mut AccountData,
    no_gas: bool,
    chain_id: ChainId,
) -> SignedTransaction {
    let transaction = create_user_txn(
        &sender_account.key_pair,
        TransactionPayload::Script(script),
        sender_account.address,
        sender_account.sequence_number,
        MAX_GAS_AMOUNT,
        if no_gas { 0 } else { *GAS_UNIT_PRICE },
        GAS_CURRENCY_CODE.to_owned(),
        TXN_EXPIRATION_SECONDS,
        chain_id,
    )
    .expect("Failed to create signed transaction");
    sender_account.sequence_number += 1;
    transaction
}

fn gen_mint_request(
    faucet_account: &mut AccountData,
    num_coins: u64,
    chain_id: ChainId,
) -> SignedTransaction {
    let receiver = faucet_account.address;
    gen_submit_transaction_request(
        transaction_builder::encode_peer_to_peer_with_metadata_script(
            account_config::coin1_tag(),
            receiver,
            num_coins,
            vec![],
            vec![],
        ),
        faucet_account,
        true,
        chain_id,
    )
}

fn gen_transfer_txn_request(
    sender: &mut AccountData,
    receiver: &AccountAddress,
    num_coins: u64,
    chain_id: ChainId,
) -> SignedTransaction {
    gen_submit_transaction_request(
        transaction_builder::encode_peer_to_peer_with_metadata_script(
            account_config::coin1_tag(),
            *receiver,
            num_coins,
            vec![],
            vec![],
        ),
        sender,
        false,
        chain_id,
    )
}

fn gen_create_child_txn_request(
    sender: &mut AccountData,
    receiver: &AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    num_coins: u64,
    chain_id: ChainId,
) -> SignedTransaction {
    let add_all_currencies = false;
    gen_submit_transaction_request(
        transaction_builder::encode_create_child_vasp_account_script(
            account_config::coin1_tag(),
            *receiver,
            receiver_auth_key_prefix,
            add_all_currencies,
            num_coins,
        ),
        sender,
        true,
        chain_id,
    )
}

fn gen_create_account_txn_request(
    sender: &mut AccountData,
    receiver: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    chain_id: ChainId,
) -> SignedTransaction {
    gen_submit_transaction_request(
        transaction_builder::encode_create_parent_vasp_account_script(
            account_config::coin1_tag(),
            0,
            *receiver,
            auth_key_prefix,
            vec![],
            false,
        ),
        sender,
        true,
        chain_id,
    )
}

fn gen_mint_txn_request(
    sender: &mut AccountData,
    receiver: &AccountAddress,
    num_coins: u64,
    chain_id: ChainId,
) -> SignedTransaction {
    gen_submit_transaction_request(
        transaction_builder::encode_peer_to_peer_with_metadata_script(
            account_config::coin1_tag(),
            *receiver,
            num_coins,
            vec![],
            vec![],
        ),
        sender,
        true,
        chain_id,
    )
}

fn gen_random_account(rng: &mut StdRng) -> AccountData {
    let key_pair = KeyPair::generate(rng);
    AccountData {
        address: libra_types::account_address::from_public_key(&key_pair.public_key),
        key_pair,
        sequence_number: 0,
    }
}

fn gen_random_accounts(num_accounts: usize) -> Vec<AccountData> {
    let seed: [u8; 32] = OsRng.gen();
    let mut rng = StdRng::from_seed(seed);
    (0..num_accounts)
        .map(|_| gen_random_account(&mut rng))
        .collect()
}

fn gen_create_child_txn_requests(
    source_account: &mut AccountData,
    accounts: &[AccountData],
    amount: u64,
    chain_id: ChainId,
) -> Vec<SignedTransaction> {
    accounts
        .iter()
        .map(|account| {
            gen_create_child_txn_request(
                source_account,
                &account.address,
                account.auth_key_prefix(),
                amount,
                chain_id,
            )
        })
        .collect()
}

fn gen_account_creation_txn_requests(
    sending_account: &mut AccountData,
    accounts: &[AccountData],
    chain_id: ChainId,
) -> Vec<SignedTransaction> {
    accounts
        .iter()
        .map(|account| {
            gen_create_account_txn_request(
                sending_account,
                &account.address,
                account.auth_key_prefix(),
                chain_id,
            )
        })
        .collect()
}

fn gen_mint_txn_requests(
    sending_account: &mut AccountData,
    accounts: &[AccountData],
    amount: u64,
    chain_id: ChainId,
) -> Vec<SignedTransaction> {
    accounts
        .iter()
        .map(|account| gen_mint_txn_request(sending_account, &account.address, amount, chain_id))
        .collect()
}

pub async fn execute_and_wait_transactions(
    client: &mut JsonRpcAsyncClient,
    account: &mut AccountData,
    txn: Vec<SignedTransaction>,
) -> Result<()> {
    debug!(
        "[{:?}] Submitting transactions {} - {} for {}",
        client,
        account.sequence_number - txn.len() as u64,
        account.sequence_number,
        account.address
    );
    for request in txn {
        libra_retrier::retry_async(libra_retrier::fixed_retry_strategy(5_000, 20), || {
            let request = request.clone();
            let c = client.clone();
            let client_name = format!("{:?}", client);
            Box::pin(async move {
                let txn_str = format!("{}::{}", request.sender(), request.sequence_number());
                debug!("Submitting txn {}", txn_str);
                let resp = c.submit_transaction(request).await;
                debug!("txn {} status: {:?}", txn_str, resp);

                resp.map_err(|e| format_err!("[{}] Failed to submit request: {:?}", client_name, e))
            })
        })
        .await?;
    }
    let r = wait_for_accounts_sequence(client, slice::from_mut(account))
        .await
        .map_err(|_| format_err!("Mint transactions were not committed before expiration"));
    debug!(
        "[{:?}] Account {} is at sequence number {} now",
        client, account.address, account.sequence_number
    );
    r
}

/// Create `num_new_accounts` by transferring libra from `source_account`. Return Vec of created
/// accounts
async fn create_new_accounts(
    mut source_account: AccountData,
    num_new_accounts: usize,
    libra_per_new_account: u64,
    max_num_accounts_per_batch: u64,
    mut client: JsonRpcAsyncClient,
    chain_id: ChainId,
) -> Result<Vec<AccountData>> {
    let mut i = 0;
    let mut accounts = vec![];
    while i < num_new_accounts {
        let mut batch = gen_random_accounts(min(
            max_num_accounts_per_batch as usize,
            min(MAX_TXN_BATCH_SIZE, num_new_accounts - i),
        ));
        let requests = gen_create_child_txn_requests(
            &mut source_account,
            &batch,
            libra_per_new_account,
            chain_id,
        );
        execute_and_wait_transactions(&mut client, &mut source_account, requests).await?;
        i += batch.len();
        accounts.append(&mut batch);
    }
    Ok(accounts)
}

/// Create `num_new_accounts`. Return Vec of created accounts
async fn create_seed_accounts(
    creation_account: &mut AccountData,
    num_new_accounts: usize,
    max_num_accounts_per_batch: u64,
    mut client: JsonRpcAsyncClient,
    chain_id: ChainId,
) -> Result<Vec<AccountData>> {
    let mut i = 0;
    let mut accounts = vec![];
    while i < num_new_accounts {
        let mut batch = gen_random_accounts(min(
            max_num_accounts_per_batch as usize,
            min(MAX_TXN_BATCH_SIZE, num_new_accounts - i),
        ));
        let create_requests = gen_account_creation_txn_requests(creation_account, &batch, chain_id);
        execute_and_wait_transactions(&mut client, creation_account, create_requests).await?;
        i += batch.len();
        accounts.append(&mut batch);
    }
    Ok(accounts)
}

/// Mint `libra_per_new_account` from `minting_account` to each account in `accounts`.
async fn mint_to_new_accounts(
    minting_account: &mut AccountData,
    accounts: &[AccountData],
    libra_per_new_account: u64,
    max_num_accounts_per_batch: u64,
    mut client: JsonRpcAsyncClient,
    chain_id: ChainId,
) -> Result<()> {
    let mut left = accounts;
    let mut i = 0;
    let num_accounts = accounts.len();
    while !left.is_empty() {
        let batch_size = OsRng.gen::<usize>()
            % min(
                max_num_accounts_per_batch as usize,
                min(MAX_TXN_BATCH_SIZE, num_accounts - i),
            );
        let (to_batch, rest) = left.split_at(batch_size + 1);
        let mint_requests =
            gen_mint_txn_requests(minting_account, to_batch, libra_per_new_account, chain_id);
        execute_and_wait_transactions(&mut client, minting_account, mint_requests).await?;
        i += to_batch.len();
        left = rest;
    }
    Ok(())
}

#[derive(Clone)]
pub struct AccountData {
    pub address: AccountAddress,
    pub key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    pub sequence_number: u64,
}

impl AccountData {
    pub fn auth_key_prefix(&self) -> Vec<u8> {
        AuthenticationKey::ed25519(&self.key_pair.public_key)
            .prefix()
            .to_vec()
    }
}

impl StatsAccumulator {
    pub fn accumulate(&self) -> TxStats {
        TxStats {
            submitted: self.submitted.load(Ordering::Relaxed),
            committed: self.committed.load(Ordering::Relaxed),
            expired: self.expired.load(Ordering::Relaxed),
            latency: self.latency.load(Ordering::Relaxed),
            latency_buckets: self.latencies.snapshot(),
        }
    }
}

impl TxStats {
    pub fn rate(&self, window: Duration) -> TxStatsRate {
        TxStatsRate {
            submitted: self.submitted / window.as_secs(),
            committed: self.committed / window.as_secs(),
            expired: self.expired / window.as_secs(),
            latency: if self.committed == 0 {
                0u64
            } else {
                self.latency / self.committed
            },
            p99_latency: self.latency_buckets.percentile(99, 100),
        }
    }
}

impl Sub for &TxStats {
    type Output = TxStats;

    fn sub(self, other: &TxStats) -> TxStats {
        TxStats {
            submitted: self.submitted - other.submitted,
            committed: self.committed - other.committed,
            expired: self.expired - other.expired,
            latency: self.latency - other.latency,
            latency_buckets: &self.latency_buckets - &other.latency_buckets,
        }
    }
}

impl fmt::Display for TxStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "submitted: {}, committed: {}, expired: {}",
            self.submitted, self.committed, self.expired,
        )
    }
}

impl fmt::Display for TxStatsRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "submitted: {} txn/s, committed: {} txn/s, expired: {} txn/s, latency: {} ms, p99 latency: {} ms",
            self.submitted, self.committed, self.expired, self.latency, self.p99_latency,
        )
    }
}

#[cfg(test)]
mod test {
    use crate::tx_emitter::EmitJobRequest;

    #[test]
    pub fn test_fixed_tps_params() {
        let inst_num = 30;
        let target_tps = 10;
        let (num_workers, wait_time) = EmitJobRequest::fixed_tps_params(inst_num, target_tps);
        assert_eq!(num_workers, 1usize);
        assert_eq!(wait_time, 3000u64);
        let target_tps = 30;
        let (num_workers, wait_time) = EmitJobRequest::fixed_tps_params(inst_num, target_tps);
        assert_eq!(num_workers, 2usize);
        assert_eq!(wait_time, 2000u64);
    }
}
