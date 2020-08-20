use crate::{
    protocols::wire::handshake::v1::{HandshakeMsg, MessagingProtocolVersion, SupportedProtocols},
    testutils::fake_socket::ReadOnlyTestSocketVec,
    transport::perform_handshake,
};
use futures::executor::block_on;
use libra_config::network_id::NetworkId;
use libra_network_address::NetworkAddress;
use libra_types::{chain_id::ChainId, PeerId};
use netcore::transport::ConnectionOrigin;
use proptest::{collection::btree_map, prelude::*};

//
// Handshake Protocol Fuzzer
// =========================
//

/// Serializes a HandshakeMsg by simulating sending it over a socket
fn serialize_handshake_message(handshake_msg: &HandshakeMsg) -> Vec<u8> {
    // serialize with LCS
    let handshake_msg = lcs::to_bytes(handshake_msg).unwrap();
    // prepend a 2-byte prefix indicating the message length
    let mut serialized = (handshake_msg.len() as u16).to_be_bytes().to_vec();
    serialized.extend_from_slice(&handshake_msg);
    serialized
}

/// Fuzzing the handshake protocol, which negotiates protocols supported by both
/// the client and the server.
/// At the moment, fuzzing the client or the server leads to the same logic.
pub fn fuzz_network_handshake_protocol_negotiation(own_handshake: &HandshakeMsg, data: Vec<u8>) {
    // the logic is not impacted by these variables
    let peer_id = PeerId::ZERO;
    let origin = ConnectionOrigin::Outbound;
    let network_addr = NetworkAddress::mock();

    // put our own HandshakeMsg fake socket
    let mut fake_socket = ReadOnlyTestSocketVec::new(data);
    fake_socket.set_trailing();

    // fuzz it!
    let _ = block_on(async move {
        perform_handshake(peer_id, fake_socket, network_addr, origin, own_handshake).await
    });
}

prop_compose! {
  /// Builds an arbitrary HandshakeMsg
  fn build_handshake_msg()(
    supported_protocols in btree_map(
      any::<MessagingProtocolVersion>(),
      any::<SupportedProtocols>(),
      0..5
    ),
  ) -> HandshakeMsg {
    HandshakeMsg {
      supported_protocols,
      chain_id: ChainId::new(1), // doesn't matter for handshake protocol
      network_id: NetworkId::Validator, // doesn't matter for handshake protocol
    }
  }
}

prop_compose! {
  /// Builds two HandshakeMsg and serializes the second one.
  /// It is the input expected for the fuzzer.
  pub fn build_perform_handshake_input()(
    own_handshake in build_handshake_msg(),
    their_handshake in build_handshake_msg(),
  ) -> (HandshakeMsg, Vec<u8>) {
    (own_handshake, serialize_handshake_message(&their_handshake))
  }
}

proptest! {
  #[test]
  fn test_handshake_fuzzer((own_handshake, their_handshake) in build_perform_handshake_input()) {
    fuzz_network_handshake_protocol_negotiation(&own_handshake, their_handshake);
  }

  #[test]
  fn test33(a in any::<MessagingProtocolVersion>()) {

  }
}
