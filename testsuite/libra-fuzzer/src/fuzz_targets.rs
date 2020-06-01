// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{FuzzTarget, FuzzTargetImpl};
use anyhow::{format_err, Result};
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, env};

/// Convenience macro to return the module name.
macro_rules! module_name {
    () => {
        module_path!()
            .rsplit("::")
            .next()
            .expect("module path must have at least one component")
    };
}

// List fuzz target modules here.
mod compiled_module;
mod consensus_proposal;
mod inbound_rpc_protocol;
mod inner_signed_transaction;
mod json_rpc_service;
mod network_noise_initiator;
mod network_noise_responder;
//mod storage_save_blocks;
mod storage_schema_decode;
mod vm_value;

static ALL_TARGETS: Lazy<BTreeMap<&'static str, Box<dyn FuzzTargetImpl>>> = Lazy::new(|| {
    let targets: Vec<Box<dyn FuzzTargetImpl>> = vec![
        // List fuzz targets here in this format.
        Box::new(compiled_module::CompiledModuleTarget::default()),
        Box::new(consensus_proposal::ConsensusProposal::default()),
        Box::new(inbound_rpc_protocol::RpcInboundRequest::default()),
        Box::new(inner_signed_transaction::SignedTransactionTarget::default()),
        Box::new(json_rpc_service::JsonRpcSubmitTransactionRequest::default()),
        Box::new(network_noise_initiator::NetworkNoiseInitiator::default()),
        Box::new(network_noise_responder::NetworkNoiseResponder::default()),
        //        Box::new(storage_save_blocks::StorageSaveBlocks::default()),
        Box::new(storage_schema_decode::StorageSchemaDecode::default()),
        Box::new(vm_value::ValueTarget::default()),
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
