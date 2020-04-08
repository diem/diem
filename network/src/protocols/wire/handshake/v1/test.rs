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

#[test]
fn protocols_to_from_vec() {
    let supported_protocols = vec![ProtocolId::ConsensusRpc, ProtocolId::MempoolDirectSend];
    assert_eq!(
        SupportedProtocols::from(&supported_protocols).try_into(),
        Ok(supported_protocols)
    );
}

#[test]
fn common_protocols() {
    let h1: BTreeMap<_, _> = [(
        MessagingProtocolVersion::V1,
        SupportedProtocols::from([ProtocolId::ConsensusRpc, ProtocolId::DiscoveryDirectSend]),
    )]
    .iter()
    .cloned()
    .collect();
    let h1 = HandshakeMsg {
        supported_protocols: h1,
    };

    // Case 1: One intersecting protocol is found for common messaging protocol version.
    let h2: BTreeMap<_, _> = [(
        MessagingProtocolVersion::V1,
        SupportedProtocols::from([ProtocolId::ConsensusRpc, ProtocolId::MempoolDirectSend]),
    )]
    .iter()
    .cloned()
    .collect();
    let h2 = HandshakeMsg {
        supported_protocols: h2,
    };
    assert_eq!(
        Some((MessagingProtocolVersion::V1, vec![ProtocolId::ConsensusRpc])),
        h1.find_common_protocols(&h2)
    );

    // Case 2: No intersecting messaging protocol version.
    let h2 = HandshakeMsg {
        supported_protocols: BTreeMap::default(),
    };
    assert_eq!(None, h1.find_common_protocols(&h2));

    // Case 3: Intersecting messaging protocol version is present, but no intersecting protocols.
    let h2: BTreeMap<_, _> = [(MessagingProtocolVersion::V1, SupportedProtocols::default())]
        .iter()
        .cloned()
        .collect();
    let h2 = HandshakeMsg {
        supported_protocols: h2,
    };
    assert_eq!(
        Some((MessagingProtocolVersion::V1, vec![])),
        h1.find_common_protocols(&h2)
    );
}
