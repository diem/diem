// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    protocols::{
        identity::exchange_handshake,
        wire::handshake::v1::{HandshakeMsg, MessagingProtocolVersion, SupportedProtocols},
    },
    testutils::fake_socket::ReadOnlyTestSocketVec,
};
use diem_config::network_id::NetworkId;
use diem_types::chain_id::ChainId;
use futures::executor::block_on;
use proptest::{collection::btree_map, prelude::*};

//
// Handshake Protocol Fuzzer
// =========================
//

/// Serializes a HandshakeMsg by simulating sending it over a socket
fn serialize_handshake_message(handshake_msg: &HandshakeMsg) -> Vec<u8> {
    // serialize with BCS
    let handshake_msg = bcs::to_bytes(handshake_msg).unwrap();
    // prepend a 2-byte prefix indicating the message length
    let mut serialized = (handshake_msg.len() as u16).to_be_bytes().to_vec();
    serialized.extend_from_slice(&handshake_msg);
    serialized
}

/// Fuzzing the handshake protocol, which negotiates protocols supported by both
/// the client and the server.
/// At the moment, fuzzing the client or the server leads to the same logic.
pub fn fuzz_network_handshake_protocol_exchange(self_handshake: &HandshakeMsg, data: Vec<u8>) {
    // fake socket to read the other peer's serialized HandshakeMsg from
    let mut fake_socket = ReadOnlyTestSocketVec::new(data);
    fake_socket.set_trailing();

    // fuzz the network exchange of HandshakeMsg first
    let _ = block_on(async move {
        if let Ok(remote_handshake_msg) = exchange_handshake(self_handshake, &mut fake_socket).await
        {
            // then perform the negotiation
            let _ = self_handshake.perform_handshake(&remote_handshake_msg);
        }
    });
}

/// Same function as fuzz_network_handshake_protocol_exchange except that the network exchange is skipped,
/// letting us skip BCS deserialization (and potentially other logic) and fuzz the negotiation of protocols directly.
pub fn fuzz_network_handshake_protocol_negotiation(
    self_handshake: &HandshakeMsg,
    remote_handshake: &HandshakeMsg,
) {
    let _ = self_handshake.perform_handshake(remote_handshake);
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
  pub fn exchange_handshake_input()(
    self_handshake in build_handshake_msg(),
    remote_handshake in build_handshake_msg(),
  ) -> (HandshakeMsg, Vec<u8>) {
    (self_handshake, serialize_handshake_message(&remote_handshake))
  }
}

prop_compose! {
  /// Builds two HandshakeMsg and serializes the second one.
  /// It is the input expected for the fuzzer.
  pub fn perform_handshake_input()(
    self_handshake in build_handshake_msg(),
    remote_handshake in build_handshake_msg(),
  ) -> (HandshakeMsg, HandshakeMsg) {
    (self_handshake, remote_handshake)
  }
}

proptest! {
  #[test]
  fn test_handshake_exchange_fuzzer((self_handshake, remote_handshake) in exchange_handshake_input()) {
    fuzz_network_handshake_protocol_exchange(&self_handshake, remote_handshake);
  }

  #[test]
  fn test_handshake_negotiation_fuzzer((self_handshake, remote_handshake) in perform_handshake_input()) {
    fuzz_network_handshake_protocol_negotiation(&self_handshake, &remote_handshake);
  }
}
