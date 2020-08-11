// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{FuzzTarget, FuzzTargetImpl};
use anyhow::{format_err, Result};
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, env};

// List fuzz target modules here.
mod compiled_module;
mod consensus_proposal;
mod execute_and_commit_chunk;
mod inbound_rpc_protocol;
mod inner_signed_transaction;
mod json_rpc_service;
mod language_transaction_execution;
mod mempool;
mod network_noise;
mod state_sync_msg;
mod storage;
mod vm_value;

static ALL_TARGETS: Lazy<BTreeMap<&'static str, Box<dyn FuzzTargetImpl>>> = Lazy::new(|| {
    let targets: Vec<Box<dyn FuzzTargetImpl>> = vec![
        // List fuzz targets here in this format.
        Box::new(compiled_module::CompiledModuleTarget::default()),
        Box::new(consensus_proposal::ConsensusProposal::default()),
        Box::new(inbound_rpc_protocol::RpcInboundRequest::default()),
        Box::new(inner_signed_transaction::SignedTransactionTarget::default()),
        Box::new(json_rpc_service::JsonRpcSubmitTransactionRequest::default()),
        Box::new(mempool::MempoolIncomingTransactions::default()),
        Box::new(network_noise::NetworkNoiseInitiator::default()),
        Box::new(network_noise::NetworkNoiseResponder::default()),
        Box::new(network_noise::NetworkNoiseStream::default()),
        Box::new(state_sync_msg::StateSyncMsg::default()),
        Box::new(language_transaction_execution::LanguageTransactionExecution::default()),
        //        Box::new(storage_save_blocks::StorageSaveBlocks::default()),
        Box::new(storage::StorageSchemaDecode::default()),
        Box::new(vm_value::ValueTarget::default()),
        Box::new(execute_and_commit_chunk::ExecuteAndCommitChunk::default()),
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
