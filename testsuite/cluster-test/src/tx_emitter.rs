// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{cluster::Cluster, instance::Instance};
use std::{
    fmt, slice,
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
    account_config::{association_address, lbr_type_tag, LBR_NAME},
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
use reqwest::{Client, Url};
use std::{
    cmp::{max, min},
    ops::Sub,
    str::FromStr,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};
use tokio::{task::JoinHandle, time};
use util::retry;

const MAX_TXN_BATCH_SIZE: usize = 100; // Max transactions per account in mempool

pub struct TxEmitter {
    accounts: Vec<AccountData>,
    mint_key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    http_client: Client,
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
}

#[derive(Debug, Default)]
pub struct TxStats {
    pub submitted: u64,
    pub committed: u64,
    pub expired: u64,
    pub latency: u64,
}

#[derive(Debug, Default)]
pub struct TxStatsRate {
    pub submitted: u64,
    pub committed: u64,
    pub expired: u64,
    pub latency: u64,
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
}

impl TxEmitter {
    pub fn new(cluster: &Cluster) -> Self {
        Self {
            accounts: vec![],
            mint_key_pair: cluster.mint_key_pair().clone(),
            http_client: Client::new(),
        }
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
        self.make_client(self.pick_mint_instance(instances))
    }

    pub async fn submit_single_transaction(
        &self,
        instance: &Instance,
        account: &mut AccountData,
    ) -> Result<Instant> {
        let client = self.make_client(instance);
        client
            .submit_transaction(gen_mint_request(account, 10))
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
                let client = self.make_client(&instance);
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
        let client = self.make_client(instance);
        let address = association_address();
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

    pub async fn mint_accounts(&mut self, req: &EmitJobRequest, num_accounts: usize) -> Result<()> {
        if self.accounts.len() >= num_accounts {
            info!("Not minting accounts");
            return Ok(()); // Early return to skip printing 'Minting ...' logs
        }
        let mut faucet_account = self
            .load_faucet_account(self.pick_mint_instance(&req.instances))
            .await?;
        let mint_txn = gen_mint_request(
            &mut faucet_account,
            LIBRA_PER_NEW_ACCOUNT * num_accounts as u64,
        );
        execute_and_wait_transactions(
            &mut self.pick_mint_client(&req.instances),
            &mut faucet_account,
            vec![mint_txn],
        )
        .await
        .map_err(|e| format_err!("Failed to mint into faucet account: {}", e))?;
        let libra_per_seed =
            (LIBRA_PER_NEW_ACCOUNT * num_accounts as u64) / req.instances.len() as u64;
        // Create seed accounts with which we can create actual accounts concurrently
        let seed_accounts = mint_to_new_accounts(
            &mut faucet_account,
            req.instances.len(),
            libra_per_seed,
            100,
            self.pick_mint_client(&req.instances),
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
                let num_new_accounts = num_accounts / req.instances.len();
                let client = self.make_client(&instance);
                create_new_accounts(
                    seed_account,
                    num_new_accounts,
                    LIBRA_PER_NEW_ACCOUNT,
                    20,
                    client,
                )
            });
        self.accounts = try_join_all(account_futures)
            .await
            .map_err(|e| format_err!("Failed to mint accounts {}", e))?
            .into_iter()
            .flatten()
            .collect();
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

    fn make_client(&self, instance: &Instance) -> JsonRpcAsyncClient {
        JsonRpcAsyncClient::new_with_client(
            self.http_client.clone(),
            Url::from_str(format!("http://{}:{}", instance.ip(), instance.ac_port()).as_str())
                .expect("Invalid URL."),
        )
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
        let client = self.make_client(instance);
        let resp = client
            .get_accounts_state(slice::from_ref(address))
            .await
            .map_err(|e| format_err!("[{:?}] get_accounts_state failed: {:?} ", client, e))?;
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
}

impl SubmissionWorker {
    #[allow(clippy::collapsible_if)]
    async fn run(mut self) -> Vec<AccountData> {
        let wait = Duration::from_millis(self.params.wait_millis);
        while !self.stop.load(Ordering::Relaxed) {
            let requests = self.gen_requests();
            let num_requests = requests.len();
            let start_time = Instant::now();
            let mut tx_offset_time = 0u64;
            for request in requests {
                let cur_time = Instant::now();
                let wait_util = cur_time + wait;
                tx_offset_time += (cur_time - start_time).as_millis() as u64;
                self.stats.submitted.fetch_add(1, Ordering::Relaxed);
                let resp = self.client.submit_transaction(request).await;
                if let Err(e) = resp {
                    warn!("[{:?}] Failed to submit request: {:?}", self.client, e);
                }
                let now = Instant::now();
                if wait_util > now {
                    time::delay_for(wait_util - now).await;
                }
            }
            if self.params.wait_committed {
                if let Err(uncommitted) =
                    wait_for_accounts_sequence(&self.client, &mut self.accounts).await
                {
                    let end_time = (Instant::now() - start_time).as_millis() as u64;
                    let num_committed = (num_requests - uncommitted.len()) as u64;
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
                        (end_time - tx_offset_time / num_requests as u64) * num_committed as u64,
                        Ordering::Relaxed,
                    );
                    info!(
                        "[{:?}] Transactions were not committed before expiration: {:?}",
                        self.client, uncommitted
                    );
                } else {
                    let end_time = (Instant::now() - start_time).as_millis() as u64;
                    self.stats
                        .committed
                        .fetch_add(num_requests as u64, Ordering::Relaxed);
                    self.stats.latency.fetch_add(
                        end_time * num_requests as u64 - tx_offset_time,
                        Ordering::Relaxed,
                    );
                }
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
            let request = gen_transfer_txn_request(sender, receiver, Vec::new(), 1);
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
            Err(e) => info!(
                "Failed to query ledger info for instance {:?} : {:?}",
                client, e
            ),
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
            .get_accounts_state(addresses_batch)
            .await
            .map_err(|e| format_err!("[{:?}] get_accounts_state failed: {:?} ", client, e))?;

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
const GAS_UNIT_PRICE: u64 = 0;
const GAS_CURRENCY_CODE: &str = LBR_NAME;
const TXN_EXPIRATION_SECONDS: i64 = 50;
const TXN_MAX_WAIT: Duration = Duration::from_secs(TXN_EXPIRATION_SECONDS as u64 + 30);
const LIBRA_PER_NEW_ACCOUNT: u64 = 1_000_000;

fn gen_submit_transaction_request(
    script: Script,
    sender_account: &mut AccountData,
) -> SignedTransaction {
    let transaction = create_user_txn(
        &sender_account.key_pair,
        TransactionPayload::Script(script),
        sender_account.address,
        sender_account.sequence_number,
        MAX_GAS_AMOUNT,
        GAS_UNIT_PRICE,
        GAS_CURRENCY_CODE.to_owned(),
        TXN_EXPIRATION_SECONDS,
    )
    .expect("Failed to create signed transaction");
    sender_account.sequence_number += 1;
    transaction
}

fn gen_mint_request(faucet_account: &mut AccountData, num_coins: u64) -> SignedTransaction {
    let receiver = faucet_account.address;
    let auth_key_prefix = faucet_account.auth_key_prefix();
    gen_submit_transaction_request(
        transaction_builder::encode_mint_script(
            lbr_type_tag(),
            &receiver,
            auth_key_prefix,
            num_coins,
        ),
        faucet_account,
    )
}

fn gen_transfer_txn_request(
    sender: &mut AccountData,
    receiver: &AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    num_coins: u64,
) -> SignedTransaction {
    gen_submit_transaction_request(
        transaction_builder::encode_transfer_with_metadata_script(
            lbr_type_tag(),
            receiver,
            receiver_auth_key_prefix,
            num_coins,
            vec![],
            vec![],
        ),
        sender,
    )
}

fn gen_mint_txn_request(
    sender: &mut AccountData,
    receiver: &AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    num_coins: u64,
) -> SignedTransaction {
    gen_submit_transaction_request(
        transaction_builder::encode_mint_script(
            lbr_type_tag(),
            receiver,
            receiver_auth_key_prefix,
            num_coins,
        ),
        sender,
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

fn gen_transfer_txn_requests(
    source_account: &mut AccountData,
    accounts: &[AccountData],
    amount: u64,
) -> Vec<SignedTransaction> {
    accounts
        .iter()
        .map(|account| {
            gen_transfer_txn_request(
                source_account,
                &account.address,
                account.auth_key_prefix(),
                amount,
            )
        })
        .collect()
}

fn gen_mint_txn_requests(
    sending_account: &mut AccountData,
    accounts: &[AccountData],
    amount: u64,
) -> Vec<SignedTransaction> {
    accounts
        .iter()
        .map(|account| {
            gen_mint_txn_request(
                sending_account,
                &account.address,
                account.auth_key_prefix(),
                amount,
            )
        })
        .collect()
}

async fn execute_and_wait_transactions(
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
        retry::retry_async(retry::fixed_retry_strategy(5_000, 20), || {
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
) -> Result<Vec<AccountData>> {
    let mut i = 0;
    let mut accounts = vec![];
    while i < num_new_accounts {
        let mut batch = gen_random_accounts(min(
            max_num_accounts_per_batch as usize,
            min(MAX_TXN_BATCH_SIZE, num_new_accounts - i),
        ));
        let requests =
            gen_transfer_txn_requests(&mut source_account, &batch, libra_per_new_account);
        execute_and_wait_transactions(&mut client, &mut source_account, requests).await?;
        i += batch.len();
        accounts.append(&mut batch);
    }
    Ok(accounts)
}

/// Create `num_new_accounts` by minting libra. Return Vec of created accounts
async fn mint_to_new_accounts(
    source_account: &mut AccountData,
    num_new_accounts: usize,
    libra_per_new_account: u64,
    max_num_accounts_per_batch: u64,
    mut client: JsonRpcAsyncClient,
) -> Result<Vec<AccountData>> {
    let mut i = 0;
    let mut accounts = vec![];
    while i < num_new_accounts {
        let mut batch = gen_random_accounts(min(
            max_num_accounts_per_batch as usize,
            min(MAX_TXN_BATCH_SIZE, num_new_accounts - i),
        ));
        let requests = gen_mint_txn_requests(source_account, &batch, libra_per_new_account);
        execute_and_wait_transactions(&mut client, source_account, requests).await?;
        i += batch.len();
        accounts.append(&mut batch);
    }
    Ok(accounts)
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
        }
    }
}

impl TxStats {
    pub fn rate(&self, window: Duration) -> TxStatsRate {
        TxStatsRate {
            submitted: self.submitted / window.as_secs(),
            committed: self.committed / window.as_secs(),
            expired: self.expired / window.as_secs(),
            latency: self.latency / self.committed,
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
        }
    }
}

impl fmt::Display for TxStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "submitted: {}, committed: {}, expired: {}",
            self.submitted, self.committed, self.expired
        )
    }
}

impl fmt::Display for TxStatsRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "submitted: {} txn/s, committed: {} txn/s, expired: {} txn/s, latency: {} ms",
            self.submitted, self.committed, self.expired, self.latency
        )
    }
}
