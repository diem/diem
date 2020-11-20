// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants,
    peer::{PeerHandle, PeerRequest},
    protocols::{
        rpc::{self, RpcNotification},
        wire::messaging::v1::{
            network_message_frame_codec, NetworkMessage, RpcRequest, RpcResponse,
        },
    },
    transport::ConnectionMetadata,
    ProtocolId,
};
use bytes::{Bytes, BytesMut};
use diem_config::network_id::NetworkContext;
use diem_proptest_helpers::ValueGenerator;
use diem_types::PeerId;
use futures::{
    future::{self, FutureExt},
    stream::StreamExt,
};
use proptest::{arbitrary::any, collection::vec, prop_oneof, strategy::Strategy};
use std::io;
use tokio::runtime;
use tokio_util::codec::Encoder;

// Length of unsigned varint prefix in bytes for a u128-sized length
const MAX_UVI_PREFIX_BYTES: usize = 19;

// Corpus size classes
const MAX_SMALL_MSG_BYTES: usize = 32;
const MAX_MEDIUM_MSG_BYTES: usize = 280;

const MOCK_PEER_ID_1: PeerId = PeerId::new([1u8; PeerId::LENGTH]);
const MOCK_PEER_ID_2: PeerId = PeerId::new([2u8; PeerId::LENGTH]);
const MOCK_PROTOCOL_ID: ProtocolId = ProtocolId::ConsensusRpc;

#[test]
fn test_fuzzer() {
    let mut gen = ValueGenerator::new();
    let data = generate_corpus(&mut gen);
    fuzzer(&data);
}

// Generate some random, well-formed, unsigned-varint length-prefixed byte arrays
// for our fuzzer corpus to act as serialized inbound rpc calls.
pub fn generate_corpus(gen: &mut ValueGenerator) -> Vec<u8> {
    let small_data_strat = vec(any::<u8>(), 0..MAX_SMALL_MSG_BYTES);
    let medium_data_strat = vec(any::<u8>(), 0..MAX_MEDIUM_MSG_BYTES);

    // bias corpus generation to prefer small message sizes
    let data_strat = prop_oneof![small_data_strat, medium_data_strat];

    let length_prefixed_data_strat = data_strat.prop_map(|data| {
        let max_len = data.len() + MAX_UVI_PREFIX_BYTES;
        let mut buf = BytesMut::with_capacity(max_len);
        let mut codec = network_message_frame_codec(constants::MAX_FRAME_SIZE);
        codec
            .encode(Bytes::from(data), &mut buf)
            .expect("Failed to create uvi-prefixed data for corpus");
        buf.freeze().to_vec()
    });

    gen.generate(length_prefixed_data_strat)
}

// Fuzz the inbound rpc protocol.
pub fn fuzzer(data: &[u8]) {
    let network_context = NetworkContext::mock_with_peer_id(MOCK_PEER_ID_1);
    let connection_metadata = ConnectionMetadata::mock(MOCK_PEER_ID_2);
    let (notification_tx, mut notification_rx) = channel::new_test(8);
    let (peer_reqs_tx, mut peer_reqs_rx) = channel::new_test(8);
    let raw_request = Vec::from(data);
    let inbound_request = RpcRequest {
        protocol_id: MOCK_PROTOCOL_ID,
        request_id: 0,
        priority: 0,
        // write the fuzzer data into the in-memory substream
        raw_request: raw_request.clone(),
    };
    // run the rpc inbound protocol using the in-memory substream
    let f_handle_inbound = rpc::handle_inbound_request_inner(
        &network_context,
        notification_tx,
        inbound_request,
        PeerHandle::new(network_context.clone(), connection_metadata, peer_reqs_tx),
    )
    .map(|_| io::Result::Ok(()));

    // mock the notification channel to echo the fuzzer data back to the dialer
    // as an rpc response
    let f_respond_inbound = async move {
        // Wait for notification from RPC actor about inbound RPC.
        let notif = notification_rx.next().await.unwrap();
        match notif {
            RpcNotification::RecvRpc(req) => {
                let protocol_id = req.protocol_id;
                let data = req.data;
                let res_tx = req.res_tx;
                assert_eq!(protocol_id, MOCK_PROTOCOL_ID);
                let _ = res_tx.send(Ok(data));
            }
        }

        // Echo the fuzzer data back in RpcResponse.
        let outbound_response = NetworkMessage::RpcResponse(RpcResponse {
            request_id: 0,
            priority: 0,
            raw_response: raw_request.clone(),
        });
        if let Some(req) = peer_reqs_rx.next().await {
            match req {
                PeerRequest::SendMessage(response_message, protocol_id, res_tx) => {
                    // when testing, we only run the fuzzer with well-formed inputs, so we
                    // should successfully reach this point and read the same data back
                    if cfg!(test) {
                        assert_eq!(protocol_id, MOCK_PROTOCOL_ID);
                        assert_eq!(response_message, outbound_response);
                    }
                    res_tx.send(Ok(())).unwrap();
                }
                _ => {
                    panic!("unexpected peer request");
                }
            }
        }

        io::Result::Ok(())
    };

    let f = future::try_join(f_handle_inbound, f_respond_inbound);
    // we need to use tokio runtime since Rpc uses tokio timers
    let res = runtime::Builder::new()
        .basic_scheduler()
        .build()
        .unwrap()
        .block_on(f);

    // there should be no errors when testing with well-formed inputs
    if cfg!(test) {
        res.expect("Fuzzing should succeed when run on the corpus");
    }
}
