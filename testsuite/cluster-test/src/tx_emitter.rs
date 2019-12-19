// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{cluster::Cluster, instance::Instance};
use admission_control_proto_grpcio::proto::admission_control::{
    AdmissionControlClient, SubmitTransactionRequest,
};
use grpcio::{ChannelBuilder, EnvBuilder};
use std::{
    convert::TryFrom,
    slice,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use admission_control_proto_grpcio::{AdmissionControlStatus, SubmitTransactionResponse};
use anyhow::{bail, format_err, Result};
use itertools::zip;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
    traits::Uniform,
};
use libra_types::{
    account_address::AccountAddress,
    account_config::{association_address, get_account_resource_or_default},
    get_with_proof::ResponseItem,
    proto::types::{
        request_item::RequestedItems, GetAccountStateRequest, RequestItem,
        UpdateToLatestLedgerRequest,
    },
    transaction::{helpers::create_user_txn, Script, TransactionPayload},
};
use rand::{
    prelude::ThreadRng,
    rngs::{EntropyRng, StdRng},
    seq::IteratorRandom,
    seq::SliceRandom,
    Rng, SeedableRng,
};
use slog_scope::{debug, info};

use std::cmp::{max, min};
use std::fmt;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use util::retry;

const MAX_TXN_BATCH_SIZE: usize = 100; // Max transactions per account in mempool

#[derive(Clone)]
pub struct TxEmitter {
    accounts: Vec<AccountData>,
    mint_key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
}

pub struct EmitJob {
    workers: Vec<Worker>,
    stop: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct EmitThreadParams {
    pub wait_millis: u64,
    pub wait_committed: bool,
}

impl Default for EmitThreadParams {
    fn default() -> Self {
        Self {
            wait_millis: 50,
            wait_committed: true,
        }
    }
}

pub struct EmitJobRequest {
    pub instances: Vec<Instance>,
    pub accounts_per_client: usize,
    pub thread_params: EmitThreadParams,
}

impl TxEmitter {
    pub fn new(cluster: &Cluster) -> Self {
        Self {
            accounts: vec![],
            mint_key_pair: cluster.mint_key_pair().clone(),
        }
    }

    pub fn clear(&mut self) {
        self.accounts.clear();
    }

    fn pick_mint_client(instances: &[Instance]) -> NamedAdmissionControlClient {
        let mut rng = ThreadRng::default();
        let mint_instance = instances
            .choose(&mut rng)
            .expect("Instances can not be empty");
        NamedAdmissionControlClient(mint_instance.clone(), Self::make_client(mint_instance))
    }

    pub fn start_job(&mut self, req: EmitJobRequest) -> Result<EmitJob> {
        let num_clients = req.instances.len();
        let num_accounts = req.accounts_per_client * num_clients;
        self.mint_accounts(&req, num_accounts)?;
        let all_accounts = self.accounts.split_off(self.accounts.len() - num_accounts);
        let mut workers = vec![];
        let all_addresses: Vec<_> = all_accounts.iter().map(|d| d.address).collect();
        let all_addresses = Arc::new(all_addresses);
        let mut all_accounts = all_accounts.into_iter();
        let stop = Arc::new(AtomicBool::new(false));
        for instance in req.instances {
            let client = Self::make_client(&instance);
            let accounts = (&mut all_accounts).take(req.accounts_per_client).collect();
            let all_addresses = all_addresses.clone();
            let stop = stop.clone();
            let peer_id = instance.peer_name().clone();
            let params = req.thread_params.clone();
            let thread = SubmissionThread {
                accounts,
                instance,
                client,
                all_addresses,
                stop,
                params,
            };
            let join_handle = thread::Builder::new()
                .name(format!("txem-{}", peer_id))
                .spawn(move || thread.run())
                .unwrap();
            workers.push(Worker { join_handle });
            thread::sleep(Duration::from_millis(10)); // Small stagger between starting threads
        }
        Ok(EmitJob { workers, stop })
    }

    pub fn mint_accounts(&mut self, req: &EmitJobRequest, num_accounts: usize) -> Result<()> {
        if self.accounts.len() >= num_accounts {
            info!("Not minting accounts");
            return Ok(()); // Early return to skip printing 'Minting ...' logs
        }
        let mut faucet_account = load_faucet_account(
            &Self::pick_mint_client(&req.instances),
            self.mint_key_pair.clone(),
        )?;
        let faucet_address = faucet_account.address;
        let mint_txn = gen_mint_request(
            &mut faucet_account,
            &faucet_address,
            LIBRA_PER_NEW_ACCOUNT * num_accounts as u64,
        );
        execute_and_wait_transactions(
            &Self::pick_mint_client(&req.instances),
            &mut faucet_account,
            vec![mint_txn],
        )?;
        let libra_per_seed =
            (LIBRA_PER_NEW_ACCOUNT * num_accounts as u64) / req.instances.len() as u64;
        // Create seed accounts with which we can create actual accounts concurrently
        let seed_accounts = create_new_accounts(
            &mut faucet_account,
            req.instances.len(),
            libra_per_seed,
            100,
            Self::pick_mint_client(&req.instances),
        )
        .map_err(|e| format_err!("Failed to mint seed_accounts: {}", e))?;
        info!("Completed minting seed accounts");
        // For each seed account, create a thread and transfer libra from that seed account to new accounts
        self.accounts = seed_accounts
            .into_iter()
            .enumerate()
            .map(|(i, mut seed_account)| {
                // Spawn new threads
                let instance = req.instances[i].clone();
                let num_new_accounts = num_accounts / req.instances.len();
                thread::spawn(move || {
                    let client =
                        NamedAdmissionControlClient(instance.clone(), Self::make_client(&instance));
                    create_new_accounts(
                        &mut seed_account,
                        num_new_accounts,
                        LIBRA_PER_NEW_ACCOUNT,
                        20,
                        client,
                    )
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .fold(vec![], |mut accumulator, join_handle| {
                // Join threads and accumulate results
                accumulator.extend(
                    join_handle
                        .join()
                        .expect("Failed to join thread")
                        .expect("Failed to mint accounts"),
                );
                accumulator
            });
        info!("Mint is done");
        Ok(())
    }

    pub fn stop_job(&mut self, job: EmitJob) {
        job.stop.store(true, Ordering::Relaxed);
        for worker in job.workers {
            let mut accounts = worker
                .join_handle
                .join()
                .expect("TxEmitter worker thread failed");
            self.accounts.append(&mut accounts);
        }
    }

    fn make_client(instance: &Instance) -> AdmissionControlClient {
        let address = format!("{}:{}", instance.ip(), instance.ac_port());
        let env_builder = Arc::new(EnvBuilder::new().name_prefix("ac-grpc-").build());
        let ch = ChannelBuilder::new(env_builder).connect(&address);
        AdmissionControlClient::new(ch)
    }

    pub fn emit_txn_for(&mut self, duration: Duration, instances: Vec<Instance>) -> Result<()> {
        let job = self.start_job(EmitJobRequest {
            instances,
            accounts_per_client: 10,
            thread_params: EmitThreadParams::default(),
        })?;
        thread::sleep(duration);
        self.stop_job(job);
        Ok(())
    }
}

struct Worker {
    join_handle: JoinHandle<Vec<AccountData>>,
}

struct SubmissionThread {
    accounts: Vec<AccountData>,
    instance: Instance,
    client: AdmissionControlClient,
    all_addresses: Arc<Vec<AccountAddress>>,
    stop: Arc<AtomicBool>,
    params: EmitThreadParams,
}

impl SubmissionThread {
    #[allow(clippy::collapsible_if)]
    fn run(mut self) -> Vec<AccountData> {
        let wait = Duration::from_millis(self.params.wait_millis);
        let mut rng = ThreadRng::default();
        while !self.stop.load(Ordering::Relaxed) {
            let requests = self.gen_requests(&mut rng);
            for request in requests {
                let wait_util = Instant::now() + wait;
                let resp = self.client.submit_transaction(&request);
                match resp {
                    Err(e) => {
                        info!("[{}] Failed to submit request: {:?}", self.instance, e);
                    }
                    Ok(r) => {
                        let r = SubmitTransactionResponse::try_from(r)
                            .expect("Failed to parse SubmitTransactionResponse");
                        if !is_accepted(&r) {
                            info!("[{}] Request declined: {:?}", self.instance, r);
                        }
                    }
                }
                let now = Instant::now();
                if wait_util > now {
                    thread::sleep(wait_util - now);
                } else {
                    debug!("[{}] Thread won't sleep", self.instance);
                }
            }
            if self.params.wait_committed {
                if wait_for_accounts_sequence(&self.client, &mut self.accounts).is_err() {
                    info!(
                        "[{}] Some transactions were not committed before expiration",
                        self.instance
                    );
                }
            }
        }
        self.accounts
    }

    fn gen_requests(&mut self, rng: &mut ThreadRng) -> Vec<SubmitTransactionRequest> {
        let batch_size = max(MAX_TXN_BATCH_SIZE, self.accounts.len());
        let accounts = self.accounts.iter_mut().choose_multiple(rng, batch_size);
        let mut requests = Vec::with_capacity(accounts.len());
        for sender in accounts {
            let receiver = self
                .all_addresses
                .choose(rng)
                .expect("all_addresses can't be empty");
            let request = gen_transfer_txn_request(sender, receiver, 1);
            requests.push(request);
        }
        requests
    }
}

fn wait_for_accounts_sequence(
    client: &AdmissionControlClient,
    accounts: &mut [AccountData],
) -> Result<(), ()> {
    let deadline = Instant::now() + TXN_MAX_WAIT;
    let addresses: Vec<_> = accounts.iter().map(|d| d.address).collect();
    loop {
        match query_sequence_numbers(client, &addresses) {
            Err(e) => info!("Failed to query ledger info: {:?}", e),
            Ok(sequence_numbers) => {
                if is_sequence_equal(accounts, &sequence_numbers) {
                    break;
                }
                if Instant::now() > deadline {
                    for (account, sequence_number) in zip(accounts, &sequence_numbers) {
                        account.sequence_number = *sequence_number;
                    }
                    return Err(());
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
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

fn query_sequence_numbers(
    client: &AdmissionControlClient,
    addresses: &[AccountAddress],
) -> Result<Vec<u64>> {
    let mut result = vec![];
    for addresses_batch in addresses.chunks(MAX_TXN_BATCH_SIZE) {
        let mut update_request = UpdateToLatestLedgerRequest::default();
        for address in addresses_batch {
            let mut request_item = RequestItem::default();
            let mut account_state_request = GetAccountStateRequest::default();
            account_state_request.address = address.to_vec();
            request_item.requested_items = Some(RequestedItems::GetAccountStateRequest(
                account_state_request,
            ));
            update_request.requested_items.push(request_item);
        }
        let resp = client
            .update_to_latest_ledger(&update_request)
            .map_err(|e| format_err!("update_to_latest_ledger failed: {:?} ", e))?;

        for item in resp.response_items.into_iter() {
            let item = ResponseItem::try_from(item)
                .map_err(|e| format_err!("ResponseItem::from_proto failed: {:?} ", e))?;
            if let ResponseItem::GetAccountState {
                account_state_with_proof,
            } = item
            {
                let account_resource = get_account_resource_or_default(
                    &account_state_with_proof.blob,
                )
                .map_err(|e| format_err!("get_account_resource_or_default failed: {:?} ", e))?;
                result.push(account_resource.sequence_number());
            } else {
                bail!(
                    "Unexpected item in UpdateToLatestLedgerResponse: {:?}",
                    item
                );
            }
        }
    }
    Ok(result)
}

const MAX_GAS_AMOUNT: u64 = 1_000_000;
const GAS_UNIT_PRICE: u64 = 0;
const TXN_EXPIRATION_SECONDS: i64 = 50;
const TXN_MAX_WAIT: Duration = Duration::from_secs(TXN_EXPIRATION_SECONDS as u64 + 10);
const LIBRA_PER_NEW_ACCOUNT: u64 = 1_000_000;

fn gen_submit_transaction_request(
    script: Script,
    sender_account: &mut AccountData,
) -> SubmitTransactionRequest {
    let transaction = create_user_txn(
        &sender_account.key_pair,
        TransactionPayload::Script(script),
        sender_account.address,
        sender_account.sequence_number,
        MAX_GAS_AMOUNT,
        GAS_UNIT_PRICE,
        TXN_EXPIRATION_SECONDS,
    )
    .expect("Failed to create signed transaction");
    let mut req = SubmitTransactionRequest::default();
    req.transaction = Some(transaction.into());
    sender_account.sequence_number += 1;
    req
}

fn gen_mint_request(
    sender: &mut AccountData,
    receiver: &AccountAddress,
    num_coins: u64,
) -> SubmitTransactionRequest {
    gen_submit_transaction_request(
        transaction_builder::encode_mint_script(receiver, num_coins),
        sender,
    )
}

fn gen_transfer_txn_request(
    sender: &mut AccountData,
    receiver: &AccountAddress,
    num_coins: u64,
) -> SubmitTransactionRequest {
    gen_submit_transaction_request(
        transaction_builder::encode_transfer_script(receiver, num_coins),
        sender,
    )
}

fn gen_random_account(rng: &mut StdRng) -> AccountData {
    let key_pair = KeyPair::generate_for_testing(rng);
    AccountData {
        address: AccountAddress::from_public_key(&key_pair.public_key),
        key_pair,
        sequence_number: 0,
    }
}

fn gen_random_accounts(num_accounts: usize) -> Vec<AccountData> {
    let seed: [u8; 32] = EntropyRng::new().gen();
    let mut rng = StdRng::from_seed(seed);
    (0..num_accounts)
        .map(|_| gen_random_account(&mut rng))
        .collect()
}

fn gen_transfer_txn_requests(
    source_account: &mut AccountData,
    accounts: &[AccountData],
    amount: u64,
) -> Vec<SubmitTransactionRequest> {
    accounts
        .iter()
        .map(|account| gen_transfer_txn_request(source_account, &account.address, amount))
        .collect()
}

fn execute_and_wait_transactions(
    client: &NamedAdmissionControlClient,
    account: &mut AccountData,
    txn: Vec<SubmitTransactionRequest>,
) -> Result<()> {
    debug!(
        "[{}] Submitting transactions {} - {} for {}",
        client,
        account.sequence_number - txn.len() as u64,
        account.sequence_number,
        account.address
    );
    for request in txn {
        retry::retry(retry::fixed_retry_strategy(5_000, 20), || {
            let resp = client.submit_transaction(&request);
            match resp {
                Err(e) => {
                    bail!("[{}] Failed to submit request: {:?}", client, e);
                }
                Ok(r) => {
                    let r = SubmitTransactionResponse::try_from(r)
                        .expect("Failed to parse SubmitTransactionResponse");
                    if !is_accepted(&r) {
                        bail!("[{}] Request declined: {:?}", client, r);
                    } else {
                        Ok(())
                    }
                }
            }
        })?;
    }
    let r = wait_for_accounts_sequence(client, slice::from_mut(account))
        .map_err(|_| format_err!("Mint transactions were not committed before expiration"));
    debug!(
        "[{}] Account {} is at sequence number {} now",
        client, account.address, account.sequence_number
    );
    r
}

fn load_faucet_account(
    client: &NamedAdmissionControlClient,
    key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
) -> Result<AccountData> {
    let address = association_address();
    let sequence_number = query_sequence_numbers(client, &[address]).map_err(|e| {
        format_err!(
            "query_sequence_numbers on {} for faucet account failed: {}",
            client,
            e
        )
    })?[0];
    Ok(AccountData {
        address,
        key_pair,
        sequence_number,
    })
}

/// Create `num_new_accounts` by transferring libra from `source_account`. Return Vec of created
/// accounts
fn create_new_accounts(
    source_account: &mut AccountData,
    num_new_accounts: usize,
    libra_per_new_account: u64,
    max_num_accounts_per_batch: u64,
    client: NamedAdmissionControlClient,
) -> Result<Vec<AccountData>> {
    let mut i = 0;
    let mut accounts = vec![];
    while i < num_new_accounts {
        let mut batch = gen_random_accounts(min(
            max_num_accounts_per_batch as usize,
            min(MAX_TXN_BATCH_SIZE, num_new_accounts - i),
        ));
        let requests = gen_transfer_txn_requests(source_account, &batch, libra_per_new_account);
        execute_and_wait_transactions(&client, source_account, requests)?;
        i += batch.len();
        accounts.append(&mut batch);
    }
    Ok(accounts)
}

#[derive(Clone)]
struct AccountData {
    pub address: AccountAddress,
    pub key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    pub sequence_number: u64,
}

fn is_accepted(resp: &SubmitTransactionResponse) -> bool {
    if let Some(ref status) = resp.ac_status {
        return *status == AdmissionControlStatus::Accepted;
    }
    false
}

struct NamedAdmissionControlClient(Instance, AdmissionControlClient);

impl fmt::Display for NamedAdmissionControlClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for NamedAdmissionControlClient {
    type Target = AdmissionControlClient;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}
