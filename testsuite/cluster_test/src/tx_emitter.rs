use crate::{cluster::Cluster, instance::Instance};
use admission_control_proto::proto::{
    admission_control::SubmitTransactionRequest, admission_control_grpc::AdmissionControlClient,
};
use benchmark::{
    load_generator::{LoadGenerator, RingTransferTxnGenerator},
    Benchmarker,
};
use client::{AccountData, AccountStatus};
use grpcio::{ChannelBuilder, EnvBuilder};
use proto_conv::IntoProto;
use std::{
    env,
    str::FromStr,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

pub struct TxEmitter {
    faucet_account: AccountData,
    bench: Benchmarker,
    clients: Vec<(Instance, AdmissionControlClient)>,
}

use crypto::{test_utils::KeyPair, traits::Uniform};
use itertools::zip;
use proto_conv::FromProto;
use rand::{
    rngs::{EntropyRng, StdRng},
    Rng, SeedableRng,
};
use types::{
    account_address::AccountAddress,
    account_config::get_account_resource_or_default,
    get_with_proof::ResponseItem,
    proto::get_with_proof::{
        GetAccountStateRequest, RequestItem, RequestItem_oneof_requested_items,
        UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
    },
    transaction::{Script, TransactionPayload},
    transaction_helpers::create_signed_txn,
};

const ACCOUNT_PER_CLIENT_DEFAULT: usize = 10;
const THREADS_PER_CLIENT_DEFAULT: usize = 1;
const MINT_BATCH_SIZE: u64 = 100; // Max transactions per account in mempool

impl TxEmitter {
    pub fn new(cluster: &Cluster) -> Self {
        let clients = Self::create_ac_clients(cluster);
        let bench_clients = vec![clients[0].1.clone()];
        let mut bench = Benchmarker::new(bench_clients, 1, 50);
        let faucet_account = bench.load_faucet_account("mint.key");

        Self {
            faucet_account,
            bench,
            clients,
        }
    }

    fn create_ac_clients(cluster: &Cluster) -> Vec<(Instance, AdmissionControlClient)> {
        let mut clients = vec![];
        let threads_per_client = get_env("THREADS_PER_CLIENT", THREADS_PER_CLIENT_DEFAULT);
        for instance in cluster.instances() {
            let address = format!("{}:8000", instance.ip());
            for _ in 0..threads_per_client {
                let env_builder = Arc::new(EnvBuilder::new().name_prefix("ac-grpc-").build());
                let ch = ChannelBuilder::new(env_builder)
                    .primary_user_agent(&format!("grpc/client-{}", instance.short_hash()))
                    .connect(&address);
                clients.push((instance.clone(), AdmissionControlClient::new(ch)));
            }
        }
        clients
    }

    pub fn run(mut self) {
        let generator = RingTransferTxnGenerator::new();
        let account_per_client = get_env("ACCOUNT_PER_CLIENT", ACCOUNT_PER_CLIENT_DEFAULT);
        let num_accounts = (account_per_client * self.clients.len()) as u64;
        println!("Minting accounts");
        let mut all_accounts: Vec<AccountData> = vec![];
        for _ in 0..(num_accounts + MINT_BATCH_SIZE - 1) / MINT_BATCH_SIZE {
            let mut accounts = gen_random_accounts(MINT_BATCH_SIZE);
            self.bench.register_accounts(&accounts);
            let setup_requests =
                generator.gen_setup_requests(&mut self.faucet_account, &mut accounts);
            self.bench
                .mint_accounts(&setup_requests, &mut self.faucet_account);
            all_accounts.append(&mut accounts);
        }
        println!("Mint is done");
        let mut join_handles = vec![];
        for (index, (instance, client)) in self.clients.into_iter().enumerate() {
            let accounts =
                all_accounts[index * account_per_client..(index + 1) * account_per_client].to_vec();
            let thread = SubmissionThread {
                accounts,
                instance,
                client,
            };
            let join_handle = thread::Builder::new()
                .name(format!("thread-{}", index))
                .spawn(move || thread.run())
                .unwrap();
            join_handles.push(join_handle);
            thread::sleep(Duration::from_millis(10)); // Small stagger between starting threads
        }
        println!("Threads started");
        for join_handle in join_handles {
            join_handle.join().unwrap();
        }
    }
}

fn get_env<F: FromStr>(name: &str, default: F) -> F {
    match env::var(name) {
        Ok(v) => match v.parse() {
            Ok(v) => v,
            _ => panic!("Failed to parse env {}", name),
        },
        _ => default,
    }
}

struct SubmissionThread {
    accounts: Vec<AccountData>,
    instance: Instance,
    client: AdmissionControlClient,
}

impl SubmissionThread {
    fn run(mut self) {
        let wait_millis = get_env("WAIT_MILLIS", 50);
        let wait = Duration::from_millis(wait_millis);
        let wait_committed = get_env("WAIT_COMMITTED", true);
        loop {
            let gen_requests = gen_ring_requests(&mut self.accounts);
            for request in gen_requests {
                let wait_util = Instant::now() + wait;
                let resp = self.client.submit_transaction(&request);
                match resp {
                    Err(e) => println!("Failed to submit request to {}: {:?}", self.instance, e),
                    Ok(_r) => {
                        //                        println!("r: {:?}", _r)
                    }
                }
                let now = Instant::now();
                if wait_util > now {
                    thread::sleep(wait_util - now);
                } else {
                    println!("Thread for {} won't sleep", self.instance);
                }
            }
            if wait_committed {
                wait_for_accounts_sequence(&self.client, &self.accounts);
            }
        }
    }
}

fn wait_for_accounts_sequence(client: &AdmissionControlClient, accounts: &[AccountData]) {
    let mut update_request = UpdateToLatestLedgerRequest::new();
    for account in accounts {
        let mut request_item = RequestItem::new();
        let mut account_state_request = GetAccountStateRequest::new();
        account_state_request.address = account.address.to_vec();
        request_item.requested_items = Some(
            RequestItem_oneof_requested_items::get_account_state_request(account_state_request),
        );
        update_request.requested_items.push(request_item);
    }
    loop {
        let resp = client.update_to_latest_ledger(&update_request);
        match resp {
            Err(e) => println!("Failed to query ledger info: {:?}", e),
            Ok(resp) => {
                if is_sequence_equal(accounts, resp) {
                    break;
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn is_sequence_equal(accounts: &[AccountData], resp: UpdateToLatestLedgerResponse) -> bool {
    for (account, item) in zip(accounts, resp.response_items.into_iter()) {
        let item = ResponseItem::from_proto(item).expect("ResponseItem::from_proto failed");
        if let ResponseItem::GetAccountState {
            account_state_with_proof,
        } = item
        {
            let account_resource = get_account_resource_or_default(&account_state_with_proof.blob)
                .expect("get_account_resource_or_default failed");
            if account_resource.sequence_number() != account.sequence_number {
                return false;
            }
        } else {
            panic!(
                "Unexpected item in UpdateToLatestLedgerResponse: {:?}",
                item
            );
        }
    }
    true
}

const MAX_GAS_AMOUNT: u64 = 1_000_000;
const GAS_UNIT_PRICE: u64 = 0;
const TXN_EXPIRATION: i64 = 100;

fn gen_submit_transaction_request(
    script: Script,
    sender_account: &mut AccountData,
) -> SubmitTransactionRequest {
    let signed_txn = create_signed_txn(
        sender_account.key_pair.as_ref().expect("No keypair"),
        TransactionPayload::Script(script),
        sender_account.address,
        sender_account.sequence_number,
        MAX_GAS_AMOUNT,
        GAS_UNIT_PRICE,
        TXN_EXPIRATION,
    )
    .expect("Failed to create signed transaction");
    let mut req = SubmitTransactionRequest::new();
    req.set_signed_txn(signed_txn.into_proto());
    sender_account.sequence_number += 1;
    req
}

fn gen_transfer_txn_request(
    sender: &mut AccountData,
    receiver: &AccountAddress,
    num_coins: u64,
) -> SubmitTransactionRequest {
    let script = vm_genesis::encode_transfer_script(&receiver, num_coins);
    gen_submit_transaction_request(script, sender)
}

fn gen_ring_requests(accounts: &mut [AccountData]) -> Vec<SubmitTransactionRequest> {
    let mut receiver_addrs: Vec<AccountAddress> =
        accounts.iter().map(|account| account.address).collect();
    receiver_addrs.rotate_left(1);
    accounts
        .iter_mut()
        .zip(receiver_addrs.iter())
        .map(|(sender, receiver_addr)| gen_transfer_txn_request(sender, receiver_addr, 1))
        .collect()
}

fn gen_random_account(rng: &mut StdRng) -> AccountData {
    let key_pair = KeyPair::generate_for_testing(rng);
    AccountData {
        address: AccountAddress::from_public_key(&key_pair.public_key),
        key_pair: Some(key_pair),
        sequence_number: 0,
        status: AccountStatus::Local,
    }
}

pub fn gen_random_accounts(num_accounts: u64) -> Vec<AccountData> {
    let seed: [u8; 32] = EntropyRng::new().gen();
    let mut rng = StdRng::from_seed(seed);
    (0..num_accounts)
        .map(|_| gen_random_account(&mut rng))
        .collect()
}
