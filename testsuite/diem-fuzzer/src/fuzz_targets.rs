// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{FuzzTarget, FuzzTargetImpl};
use anyhow::{format_err, Result};
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, env};

// List fuzz target modules here.
mod consensus;
mod executor;
mod json_rpc_service;
mod mempool;
mod move_vm;
mod network;
mod proof;
mod safety_rules;
mod secure_json_rpc_client;
mod secure_storage_vault;
mod state_sync;
mod storage;
mod transaction;
mod vm;

static ALL_TARGETS: Lazy<BTreeMap<&'static str, Box<dyn FuzzTargetImpl>>> = Lazy::new(|| {
    // List fuzz targets here in this format:
    let targets: Vec<Box<dyn FuzzTargetImpl>> = vec![
        // Consensus
        Box::new(consensus::ConsensusProposal::default()),
        // Executor
        Box::new(executor::ExecuteAndCommitBlocks::default()),
        Box::new(executor::ExecuteAndCommitChunk::default()),
        // JSON RPC Service
        Box::new(json_rpc_service::JsonRpcSubmitTransactionRequest::default()),
        Box::new(json_rpc_service::JsonRpcGetAccountRequest::default()),
        Box::new(json_rpc_service::JsonRpcGetAccountTransactionRequest::default()),
        Box::new(json_rpc_service::JsonRpcGetAccountTransactionsRequest::default()),
        Box::new(json_rpc_service::JsonRpcGetTransactionsRequest::default()),
        Box::new(json_rpc_service::JsonRpcGetEventsRequest::default()),
        Box::new(json_rpc_service::JsonRpcGetMetadataRequest::default()),
        Box::new(json_rpc_service::JsonRpcGetCurrenciesRequest::default()),
        Box::new(json_rpc_service::JsonRpcGetStateProofRequest::default()),
        Box::new(json_rpc_service::JsonRpcGetAccountStateWithProofRequest::default()),
        Box::new(json_rpc_service::JsonRpcGetNetworkStatusRequest::default()),
        // Mempool
        Box::new(mempool::MempoolIncomingTransactions::default()),
        // Move VM
        Box::new(move_vm::ValueTarget::default()),
        // Proof
        Box::new(proof::TestAccumulatorProofFuzzer::default()),
        Box::new(proof::SparseMerkleProofFuzzer::default()),
        Box::new(proof::TestAccumulatorRangeProofFuzzer::default()),
        Box::new(proof::TransactionInfoWithProofFuzzer::default()),
        Box::new(proof::AccountStateProofFuzzer::default()),
        Box::new(proof::EventProofFuzzer::default()),
        Box::new(proof::TransactionListProofFuzzer::default()),
        // Network
        Box::new(network::RpcInboundRequest::default()),
        Box::new(network::NetworkNoiseInitiator::default()),
        Box::new(network::NetworkNoiseResponder::default()),
        Box::new(network::NetworkNoiseStream::default()),
        Box::new(network::NetworkHandshakeExchange::default()),
        Box::new(network::NetworkHandshakeNegotiation::default()),
        Box::new(network::PeerNetworkMessagesReceive::default()),
        // Safety Rules Server (LSR)
        Box::new(safety_rules::SafetyRulesConstructAndSignVote::default()),
        Box::new(safety_rules::SafetyRulesInitialize::default()),
        Box::new(safety_rules::SafetyRulesHandleMessage::default()),
        Box::new(safety_rules::SafetyRulesSignProposal::default()),
        Box::new(safety_rules::SafetyRulesSignTimeout::default()),
        // Secure JSON RPC Client
        Box::new(secure_json_rpc_client::SecureJsonRpcSubmitTransaction::default()),
        Box::new(secure_json_rpc_client::SecureJsonRpcGetAccountState::default()),
        Box::new(secure_json_rpc_client::SecureJsonRpcGetAccountTransaction::default()),
        // Secure Storage Vault
        Box::new(secure_storage_vault::VaultGenericResponse::default()),
        Box::new(secure_storage_vault::VaultPolicyReadResponse::default()),
        Box::new(secure_storage_vault::VaultPolicyListResponse::default()),
        Box::new(secure_storage_vault::VaultSecretListResponse::default()),
        Box::new(secure_storage_vault::VaultSecretReadResponse::default()),
        Box::new(secure_storage_vault::VaultTokenCreateResponse::default()),
        Box::new(secure_storage_vault::VaultTokenRenewResponse::default()),
        Box::new(secure_storage_vault::VaultTransitCreateResponse::default()),
        Box::new(secure_storage_vault::VaultTransitExportResponse::default()),
        Box::new(secure_storage_vault::VaultTransitListResponse::default()),
        Box::new(secure_storage_vault::VaultTransitReadResponse::default()),
        Box::new(secure_storage_vault::VaultTransitRestoreResponse::default()),
        Box::new(secure_storage_vault::VaultTransitSignResponse::default()),
        Box::new(secure_storage_vault::VaultUnsealedResponse::default()),
        // State Sync
        Box::new(state_sync::StateSyncMsg::default()),
        // Storage
        // Box::new(storage::StorageSaveBlocks::default()),
        Box::new(storage::StorageSchemaDecode::default()),
        //Box::new(storage::JellyfishGetWithProof::default()),
        Box::new(storage::JellyfishGetWithProofWithDistinctLastNibble::default()),
        Box::new(storage::JellyfishGetRangeProof::default()),
        Box::new(storage::AccumulatorFrozenSubtreeHashes::default()),
        Box::new(storage::AccumulatorProof::default()),
        Box::new(storage::AccumulatorConsistencyProof::default()),
        Box::new(storage::AccumulatorRangeProof::default()),
        Box::new(storage::AccumulatorAppendMany::default()),
        Box::new(storage::AccumulatorAppendEmpty::default()),
        // Transaction
        Box::new(transaction::LanguageTransactionExecution::default()),
        Box::new(transaction::SignedTransactionTarget::default()),
        Box::new(transaction::MutatedSignedTransaction::default()),
        Box::new(transaction::TwoSignedTransactions::default()),
        // VM
        Box::new(vm::CompiledModuleTarget::default()),
    ];
    targets
        .into_iter()
        .map(|target| (target.name(), target))
        .collect()
});

impl FuzzTarget {
    /// The environment variable used for passing fuzz targets to child processes.
    pub(crate) const ENV_VAR: &'static str = "FUZZ_TARGET";

    /// Get the current fuzz target from the environment.
    pub fn from_env() -> Result<Self> {
        let name = env::var(Self::ENV_VAR)?;
        Self::by_name(&name).ok_or_else(|| format_err!("Unknown fuzz target '{}'", name))
    }

    /// Get a fuzz target by name.
    pub fn by_name(name: &str) -> Option<Self> {
        ALL_TARGETS.get(name).map(|target| FuzzTarget(&**target))
    }

    /// A list of all fuzz targets.
    pub fn all_targets() -> impl Iterator<Item = Self> {
        ALL_TARGETS.values().map(|target| FuzzTarget(&**target))
    }
}
