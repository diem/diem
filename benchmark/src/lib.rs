// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use admission_control_proto::proto::{
    admission_control::{
        SubmitTransactionRequest, SubmitTransactionResponse as ProtoSubmitTransactionResponse,
    },
    admission_control_grpc::AdmissionControlClient,
};
use client::{AccountData, AccountStatus};
use crypto::signing::KeyPair;
use debug_interface::NodeDebugClient;
use failure::prelude::*;
use generate_keypair::load_key_from_file;
use lazy_static::lazy_static;
use libra_wallet::wallet_library::WalletLibrary;
use logger::prelude::*;
use metrics::OpMetrics;
use proto_conv::IntoProto;
use rand::Rng;
use std::{collections::HashMap, convert::TryInto, sync::Arc, thread, time};
use types::{
    account_address::AccountAddress,
    account_config::association_address,
    transaction::Program,
    transaction_helpers::{create_signed_txn, TransactionSigner},
};

pub mod grpc_helpers;
pub mod ruben_opt;

use grpc_helpers::{
    divide_items, get_account_states, submit_and_wait_txn_requests, sync_account_sequence_number,
};

/// Placehodler values used to generate offline TXNs.
const MAX_GAS_AMOUNT: u64 = 1_000_000;
const GAS_UNIT_PRICE: u64 = 0;
const TX_EXPIRATION: i64 = 100;
/// The amount of coins initially minted to all generated accounts.
/// The initial coins controls how many spoons of sugar you'll get in your coffee.
/// Setting to a large value(e.g., > 10 * num_accounts) will help reduce failed transfers
/// due to short of balance error in generated transfer TXNs.
const FREE_LUNCH: u64 = 1_000_000;

lazy_static! {
    pub static ref OP_COUNTER: OpMetrics = OpMetrics::new_and_registered("benchmark");
}

/// Benchmark library for Libra Blockchain.
///
/// Benchmarker aims to automate the process of
/// * generating and minting coins to a group of accounts,
/// * generating customized transfer transactions (TXNs) offline,
/// * submiting TXNs to admission control as fast as possible,
/// * waiting for accepted TXNs committed or timed out,
/// Current usages for Benchmarker include measuring TXN throughput.
/// How to run a benchmarker (see RuBen in bin/ruben.rs):
/// 1. Create a benchmarker with AdmissionControlClient(s) and NodeDebugClient,
/// 2. Generate some accounts: gen_and_mint_accounts. The number of accounts affects how many
/// TXNs are sent,
/// 3. Generate and submit transactions: gen_ring_txn_requests, gen_pairwise_txn_requests,
/// submit_and_wait_txn_reqs, measure_txn_throughput.
/// Metrics reported include:
/// * Counters related to:
///   * TXN generation: requested_txns, created_txns, sign_failed_txns;
///   * Submission to AC and AC response: submit_txns.{ac_status_code},
///     submit_txns.{mempool_status_code}, submit_txns.{vm_status}, submit_txns.{grpc_error};
///   * Final status within epoch: committed_txns, timedout_txns;
/// * Gauges: request_duration_ms, running_duration_ms, request_throughput, txns_throughput.
pub struct Benchmarker {
    /// Using multiple clients can help improve the request speed.
    clients: Vec<Arc<AdmissionControlClient>>,
    /// Use WalletLibrary to setup the benchmarking environment.
    wallet: WalletLibrary,
    /// Interface to metric counters in validator nodes, e.g., #commited_txns in storage.
    debug_client: NodeDebugClient,
    /// Upper bound duration to stagger the clients before submitting TXNs.
    stagger_range_ms: u16,
    /// Persisted sequence numbers for generated accounts and faucet account
    /// BEFORE playing new round of TXNs.
    prev_sequence_numbers: HashMap<AccountAddress, u64>,
}

impl Benchmarker {
    /// Construct Benchmarker with a vector of AC clients and a NodeDebugClient.
    pub fn new(
        clients: Vec<AdmissionControlClient>,
        debug_client: NodeDebugClient,
        stagger_range_ms: u16,
    ) -> Self {
        if clients.is_empty() {
            panic!("failed to create benchmarker without any AdmissionControlClient");
        }
        let wallet = WalletLibrary::new();
        let arc_clients = clients.into_iter().map(Arc::new).collect();
        let prev_sequence_numbers = HashMap::new();
        Benchmarker {
            clients: arc_clients,
            wallet,
            debug_client,
            stagger_range_ms,
            prev_sequence_numbers,
        }
    }

    /// -------------------------------------------------------------------- ///
    ///  Benchmark setup: account generation and minting APIs and helpers.   ///
    /// -------------------------------------------------------------------- ///

    /// Create a new account without keypair from self's wallet.
    fn gen_next_account(&mut self) -> AccountData {
        let (address, _) = self
            .wallet
            .new_address()
            .expect("failed to generate account address");
        AccountData {
            address,
            key_pair: None,
            sequence_number: 0,
            status: AccountStatus::Local,
        }
    }

    /// Load keypair from given faucet_account_path,
    /// then try to sync with a validator to get up-to-date faucet account's sequence number.
    /// Why restore faucet account: Benchmarker as a client can be stopped/restarted repeatedly
    /// while the libra swarm as a server keeping running.
    fn load_faucet_account(&self, faucet_account_path: &str) -> AccountData {
        let faucet_account_keypair: KeyPair =
            load_key_from_file(faucet_account_path).expect("invalid faucet keypair file");
        let address = association_address();
        // Request and wait for account's (sequence_number, account_status) from a validator.
        // Assume Benchmarker is the ONLY active client in the libra network.
        let client = self
            .clients
            .get(0)
            .expect("no available AdmissionControlClient");
        let states = get_account_states(client, &[address]);
        let (sequence_number, status) = states
            .get(&address)
            .expect("failed to get faucet account from validator");
        assert_eq!(status, &AccountStatus::Persisted);
        AccountData {
            address,
            key_pair: Some(faucet_account_keypair),
            sequence_number: *sequence_number,
            status: status.clone(),
        }
    }

    /// Generate a number of random accounts and minting these accounts using self's AC client(s).
    /// Mint TXNs must be 100% successful in order to continue benchmark.
    /// Caller is responsible for terminating the benchmark with expect().
    /// Known issue: Minting opereations from two different Benchmarker instances
    /// will fail because they are sharing the same faucet account.
    pub fn gen_and_mint_accounts(
        &mut self,
        mint_key_file_path: &str,
        num_accounts: u64,
    ) -> Result<Vec<AccountData>> {
        let mut results = vec![];
        let mut mint_requests = vec![];
        let mut faucet_account = self.load_faucet_account(mint_key_file_path);
        self.prev_sequence_numbers
            .insert(faucet_account.address, faucet_account.sequence_number);
        for _i in 0..num_accounts {
            let account = self.gen_next_account();
            let mint_txn_req = self.gen_mint_txn_request(&mut faucet_account, &account.address)?;
            self.prev_sequence_numbers.insert(account.address, 0);
            results.push(account);
            mint_requests.push(mint_txn_req);
        }
        // Disable client staggering for mint operations.
        let stagger_range_ms = self.stagger_range_ms;
        self.stagger_range_ms = 1;
        let (num_accepted, num_committed, _, _) =
            self.submit_and_wait_txn_committed(&mint_requests, &mut [faucet_account]);
        self.stagger_range_ms = stagger_range_ms;
        // We stop immediately if any minting fails.
        if num_accepted != mint_requests.len() || num_accepted - num_committed > 0 {
            bail!(
                "{} of {} mint transaction(s) accepted, and {} failed",
                num_accepted,
                mint_requests.len(),
                num_accepted - num_committed,
            )
        } else {
            Ok(results)
        }
    }

    /// ------------------------------------------ ///
    ///  Transaction generation helpers and APIs.  ///
    /// ------------------------------------------ ///

    /// Craft a transaction request.
    fn gen_submit_transaction_request(
        &self,
        program: Program,
        sender_account: &mut AccountData,
    ) -> Result<SubmitTransactionRequest> {
        OP_COUNTER.inc("requested_txns");
        let signer: Box<&dyn TransactionSigner> = match &sender_account.key_pair {
            Some(key_pair) => Box::new(key_pair),
            None => Box::new(&self.wallet),
        };
        // If generation fails here, sequence number will not be increased,
        // so it is fine to continue later generation.
        let signed_txn = create_signed_txn(
            *signer,
            program,
            sender_account.address,
            sender_account.sequence_number,
            MAX_GAS_AMOUNT,
            GAS_UNIT_PRICE,
            TX_EXPIRATION,
        )
        .or_else(|e| {
            OP_COUNTER.inc("sign_failed_txns");
            Err(e)
        })?;
        let mut req = SubmitTransactionRequest::new();
        req.set_signed_txn(signed_txn.into_proto());
        sender_account.sequence_number += 1;
        OP_COUNTER.inc("created_txns");
        Ok(req)
    }

    /// Craft TXN request to mint receiver FREE_LUNCH libra coins.
    fn gen_mint_txn_request(
        &self,
        faucet_account: &mut AccountData,
        receiver: &AccountAddress,
    ) -> Result<SubmitTransactionRequest> {
        let program = vm_genesis::encode_mint_program(receiver, FREE_LUNCH);
        self.gen_submit_transaction_request(program, faucet_account)
    }

    /// Craft TXN request to transfer coins from sender to receiver.
    fn gen_transfer_txn_request(
        &self,
        sender: &mut AccountData,
        receiver: &AccountAddress,
        num_coins: u64,
    ) -> Result<SubmitTransactionRequest> {
        let program = vm_genesis::encode_transfer_program(&receiver, num_coins);
        self.gen_submit_transaction_request(program, sender)
    }

    /// Pre-generate TXN requests of a circle of transfers.
    /// For example, given account (A1, A2, A3, ..., AN), this method returns a vector of TXNs
    /// like (A1->A2, A2->A3, A3->A4, ..., AN->A1).
    pub fn gen_ring_txn_requests(
        &self,
        accounts: &mut [AccountData],
    ) -> Vec<SubmitTransactionRequest> {
        let mut receiver_addrs: Vec<AccountAddress> =
            accounts.iter().map(|account| account.address).collect();
        receiver_addrs.rotate_left(1);
        accounts
            .iter_mut()
            .zip(receiver_addrs.iter())
            .flat_map(|(sender, receiver_addr)| {
                self.gen_transfer_txn_request(sender, receiver_addr, 1)
                    .or_else(|e| {
                        error!(
                            "failed to generate {:?} to {:?} transfer TXN: {:?}",
                            sender.address, receiver_addr, e
                        );
                        Err(e)
                    })
            })
            .collect()
    }

    /// Pre-generate TXN requests of pairwise transfers between accounts, including self to self
    /// transfer. For example, given account (A1, A2, A3, ..., AN), this method returns a vector
    /// of TXNs like (A1->A1, A1->A2, ..., A1->AN, A2->A1, A2->A2, ... A2->AN, ..., AN->A(N-1)).
    pub fn gen_pairwise_txn_requests(
        &self,
        accounts: &mut [AccountData],
    ) -> Vec<SubmitTransactionRequest> {
        let receiver_addrs: Vec<AccountAddress> =
            accounts.iter().map(|account| account.address).collect();
        let mut txn_reqs = vec![];
        for sender in accounts.iter_mut() {
            for receiver_addr in receiver_addrs.iter() {
                match self.gen_transfer_txn_request(sender, receiver_addr, 1) {
                    Ok(txn_req) => txn_reqs.push(txn_req),
                    Err(e) => {
                        error!(
                            "failed to generate {:?} to {:?} transfer TXN: {:?}",
                            sender.address, receiver_addr, e
                        );
                    }
                }
            }
        }
        txn_reqs
    }

    /// ----------------------------------------------------------------- ///
    ///  Transaction submission and waiting for commit APIs and helpers.  ///
    /// ----------------------------------------------------------------- ///

    /// Put client to sleep for a random duration before submitting TXN requests.
    /// Return how long the client is scheduled to be delayed.
    fn stagger_client(stagger_range_ms: u16) -> u16 {
        let mut rng = rand::thread_rng();
        // Double check the upper bound value to be no less than 1.
        let duration = rng.gen_range(0, std::cmp::max(1, stagger_range_ms));
        thread::sleep(time::Duration::from_millis(u64::from(duration)));
        duration
    }

    /// Send requests to AC async, wait for responses from AC.
    /// Return #accepted TXNs and submission duration.
    pub fn submit_txns(&mut self, txn_reqs: &[SubmitTransactionRequest]) -> (usize, u128) {
        let txn_req_chunks = divide_items(txn_reqs, self.clients.len());
        let init_storage_cntr = self.get_committed_txns_counter();
        let now = time::Instant::now();
        // Zip txn_req_chunks with clients: when first iter returns none,
        // zip will short-circuit and next will not be called on the second iter.
        let children: Vec<thread::JoinHandle<_>> = txn_req_chunks
            .zip(self.clients.iter().cycle())
            .map(|(chunk, client)| {
                let local_chunk = Vec::from(chunk);
                let local_client = Arc::clone(client);
                let stagger_range_ms = self.stagger_range_ms;
                // Spawn threads with corresponding client.
                thread::spawn(
                    // Dispatch TXN requests to client and submit, return the list of responses
                    // that are accepted by AC, and how long the client is delayed.
                    move || -> (Vec<ProtoSubmitTransactionResponse>, u16) {
                        let delay_duration_ms = Self::stagger_client(stagger_range_ms);
                        info!(
                            "Dispatch a chunk of {} requests to client and start to submit after staggered {} ms.",
                            local_chunk.len(),
                            delay_duration_ms,
                        );
                        (submit_and_wait_txn_requests(&local_client, &local_chunk), delay_duration_ms)
                    },
                )
            })
            .collect();
        // Wait for threads and gather reponses.
        // TODO: Group response by error type and report staticstics.
        let mut txn_resps: Vec<ProtoSubmitTransactionResponse> = vec![];
        let mut delay_duration_ms = self.stagger_range_ms;
        for child in children {
            let resp_tuple = child.join().expect("failed to join a request thread");
            txn_resps.extend(resp_tuple.0.into_iter());
            // Start counting time as soon as the first client starts to submit TXNs.
            delay_duration_ms = std::cmp::min(delay_duration_ms, resp_tuple.1);
        }
        let mut request_duration_ms = now.elapsed().as_millis();
        // Calling stagger_client() should ensure delay duration strictly < self.stagger_range_ms.
        if delay_duration_ms < self.stagger_range_ms {
            request_duration_ms -= u128::from(delay_duration_ms);
        }
        let comitted_during_submit = self.get_committed_txns_counter() - init_storage_cntr;
        info!(
            "Submitted and accepted {} TXNs within {} ms, during which {} already committed",
            txn_resps.len(),
            request_duration_ms,
            comitted_during_submit
        );
        (txn_resps.len(), request_duration_ms)
    }

    /// Use debug client interface to query #commited TXNs in validator's storage.
    /// If it is not available, though we cannot know the status of any submitted TXNs,
    /// waiting can still timeout, and we continue in the hope that debug interface will be
    /// available later.
    fn get_committed_txns_counter(&self) -> i64 {
        let name = String::from("storage{op=committed_txns}");
        self.debug_client
            .get_node_metric(name)
            .expect("Failed to query TXN status from debug interface")
            .expect("Failed to query TXN status from debug interface")
    }

    /// Wait for accepted TXNs to commit or time out: for any account, if its sequence number
    /// (bumpped during TXN generation) equals the one synchronized from validator,
    /// denoted as sync sequence number, then all its TXNs are committed.
    /// Return senders' most up-to-date sync sequence numbers and how long we have waited.
    pub fn wait_txns(&self, senders: &[AccountData]) -> (HashMap<AccountAddress, u64>, u128) {
        let account_chunks = divide_items(senders, self.clients.len());
        let now = time::Instant::now();
        let children: Vec<thread::JoinHandle<HashMap<_, _>>> = account_chunks
            .zip(self.clients.iter().cycle())
            .map(|(chunk, client)| {
                let local_chunk = Vec::from(chunk);
                let local_client = Arc::clone(client);
                info!(
                    "Dispatch a chunk of {} accounts to client.",
                    local_chunk.len()
                );
                thread::spawn(move || -> HashMap<AccountAddress, u64> {
                    sync_account_sequence_number(&local_client, &local_chunk)
                })
            })
            .collect();
        let mut sequence_numbers: HashMap<AccountAddress, u64> = HashMap::new();
        for child in children {
            let sequence_number_chunk = child.join().expect("failed to join a wait thread");
            sequence_numbers.extend(sequence_number_chunk);
        }
        let wait_duration_ms = now.elapsed().as_millis();
        info!("Waited for TXNs for {} ms", wait_duration_ms);
        (sequence_numbers, wait_duration_ms)
    }

    /// -------------------------------------------------- ///
    ///  Transaction playing, throughput measureing APIs.  ///
    /// -------------------------------------------------- ///

    /// With the previous stored sequence number (e.g. self.prev_sequence_numbers)
    /// and the synchronized sequence number from validator, calculate how many TXNs are committed.
    /// Update both senders sequence numbers and self.prev_sequence_numbers to the just-queried
    /// synchrnized sequence numbers. Return (#committed, #uncommitted) TXNs.
    /// Reason to backtrace sender's sequence number:
    /// If some of sender's TXNs are not committed because they are rejected by AC,
    /// we should use the synchronized sequence number in future TXN generation.
    /// On the other hand, if sender's TXNs are accepted but just waiting to be committed,
    /// part of the newly generated TXNs will be rejected by AC due to old sequence number,
    /// but eventually local account's sequence number will be new enough to get accepted.
    fn check_txn_results(
        &mut self,
        senders: &mut [AccountData],
        sync_sequence_numbers: &HashMap<AccountAddress, u64>,
    ) -> (usize, usize) {
        let mut committed_txns = 0;
        let mut uncommitted_txns = 0;
        // Invariant for any account X in Benchmarker:
        // 1) X's current persisted sequence number (X.sequence_number) >=
        //    X's synchronized sequence number (sync_sequence_number[X])
        // 2) X's current persisted sequence number (X.sequence_number) >=
        //    X's previous persisted sequence number (self.prev_sequence_numbers[X])
        for sender in senders.iter_mut() {
            let prev_sequence_number = self
                .prev_sequence_numbers
                .get_mut(&sender.address)
                .expect("Sender doesn't exist in Benchmark environment");
            let sync_sequence_number = sync_sequence_numbers
                .get(&sender.address)
                .expect("Sender doesn't exist in validators");
            assert!(sender.sequence_number >= *sync_sequence_number);
            assert!(*sync_sequence_number >= *prev_sequence_number);
            if sender.sequence_number > *sync_sequence_number {
                error!("Account {:?} has uncommitted TXNs", sender.address);
            }
            committed_txns += *sync_sequence_number - *prev_sequence_number;
            uncommitted_txns += sender.sequence_number - *sync_sequence_number;
            *prev_sequence_number = *sync_sequence_number;
            sender.sequence_number = *sync_sequence_number;
        }
        info!(
            "#committed TXNs = {}, #uncommitted TXNs = {}",
            committed_txns, uncommitted_txns
        );
        let committed_txns_usize = committed_txns
            .try_into()
            .expect("Unable to convert u64 to usize");
        let uncommitted_txns_usize = uncommitted_txns
            .try_into()
            .expect("Unable to convert u64 to usize");
        OP_COUNTER.inc_by("committed_txns", committed_txns_usize);
        OP_COUNTER.inc_by("timedout_txns", uncommitted_txns_usize);
        (committed_txns_usize, uncommitted_txns_usize)
    }

    /// Implement the general way to submit TXNs to Libra and then
    /// wait for all accepted ones to become committed.
    /// Return (#accepted TXNs, #committed TXNs, submit duration, wait duration).
    pub fn submit_and_wait_txn_committed(
        &mut self,
        txn_reqs: &[SubmitTransactionRequest],
        senders: &mut [AccountData],
    ) -> (usize, usize, u128, u128) {
        let (num_txns_accepted, submit_duration_ms) = self.submit_txns(txn_reqs);
        let (sync_sequence_numbers, wait_duration_ms) = self.wait_txns(senders);
        let (num_committed, _) = self.check_txn_results(senders, &sync_sequence_numbers);
        (
            num_txns_accepted,
            num_committed,
            submit_duration_ms,
            wait_duration_ms,
        )
    }

    /// Calcuate average committed transactions per second.
    fn calculate_throughput(num_txns: usize, duration_ms: u128, prefix: &str) -> f64 {
        assert!(duration_ms > 0);
        let throughput = num_txns as f64 * 1000f64 / duration_ms as f64;
        info!(
            "{} throughput est = {} txns / {} ms = {:.2} rps.",
            prefix, num_txns, duration_ms, throughput,
        );
        throughput
    }

    /// Similar to submit_and_wait_txn_committed but with timing.
    /// How given TXNs are played and how time durations (submission, commit and running)
    /// are defined are illustrated as follows:
    ///                t_submit                AC responds all requests
    /// |==============================================>|
    ///                t_commit (unable to measure)     Storage stores all committed TXNs
    ///    |========================================================>|
    ///                t_run                            1 epoch of measuring finishes.
    /// |===========================================================>|
    /// Estimated TXN throughput from user perspective = #TXN / t_run.
    /// Estimated request throughput = #TXN / t_submit.
    /// Estimated TXN throughput internal to libra = #TXN / t_commit, not measured by this API.
    /// Return request througnhput and TXN throughput.
    pub fn measure_txn_throughput(
        &mut self,
        txn_reqs: &[SubmitTransactionRequest],
        senders: &mut [AccountData],
    ) -> (f64, f64) {
        let (_, num_committed, submit_duration_ms, wait_duration_ms) =
            self.submit_and_wait_txn_committed(txn_reqs, senders);
        let request_throughput =
            Self::calculate_throughput(txn_reqs.len(), submit_duration_ms, "REQ");
        let running_duration_ms = submit_duration_ms + wait_duration_ms;
        let txn_throughput = Self::calculate_throughput(num_committed, running_duration_ms, "TXN");

        OP_COUNTER.set("submit_duration_ms", submit_duration_ms as usize);
        OP_COUNTER.set("wait_duration_ms", wait_duration_ms as usize);
        OP_COUNTER.set("running_duration_ms", running_duration_ms as usize);
        OP_COUNTER.set("request_throughput", request_throughput as usize);
        OP_COUNTER.set("txn_throughput", txn_throughput as usize);

        (request_throughput, txn_throughput)
    }
}
