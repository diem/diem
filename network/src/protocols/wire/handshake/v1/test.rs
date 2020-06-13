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
fn represents_same_network() {
    let network_id = NetworkId::Private("h1".to_string());
    let chain_id = ChainId::new("h1");

    // Positive case
    let h1 = HandshakeMsg::new(chain_id.clone(), network_id.clone());
    let h2 = HandshakeMsg::new(chain_id.clone(), network_id.clone());
    assert!(h1.verify(&h2));

    // Negative cases
    let h2 = HandshakeMsg::new(chain_id.clone(), NetworkId::Private("h2".to_string()));
    assert!(!h1.verify(&h2));
    let h2 = HandshakeMsg::new(chain_id, NetworkId::Public);
    assert!(!h1.verify(&h2));
    let h2 = HandshakeMsg::new(ChainId::new("h2"), network_id);
    assert!(!h1.verify(&h2));
}

#[test]
fn common_protocols() {
    let network_id = NetworkId::default();
    let chain_id = ChainId::default();

    let mut h1 = HandshakeMsg::new(chain_id.clone(), network_id.clone());
    h1.add(
        MessagingProtocolVersion::V1,
        [ProtocolId::ConsensusRpc, ProtocolId::DiscoveryDirectSend]
            .iter()
            .into(),
    );

    // Case 1: One intersecting protocol is found for common messaging protocol version.
    let mut h2 = HandshakeMsg::new(chain_id.clone(), network_id.clone());
    h2.add(
        MessagingProtocolVersion::V1,
        [ProtocolId::ConsensusRpc, ProtocolId::MempoolDirectSend]
            .iter()
            .into(),
    );
    assert_eq!(
        Some((
            MessagingProtocolVersion::V1,
            [ProtocolId::ConsensusRpc].iter().into()
        )),
        h1.find_common_protocols(&h2)
    );

    // Case 2: No intersecting messaging protocol version.
    let h2 = HandshakeMsg {
        network_id: network_id.clone(),
        supported_protocols: BTreeMap::default(),
        chain_id: chain_id.clone(),
    };
    assert_eq!(None, h1.find_common_protocols(&h2));

    // Case 3: Intersecting messaging protocol version is present, but no intersecting protocols.
    let mut h2 = HandshakeMsg::new(chain_id, network_id);
    h2.add(MessagingProtocolVersion::V1, SupportedProtocols::default());
    assert_eq!(
        Some((MessagingProtocolVersion::V1, [].iter().into())),
        h1.find_common_protocols(&h2)
    );
}
