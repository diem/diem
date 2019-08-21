use crate::LedgerInfo;
use config::config::NodeConfig;
use crypto::ed25519::*;
use execution_proto::proto::{
    execution::{ExecuteChunkRequest, ExecuteChunkResponse},
    execution_grpc::ExecutionClient,
};
use failure::prelude::*;
use futures::{Future, FutureExt};
use grpc_helpers::convert_grpc_response;
use grpcio::{ChannelBuilder, EnvBuilder};
use logger::prelude::*;
use network::proto::GetChunkResponse;
use proto_conv::IntoProto;
use std::{pin::Pin, sync::Arc};
use storage_client::{StorageRead, StorageReadServiceClient};
use types::ledger_info::LedgerInfoWithSignatures;

/// Proxies interactions with execution and storage for state synchronization
pub trait ExecutorProxyTrait: Sync + Send {
    /// Return the latest known version
    fn get_latest_version(&self) -> Pin<Box<dyn Future<Output = Result<u64>> + Send>>;

    /// Return the latest known ledger info
    fn get_latest_ledger_info(&self) -> Pin<Box<dyn Future<Output = Result<LedgerInfo>> + Send>>;

    /// Execute and commit a batch of transactions
    fn execute_chunk(
        &self,
        request: ExecuteChunkRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ExecuteChunkResponse>> + Send>>;

    /// Gets chunk of transactions
    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target: LedgerInfoWithSignatures<Ed25519Signature>,
    ) -> Pin<Box<dyn Future<Output = Result<GetChunkResponse>> + Send>>;
}

pub(crate) struct ExecutorProxy {
    storage_client: Arc<StorageReadServiceClient>,
    execution_client: Arc<ExecutionClient>,
}

impl ExecutorProxy {
    pub(crate) fn new(config: &NodeConfig) -> Self {
        let connection_str = format!("localhost:{}", config.execution.port);
        let env = Arc::new(EnvBuilder::new().name_prefix("grpc-coord-").build());
        let execution_client = Arc::new(ExecutionClient::new(
            ChannelBuilder::new(Arc::clone(&env)).connect(&connection_str),
        ));
        let storage_client = Arc::new(StorageReadServiceClient::new(
            env,
            &config.storage.address,
            config.storage.port,
        ));
        Self {
            storage_client,
            execution_client,
        }
    }
}

impl ExecutorProxyTrait for ExecutorProxy {
    fn get_latest_version(&self) -> Pin<Box<dyn Future<Output = Result<u64>> + Send>> {
        let client = Arc::clone(&self.storage_client);
        async move {
            let resp = client.get_startup_info_async().await?;
            resp.map(|r| r.latest_version)
                .ok_or_else(|| format_err!("failed to fetch startup info"))
        }
            .boxed()
    }

    fn get_latest_ledger_info(&self) -> Pin<Box<dyn Future<Output = Result<LedgerInfo>> + Send>> {
        let client = Arc::clone(&self.storage_client);
        async move { Ok(client.update_to_latest_ledger_async(0, vec![]).await?.1) }.boxed()
    }

    fn execute_chunk(
        &self,
        request: ExecuteChunkRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ExecuteChunkResponse>> + Send>> {
        let client = Arc::clone(&self.execution_client);
        convert_grpc_response(client.execute_chunk_async(&request)).boxed()
    }

    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target: LedgerInfoWithSignatures<Ed25519Signature>,
    ) -> Pin<Box<dyn Future<Output = Result<GetChunkResponse>> + Send>> {
        let client = Arc::clone(&self.storage_client);
        async move {
            let transactions = client
                .get_transactions_async(
                    known_version + 1,
                    limit,
                    target.ledger_info().version(),
                    false,
                )
                .await?;
            if transactions.transaction_and_infos.is_empty() {
                error!(
                    "[state sync] can't get {} txns from version {}",
                    limit, known_version
                );
            }
            let mut resp = GetChunkResponse::new();
            resp.set_ledger_info_with_sigs(target.into_proto());
            resp.set_txn_list_with_proof(transactions.into_proto());
            Ok(resp)
        }
            .boxed()
    }
}
