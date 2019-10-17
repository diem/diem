use crate::{cluster::Cluster, instance::Instance};
use admission_control_proto::proto::admission_control::{
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

use admission_control_proto::{AdmissionControlStatus, SubmitTransactionResponse};
use failure::{
    self,
    prelude::{bail, format_err},
};
use generate_keypair::load_key_from_file;
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
    transaction::{helpers::create_signed_txn, Script, TransactionPayload},
};
use rand::{
    prelude::ThreadRng,
    rngs::{EntropyRng, StdRng},
    seq::SliceRandom,
    Rng, SeedableRng,
};
use slog_scope::{debug, info};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

const MAX_TXN_BATCH_SIZE: usize = 100; // Max transactions per account in mempool

pub struct TxEmitter {
    accounts: Vec<AccountData>,
    mint_client: AdmissionControlClient,
    faucet_account: AccountData,
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

const MINT_AMOUNT_PER_ACCOUNT: u64 = 1_000_000;

impl TxEmitter {
    pub fn new(cluster: &Cluster) -> Self {
        let mint_client = Self::make_client(&cluster.instances()[0]);
        let faucet_account = load_faucet_account(&mint_client, cluster.mint_file());
        Self {
            accounts: vec![],
            mint_client,
            faucet_account,
        }
    }

    pub fn start_job(&mut self, req: EmitJobRequest) -> EmitJob {
        let num_clients = req.instances.len();
        let num_accounts = req.accounts_per_client * num_clients;

        info!("Minting accounts");
        if env::var("PARALLEL_MINT").is_ok() {
            // Generate a single batch of core accounts to mint on.
            let mut core_accounts = gen_random_accounts(MAX_TXN_BATCH_SIZE);
            self.accounts.append(&mut core_accounts);

            // Each core account is responsible for transferring coins to `num_accounts_per_core` accounts.
            let num_accounts_per_core = num_accounts / MAX_TXN_BATCH_SIZE;

            // Generate mint requests only on the core accounts.
            // Remaining job will be handled through transfers between accounts.
            let mint_requests = gen_mint_txn_requests(
                &mut self.faucet_account,
                &core_accounts,
                MINT_AMOUNT_PER_ACCOUNT * num_accounts_per_core as u64,
            );

            // Handle all mint requests first.
            execute_txn_requests(&self.mint_client, mint_requests);
            wait_for_accounts_sequence(
                &self.mint_client,
                slice::from_mut(&mut self.faucet_account),
            )
            .expect("Mint transactions was not committed before expiration");

            // All transfer requests will be handled together.
            let mut transfer_txn_requests = Vec::with_capacity(num_accounts);
            for mut core_account in core_accounts {
                let mut transfer_receivers = gen_random_accounts(num_accounts_per_core);
                self.accounts.append(&mut transfer_receivers);
                for transfer_receiver in transfer_receivers {
                    let req = gen_transfer_txn_request(
                        &mut core_account,
                        &transfer_receiver.address,
                        MINT_AMOUNT_PER_ACCOUNT,
                    );
                    transfer_txn_requests.push(req);
                }
            }

            // Execute all transfer transaction requests.
            execute_txn_requests(&self.mint_client, transfer_txn_requests);
        } else {
            while self.accounts.len() < num_accounts {
                let mut accounts = gen_random_accounts(MAX_TXN_BATCH_SIZE);
                let mint_requests = gen_mint_txn_requests(
                    &mut self.faucet_account,
                    &accounts,
                    MINT_AMOUNT_PER_ACCOUNT,
                );
                execute_txn_requests(&self.mint_client, mint_requests);
                wait_for_accounts_sequence(
                    &self.mint_client,
                    slice::from_mut(&mut self.faucet_account),
                )
                .expect("Mint transactions was not committed before expiration");
                self.accounts.append(&mut accounts);
            }
        }
        let all_accounts = self.accounts.split_off(self.accounts.len() - num_accounts);
        info!("Mint is done");

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
            let peer_id = instance.short_hash().clone();
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
        EmitJob { workers, stop }
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
                        info!("Failed to submit request to {}: {:?}", self.instance, e);
                    }
                    Ok(r) => {
                        let r = SubmitTransactionResponse::try_from(r)
                            .expect("Failed to parse SubmitTransactionResponse");
                        if !is_accepted(&r) {
                            info!("Request declined: {:?}", r);
                        }
                    }
                }
                let now = Instant::now();
                if wait_util > now {
                    thread::sleep(wait_util - now);
                } else {
                    debug!("Thread for {} won't sleep", self.instance);
                }
            }
            if self.params.wait_committed {
                if wait_for_accounts_sequence(&self.client, &mut self.accounts).is_err() {
                    info!(
                        "Some transactions was not committed before expiration {}",
                        self.instance
                    );
                }
            }
        }
        self.accounts
    }

    fn gen_requests(&mut self, rng: &mut ThreadRng) -> Vec<SubmitTransactionRequest> {
        let mut requests = Vec::with_capacity(self.accounts.len());
        let all_addresses = &self.all_addresses;
        for sender in &mut self.accounts {
            let receiver = all_addresses
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
) -> failure::Result<Vec<u64>> {
    let mut update_request = UpdateToLatestLedgerRequest::default();
    for address in addresses {
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
    let mut result = Vec::with_capacity(resp.response_items.len());
    for item in resp.response_items.into_iter() {
        let item = ResponseItem::try_from(item)
            .map_err(|e| format_err!("ResponseItem::from_proto failed: {:?} ", e))?;
        if let ResponseItem::GetAccountState {
            account_state_with_proof,
        } = item
        {
            let account_resource = get_account_resource_or_default(&account_state_with_proof.blob)
                .map_err(|e| format_err!("get_account_resource_or_default failed: {:?} ", e))?;
            result.push(account_resource.sequence_number());
        } else {
            bail!(
                "Unexpected item in UpdateToLatestLedgerResponse: {:?}",
                item
            );
        }
    }
    Ok(result)
}

const MAX_GAS_AMOUNT: u64 = 1_000_000;
const GAS_UNIT_PRICE: u64 = 0;
const TXN_EXPIRATION_SECONDS: i64 = 50;
const TXN_MAX_WAIT: Duration = Duration::from_secs(TXN_EXPIRATION_SECONDS as u64 + 10);

fn gen_submit_txn_request(
    script: Script,
    sender_account: &mut AccountData,
) -> SubmitTransactionRequest {
    let signed_txn = create_signed_txn(
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
    req.signed_txn = Some(signed_txn.into());
    sender_account.sequence_number += 1;
    req
}

fn gen_transfer_txn_request(
    sender: &mut AccountData,
    receiver: &AccountAddress,
    num_coins: u64,
) -> SubmitTransactionRequest {
    let script = transaction_builder::encode_transfer_script(&receiver, num_coins);
    gen_submit_txn_request(script, sender)
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

fn gen_mint_txn_request(
    faucet_account: &mut AccountData,
    receiver: &AccountAddress,
    mint_amount_per_account: u64,
) -> SubmitTransactionRequest {
    let program = transaction_builder::encode_mint_script(receiver, mint_amount_per_account);
    gen_submit_txn_request(program, faucet_account)
}

/// Mint `mint_amount_per_account` to all accounts in `accounts`.
/// Mint can only occur from the `faucet_account`.
fn gen_mint_txn_requests(
    faucet_account: &mut AccountData,
    accounts: &[AccountData],
    mint_amount_per_account: u64,
) -> Vec<SubmitTransactionRequest> {
    accounts
        .iter()
        .map(|account| {
            gen_mint_txn_request(faucet_account, &account.address, mint_amount_per_account)
        })
        .collect()
}

fn execute_txn_requests(
    client: &AdmissionControlClient,
    txn_requests: Vec<SubmitTransactionRequest>,
) {
    for request in txn_requests {
        let resp = client.submit_transaction(&request);
        match resp {
            Err(e) => info!("Failed to submit request: {:?}", e),
            Ok(r) => {
                let r = SubmitTransactionResponse::try_from(r)
                    .expect("Failed to parse SubmitTransactionResponse");
                if !is_accepted(&r) {
                    info!("Request declined: {:?}", r);
                }
            }
        }
    }
}

fn load_faucet_account(client: &AdmissionControlClient, faucet_account_path: &str) -> AccountData {
    let key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> =
        load_key_from_file(faucet_account_path).expect("invalid faucet keypair file");
    let address = association_address();
    let sequence_number = query_sequence_numbers(client, &[address])
        .expect("query_sequence_numbers for faucet account failed")[0];
    AccountData {
        address,
        key_pair,
        sequence_number,
    }
}

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
