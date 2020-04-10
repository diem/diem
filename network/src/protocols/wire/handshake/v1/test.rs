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
    let supported_protocols: SupportedProtocols =
        [ProtocolId::ConsensusRpc, ProtocolId::MempoolDirectSend]
            .iter()
            .into();
    assert_eq!(
        SupportedProtocols::from(
            (supported_protocols.clone().try_into() as Result<Vec<ProtocolId>, _>)
                .unwrap()
                .iter()
        ),
        supported_protocols
    );
}

#[test]
fn common_protocols() {
    let h1: BTreeMap<_, _> = [(
        MessagingProtocolVersion::V1,
        [ProtocolId::ConsensusRpc, ProtocolId::DiscoveryDirectSend]
            .iter()
            .into(),
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
        [ProtocolId::ConsensusRpc, ProtocolId::MempoolDirectSend]
            .iter()
            .into(),
    )]
    .iter()
    .cloned()
    .collect();
    let h2 = HandshakeMsg {
        supported_protocols: h2,
    };
    assert_eq!(
        Some((
            MessagingProtocolVersion::V1,
            [ProtocolId::ConsensusRpc].iter().into()
        )),
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
        Some((MessagingProtocolVersion::V1, [].iter().into())),
        h1.find_common_protocols(&h2)
    );
}
