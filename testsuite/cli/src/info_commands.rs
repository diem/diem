// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{client_proxy::ClientProxy, commands::Command};

/// Major command for account related operations.
pub struct InfoCommand {}

impl Command for InfoCommand {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["info", "i"]
    }
    fn get_description(&self) -> &'static str {
        "Print cli config and client internal information"
    }
    fn execute(&self, client: &mut ClientProxy, _params: &[&str]) {
        println!("ChainID: {}", client.chain_id);
        println!("Trusted State: {:#?}", client.client.trusted_state());
        println!("LedgerInfo: {:#?}", client.client.latest_epoch_change_li());
    }
}
