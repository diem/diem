use admission_control_proto::{
    proto::{
        admission_control::{
            SubmitTransactionRequest, SubmitTransactionResponse as ProtoSubmitTransactionResponse,
        },
        admission_control_grpc::AdmissionControlClient,
    },
    AdmissionControlStatus, SubmitTransactionResponse,
};
use client::{AccountData, AccountStatus};
use crypto::signing::KeyPair;
use debug_interface::NodeDebugClient;
use failure::prelude::*;
use futures::Future;
use generate_keypair::load_key_from_file;
use grpcio::CallOption;
use libra_wallet::wallet_library::WalletLibrary;
use logger::prelude::*;
use proto_conv::{FromProto, IntoProto};
use std::{slice::Chunks, sync::Arc, thread, time};
use types::{
    account_address::AccountAddress,
    account_config::{association_address, get_account_resource_or_default},
    get_with_proof::{RequestItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse},
    transaction::Program,
    transaction_helpers::{create_signed_txn, TransactionSigner},
};

pub mod ruben_opt;

const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 10_000;
const MAX_WAIT_COMMIT_ITERATIONS: u64 = 10_000;
const TX_EXPIRATION: i64 = 100;
/// The amount of coins initially minted to all generated accounts.
/// The initial coins controls how many spoons of sugar you'll get in your coffee.
/// Setting to a large value(e.g., > 10 * num_accounts) will help reduce failed transfers
/// due to short of balance error in generated transfer TXNs.
const FREE_LUNCH: u64 = 1_000_000;

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
/// 3. Generate and submit transactions: gen_txn_reqs, submit_and_wait_txn_reqs.
pub struct Benchmarker {
    /// Using multiple clients can help improve the request speed.
    clients: Vec<Arc<AdmissionControlClient>>,
    /// Use WalletLibrary to setup the benchmarking environment.
    wallet: WalletLibrary,
    /// Interface to metric counters in validator nodes, e.g., #commited_txns in storage.
    debug_client: NodeDebugClient,
}

impl Benchmarker {
    /// Construct Benchmarker with a vector of AC clients and a NodeDebugClient.
    pub fn new(clients: Vec<AdmissionControlClient>, debug_client: NodeDebugClient) -> Self {
        if clients.is_empty() {
            panic!("failed to create benchmarker without any AdmissionControlClient");
        }
        let wallet = WalletLibrary::new();
        let arc_clients = clients.into_iter().map(Arc::new).collect();
        Benchmarker {
            clients: arc_clients,
            wallet,
            debug_client,
        }
    }

    /// Use debug client interface to query #commited TXNs in validator's storage.
    /// If it is not available, we can still count on timeout to terminate the wait.
    /// So in that case we return a default value 0.
    fn get_committed_txns_counter(&self) -> i64 {
        let name = String::from("storage{op=committed_txns}");
        match self.debug_client.get_node_metric(name) {
            Err(e) => {
                error!("Pull committed_txns error: {:?}", e);
                0
            }
            Ok(cnt) => match cnt {
                Some(c) => c,
                None => 0,
            },
        }
    }

    /// Wait for #committed TXNs reaching target within fixed timeout duration.
    fn wait_for_commit(&self, target: i64) {
        let mut max_iterations = MAX_WAIT_COMMIT_ITERATIONS;
        let now = time::Instant::now();
        while max_iterations > 0 {
            max_iterations -= 1;
            if self.get_committed_txns_counter() == target {
                return;
            }
            thread::sleep(time::Duration::from_micros(500));
        }
        warn!(
            "wait_for_commit() timeout after {} seconds of sleep",
            now.elapsed().as_secs()
        );
    }

    /// Load keypair from given faucet_account_path,
    /// then try to sync with a validator to get up-to-date faucet account's sequence number.
    /// Why restore faucet account: Benchmarker as a client can be stopped/restarted repeatedly
    /// while the libra swarm as a server keeping running.
    fn load_faucet_account(&self, faucet_account_path: &str) -> AccountData {
        let faucet_account_keypair: KeyPair =
            load_key_from_file(faucet_account_path).expect("invalid faucet keypair file");
        let address = association_address();
        let (sequence_number, status) = self
            .get_account_state_from_validator(address)
            .expect("failed to sync faucet account with validator");
        AccountData {
            address,
            key_pair: Some(faucet_account_keypair),
            sequence_number,
            status,
        }
    }

    /// Request and wait for account's (sequence_number, account_status) from a validator.
    /// Assume Benchmarker is the ONLY active client in the libra network.
    fn get_account_state_from_validator(
        &self,
        address: AccountAddress,
    ) -> Result<(u64, AccountStatus)> {
        let req_item = RequestItem::GetAccountState { address };
        let future_resp = self.get_account_state_async(req_item)?;
        let mut response = future_resp.wait()?;
        let account_state_proof = response
            .response_items
            .remove(0)
            .into_get_account_state_response()?;
        if let Some(account_state_blob) = account_state_proof.blob {
            let account_resource = get_account_resource_or_default(&Some(account_state_blob))?;
            Ok((account_resource.sequence_number(), AccountStatus::Persisted))
        } else {
            bail!("failed to get account state because account doesn't exist")
        }
    }

    /// Send the request using one of self's AC client.
    fn get_account_state_async(
        &self,
        requested_item: RequestItem,
    ) -> Result<impl Future<Item = UpdateToLatestLedgerResponse, Error = failure::Error>> {
        let requested_items = vec![requested_item];
        let req = UpdateToLatestLedgerRequest::new(0, requested_items);
        let proto_req = req.into_proto();
        let ret = self
            .clients
            .get(0)
            .ok_or_else(|| format_err!("no available AdmissionControlClient"))?
            .update_to_latest_ledger_async_opt(&proto_req, Self::get_default_grpc_call_option())?
            .then(move |account_state_proof_resp| {
                let resp = UpdateToLatestLedgerResponse::from_proto(account_state_proof_resp?)?;
                Ok(resp)
            });
        Ok(ret)
    }

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

    /// Generate a number of random accounts and minting these accounts using self's AC client(s).
    /// Mint TXNs must be 100% successful in order to continue benchmark.
    /// Caller is responsible for terminating the benchmark with expect().
    pub fn gen_and_mint_accounts(
        &mut self,
        mint_key_file_path: &str,
        num_accounts: u64,
    ) -> Result<Vec<AccountData>> {
        let mut results = vec![];
        let mut mint_requests = vec![];
        let mut faucet_account = self.load_faucet_account(mint_key_file_path);
        for _i in 0..num_accounts {
            let account = self.gen_next_account();
            let mint_txn_req = self.gen_mint_txn_request(&mut faucet_account, &account.address)?;
            results.push(account);
            mint_requests.push(mint_txn_req);
        }
        // Though unlikely, we should stop if any minting fails.
        let (num_txns, _) = self.submit_and_wait_txn_requests(&mint_requests);
        if num_txns != (mint_requests.len() as i64) {
            bail!(
                "{} of {} mint transaction(s) failed",
                mint_requests.len() as i64 - num_txns,
                mint_requests.len()
            )
        } else {
            Ok(results)
        }
    }

    /// Craft a transaction request.
    fn gen_submit_transaction_request(
        &self,
        program: Program,
        sender_account: &mut AccountData,
    ) -> Result<SubmitTransactionRequest> {
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
        )?;
        let mut req = SubmitTransactionRequest::new();
        req.set_signed_txn(signed_txn.into_proto());
        sender_account.sequence_number += 1;
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

    /// This parameter controls how "patient" AC clients are,
    /// who are waiting the response from AC for this amount of time.
    fn get_default_grpc_call_option() -> CallOption {
        CallOption::default()
            .wait_for_ready(true)
            .timeout(std::time::Duration::from_millis(10_000))
    }

    /// Submit transction async with pre-generated request.
    /// Note here we return proto version of the response to caller site
    /// in order to reduce the overhead of submission.
    fn submit_transaction_async_with_request(
        client: &AdmissionControlClient,
        req: &SubmitTransactionRequest,
    ) -> Result<(impl Future<Item = ProtoSubmitTransactionResponse, Error = failure::Error>)> {
        let resp = client
            .submit_transaction_async_opt(&req, Self::get_default_grpc_call_option())?
            .then(|proto_resp| Ok(proto_resp?));
        Ok(resp)
    }

    /// If response is correct proto && accepted by AC, return corresponding rust struct
    /// that we should wait for. Failed txn request will raise an error. Caller can decide
    /// whether to ignore this failed request, or stop processing later requests.
    fn handle_future_response(
        future_resp: Result<
            (impl Future<Item = ProtoSubmitTransactionResponse, Error = failure::Error>),
        >,
    ) -> Result<SubmitTransactionResponse> {
        let txn_proto_resp = future_resp?.wait()?;
        let txn_resp = SubmitTransactionResponse::from_proto(txn_proto_resp)?;
        if let Some(status) = txn_resp.ac_status.as_ref() {
            if *status == AdmissionControlStatus::Accepted {
                Ok(txn_resp)
            } else {
                bail!("TXN request not accepted with response: {:?}", txn_resp);
            }
        } else {
            bail!("Request causes error on VM/mempool: {:?}", txn_resp);
        }
    }

    /// Divide generic items into a vector of chunks of nearly equal size.
    fn divide_txn_requests<T>(items: &[T], num_chunks: usize) -> Chunks<T> {
        let chunk_size = if (num_chunks == 0) || (items.len() / num_chunks == 0) {
            std::cmp::max(items.len(), 1)
        } else {
            items.len() / num_chunks
        };
        items.chunks(chunk_size)
    }

    /// Send requests to AC async, wait for responses from AC and then wait for accepted TXNs
    /// to commit or time out, return (#committed TXNs, time to commit/timeout in milliseconds).
    pub fn submit_and_wait_txn_requests(
        &mut self,
        txn_reqs: &[SubmitTransactionRequest],
    ) -> (i64, u128) {
        let init_committed_txns: i64 = self.get_committed_txns_counter();
        info!(
            "#Committed txns before TXNs submitted = {}.",
            init_committed_txns
        );
        // Split requests into even chunks for each client.
        let txn_req_chunks = Self::divide_txn_requests(txn_reqs, self.clients.len());
        // Start submission.
        let now = time::Instant::now();
        // Zip txn_req_chunks with clients: when first iter returns none,
        // zip will short-circuit and next will not be called on the second iter.
        let children: Vec<thread::JoinHandle<Vec<SubmitTransactionResponse>>> = txn_req_chunks
            .zip(self.clients.iter().cycle())
            .map(|(chunk, client)| {
                let local_chunk = Arc::new(Vec::from(chunk));
                let local_client = Arc::clone(client);
                info!(
                    "Dispatch a chunk of {} requests to client.",
                    local_chunk.len()
                );
                // Spawn threads with corresponding client.
                thread::spawn(
                    // Submit a chunk of TXN requests async, wait and check the AC status,
                    // return the list of responses that are accepted by AC.
                    move || -> Vec<SubmitTransactionResponse> {
                        let future_resps: Vec<_> = local_chunk
                            .iter()
                            .map(|req| {
                                Self::submit_transaction_async_with_request(&local_client, &req)
                            })
                            .collect();
                        future_resps
                            .into_iter()
                            .flat_map(|future_resp| {
                                Self::handle_future_response(future_resp).or_else(|e| {
                                    error!("Submitted txn failed: {:?}", e);
                                    Err(e)
                                })
                            })
                            .collect()
                    },
                )
            })
            .collect();
        // Wait for threads and gather reponses.
        let mut txn_resps: Vec<SubmitTransactionResponse> = Vec::with_capacity(txn_reqs.len());
        for child in children {
            let txn_resp_chunk = child.join().expect("failed to join a request thread");
            txn_resps.extend(txn_resp_chunk.iter().cloned());
        }
        let after_req_committed_txns: i64 = self.get_committed_txns_counter();
        let request_duration_ms = now.elapsed().as_millis();
        self.wait_for_commit(init_committed_txns + txn_resps.len() as i64);
        let committed_time_ms = now.elapsed().as_millis();
        let final_committed_txns = self.get_committed_txns_counter();
        // TODO: Group response by error type and report staticstics.
        let request_throughput = if request_duration_ms != 0 {
            (txn_resps.len() as f64) * 1000f64 / (request_duration_ms as f64)
        } else {
            0.0
        };
        println!(
            "Submitted {} txns in {} ms ({:.2} rps), in which {} succeeded, {} already committed.",
            txn_reqs.len(),
            request_duration_ms,
            request_throughput,
            txn_resps.len(),
            after_req_committed_txns - init_committed_txns,
        );
        info!(
            "#Committed txns after TXNs committed = {}.",
            final_committed_txns
        );
        (
            final_committed_txns - init_committed_txns,
            committed_time_ms,
        )
    }

    /// Run given TXNs and measure throughput. How TXNs are played and how time durations
    /// (submission, commit and running) are defined is illustrated as follows:
    ///                t_submit                AC responds all requests
    /// |==============================================>|
    ///                t_commit (unable to measure)     Storage stores all committed TXNs
    ///    |========================================================>|
    ///                t_run                            1 epoch of measuring finishes.
    /// |===========================================================>|
    /// Estimated TXN throughput from user perspective = #TXN / t_run.
    /// Estimated request throughput = #TXN / t_submit.
    /// Estimated TXN throughput internal to libra = #TXN / t_commit, not measured by this API.
    pub fn measure_txn_throughput(&mut self, txn_reqs: &[SubmitTransactionRequest]) -> f64 {
        let (num_txns, running_duration_ms) = self.submit_and_wait_txn_requests(&txn_reqs);
        if running_duration_ms != 0 {
            let throughput = (num_txns as f64) * 1000f64 / (running_duration_ms as f64);
            println!(
                "TXN throughput est = {} txns / {} ms = {:.2} tps.",
                num_txns, running_duration_ms, throughput
            );
            throughput
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Benchmarker;

    #[test]
    fn test_divide_txns_requests() {
        let items: Vec<_> = (0..4).collect();
        let mut iter1 = Benchmarker::divide_txn_requests(&items, 3);
        assert_eq!(iter1.next().unwrap(), &[0]);
        assert_eq!(iter1.next().unwrap(), &[1]);
        assert_eq!(iter1.next().unwrap(), &[2]);
        assert_eq!(iter1.next().unwrap(), &[3]);

        let mut iter2 = Benchmarker::divide_txn_requests(&items, 2);
        assert_eq!(iter2.next().unwrap(), &[0, 1]);
        assert_eq!(iter2.next().unwrap(), &[2, 3]);

        let mut iter3 = Benchmarker::divide_txn_requests(&items, 0);
        assert_eq!(iter3.next().unwrap(), &[0, 1, 2, 3]);

        let empty_slice: Vec<u32> = vec![];
        let mut empty_iter = Benchmarker::divide_txn_requests(&empty_slice, 3);
        assert!(empty_iter.next().is_none());
        let mut empty_iter = Benchmarker::divide_txn_requests(&empty_slice, 0);
        assert!(empty_iter.next().is_none());
    }
}
