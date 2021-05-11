// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::Test;
use crate::{CoreContext, Result, Swarm};
use diem_sdk::types::{chain_id::ChainId, LocalAccount};

/// The testing interface which defines a test written with full control over an existing network.
/// Tests written against this interface will have access to both the Root account as well as the
/// nodes which comprise the network.
pub trait NetworkTest: Test {
    /// Executes the test against the given context.
    fn run<'t>(&self, ctx: &mut NetworkContext<'t>) -> Result<()>;
}

pub struct NetworkContext<'t> {
    core: CoreContext,
    swarm: &'t mut dyn Swarm,
}

impl<'t> NetworkContext<'t> {
    pub fn new(core: CoreContext, swarm: &'t mut dyn Swarm) -> Self {
        Self { core, swarm }
    }

    pub fn swarm(&mut self) -> &mut dyn Swarm {
        self.swarm
    }

    pub fn core(&mut self) -> &mut CoreContext {
        &mut self.core
    }
}

#[derive(Debug)]
pub struct ChainInfo<'t> {
    pub root_account: &'t mut LocalAccount,
    pub treasury_compliance_account: &'t mut LocalAccount,
    pub dd_account: &'t mut LocalAccount,
    pub json_rpc_url: &'t str,
    pub chain_id: ChainId,
}

impl<'t> ChainInfo<'t> {
    pub fn new(
        root_account: &'t mut LocalAccount,
        treasury_compliance_account: &'t mut LocalAccount,
        dd_account: &'t mut LocalAccount,
        json_rpc_url: &'t str,
        chain_id: ChainId,
    ) -> Self {
        Self {
            root_account,
            treasury_compliance_account,
            dd_account,
            json_rpc_url,
            chain_id,
        }
    }

    pub fn dd_account(&mut self) -> &mut LocalAccount {
        self.dd_account
    }

    pub fn root_account(&mut self) -> &mut LocalAccount {
        self.root_account
    }

    pub fn tc_account(&mut self) -> &mut LocalAccount {
        self.treasury_compliance_account
    }

    pub fn json_rpc(&self) -> &str {
        self.json_rpc_url
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }
}
