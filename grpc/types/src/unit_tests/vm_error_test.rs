// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_prost_test_helpers::assert_protobuf_encode_decode;
use libra_types::vm_error::{StatusCode, VMStatus};
use std::convert::TryFrom;

#[test]
fn status_roundtrip() {
    let max_error_number = 5000;
    for status_number in 0..max_error_number {
        let status =
            StatusCode::try_from(status_number).unwrap_or_else(|_| StatusCode::UNKNOWN_STATUS);
        if status != StatusCode::UNKNOWN_STATUS {
            let stat_number: u64 = status.into();
            assert!(stat_number == status_number);
        }
        let status = VMStatus::new(status);
        assert_protobuf_encode_decode::<crate::proto::types::VmStatus, VMStatus>(&status);
    }
}
