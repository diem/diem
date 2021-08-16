// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::transaction::TransactionPayload;

pub const RELEASE_1_2_0_PATCH: &[u8] = include_bytes!("../release/release-1.2.0-rc0.blob");
pub const RELEASE_1_4_0_PATCH: &[u8] = include_bytes!("../release/release-1.4.0-rc0.blob");

pub fn release_1_4_0_writeset() -> TransactionPayload {
    bcs::from_bytes(RELEASE_1_4_0_PATCH).expect("release-1.4.0 patch couldn't be deserialized")
}

pub fn release_1_2_0_writeset() -> TransactionPayload {
    bcs::from_bytes(RELEASE_1_2_0_PATCH).expect("release-1.2.0 patch couldn't be deserialized")
}
