// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::Schema;
use libra_types::account_address::AccountAddress;

#[derive(Schema, Default)]
pub(crate) struct MempoolSchema<'a> {
    #[schema(display)]
    sender: Option<&'a AccountAddress>,
    #[schema(display)]
    sequence_number: Option<&'a u64>,
}

impl<'a> MempoolSchema<'a> {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}
