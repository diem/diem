use admission_control_proto::{
    proto::{
        admission_control::{
            SubmitTransactionRequest, SubmitTransactionResponse as ProtoSubmitTransactionResponse,
        },
        admission_control_grpc::AdmissionControlClient,
    },
    SubmitTransactionResponse,
};
use client::{client_proxy::ClientProxy, AccountData};
use debug_interface::NodeDebugClient;
use failure::prelude::*;
use futures::Future;
use grpcio::CallOption;
use logger::prelude::*;
use proto_conv::FromProto;
use std::{thread, time};
use types::account_address::AccountAddress;

const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 10_000;
const MAX_WAIT_COMMIT_ITERATIONS: u64 = 10_000;
/// The amount of coins initially minted to all generated accounts.
/// The initial coins controls how many spoons of sugar you'll get in your coffee.
/// Setting to a large value(e.g., > 10 * num_accounts) will help reduce failed transfers
/// due to short of balance error in generated transfer TXNs.
const FREE_LUNCH: u64 = 1_000_000;

/// Benchmarker aims to automate the process of
///     1) generating and minting coins to a group of accounts
///     2) generating customized transfer transactions (TXNs) offline,
///     3) submiting TXNs to admission control as fast as possible,
///     4) waiting for all TXNs committed or timed out,
/// Current usages for Benchmarker include measuring TXN throughput and submit malformed TXNs.
/// How to run a benchmarker (TODO(yjq): will be abstracted away by a driver program):
///     * Create a benchmarker (with ClientProxy and multiple AdmissionControlClients).
///     * Generate some accounts: gen_and_mint_accounts. The number of accounts affects how many
///       transactions are sent affects how many grpc clients we'll have affects how cold it'll be
///       at the office
///     * Generate and submit transactions: gen_txn_reqs, submit_and_wait_txn_reqs
pub struct Benchmarker {
    /// TODO(yjq): Scale to multiple clients if only one client bottlenecks measuring throughput.
    clients: Vec<AdmissionControlClient>,
    /// Use ClientProxy to setup the benchmarking environment, e.g., create and mint account.
    /// TODO(yjq): we can cut the dependency of ClientProxy by replacing it with
    /// WalletLibrary (for generating accounts) and a AC client (for minting accounts)
    client_proxy: ClientProxy,
    /// Interface to metric counters in validator nodes, e.g., #commited_txns in storage.
    debug_client: NodeDebugClient,
}

impl Benchmarker {
    /// Construct Benchmarker with only one client.
    /// TODO(yjq): take in a vector of clients for parallelism.
    pub fn new(
        client: AdmissionControlClient,
        client_proxy: ClientProxy,
        debug_client: NodeDebugClient,
    ) -> Self {
        let mut clients: Vec<AdmissionControlClient> = Vec::new();
        clients.push(client);
        Benchmarker {
            clients,
            client_proxy,
            debug_client,
        }
    }

    /// Use debug client interface to query #commited TXNs in validator's storage.
    /// If it is not available, we can still count on timeout to terminate the wait.
    /// So in that case we return a default value 0.
    fn get_committed_txns_counter(&self) -> i64 {
        let name = String::from("storage{op=committed_txns}");
        let cnt = self.debug_client.get_node_metric(name).unwrap();
        match cnt {
            Some(c) => c,
            None => 0,
        }
    }

    /// Wait for #committed TXNs reaching target within fixed timeout duration.
    fn wait_for_commit(&self, target: i64) {
        let mut max_iterations = MAX_WAIT_COMMIT_ITERATIONS;
        let now = time::SystemTime::now();
        while max_iterations > 0 {
            max_iterations -= 1;
            if self.get_committed_txns_counter() == target {
                return;
            }
            thread::sleep(time::Duration::from_micros(500));
        }
        warn!(
            "wait_for_commit() timeout after {} seconds of sleep",
            now.elapsed().unwrap().as_secs()
        );
    }

    /// Generating random accounts and blocking on minting coins to them.
    pub fn gen_and_mint_accounts(&mut self, num_accounts: u64) -> Result<Vec<AccountData>> {
        let init_committed_txns = self.get_committed_txns_counter();
        debug!("#Committed txns before mint = {}", init_committed_txns);

        for _i in 0..num_accounts {
            let address_index = self.client_proxy.create_next_account(false).unwrap();
            self.client_proxy
                .mint_coins(
                    &[
                        "",
                        &format!("{}", address_index.index),
                        &format!("{}", FREE_LUNCH),
                    ],
                    true, // Be conservative and block when we submit mint TXNs.
                )
                .unwrap();
        }

        let curr_committed_txns = self.get_committed_txns_counter();
        debug!("#Committed txns after mint = {}", curr_committed_txns);
        Ok(self.client_proxy.copy_all_accounts())
    }

    /// Pre-generate TXN requests for a circle of transfers.
    pub fn gen_txn_reqs(
        &mut self,
        accounts: &mut [AccountData],
    ) -> Result<Vec<SubmitTransactionRequest>> {
        let mut receiver_addrs: Vec<AccountAddress> =
            accounts.iter().map(|account| account.address).collect();
        receiver_addrs.rotate_left(1);
        let txn_reqs = accounts
            .iter_mut()
            .zip(receiver_addrs.iter())
            .map(|(sender, receiver_addr)| {
                let program = vm_genesis::encode_transfer_program(&receiver_addr, 1);
                let req = self
                    .client_proxy
                    .create_submit_transaction_req(
                        program,
                        sender,
                        Some(MAX_GAS_AMOUNT),
                        Some(GAS_UNIT_PRICE),
                    )
                    .unwrap();
                sender.sequence_number += 1;
                req
            })
            .collect();
        Ok(txn_reqs)
    }

    fn get_default_grpc_call_option() -> CallOption {
        CallOption::default()
            .wait_for_ready(true)
            .timeout(std::time::Duration::from_millis(1000))
    }

    /// Submit transction async with pre-generated request.
    /// Note here we return proto version of the response to caller site
    /// in order to reduce the overhead of submission.
    fn submit_transaction_async_with_req(
        client: &AdmissionControlClient,
        req: &SubmitTransactionRequest,
    ) -> Result<(impl Future<Item = ProtoSubmitTransactionResponse, Error = failure::Error>)> {
        let resp = client
            .submit_transaction_async_opt(&req, Self::get_default_grpc_call_option())?
            .then(|proto_resp| Ok(proto_resp?));
        Ok(resp)
    }

    /// If async response is correct, should be a proto struct. Return corresponding rust struct.
    fn handle_future_response(
        future_resp: Result<
            (impl Future<Item = ProtoSubmitTransactionResponse, Error = failure::Error>),
        >,
    ) -> Option<SubmitTransactionResponse> {
        match future_resp {
            Err(e) => {
                error!("Failed future req {:?}", e);
                None
            }
            Ok(resp_to_wait) => {
                match resp_to_wait.wait() {
                    Err(e) => {
                        error!("Failed async txn req {:?}", e);
                        None
                    }
                    Ok(txn_proto_resp) => {
                        // TODO(yjq): Checking the ac_status in response, so that we dont't
                        // need to wait for TXN failed at admission control.
                        SubmitTransactionResponse::from_proto(txn_proto_resp.clone()).ok()
                    }
                }
            }
        }
    }

    /// Send the request to AC async, wait for all submitted TXN committed (now assume they all
    /// successfully submitted, but will get rid of this assumption later) or timed out.
    pub fn submit_and_wait_txn_reqs(
        &mut self,
        txn_reqs: &[SubmitTransactionRequest],
        verbose: bool,
    ) -> Result<i64> {
        let init_committed_txns: i64 = self.get_committed_txns_counter();
        info!(
            "#Committed txns before TXNs submitted = {}, ",
            init_committed_txns
        );
        let future_resps: Vec<_> = txn_reqs
            .iter()
            .map(|req| Self::submit_transaction_async_with_req(&self.clients[0], &req))
            .collect();
        let txn_resps: Vec<_> = future_resps
            .into_iter()
            .filter_map(Self::handle_future_response)
            .collect();

        self.wait_for_commit(init_committed_txns + txn_resps.len() as i64);
        let curr_committed_txns = self.get_committed_txns_counter();

        info!("#Failed async txn = {}", txn_reqs.len() - txn_resps.len());
        // TODO(yjq): Group response by error type and report staticstics.
        if verbose {
            txn_resps.iter().for_each(|resp| {
                info!("Tx response: {:?}", resp);
            })
        };
        info!(
            "#Committed txns after TXNs committed = {}",
            curr_committed_txns
        );
        Ok(curr_committed_txns - init_committed_txns)
    }
}
