// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;

// Ensure serialization of ProtocolId enum takes 1 byte.
#[test]
fn protocol_id_serialization() -> lcs::Result<()> {
    let protocol = ProtocolId::ConsensusRpc;
    assert_eq!(lcs::to_bytes(&protocol)?, vec![0x00]);
    Ok(())
}

#[test]
fn error_code() -> lcs::Result<()> {
    let error_code = ErrorCode::TimedOut;
    assert_eq!(lcs::to_bytes(&error_code)?, vec![0x00]);
    Ok(())
}

#[test]
fn rpc_request() -> lcs::Result<()> {
    let rpc_request = RpcRequest {
        request_id: 25,
        protocol_id: ProtocolId::ConsensusRpc,
        priority: 0,
        raw_request: [0, 1, 2, 3].to_vec(),
    };
    assert_eq!(
        lcs::to_bytes(&rpc_request)?,
        // [25, 0, 0, 0] -> request_id
        // [0] -> protocol_idx
        // [0] -> priority
        // [4] -> length of raw_request
        // [0, 1, 2, 3] -> raw_request bytes
        vec![25, 0, 0, 0, 0, 0, 4, 0, 1, 2, 3]
    );
    Ok(())
}
