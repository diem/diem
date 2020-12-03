// Copyright (c) The Diem Core Contributors
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
fn represents_same_network() {
    let mut handshake_msg = HandshakeMsg::new_for_testing();
    handshake_msg.network_id = NetworkId::Private("h1".to_string());

    // succeeds: Positive case
    let h1 = handshake_msg.clone();
    let h2 = handshake_msg.clone();
    h1.perform_handshake(&h2).unwrap();

    // fails: another private network
    let mut h2 = handshake_msg.clone();
    h2.network_id = NetworkId::Private("h2".to_string());
    h1.perform_handshake(&h2).unwrap_err();

    // fails: different network
    let mut h2 = handshake_msg.clone();
    h2.network_id = NetworkId::Public;
    h1.perform_handshake(&h2).unwrap_err();

    // fails: different chain
    let mut h2 = handshake_msg;
    h2.chain_id = ChainId::new(h1.chain_id.id() + 1);
    h1.perform_handshake(&h2).unwrap_err();
}

#[test]
fn common_protocols() {
    let network_id = NetworkId::default();
    let chain_id = ChainId::default();
    let mut supported_protocols = BTreeMap::new();
    supported_protocols.insert(
        MessagingProtocolVersion::V1,
        [ProtocolId::ConsensusRpc, ProtocolId::DiscoveryDirectSend]
            .iter()
            .into(),
    );

    let h1 = HandshakeMsg {
        chain_id,
        network_id: network_id.clone(),
        supported_protocols,
    };

    // Case 1: One intersecting protocol is found for common messaging protocol version.
    let mut supported_protocols = BTreeMap::new();
    supported_protocols.insert(
        MessagingProtocolVersion::V1,
        [ProtocolId::ConsensusRpc, ProtocolId::MempoolDirectSend]
            .iter()
            .into(),
    );
    let h2 = HandshakeMsg {
        chain_id,
        network_id: network_id.clone(),
        supported_protocols,
    };

    assert_eq!(
        (
            MessagingProtocolVersion::V1,
            [ProtocolId::ConsensusRpc].iter().into()
        ),
        h1.perform_handshake(&h2).unwrap()
    );

    // Case 2: No intersecting messaging protocol version.
    let h2 = HandshakeMsg {
        chain_id,
        network_id: network_id.clone(),
        supported_protocols: BTreeMap::new(),
    };
    h1.perform_handshake(&h2).unwrap_err();

    // Case 3: Intersecting messaging protocol version is present, but no intersecting protocols.
    let mut supported_protocols = BTreeMap::new();
    supported_protocols.insert(MessagingProtocolVersion::V1, SupportedProtocols::default());
    let h2 = HandshakeMsg {
        chain_id,
        network_id,
        supported_protocols,
    };

    assert_eq!(
        (MessagingProtocolVersion::V1, [].iter().into()),
        h1.perform_handshake(&h2).unwrap()
    );
}
