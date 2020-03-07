// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;

// Ensure serialization of MessagingProtocolVersion enum takes 1 byte.
#[test]
fn net_protocol() -> lcs::Result<()> {
    let protocol = MessagingProtocolVersion::V1;
    assert_eq!(lcs::to_bytes(&protocol)?, vec![0x00]);
    Ok(())
}
