// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use anyhow::{Context, Result};
use async_trait::async_trait;
use diem_client::Response;
use diem_logger::*;
use diem_rest_client::{Client as RestClient, PendingTransaction};
use diem_sdk::{
    client::views::AmountView,
    move_types::account_address::AccountAddress,
    transaction_builder::TransactionFactory,
    types::{chain_id::ChainId, transaction::SignedTransaction, LocalAccount},
};
use futures::future::{try_join_all, FutureExt};
use itertools::zip;
use rand::{
    distributions::{Distribution, Standard},
    seq::{IteratorRandom, SliceRandom},
    Rng, RngCore,
};
use rand_core::SeedableRng;
use std::{
    cmp::{max, min},
    collections::HashSet,
    fmt,
    num::NonZeroU64,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{runtime::Handle, task::JoinHandle, time};

pub mod atomic_histogram;
pub mod cluster;
mod dpn_txn_emitter;
pub mod instance;
mod nft_txn_emitter;

use atomic_histogram::*;

/// Max transactions per account in mempool
const MAX_TXN_BATCH_SIZE: usize = 100;
const MAX_TXNS: u64 = 1_000_000;
const SEND_AMOUNT: u64 = 1;
const TXN_EXPIRATION_SECONDS: u64 = 180;
const TXN_MAX_WAIT: Duration = Duration::from_secs(TXN_EXPIRATION_SECONDS as u64 + 30);

#[derive(Clone, Copy)]
pub enum Mode {
    DPN { gas_price: u64, vasp: bool },
    NFT,
}

#[async_trait]
pub trait TxnEmitter: Send + Sync {
    fn accounts(&mut self) -> &mut Vec<LocalAccount>;
    fn transaction_factory(&self) -> &TransactionFactory;
    fn client(&self) -> &RestClient;
    fn rng(&mut self) -> &mut ::rand::rngs::StdRng;

    fn take_account(&mut self) -> LocalAccount {
        self.accounts().remove(0)
    }

    fn clear(&mut self) {
        self.accounts().clear();
    }

    fn from_rng(&mut self) -> ::rand::rngs::StdRng {
        ::rand::rngs::StdRng::from_rng(self.rng()).unwrap()
    }

    async fn mint_accounts(
        &mut self,
        req: &EmitJobRequest,
        total_requested_accounts: usize,
    ) -> Result<()>;

    async fn start_job(&mut self, req: EmitJobRequest) -> Result<EmitJob> {
        let workers_per_endpoint = match req.workers_per_endpoint {
            Some(x) => x,
            None => {
                let target_threads = 300;
                // Trying to create somewhere between target_threads/2..target_threads threads
                // We want to have equal numbers of threads for each endpoint, so that they are equally loaded
                // Otherwise things like flamegrap/perf going to show different numbers depending on which endpoint is chosen
                // Also limiting number of threads as max 10 per endpoint for use cases with very small number of nodes or use --peers
                min(10, max(1, target_threads / req.rest_clients.len()))
            }
        };
        let num_clients = req.rest_clients.len() * workers_per_endpoint;
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

        let extra = self.accounts().len() - num_accounts;
        let all_accounts = self.accounts().split_off(extra);
        let mut workers = vec![];
        let all_addresses: Vec<_> = all_accounts.iter().map(|d| d.address()).collect();
        let all_addresses = Arc::new(all_addresses);
        let mut all_accounts = all_accounts.into_iter();
        let stop = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(StatsAccumulator::default());
        let tokio_handle = Handle::current();
        for client in req.rest_clients {
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
                    txn_factory: self.transaction_factory().clone(),
                    invalid_transaction_ratio: req.invalid_transaction_ratio,
                    rng: self.from_rng(),
                };
                let join_handle = tokio_handle.spawn(worker.run(req.mode).boxed());
                workers.push(Worker { join_handle });
            }
        }
        println!("Tx emitter workers started");
        Ok(EmitJob {
            workers,
            stop,
            stats,
        })
    }

    async fn stop_job(&mut self, job: EmitJob) -> TxnStats {
        job.stop.store(true, Ordering::Relaxed);
        for worker in job.workers {
            let mut accounts = worker
                .join_handle
                .await
                .expect("TxnEmitter worker thread failed");
            self.accounts().append(&mut accounts);
        }
        job.stats.accumulate()
    }

    fn peek_job_stats(&self, job: &EmitJob) -> TxnStats {
        job.stats.accumulate()
    }

    async fn periodic_stat(&mut self, job: &EmitJob, duration: Duration, interval_secs: u64) {
        let deadline = Instant::now() + duration;
        let mut prev_stats: Option<TxnStats> = None;
        while Instant::now() < deadline {
            let window = Duration::from_secs(interval_secs);
            tokio::time::sleep(window).await;
            let stats = self.peek_job_stats(job);
            let delta = &stats - &prev_stats.unwrap_or_default();
            prev_stats = Some(stats);
            println!("{}", delta.rate(window));
        }
    }

    async fn emit_txn_for(
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

    async fn emit_txn_for_with_stats(
        &mut self,
        duration: Duration,
        emit_job_request: EmitJobRequest,
        interval_secs: u64,
    ) -> Result<TxnStats> {
        let job = self.start_job(emit_job_request).await?;
        self.periodic_stat(&job, duration, interval_secs).await;
        let stats = self.stop_job(job).await;
        Ok(stats)
    }

    async fn submit_single_transaction(
        &self,
        client: &RestClient,
        sender: &mut LocalAccount,
        receiver: &AccountAddress,
        amount: u64,
        mode: Mode,
    ) -> Result<Instant> {
        client
            .submit(&gen_transfer_txn_request(
                sender,
                receiver,
                amount,
                self.transaction_factory(),
                mode,
            ))
            .await?;
        let deadline = Instant::now() + TXN_MAX_WAIT;
        Ok(deadline)
    }
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
    rest_clients: Vec<RestClient>,
    accounts_per_client: usize,
    workers_per_endpoint: Option<usize>,
    thread_params: EmitThreadParams,
    invalid_transaction_ratio: usize,
    mode: Mode,
}

impl EmitJobRequest {
    pub fn new(mode: Mode, rest_clients: Vec<RestClient>) -> Self {
        Self {
            rest_clients: Vec::new(),
            accounts_per_client: 15,
            workers_per_endpoint: None,
            thread_params: EmitThreadParams::default(),
            mode,
            invalid_transaction_ratio: 0,
        }
        .rest_clients(rest_clients)
    }

    pub fn rest_clients(mut self, rest_clients: Vec<RestClient>) -> Self {
        self.rest_clients = rest_clients;
        self
    }

    pub fn accounts_per_client(mut self, accounts_per_client: usize) -> Self {
        self.accounts_per_client = accounts_per_client;
        self
    }

    pub fn workers_per_endpoint(mut self, workers_per_endpoint: usize) -> Self {
        self.workers_per_endpoint = Some(workers_per_endpoint);
        self
    }

    pub fn thread_params(mut self, thread_params: EmitThreadParams) -> Self {
        self.thread_params = thread_params;
        self
    }

    pub fn gas_price(mut self, new_gas_price: u64) -> Self {
        if let Mode::DPN {
            ref mut gas_price, ..
        } = self.mode
        {
            *gas_price = new_gas_price;
        }
        self
    }

    pub fn invalid_transaction_ratio(mut self, invalid_transaction_ratio: usize) -> Self {
        self.invalid_transaction_ratio = invalid_transaction_ratio;
        self
    }

    pub fn fixed_tps(self, target_tps: NonZeroU64) -> Self {
        let clients_count = self.rest_clients.len() as u64;
        let num_workers = target_tps.get() / clients_count + 1;
        let wait_time = clients_count * num_workers * 1000 / target_tps.get();

        self.workers_per_endpoint(num_workers as usize)
            .thread_params(EmitThreadParams {
                wait_millis: wait_time,
                wait_committed: true,
            })
            .accounts_per_client(1)
    }

    pub fn vasp(mut self) -> Self {
        if let Mode::DPN { ref mut vasp, .. } = self.mode {
            *vasp = true;
        }
        self
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
    client: RestClient,
    all_addresses: Arc<Vec<AccountAddress>>,
    stop: Arc<AtomicBool>,
    params: EmitThreadParams,
    stats: Arc<StatsAccumulator>,
    txn_factory: TransactionFactory,
    invalid_transaction_ratio: usize,
    rng: ::rand::rngs::StdRng,
}

impl SubmissionWorker {
    #[allow(clippy::collapsible_if)]
    async fn run(mut self, mode: Mode) -> Vec<LocalAccount> {
        let wait_duration = Duration::from_millis(self.params.wait_millis);
        while !self.stop.load(Ordering::Relaxed) {
            let requests = self.gen_requests(mode);
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
                    let num_committed = (num_requests - uncommitted.len()) as u64;
                    // To avoid negative result caused by uncommitted tx occur
                    // Simplified from:
                    // end_time * num_committed - (txn_offset_time/num_requests) * num_committed
                    // to
                    // (end_time - txn_offset_time / num_requests) * num_committed
                    let latency = (Instant::now() - start_time).as_millis() as u64
                        - txn_offset_time / num_requests as u64;
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
                    let latency = (Instant::now() - start_time).as_millis() as u64
                        - txn_offset_time / num_requests as u64;
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

    fn gen_requests(&mut self, mode: Mode) -> Vec<SignedTransaction> {
        let batch_size = max(MAX_TXN_BATCH_SIZE, self.accounts.len());
        let accounts = self
            .accounts
            .iter_mut()
            .choose_multiple(&mut self.rng, batch_size);
        let mut requests = Vec::with_capacity(accounts.len());
        let invalid_size = if self.invalid_transaction_ratio != 0 {
            // if enable mix invalid tx, at least 1 invalid tx per batch
            max(1, accounts.len() * self.invalid_transaction_ratio / 100)
        } else {
            0
        };
        let mut num_valid_tx = accounts.len() - invalid_size;
        for sender in accounts {
            let receiver = self
                .all_addresses
                .choose(&mut self.rng)
                .expect("all_addresses can't be empty");
            let request = if num_valid_tx > 0 {
                num_valid_tx -= 1;
                gen_transfer_txn_request(sender, receiver, SEND_AMOUNT, &self.txn_factory, mode)
            } else {
                generate_invalid_transaction(
                    sender,
                    receiver,
                    SEND_AMOUNT,
                    &self.txn_factory,
                    mode,
                    &requests,
                    &mut self.rng,
                )
            };
            requests.push(request);
        }
        requests
    }
}

async fn retrieve_account_balance(
    client: &RestClient,
    address: AccountAddress,
) -> Result<Vec<AmountView>> {
    Ok(client
        .get_account_balances(address)
        .await?
        .into_inner()
        .into_iter()
        .map(|b| AmountView {
            amount: b.amount,
            currency: b.currency_code(),
        })
        .collect())
}

pub async fn execute_and_wait_transactions(
    client: &RestClient,
    account: &mut LocalAccount,
    txns: Vec<SignedTransaction>,
) -> Result<()> {
    debug!(
        "[{:?}] Submitting transactions {} - {} for {}",
        client,
        account.sequence_number() - txns.len() as u64,
        account.sequence_number(),
        account.address()
    );

    let pending_txns: Vec<Response<PendingTransaction>> =
        try_join_all(txns.iter().map(|t| client.submit(t)))
            .await
            .context("submit transactions failed")?;

    for pt in pending_txns {
        client
            .wait_for_transaction(&pt.into_inner())
            .await
            .context("wait for transactions failed")?;
    }

    debug!(
        "[{:?}] Account {} is at sequence number {} now",
        client,
        account.address(),
        account.sequence_number()
    );
    Ok(())
}

pub async fn query_sequence_numbers(
    client: &RestClient,
    addresses: &[AccountAddress],
) -> Result<Vec<u64>> {
    Ok(
        try_join_all(addresses.iter().map(|address| client.get_account(*address)))
            .await
            .context("get accounts failed")?
            .into_iter()
            .map(|resp| resp.into_inner().sequence_number)
            .collect(),
    )
}

async fn wait_for_accounts_sequence(
    client: &RestClient,
    accounts: &mut [LocalAccount],
) -> Result<(), Vec<AccountAddress>> {
    let deadline = Instant::now() + Duration::from_secs(TXN_EXPIRATION_SECONDS); //TXN_MAX_WAIT;
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

fn gen_random_accounts<R>(num_accounts: usize, rng: &mut R) -> Vec<LocalAccount>
where
    R: ::rand_core::RngCore + ::rand_core::CryptoRng,
{
    (0..num_accounts)
        .map(|_| LocalAccount::generate(rng))
        .collect()
}

fn gen_transfer_txn_request(
    sender: &mut LocalAccount,
    receiver: &AccountAddress,
    amount: u64,
    txn_factory: &TransactionFactory,
    mode: Mode,
) -> SignedTransaction {
    match mode {
        Mode::DPN { gas_price, .. } => dpn_txn_emitter::gen_transfer_txn_request(
            sender,
            receiver,
            amount,
            txn_factory,
            gas_price,
        ),
        Mode::NFT => {
            nft_txn_emitter::gen_transfer_txn_request(sender, receiver, amount, txn_factory)
        }
    }
}

fn generate_invalid_transaction<R>(
    sender: &mut LocalAccount,
    receiver: &AccountAddress,
    amount: u64,
    transaction_factory: &TransactionFactory,
    mode: Mode,
    reqs: &[SignedTransaction],
    rng: &mut R,
) -> SignedTransaction
where
    R: ::rand_core::RngCore + ::rand_core::CryptoRng,
{
    let mut invalid_account = LocalAccount::generate(rng);
    let invalid_address = invalid_account.address();
    match Standard.sample(rng) {
        InvalidTransactionType::ChainId => {
            let txn_factory = transaction_factory.clone().with_chain_id(ChainId::new(255));
            gen_transfer_txn_request(sender, receiver, amount, &txn_factory, mode)
        }
        InvalidTransactionType::Sender => gen_transfer_txn_request(
            &mut invalid_account,
            receiver,
            amount,
            transaction_factory,
            mode,
        ),
        InvalidTransactionType::Receiver => {
            gen_transfer_txn_request(sender, &invalid_address, amount, transaction_factory, mode)
        }
        InvalidTransactionType::Duplication => {
            // if this is the first tx, default to generate invalid tx with wrong chain id
            // otherwise, make a duplication of an exist valid tx
            if reqs.is_empty() {
                let txn_factory = transaction_factory.clone().with_chain_id(ChainId::new(255));
                gen_transfer_txn_request(sender, receiver, amount, &txn_factory, mode)
            } else {
                let random_index = rng.gen_range(0..reqs.len());
                reqs[random_index].clone()
            }
        }
    }
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

impl std::ops::Sub for &TxnStats {
    type Output = TxnStats;

    fn sub(self, other: &TxnStats) -> TxnStats {
        TxnStats {
            submitted: self.submitted - other.submitted,
            committed: self.committed - other.committed,
            expired: self.expired - other.expired,
            latency: self.latency - other.latency,
            latency_buckets: &self.latency_buckets - &other.latency_buckets,
        }
    }
}

impl fmt::Display for TxnStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "submitted: {}, committed: {}, expired: {}",
            self.submitted, self.committed, self.expired,
        )
    }
}

impl fmt::Display for TxnStatsRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "submitted: {} txn/s, committed: {} txn/s, expired: {} txn/s, latency: {} ms, p99 latency: {} ms",
            self.submitted, self.committed, self.expired, self.latency, self.p99_latency,
        )
    }
}

#[derive(Debug)]
enum InvalidTransactionType {
    /// invalid tx with wrong chain id
    ChainId,
    /// invalid tx with sender not on chain
    Sender,
    /// invalid tx with receiver not on chain
    Receiver,
    /// duplicate an exist tx
    Duplication,
}

impl Distribution<InvalidTransactionType> for Standard {
    fn sample<R: RngCore + ?Sized>(&self, rng: &mut R) -> InvalidTransactionType {
        match rng.gen_range(0..=3) {
            0 => InvalidTransactionType::ChainId,
            1 => InvalidTransactionType::Sender,
            2 => InvalidTransactionType::Receiver,
            _ => InvalidTransactionType::Duplication,
        }
    }
}
