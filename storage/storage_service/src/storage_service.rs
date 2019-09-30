use crate::StorageService;
use config::config::NodeConfig;
use core::borrow::{Borrow, BorrowMut};
use crypto::ed25519::Ed25519Signature;
use failure::prelude::*;
use futures::{compat::Future01CompatExt, executor::block_on, prelude::*};
use grpc_helpers::{spawn_service_thread_with_drop_closure, ServerHandle};
use proto_conv::{FromProto, IntoProto};
use std::{pin::Pin, sync::Arc};
use storage_client::{StorageRead, StorageWrite};
use storage_proto::{
    proto::{
        storage::{
            GetAccountStateWithProofByVersionRequest, GetLatestLedgerInfosPerEpochRequest,
            GetTransactionsRequest, SaveTransactionsRequest,
        },
        storage_grpc::create_storage,
    },
    GetAccountStateWithProofByVersionResponse, GetLatestLedgerInfosPerEpochResponse, StartupInfo,
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

pub fn start_storage_service_and_return_service(
    config: &NodeConfig,
) -> (ServerHandle, Arc<StorageService>) {
    let (storage_service, shutdown_receiver) = StorageService::new(&config.storage.get_dir());
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
        LedgerInfoWithSignatures<Ed25519Signature>,
        Vec<ValidatorChangeEventWithProof<Ed25519Signature>>,
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
                        LedgerInfoWithSignatures<Ed25519Signature>,
                        Vec<ValidatorChangeEventWithProof<Ed25519Signature>>,
                    )>,
                > + Send,
        >,
    > {
        let req = UpdateToLatestLedgerRequest {
            client_known_version,
            requested_items,
        };
        let resp = self
            .update_to_latest_ledger_inner(req.into_proto())
            .expect("update_to_latest_ledger_inner response err.");
        async {
            let rust_resp = UpdateToLatestLedgerResponse::from_proto(resp)?;
            Ok((
                rust_resp.response_items,
                rust_resp.ledger_info_with_sigs,
                rust_resp.validator_change_events,
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
        let mut req = GetTransactionsRequest::new();
        req.set_batch_size(batch_size);
        req.set_fetch_events(fetch_events);
        req.set_start_version(start_version);
        req.set_ledger_version(ledger_version);
        let mut resp = self
            .get_transactions_inner(req)
            .expect("get_transactions_inner response err.");
        let proof = resp.take_txn_list_with_proof();
        async { TransactionListWithProof::from_proto(proof) }.boxed()
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
        let mut req = GetAccountStateWithProofByVersionRequest::new();
        req.set_version(version);
        req.set_address(address.into_proto());
        let resp = self
            .get_account_state_with_proof_by_version_inner(req)
            .expect("get_account_state_with_proof_by_version_inner response err.");
        async {
            let response = GetAccountStateWithProofByVersionResponse::from_proto(resp)?;
            Ok(response.into())
        }
            .boxed()
    }

    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        block_on(self.get_startup_info_async())
    }

    fn get_startup_info_async(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<StartupInfo>>> + Send>> {
        let info = match self.get_startup_info_inner() {
            Ok(mut resp) => Some(resp.take_info()),
            Err(_) => None,
        };

        async {
            let tmp = match info {
                Some(i) => match StartupInfo::from_proto(i) {
                    Ok(data) => Some(data),
                    Err(e) => None,
                },
                None => None,
            };
            Ok(tmp)
        }
            .boxed()
    }

    fn get_latest_ledger_infos_per_epoch(
        &self,
        start_epoch: u64,
    ) -> Result<Vec<LedgerInfoWithSignatures<Ed25519Signature>>> {
        block_on(self.get_latest_ledger_infos_per_epoch_async(start_epoch))
    }

    fn get_latest_ledger_infos_per_epoch_async(
        &self,
        start_epoch: u64,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<LedgerInfoWithSignatures<Ed25519Signature>>>> + Send>>
    {
        let mut req = GetLatestLedgerInfosPerEpochRequest::new();
        req.set_start_epoch(start_epoch);
        let resp = self
            .get_latest_ledger_infos_per_epoch_inner(req)
            .expect("get_latest_ledger_infos_per_epoch_async response err.");
        async {
            let response = GetLatestLedgerInfosPerEpochResponse::from_proto(resp)?;
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
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures<Ed25519Signature>>,
    ) -> Result<()> {
        block_on(self.save_transactions_async(txns_to_commit, first_version, ledger_info_with_sigs))
    }

    fn save_transactions_async(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures<Ed25519Signature>>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        let mut req = SaveTransactionsRequest::new();
        req.set_first_version(first_version);
        req.set_txns_to_commit(txns_to_commit.into_proto());
        match ledger_info_with_sigs {
            Some(info) => {
                req.set_ledger_info_with_signatures(info.into_proto());
            }
            None => {}
        };

        let resp = self.save_transactions_inner(req);
        async {
            resp?;
            Ok(())
        }
            .boxed()
    }
}
