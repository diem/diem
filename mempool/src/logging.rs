// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::config::PeerNetworkId;
use libra_logger::Schema;
use libra_types::account_address::AccountAddress;

#[derive(Schema, Default)]
pub(crate) struct MempoolSchema<'a> {
    #[schema(display)]
    sender: Option<&'a AccountAddress>,
    #[schema(display)]
    sequence_number: Option<&'a u64>,
    #[schema(display)]
    peer: Option<&'a PeerNetworkId>,
    #[schema(display)]
    error: Option<&'a anyhow::Error>,
}

impl<'a> MempoolSchema<'a> {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}
