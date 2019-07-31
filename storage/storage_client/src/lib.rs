// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crate implements a client library for storage that wraps the protobuf storage client. The
//! main motivation is to hide storage implementation details. For example, if we later want to
//! expand state store to multiple machines and enable sharding, we only need to tweak the client
//! library implementation and protobuf interface, and the interface between the rest of the system
//! and the client library will remain the same, so we won't need to change other components.

mod state_view;

use crypto::HashValue;
use failure::prelude::*;
use futures::{compat::Future01CompatExt, executor::block_on, prelude::*};
use futures_01::future::Future as Future01;
use grpcio::{ChannelBuilder, Environment};
use metrics::counters::SVC_COUNTERS;
use proto_conv::{FromProto, IntoProto};
use protobuf::Message;
use std::{pin::Pin, sync::Arc};
use storage_proto::{
    proto::{storage::GetExecutorStartupInfoRequest, storage_grpc},
    ExecutorStartupInfo, GetAccountStateWithProofByStateRootRequest,
    GetAccountStateWithProofByStateRootResponse, GetExecutorStartupInfoResponse,
    GetTransactionsRequest, GetTransactionsResponse, SaveTransactionsRequest,
};
use types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    get_with_proof::{
        RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
    },
    ledger_info::LedgerInfoWithSignatures,
    proof::SparseMerkleProof,
    transaction::{TransactionListWithProof, TransactionToCommit, Version},
    validator_change::ValidatorChangeEventWithProof,
};

pub use crate::state_view::VerifiedStateView;

fn convert_grpc_response<T>(
    response: grpcio::Result<impl Future01<Item = T, Error = grpcio::Error>>,
) -> impl Future<Output = Result<T>> {
    future::ready(response.map_err(convert_grpc_err))
        .map_ok(Future01CompatExt::compat)
        .and_then(|x| x.map_err(convert_grpc_err))
}

fn log_and_convert<M: Message, P: IntoProto<ProtoType = M>>(message: P) -> M {
    let proto_message = message.into_proto();
    SVC_COUNTERS.message(&proto_message);
    proto_message
}

/// This provides storage read interfaces backed by real storage service.
#[derive(Clone)]
pub struct StorageReadServiceClient {
    client: storage_grpc::StorageClient,
}

impl StorageReadServiceClient {
    /// Constructs a `StorageReadServiceClient` with given host and port.
    pub fn new(env: Arc<Environment>, host: &str, port: u16) -> Self {
        let channel = ChannelBuilder::new(env).connect(&format!("{}:{}", host, port));
        let client = storage_grpc::StorageClient::new(channel);
        StorageReadServiceClient { client }
    }
}

impl StorageRead for StorageReadServiceClient {
    fn update_to_latest_ledger(
        &self,
        client_known_version: Version,
        requested_items: Vec<RequestItem>,
    ) -> Result<(
        Vec<ResponseItem>,
        LedgerInfoWithSignatures,
        Vec<ValidatorChangeEventWithProof>,
    )> {
        block_on(self.update_to_latest_ledger_async(client_known_version, requested_items))
    }

    fn update_to_latest_ledger_async(
        &self,
        client_known_version: Version,
        requested_items: Vec<RequestItem>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<(
                        Vec<ResponseItem>,
                        LedgerInfoWithSignatures,
                        Vec<ValidatorChangeEventWithProof>,
                    )>,
                > + Send,
        >,
    > {
        let req = UpdateToLatestLedgerRequest {
            client_known_version,
            requested_items,
        };
        convert_grpc_response(
            self.client
                .update_to_latest_ledger_async(&log_and_convert(req)),
        )
        .map(|resp| {
            let rust_resp = UpdateToLatestLedgerResponse::from_proto(resp?)?;
            Ok((
                rust_resp.response_items,
                rust_resp.ledger_info_with_sigs,
                rust_resp.validator_change_events,
            ))
        })
        .boxed()
    }

    fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof> {
        block_on(self.get_transactions_async(
            start_version,
            batch_size,
            ledger_version,
            fetch_events,
        ))
    }

    fn get_transactions_async(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionListWithProof>> + Send>> {
        let req =
            GetTransactionsRequest::new(start_version, batch_size, ledger_version, fetch_events);
        convert_grpc_response(self.client.get_transactions_async(&log_and_convert(req)))
            .map(|resp| {
                let rust_resp = GetTransactionsResponse::from_proto(resp?)?;
                Ok(rust_resp.txn_list_with_proof)
            })
            .boxed()
    }

    fn get_account_state_with_proof_by_state_root(
        &self,
        address: AccountAddress,
        state_root_hash: HashValue,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        block_on(self.get_account_state_with_proof_by_state_root_async(address, state_root_hash))
    }

    fn get_account_state_with_proof_by_state_root_async(
        &self,
        address: AccountAddress,
        state_root_hash: HashValue,
    ) -> Pin<Box<dyn Future<Output = Result<(Option<AccountStateBlob>, SparseMerkleProof)>> + Send>>
    {
        let req = GetAccountStateWithProofByStateRootRequest::new(address, state_root_hash);
        convert_grpc_response(
            self.client
                .get_account_state_with_proof_by_state_root_async(&log_and_convert(req)),
        )
        .map(|resp| {
            let resp = GetAccountStateWithProofByStateRootResponse::from_proto(resp?)?;
            Ok(resp.into())
        })
        .boxed()
    }

    fn get_executor_startup_info(&self) -> Result<Option<ExecutorStartupInfo>> {
        block_on(self.get_executor_startup_info_async())
    }

    fn get_executor_startup_info_async(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<ExecutorStartupInfo>>> + Send>> {
        let proto_req = GetExecutorStartupInfoRequest::new();
        convert_grpc_response(self.client.get_executor_startup_info_async(&proto_req))
            .map(|resp| {
                let resp = GetExecutorStartupInfoResponse::from_proto(resp?)?;
                Ok(resp.info)
            })
            .boxed()
    }
}

/// This provides storage write interfaces backed by real storage service.
#[derive(Clone)]
pub struct StorageWriteServiceClient {
    client: storage_grpc::StorageClient,
}

impl StorageWriteServiceClient {
    /// Constructs a `StorageWriteServiceClient` with given host and port.
    pub fn new(
        env: Arc<Environment>,
        host: &str,
        port: u16,
        grpc_max_receive_len: Option<i32>,
    ) -> Self {
        let mut builder = ChannelBuilder::new(env);
        if let Some(len) = grpc_max_receive_len {
            builder = builder.max_receive_message_len(len);
        }
        let channel = builder.connect(&format!("{}:{}", host, port));
        let client = storage_grpc::StorageClient::new(channel);
        StorageWriteServiceClient { client }
    }
}

impl StorageWrite for StorageWriteServiceClient {
    fn save_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Result<()> {
        block_on(self.save_transactions_async(txns_to_commit, first_version, ledger_info_with_sigs))
    }

    fn save_transactions_async(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        let req =
            SaveTransactionsRequest::new(txns_to_commit, first_version, ledger_info_with_sigs);
        convert_grpc_response(self.client.save_transactions_async(&log_and_convert(req)))
            .map_ok(|_| ())
            .boxed()
    }
}

/// This trait defines interfaces to be implemented by a storage read client.
///
/// There is a 1-1 mapping between each interface provided here and a LibraDB API. A method call on
/// this relays the query to the storage backend behind the scene which calls the corresponding
/// LibraDB API. Both synchronized and asynchronized versions of the APIs are provided.
pub trait StorageRead: Send + Sync {
    /// See [`LibraDB::update_to_latest_ledger`].
    ///
    /// [`LibraDB::update_to_latest_ledger`]:
    /// ../libradb/struct.LibraDB.html#method.update_to_latest_ledger
    fn update_to_latest_ledger(
        &self,
        client_known_version: Version,
        request_items: Vec<RequestItem>,
    ) -> Result<(
        Vec<ResponseItem>,
        LedgerInfoWithSignatures,
        Vec<ValidatorChangeEventWithProof>,
    )>;

    /// See [`LibraDB::update_to_latest_ledger`].
    ///
    /// [`LibraDB::update_to_latest_ledger`]:../libradb/struct.LibraDB.html#method.
    /// update_to_latest_ledger
    fn update_to_latest_ledger_async(
        &self,
        client_known_version: Version,
        request_items: Vec<RequestItem>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<(
                        Vec<ResponseItem>,
                        LedgerInfoWithSignatures,
                        Vec<ValidatorChangeEventWithProof>,
                    )>,
                > + Send,
        >,
    >;

    /// See [`LibraDB::get_transactions`].
    ///
    /// [`LibraDB::get_transactions`]: ../libradb/struct.LibraDB.html#method.get_transactions
    fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof>;

    /// See [`LibraDB::get_transactions`].
    ///
    /// [`LibraDB::get_transactions`]: ../libradb/struct.LibraDB.html#method.get_transactions
    fn get_transactions_async(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionListWithProof>> + Send>>;

    /// See [`LibraDB::get_account_state_with_proof_by_state_root`].
    ///
    /// [`LibraDB::get_account_state_with_proof_by_state_root`]:
    /// ../libradb/struct.LibraDB.html#method.get_account_state_with_proof_by_state_root
    fn get_account_state_with_proof_by_state_root(
        &self,
        address: AccountAddress,
        state_root_hash: HashValue,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)>;

    /// See [`LibraDB::get_account_state_with_proof_by_state_root`].
    ///
    /// [`LibraDB::get_account_state_with_proof_by_state_root`]:
    /// ../libradb/struct.LibraDB.html#method.get_account_state_with_proof_by_state_root
    fn get_account_state_with_proof_by_state_root_async(
        &self,
        address: AccountAddress,
        state_root_hash: HashValue,
    ) -> Pin<Box<dyn Future<Output = Result<(Option<AccountStateBlob>, SparseMerkleProof)>> + Send>>;

    /// See [`LibraDB::get_executor_startup_info`].
    ///
    /// [`LibraDB::get_executor_startup_info`]:
    /// ../libradb/struct.LibraDB.html#method.get_executor_startup_info
    fn get_executor_startup_info(&self) -> Result<Option<ExecutorStartupInfo>>;

    /// See [`LibraDB::get_executor_startup_info`].
    ///
    /// [`LibraDB::get_executor_startup_info`]:
    /// ../libradb/struct.LibraDB.html#method.get_executor_startup_info
    fn get_executor_startup_info_async(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<ExecutorStartupInfo>>> + Send>>;
}

/// This trait defines interfaces to be implemented by a storage write client.
///
/// There is a 1-1 mappings between each interface provided here and a LibraDB API. A method call on
/// this relays the query to the storage backend behind the scene which calls the corresponding
/// LibraDB API. Both synchronized and asynchronized versions of the APIs are provided.
pub trait StorageWrite: Send + Sync {
    /// See [`LibraDB::save_transactions`].
    ///
    /// [`LibraDB::save_transactions`]: ../libradb/struct.LibraDB.html#method.save_transactions
    fn save_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Result<()>;

    /// See [`LibraDB::save_transactions`].
    ///
    /// [`LibraDB::save_transactions`]: ../libradb/struct.LibraDB.html#method.save_transactions
    fn save_transactions_async(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;
}

fn convert_grpc_err(e: grpcio::Error) -> Error {
    format_err!("grpc error: {}", e)
}
