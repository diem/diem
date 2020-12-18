// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use diem_proptest_helpers::ValueGenerator;

//
// Noise wrapper
//

use network::noise::fuzzing::{
    fuzz_initiator, fuzz_post_handshake, fuzz_responder, generate_corpus,
};

#[derive(Clone, Debug, Default)]
pub struct NetworkNoiseInitiator;
impl FuzzTargetImpl for NetworkNoiseInitiator {
    fn description(&self) -> &'static str {
        "Network Noise crate initiator side"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(generate_corpus(gen))
    }

    fn fuzz(&self, data: &[u8]) {
        fuzz_initiator(data);
    }
}

#[derive(Clone, Debug, Default)]
pub struct NetworkNoiseResponder;
impl FuzzTargetImpl for NetworkNoiseResponder {
    fn description(&self) -> &'static str {
        "Network Noise crate responder side"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(generate_corpus(gen))
    }

    fn fuzz(&self, data: &[u8]) {
        fuzz_responder(data);
    }
}

#[derive(Clone, Debug, Default)]
pub struct NetworkNoiseStream;
impl FuzzTargetImpl for NetworkNoiseStream {
    fn description(&self) -> &'static str {
        "Network Noise crate stream"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(generate_corpus(gen))
    }

    fn fuzz(&self, data: &[u8]) {
        fuzz_post_handshake(data);
    }
}

//
// Handshake protocol
//

use network::fuzzing::{
    exchange_handshake_input, fuzz_network_handshake_protocol_exchange,
    fuzz_network_handshake_protocol_negotiation, perform_handshake_input,
};

#[derive(Clone, Debug, Default)]
pub struct NetworkHandshakeExchange;
impl FuzzTargetImpl for NetworkHandshakeExchange {
    fn description(&self) -> &'static str {
        "network handshake protocol starting with the exchange of HandshakeMsg"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(exchange_handshake_input()))
    }

    fn fuzz(&self, data: &[u8]) {
        let (own_handshake, their_handshake) = fuzz_data_to_value(data, exchange_handshake_input());
        fuzz_network_handshake_protocol_exchange(&own_handshake, their_handshake);
    }
}

#[derive(Clone, Debug, Default)]
pub struct NetworkHandshakeNegotiation;
impl FuzzTargetImpl for NetworkHandshakeNegotiation {
    fn description(&self) -> &'static str {
        "network handshake protocol skipping the exchange straight to the negotiation"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(perform_handshake_input()))
    }

    fn fuzz(&self, data: &[u8]) {
        let (own_handshake, their_handshake) = fuzz_data_to_value(data, perform_handshake_input());
        fuzz_network_handshake_protocol_negotiation(&own_handshake, &their_handshake);
    }
}

//
// Peer NetworkMessages Receive
//

use network::peer;

#[derive(Clone, Debug, Default)]
pub struct PeerNetworkMessagesReceive;
impl FuzzTargetImpl for PeerNetworkMessagesReceive {
    fn description(&self) -> &'static str {
        "network peer actor reading and deserializing multiple inbound NetworkMessages"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(peer::fuzzing::generate_corpus(gen))
    }

    fn fuzz(&self, data: &[u8]) {
        peer::fuzzing::fuzz(data);
    }
}
