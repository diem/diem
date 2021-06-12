// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{CoreContext, Test};
use crate::Result;
use diem_sdk::{
    client::BlockingClient,
    types::{chain_id::ChainId, LocalAccount},
};

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

    admin_info: AdminInfo<'t>,
}

impl<'t> AdminContext<'t> {
    pub fn new(core: CoreContext, admin_info: AdminInfo<'t>) -> Self {
        Self { core, admin_info }
    }

    pub fn core(&self) -> &CoreContext {
        &self.core
    }

    pub fn client(&self) -> BlockingClient {
        BlockingClient::new(&self.admin_info.json_rpc_url)
    }
}

#[derive(Debug)]
pub struct AdminInfo<'t> {
    root_account: &'t mut LocalAccount,
    treasury_compliance_account: &'t mut LocalAccount,

    json_rpc_url: String,
    chain_id: ChainId,
}

impl<'t> AdminInfo<'t> {
    pub fn new(
        root_account: &'t mut LocalAccount,
        treasury_compliance_account: &'t mut LocalAccount,
        json_rpc_url: String,
        chain_id: ChainId,
    ) -> Self {
        Self {
            root_account,
            treasury_compliance_account,
            json_rpc_url,
            chain_id,
        }
    }
}
