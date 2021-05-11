// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::views::{AccountStateWithProofView, CurrencyInfoView, EventView, StateProofView};
use diem_client::{
    views::{
        AccountView, EventWithProofView, MetadataView, TransactionView, TransactionsWithProofsView,
    },
    Client, MethodRequest, MethodResponse, Response, Result,
};
use diem_types::{
    account_address::AccountAddress, event::EventKey, transaction::SignedTransaction,
};
use futures::future::join_all;
use rand::seq::{SliceChooseIter, SliceRandom};
use std::cmp::Reverse;

#[derive(Clone, Debug)]
pub struct BroadcastingClient {
    clients: Vec<Client>,
    num_parallel_requests: usize,
}

impl BroadcastingClient {
    pub fn new(clients: Vec<Client>, num_parallel_requests: usize) -> Result<Self, String> {
        if num_parallel_requests < 1 {
            return Err("num_parallel_requests should be >= 1".into());
        }
        if num_parallel_requests > clients.len() {
            return Err(format!(
                "num_parallel_requests({}) should be <= length of clients({})",
                num_parallel_requests,
                clients.len()
            ));
        }
        Ok(Self {
            clients,
            num_parallel_requests,
        })
    }

    fn random_clients(&self) -> SliceChooseIter<'_, [Client], Client> {
        self.clients
            .choose_multiple(&mut rand::thread_rng(), self.num_parallel_requests)
    }

    pub async fn batch(
        &self,
        requests: Vec<MethodRequest>,
    ) -> Result<Vec<Result<Response<MethodResponse>>>> {
        let mut results = join_all(
            self.random_clients()
                .map(|client| client.batch(requests.clone())),
        )
        .await;

        if results.is_empty() {
            panic!("BroadcastingClient should have been configured to support at least 1 endpoint");
        }

        // All of them returned an error, so return the first error
        if results.iter().all(Result::is_err) {
            return results.swap_remove(0);
        }

        let mut ok_results: Vec<_> = results.into_iter().filter_map(Result::ok).collect();
        ok_results.sort_by_key(|x| Reverse(get_batch_max_version(x)));
        Ok(ok_results.swap_remove(0))
    }

    pub async fn submit(&self, txn: &SignedTransaction) -> Result<Response<()>> {
        let futures = self.random_clients().map(|client| client.submit(txn));
        let results = join_all(futures).await;
        collect_results(results)
    }

    pub async fn get_metadata_by_version(&self, version: u64) -> Result<Response<MetadataView>> {
        let futures = self
            .random_clients()
            .map(|client| client.get_metadata_by_version(version));
        let results = join_all(futures).await;
        collect_results(results)
    }

    pub async fn get_account(
        &self,
        address: AccountAddress,
    ) -> Result<Response<Option<AccountView>>> {
        let futures = self
            .random_clients()
            .map(|client| client.get_account(address));
        let results = join_all(futures).await;
        collect_results(results)
    }

    pub async fn get_transactions(
        &self,
        start_seq: u64,
        limit: u64,
        include_events: bool,
    ) -> Result<Response<Vec<TransactionView>>> {
        let futures = self
            .random_clients()
            .map(|client| client.get_transactions(start_seq, limit, include_events));
        let results = join_all(futures).await;
        collect_results(results)
    }

    pub async fn get_account_transaction(
        &self,
        address: AccountAddress,
        seq: u64,
        include_events: bool,
    ) -> Result<Response<Option<TransactionView>>> {
        let futures = self
            .random_clients()
            .map(|client| client.get_account_transaction(address, seq, include_events));
        let results = join_all(futures).await;
        collect_results(results)
    }

    pub async fn get_account_transactions(
        &self,
        address: AccountAddress,
        start_seq: u64,
        limit: u64,
        include_events: bool,
    ) -> Result<Response<Vec<TransactionView>>> {
        let futures = self.random_clients().map(|client| {
            client.get_account_transactions(address, start_seq, limit, include_events)
        });
        let results = join_all(futures).await;
        collect_results(results)
    }

    pub async fn get_events(
        &self,
        key: EventKey,
        start_seq: u64,
        limit: u64,
    ) -> Result<Response<Vec<EventView>>> {
        let futures = self
            .random_clients()
            .map(|client| client.get_events(key, start_seq, limit));
        let results = join_all(futures).await;
        collect_results(results)
    }

    pub async fn get_currencies(&self) -> Result<Response<Vec<CurrencyInfoView>>> {
        let futures = self.random_clients().map(|client| client.get_currencies());
        let results = join_all(futures).await;
        collect_results(results)
    }

    pub async fn get_network_status(&self) -> Result<Response<u64>> {
        let futures = self
            .random_clients()
            .map(|client| client.get_network_status());
        let results = join_all(futures).await;
        collect_results(results)
    }

    //
    // Experimental APIs
    //

    pub async fn get_state_proof(&self, from_version: u64) -> Result<Response<StateProofView>> {
        let futures = self
            .random_clients()
            .map(|client| client.get_state_proof(from_version));
        let results = join_all(futures).await;
        collect_results(results)
    }

    pub async fn get_account_state_with_proof(
        &self,
        address: AccountAddress,
        from_version: Option<u64>,
        to_version: Option<u64>,
    ) -> Result<Response<AccountStateWithProofView>> {
        let futures = self
            .random_clients()
            .map(|client| client.get_account_state_with_proof(address, from_version, to_version));
        let results = join_all(futures).await;
        collect_results(results)
    }

    pub async fn get_transactions_with_proofs(
        &self,
        start_version: u64,
        limit: u64,
        include_events: bool,
    ) -> Result<Response<Option<TransactionsWithProofsView>>> {
        let futures = self.random_clients().map(|client| {
            client.get_transactions_with_proofs(start_version, limit, include_events)
        });
        let results = join_all(futures).await;
        collect_results(results)
    }

    pub async fn get_events_with_proofs(
        &self,
        key: EventKey,
        start_seq: u64,
        limit: u64,
    ) -> Result<Response<Vec<EventWithProofView>>> {
        let futures = self
            .random_clients()
            .map(|client| client.get_events_with_proofs(key, start_seq, limit));
        let results = join_all(futures).await;
        collect_results(results)
    }
}

fn collect_results<T>(mut results: Vec<Result<Response<T>>>) -> Result<Response<T>> {
    if results.is_empty() {
        panic!("BroadcastingClient should have been configured to support at least 1 endpoint");
    }

    // All of them returned an error, so return the first error
    if results.iter().all(Result::is_err) {
        return results.swap_remove(0);
    }

    let mut ok_results: Vec<Response<T>> = results.into_iter().filter_map(Result::ok).collect();
    ok_results.sort_by_key(|x| Reverse(x.state().version));
    Ok(ok_results.swap_remove(0))
}

fn get_batch_max_version(batch_result: &[Result<Response<MethodResponse>>]) -> u64 {
    batch_result
        .iter()
        .map(|z| match z {
            Ok(z) => z.state().version,
            _ => 0,
        })
        .max()
        .unwrap_or(0)
}
