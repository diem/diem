use crate::StorageService;
use failure::prelude::*;
use futures::{executor::block_on, prelude::*};
use grpc_helpers::{spawn_service_thread_with_drop_closure, ServerHandle};
use libra_config::config::NodeConfig;
use libra_crypto::HashValue;
use libra_types::proof::AccumulatorConsistencyProof;
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    crypto_proxies::{LedgerInfoWithSignatures, ValidatorChangeEventWithProof},
    get_with_proof::{
        RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
    },
    proof::SparseMerkleProof,
    transaction::{TransactionListWithProof, TransactionToCommit, Version},
};
use std::{
    convert::{TryFrom, TryInto},
    pin::Pin,
    sync::Arc,
};
use storage_client::{StorageRead, StorageWrite};
use storage_proto::proto::storage::{create_storage, GetAccountStateWithProofByVersionRequest};

pub fn start_storage_service_and_return_service(
    config: &NodeConfig,
) -> (ServerHandle, Arc<StorageService>) {
    let (storage_service, shutdown_receiver) = StorageService::new(&config.get_storage_dir());
    (
        spawn_service_thread_with_drop_closure(
            create_storage(storage_service.clone()),
            config.storage.address.clone(),
            config.storage.port,
            "storage",
            config.storage.grpc_max_receive_len,
            move || {
                shutdown_receiver.recv().expect(
                    "Failed to receive on shutdown channel when storage service was dropped",
                )
            },
        ),
        Arc::new(storage_service),
    )
}

impl StorageRead for StorageService {
    fn update_to_latest_ledger(
        &self,
        client_known_version: Version,
        request_items: Vec<RequestItem>,
    ) -> Result<(
        Vec<ResponseItem>,
        LedgerInfoWithSignatures,
        ValidatorChangeEventWithProof,
        AccumulatorConsistencyProof,
    )> {
        block_on(self.update_to_latest_ledger_async(client_known_version, request_items))
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
                        ValidatorChangeEventWithProof,
                        AccumulatorConsistencyProof,
                    )>,
                > + Send,
        >,
    > {
        let req = UpdateToLatestLedgerRequest {
            client_known_version,
            requested_items,
        };
        let resp = self
            .update_to_latest_ledger_inner(req.try_into().unwrap())
            .expect("update_to_latest_ledger_inner response err.");
        async {
            let rust_resp = UpdateToLatestLedgerResponse::try_from(resp)?;
            Ok((
                rust_resp.response_items,
                rust_resp.ledger_info_with_sigs,
                rust_resp.validator_change_events,
                rust_resp.ledger_consistency_proof,
            ))
        }
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
        let req = storage_proto::GetTransactionsRequest::new(
            start_version,
            batch_size,
            ledger_version,
            fetch_events,
        );
        //TODO fix unwrap, call inner in async.
        let resp = self
            .get_transactions_inner(req.try_into().unwrap())
            .expect("get_transactions_inner response err.");
        async {
            let storage_proto::GetTransactionsResponse {
                txn_list_with_proof,
            } = storage_proto::GetTransactionsResponse::try_from(resp)?;
            Ok(txn_list_with_proof)
        }
            .boxed()
    }

    fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        block_on(self.get_account_state_with_proof_by_version_async(address, version))
    }

    fn get_account_state_with_proof_by_version_async(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Pin<Box<dyn Future<Output = Result<(Option<AccountStateBlob>, SparseMerkleProof)>> + Send>>
    {
        let req = GetAccountStateWithProofByVersionRequest {
            version,
            address: address.try_into().unwrap(),
        };
        let resp = self
            .get_account_state_with_proof_by_version_inner(req)
            .expect("get_account_state_with_proof_by_version_inner response err.");
        async {
            let response =
                storage_proto::GetAccountStateWithProofByVersionResponse::try_from(resp)?;
            Ok(response.into())
        }
            .boxed()
    }

    fn get_startup_info(&self) -> Result<Option<storage_proto::StartupInfo>> {
        block_on(self.get_startup_info_async())
    }

    fn get_history_startup_info_by_block_id(
        &self,
        block_id: HashValue,
    ) -> Result<Option<storage_proto::StartupInfo>> {
        let info = self
            .get_history_startup_info_by_block_id_inner(&block_id)
            .and_then(|resp| storage_proto::GetStartupInfoResponse::try_from(resp))
            .unwrap();
        Ok(info.info)
    }

    fn get_startup_info_async(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<storage_proto::StartupInfo>>> + Send>> {
        let info = self
            .get_startup_info_inner()
            .and_then(|resp| storage_proto::GetStartupInfoResponse::try_from(resp))
            .unwrap();
        async { Ok(info.info) }.boxed()
    }

    fn get_latest_ledger_infos_per_epoch(
        &self,
        start_epoch: u64,
    ) -> Result<Vec<LedgerInfoWithSignatures>> {
        block_on(self.get_latest_ledger_infos_per_epoch_async(start_epoch))
    }

    fn get_latest_ledger_infos_per_epoch_async(
        &self,
        start_epoch: u64,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<LedgerInfoWithSignatures>>> + Send>> {
        let req = storage_proto::GetLatestLedgerInfosPerEpochRequest::new(start_epoch);
        let resp = self
            .get_latest_ledger_infos_per_epoch_inner(req.try_into().unwrap())
            .expect("get_latest_ledger_infos_per_epoch_async response err.");
        async {
            let response = storage_proto::GetLatestLedgerInfosPerEpochResponse::try_from(resp)?;
            Ok(response.into())
        }
            .boxed()
    }
}

impl StorageWrite for StorageService {
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
        let req = storage_proto::SaveTransactionsRequest::new(
            txns_to_commit,
            first_version,
            ledger_info_with_sigs,
        );

        let resp = self.save_transactions_inner(req.try_into().unwrap());
        async {
            resp?;
            Ok(())
        }
            .boxed()
    }

    fn rollback_by_block_id(&self, block_id: HashValue) {
        self.rollback_by_block_id_inner(&block_id)
            .expect("rollback failed.");
    }
}
