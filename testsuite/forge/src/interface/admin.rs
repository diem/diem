// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{ChainInfo, CoreContext, Test};
use crate::Result;
use diem_sdk::client::BlockingClient;

/// The testing interface which defines a test written from the perspective of the Admin of the
/// network. This means that the test will have access to the Root account but do not control any
/// of the validators or full nodes running on the network.
pub trait AdminTest: Test {
    /// Executes the test against the given context.
    fn run<'t>(&self, ctx: &mut AdminContext<'t>) -> Result<()>;
}

#[derive(Debug)]
pub struct AdminContext<'t> {
    core: CoreContext,

    chain_info: ChainInfo<'t>,
}

impl<'t> AdminContext<'t> {
    pub fn new(core: CoreContext, chain_info: ChainInfo<'t>) -> Self {
        Self { core, chain_info }
    }

    pub fn core(&self) -> &CoreContext {
        &self.core
    }

    pub fn client(&self) -> BlockingClient {
        BlockingClient::new(&self.chain_info.json_rpc_url)
    }
}
