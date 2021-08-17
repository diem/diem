// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ChainInfo;
use anyhow::{anyhow, format_err, Context, Result};
use diem_logger::*;
use diem_sdk::{
    client::{views::AmountView, Client as JsonRpcClient, MethodRequest},
    crypto::hash::CryptoHash,
    move_types::account_address::AccountAddress,
    transaction_builder::{Currency, TransactionFactory},
    types::{
        account_config::XUS_NAME,
        transaction::{authenticator::AuthenticationKey, SignedTransaction, Transaction},
        LocalAccount,
    },
};
use futures::future::{try_join_all, FutureExt};
use itertools::zip;
use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};
use rand_core::SeedableRng;
use std::{
    cmp::{max, min},
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime},
};
use tokio::{runtime::Handle, task::JoinHandle, time};

pub mod atomic_histogram;
use atomic_histogram::*;

/// Max transactions per account in mempool
const MAX_TXN_BATCH_SIZE: usize = 100;
const MAX_TXNS: u64 = 1_000_000;
const SEND_AMOUNT: u64 = 1;

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

pub struct EmitJobRequest {
    pub json_rpc_clients: Vec<JsonRpcClient>,
    pub accounts_per_client: usize,
    pub workers_per_endpoint: Option<usize>,
    pub thread_params: EmitThreadParams,
}

impl EmitJobRequest {
    pub fn default(json_rpc_clients: Vec<JsonRpcClient>) -> Self {
        Self {
            json_rpc_clients,
            accounts_per_client: 15,
            workers_per_endpoint: None,
            thread_params: EmitThreadParams::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct TxnStats {
    pub submitted: u64,
    pub committed: u64,
    pub expired: u64,
    pub latency: u64,
    pub latency_buckets: AtomicHistogramSnapshot,
}

#[derive(Debug, Default)]
pub struct TxnStatsRate {
    pub submitted: u64,
    pub committed: u64,
    pub expired: u64,
    pub latency: u64,
    pub p99_latency: u64,
}

#[derive(Default)]
struct StatsAccumulator {
    submitted: AtomicU64,
    committed: AtomicU64,
    expired: AtomicU64,
    latency: AtomicU64,
    latencies: Arc<AtomicHistogramAccumulator>,
}

struct Worker {
    join_handle: JoinHandle<Vec<LocalAccount>>,
}

pub struct EmitJob {
    workers: Vec<Worker>,
    stop: Arc<AtomicBool>,
    stats: Arc<StatsAccumulator>,
}

struct SubmissionWorker {
    accounts: Vec<LocalAccount>,
    client: JsonRpcClient,
    all_addresses: Arc<Vec<AccountAddress>>,
    stop: Arc<AtomicBool>,
    params: EmitThreadParams,
    stats: Arc<StatsAccumulator>,
    txn_factory: TransactionFactory,
    rng: ::rand::rngs::StdRng,
}

impl SubmissionWorker {
    #[allow(clippy::collapsible_if)]
    async fn run(mut self) -> Vec<LocalAccount> {
        let wait_duration = Duration::from_millis(self.params.wait_millis);
        while !self.stop.load(Ordering::Relaxed) {
            let requests = self.gen_requests();
            let num_requests = requests.len();
            let start_time = Instant::now();
            let wait_until = start_time + wait_duration;
            let mut txn_offset_time = 0u64;
            for request in requests {
                let cur_time = Instant::now();
                txn_offset_time += (cur_time - start_time).as_millis() as u64;
                self.stats.submitted.fetch_add(1, Ordering::Relaxed);
                let resp = self.client.submit(&request).await;
                if let Err(e) = resp {
                    warn!("[{:?}] Failed to submit request: {:?}", self.client, e);
                }
            }
            if self.params.wait_committed {
                if let Err(uncommitted) =
                    wait_for_accounts_sequence(&self.client, &mut self.accounts).await
                {
                    let total_duration = (Instant::now() - start_time).as_millis() as u64;
                    let num_committed = (num_requests - uncommitted.len()) as u64;
                    // To avoid negative result caused by uncommitted tx occur
                    // Simplified from:
                    // end_time * num_committed - (txn_offset_time/num_requests) * num_committed
                    // to
                    // (end_time - txn_offset_time / num_requests) * num_committed
                    let latency = total_duration - txn_offset_time / num_requests as u64;
                    let committed_latency = latency * num_committed as u64;
                    self.stats
                        .committed
                        .fetch_add(num_committed, Ordering::Relaxed);
                    self.stats
                        .expired
                        .fetch_add(uncommitted.len() as u64, Ordering::Relaxed);
                    self.stats
                        .latency
                        .fetch_add(committed_latency, Ordering::Relaxed);
                    self.stats
                        .latencies
                        .record_data_point(latency, num_committed);
                    info!(
                        "[{:?}] Transactions were not committed before expiration: {:?}",
                        self.client, uncommitted
                    );
                } else {
                    let total_duration = (Instant::now() - start_time).as_millis() as u64;
                    let latency = total_duration - txn_offset_time / num_requests as u64;
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
            if wait_until > now {
                time::sleep(wait_until - now).await;
            }
        }
        self.accounts
    }

    fn gen_requests(&mut self) -> Vec<SignedTransaction> {
        let batch_size = max(MAX_TXN_BATCH_SIZE, self.accounts.len());
        let accounts = self
            .accounts
            .iter_mut()
            .choose_multiple(&mut self.rng, batch_size);
        let mut requests = Vec::with_capacity(accounts.len());
        for sender in accounts {
            let receiver = self
                .all_addresses
                .choose(&mut self.rng)
                .expect("all_addresses can't be empty");
            let request =
                gen_transfer_txn_request(sender, receiver, SEND_AMOUNT, &self.txn_factory);
            requests.push(request);
        }
        requests
    }
}

#[derive(Debug)]
pub struct TxnEmitter<'t> {
    accounts: Vec<LocalAccount>,
    txn_factory: TransactionFactory,
    chain_info: ChainInfo<'t>,
    client: JsonRpcClient,
    rng: ::rand::rngs::StdRng,
}

impl<'t> TxnEmitter<'t> {
    pub fn new(chain_info: ChainInfo<'t>, rng: ::rand::rngs::StdRng) -> Self {
        let txn_factory = TransactionFactory::new(chain_info.chain_id());
        let client = JsonRpcClient::new(chain_info.json_rpc());
        Self {
            accounts: vec![],
            txn_factory,
            chain_info,
            client,
            rng,
        }
    }

    pub fn rng(&mut self) -> &mut ::rand::rngs::StdRng {
        &mut self.rng
    }

    pub fn from_rng(&mut self) -> ::rand::rngs::StdRng {
        ::rand::rngs::StdRng::from_rng(self.rng()).unwrap()
    }

    pub async fn get_money_source(&mut self, coins_total: u64) -> Result<&mut LocalAccount> {
        let client = self.client.clone();
        println!("Creating and minting faucet account");
        let faucet_account = &mut self.chain_info.designated_dealer_account;
        let mint_txn = gen_transfer_txn_request(
            faucet_account,
            &faucet_account.address(),
            coins_total,
            &self.txn_factory,
        );
        execute_and_wait_transactions(&client, faucet_account, vec![mint_txn])
            .await
            .map_err(|e| format_err!("Failed to mint into faucet account: {}", e))?;
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

    pub async fn get_seed_accounts(
        &mut self,
        json_rpc_clients: &[JsonRpcClient],
        seed_account_num: usize,
    ) -> Result<Vec<LocalAccount>> {
        info!("Creating and minting seeds accounts");
        let mut i = 0;
        let mut seed_accounts = vec![];
        while i < seed_account_num {
            let client = self.pick_mint_client(json_rpc_clients).clone();
            let batch_size = min(MAX_TXN_BATCH_SIZE, seed_account_num - i);
            let mut batch = gen_random_accounts(batch_size, self.rng());
            let creation_account = &mut self.chain_info.treasury_compliance_account;
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

    /// workflow of mint accounts:
    /// 1. mint faucet account as the money source
    /// 2. load tc account to create seed accounts(parent VASP), one seed account for each endpoint
    /// 3. mint coins from faucet to new created seed accounts
    /// 4. split number of requested accounts(child VASP) into equally size of groups
    /// 5. each seed account take responsibility to create one size of group requested accounts and mint coins to them
    /// example:
    /// requested totally 100 new accounts with 10 endpoints
    /// will create 10 seed accounts(parent VASP), each seed account create 10 new accounts
    pub async fn mint_accounts(
        &mut self,
        req: &EmitJobRequest,
        total_requested_accounts: usize,
    ) -> Result<()> {
        if self.accounts.len() >= total_requested_accounts {
            info!("Already have enough accounts exist, do not need to mint more");
            return Ok(());
        }
        let num_accounts = total_requested_accounts - self.accounts.len(); // Only minting extra accounts
        let coins_per_account = SEND_AMOUNT * MAX_TXNS;
        let coins_total = coins_per_account * num_accounts as u64;
        let txn_factory = self.txn_factory.clone();
        let client = self.pick_mint_client(&req.json_rpc_clients);

        // Create seed accounts with which we can create actual accounts concurrently
        let seed_accounts = self
            .get_seed_accounts(&req.json_rpc_clients, req.json_rpc_clients.len())
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

        // For each seed account, create a future and transfer diem from that seed account to new accounts
        let account_futures = seed_accounts
            .into_iter()
            .enumerate()
            .map(|(i, seed_account)| {
                // Spawn new threads
                let index = i % req.json_rpc_clients.len();
                let cur_client = req.json_rpc_clients[index].clone();
                create_new_accounts(
                    seed_account,
                    num_new_child_accounts,
                    coins_per_account,
                    20,
                    cur_client,
                    &txn_factory,
                    self.from_rng(),
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
        Ok(())
    }

    pub async fn start_job(&mut self, req: EmitJobRequest) -> Result<EmitJob> {
        let workers_per_endpoint = match req.workers_per_endpoint {
            Some(x) => x,
            None => {
                let target_threads = 300;
                // Trying to create somewhere between target_threads/2..target_threads threads
                // We want to have equal numbers of threads for each endpoint, so that they are equally loaded
                // Otherwise things like flamegrap/perf going to show different numbers depending on which endpoint is chosen
                // Also limiting number of threads as max 10 per endpoint for use cases with very small number of nodes or use --peers
                min(10, max(1, target_threads / req.json_rpc_clients.len()))
            }
        };
        let num_clients = req.json_rpc_clients.len() * workers_per_endpoint;
        println!(
            "Will use {} workers per endpoint with total {} endpoint clients",
            workers_per_endpoint, num_clients
        );
        let num_accounts = req.accounts_per_client * num_clients;
        println!(
            "Will create {} accounts_per_client with total {} accounts",
            req.accounts_per_client, num_accounts
        );
        self.mint_accounts(&req, num_accounts).await?;
        let all_accounts = self.accounts.split_off(self.accounts.len() - num_accounts);
        let mut workers = vec![];
        let all_addresses: Vec<_> = all_accounts.iter().map(|d| d.address()).collect();
        let all_addresses = Arc::new(all_addresses);
        let mut all_accounts = all_accounts.into_iter();
        let stop = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(StatsAccumulator::default());
        let tokio_handle = Handle::current();
        for client in req.json_rpc_clients {
            for _ in 0..workers_per_endpoint {
                let accounts = (&mut all_accounts).take(req.accounts_per_client).collect();
                let all_addresses = all_addresses.clone();
                let stop = stop.clone();
                let params = req.thread_params.clone();
                let stats = Arc::clone(&stats);
                let worker = SubmissionWorker {
                    accounts,
                    client: client.clone(),
                    all_addresses,
                    stop,
                    params,
                    stats,
                    txn_factory: self.txn_factory.clone(),
                    rng: self.from_rng(),
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

    pub async fn stop_job(&mut self, job: EmitJob) -> TxnStats {
        job.stop.store(true, Ordering::Relaxed);
        for worker in job.workers {
            let mut accounts = worker
                .join_handle
                .await
                .expect("TxnEmitter worker thread failed");
            self.accounts.append(&mut accounts);
        }
        job.stats.accumulate()
    }

    pub async fn emit_txn_for(
        &mut self,
        duration: Duration,
        emit_job_request: EmitJobRequest,
    ) -> Result<TxnStats> {
        let job = self.start_job(emit_job_request).await?;
        println!("starting emitting txns for {} secs", duration.as_secs());
        tokio::time::sleep(duration).await;
        let stats = self.stop_job(job).await;
        Ok(stats)
    }

    fn pick_mint_client<'a>(&mut self, clients: &'a [JsonRpcClient]) -> &'a JsonRpcClient {
        clients
            .choose(self.rng())
            .expect("json-rpc clients can not be empty")
    }
}

async fn retrieve_account_balance(
    client: &JsonRpcClient,
    address: AccountAddress,
) -> Result<Vec<AmountView>> {
    let resp = client
        .get_account(address)
        .await
        .map_err(|e| format_err!("[{:?}] get_accounts failed: {:?} ", client, e))?
        .into_inner();
    Ok(resp
        .ok_or_else(|| format_err!("account does not exist"))?
        .balances)
}

pub async fn execute_and_wait_transactions(
    client: &JsonRpcClient,
    account: &mut LocalAccount,
    txn: Vec<SignedTransaction>,
) -> Result<()> {
    debug!(
        "[{:?}] Submitting transactions {} - {} for {}",
        client,
        account.sequence_number() - txn.len() as u64,
        account.sequence_number(),
        account.address()
    );

    // Batch submit all the txns
    for txn_batch in txn.chunks(20) {
        client
            .batch(
                txn_batch
                    .iter()
                    .map(MethodRequest::submit)
                    .collect::<Result<_, _>>()?,
            )
            .await?
            .into_iter()
            .map(|r| r.and_then(|response| response.into_inner().try_into_submit()))
            .collect::<Result<Vec<()>, _>>()
            .context("failed to submit transactions")?;
    }

    wait_for_signed_transactions(client, &txn).await?;

    debug!(
        "[{:?}] Account {} is at sequence number {} now",
        client,
        account.address(),
        account.sequence_number()
    );
    Ok(())
}

async fn wait_for_signed_transactions(
    client: &JsonRpcClient,
    txns: &[SignedTransaction],
) -> Result<()> {
    let deadline = Instant::now()
        + txns
            .iter()
            .map(SignedTransaction::expiration_timestamp_secs)
            .max()
            .map(Duration::from_secs)
            .ok_or_else(|| anyhow!("Expected at least 1 txn"))?
            .saturating_sub(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?);

    #[allow(clippy::mutable_key_type)]
    let mut uncommitted_txns = txns.iter().collect::<HashSet<_>>();

    while Instant::now() < deadline {
        if uncommitted_txns.is_empty() {
            return Ok(());
        }

        let (batch, queried_txns): (Vec<MethodRequest>, Vec<&SignedTransaction>) = uncommitted_txns
            .iter()
            .take(20)
            .map(|txn| {
                (
                    MethodRequest::get_account_transaction(
                        txn.sender(),
                        txn.sequence_number(),
                        false,
                    ),
                    txn,
                )
            })
            .unzip();
        let responses = client
            .batch(batch)
            .await
            .context("failed to query account transactions")?
            .into_iter()
            .map(|r| {
                r.and_then(|response| response.into_inner().try_into_get_account_transaction())
            })
            .collect::<Result<Vec<_>, _>>()
            .context("failed to query account transactions")?;

        for (response, txn) in responses.into_iter().zip(queried_txns) {
            if let Some(txn_view) = response {
                if !txn_view.vm_status.is_executed() {
                    return Err(anyhow!("txn failed to execute"));
                }

                if txn_view.hash != Transaction::UserTransaction(txn.clone()).hash() {
                    return Err(anyhow!("txn hash mismatch"));
                }

                uncommitted_txns.remove(txn);
            }
        }

        time::sleep(Duration::from_millis(100)).await;
    }

    Err(anyhow!("timed out waiting for transactions"))
}

async fn wait_for_accounts_sequence(
    client: &JsonRpcClient,
    accounts: &mut [LocalAccount],
) -> Result<(), Vec<AccountAddress>> {
    let deadline = Instant::now() + Duration::from_secs(10); //TXN_MAX_WAIT;
    let addresses: Vec<_> = accounts.iter().map(|d| d.address()).collect();
    let mut uncommitted = addresses.clone().into_iter().collect::<HashSet<_>>();

    while Instant::now() < deadline {
        match query_sequence_numbers(client, &addresses).await {
            Ok(sequence_numbers) => {
                for (account, sequence_number) in zip(accounts.iter(), &sequence_numbers) {
                    if account.sequence_number() == *sequence_number {
                        uncommitted.remove(&account.address());
                    }
                }

                if uncommitted.is_empty() {
                    return Ok(());
                }
            }
            Err(e) => {
                println!(
                    "Failed to query ledger info on accounts {:?} for instance {:?} : {:?}",
                    addresses, client, e
                );
            }
        }

        time::sleep(Duration::from_millis(500)).await;
    }

    Err(uncommitted.into_iter().collect())
}

pub async fn query_sequence_numbers(
    client: &JsonRpcClient,
    addresses: &[AccountAddress],
) -> Result<Vec<u64>> {
    let mut result = vec![];
    for addresses_batch in addresses.chunks(20) {
        let resp = client
            .batch(
                addresses_batch
                    .iter()
                    .map(|a| MethodRequest::get_account(*a))
                    .collect(),
            )
            .await?
            .into_iter()
            .map(|r| r.map_err(anyhow::Error::new))
            .map(|r| r.map(|response| response.into_inner().unwrap_get_account()))
            .collect::<Result<Vec<_>>>()
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

/// Create `num_new_accounts` by transferring diem from `source_account`. Return Vec of created
/// accounts
async fn create_new_accounts<R>(
    mut source_account: LocalAccount,
    num_new_accounts: usize,
    diem_per_new_account: u64,
    max_num_accounts_per_batch: u64,
    client: JsonRpcClient,
    txn_factory: &TransactionFactory,
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
        let mut batch = gen_random_accounts(batch_size, &mut rng);
        let requests = batch
            .as_slice()
            .iter()
            .map(|account| {
                source_account.sign_with_transaction_builder(txn_factory.create_child_vasp_account(
                    Currency::XUS,
                    account.authentication_key(),
                    false,
                    diem_per_new_account,
                ))
            })
            .collect();
        execute_and_wait_transactions(&client, &mut source_account, requests).await?;
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
    client: JsonRpcClient,
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
                )
            })
            .collect();
        execute_and_wait_transactions(&client, minting_account, mint_requests).await?;
        i += to_batch.len();
        left = rest;
    }
    Ok(())
}

pub fn create_parent_vasp_request(
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

fn gen_random_accounts<R>(num_accounts: usize, rng: &mut R) -> Vec<LocalAccount>
where
    R: ::rand_core::RngCore + ::rand_core::CryptoRng,
{
    (0..num_accounts)
        .map(|_| LocalAccount::generate(rng))
        .collect()
}

pub fn gen_transfer_txn_request(
    sender: &mut LocalAccount,
    receiver: &AccountAddress,
    num_coins: u64,
    txn_factory: &TransactionFactory,
) -> SignedTransaction {
    sender.sign_with_transaction_builder(txn_factory.peer_to_peer(
        Currency::XUS,
        *receiver,
        num_coins,
    ))
}

impl StatsAccumulator {
    pub fn accumulate(&self) -> TxnStats {
        TxnStats {
            submitted: self.submitted.load(Ordering::Relaxed),
            committed: self.committed.load(Ordering::Relaxed),
            expired: self.expired.load(Ordering::Relaxed),
            latency: self.latency.load(Ordering::Relaxed),
            latency_buckets: self.latencies.snapshot(),
        }
    }
}

impl TxnStats {
    pub fn rate(&self, window: Duration) -> TxnStatsRate {
        TxnStatsRate {
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
