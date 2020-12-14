// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::testutils::fake_socket::{ReadOnlyTestSocket, ReadWriteTestSocket};
use bcs::test_helpers::assert_canonical_encode_decode;
use futures::{executor::block_on, future, sink::SinkExt, stream::StreamExt};
use memsocket::MemorySocket;
use proptest::{collection::vec, prelude::*};

// Ensure serialization of ProtocolId enum takes 1 byte.
#[test]
fn protocol_id_serialization() -> bcs::Result<()> {
    let protocol = ProtocolId::ConsensusRpc;
    assert_eq!(bcs::to_bytes(&protocol)?, vec![0x00]);
    Ok(())
}

#[test]
fn error_code() -> bcs::Result<()> {
    let error_code = ErrorCode::ParsingError(ParsingErrorType {
        message: 9,
        protocol: 5,
    });
    assert_eq!(bcs::to_bytes(&error_code)?, vec![0, 9, 5]);
    Ok(())
}

#[test]
fn rpc_request() -> bcs::Result<()> {
    let rpc_request = RpcRequest {
        request_id: 25,
        protocol_id: ProtocolId::ConsensusRpc,
        priority: 0,
        raw_request: [0, 1, 2, 3].to_vec(),
    };
    assert_eq!(
        bcs::to_bytes(&rpc_request)?,
        // [0] -> protocol_id
        // [25, 0, 0, 0] -> request_id
        // [0] -> priority
        // [4] -> length of raw_request
        // [0, 1, 2, 3] -> raw_request bytes
        vec![0, 25, 0, 0, 0, 0, 4, 0, 1, 2, 3]
    );
    Ok(())
}

#[test]
fn libranet_wire_test_vectors() {
    let message = NetworkMessage::DirectSendMsg(DirectSendMsg {
        protocol_id: ProtocolId::MempoolDirectSend,
        priority: 0,
        raw_msg: Vec::from("hello world"),
    });
    let message_bytes = [
        // [0, 0, 0, 15] -> frame length
        // [3] -> network message type
        // [2] -> protocol_id
        // [0] -> priority
        // [11] -> raw message length
        // [104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100] -> raw message bytes
        0_u8, 0, 0, 15, 3, 2, 0, 11, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100,
    ];

    // test reading and deserializing gives us the expected message

    let socket_rx = ReadOnlyTestSocket::new(&message_bytes);
    let message_rx = NetworkMessageStream::new(socket_rx, 128);

    let recv_messages = block_on(message_rx.collect::<Vec<_>>());
    let recv_messages = recv_messages
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(vec![message.clone()], recv_messages);

    // test serializing and writing gives us the expected bytes

    let (mut socket_tx, _socket_rx) = ReadWriteTestSocket::new_pair();
    let mut write_buf = Vec::new();
    socket_tx.save_writing(&mut write_buf);

    let mut message_tx = NetworkMessageSink::new(socket_tx, 128);
    block_on(message_tx.send(&message)).unwrap();

    assert_eq!(&write_buf, &message_bytes);
}

#[test]
fn send_fails_when_larger_than_frame_limit() {
    let (memsocket_tx, _memsocket_rx) = MemorySocket::new_pair();
    let mut message_tx = NetworkMessageSink::new(memsocket_tx, 64);

    // attempting to send an outbound message larger than your frame size will
    // return an Err
    let message = NetworkMessage::DirectSendMsg(DirectSendMsg {
        protocol_id: ProtocolId::ConsensusRpc,
        priority: 0,
        raw_msg: vec![0; 123],
    });
    block_on(message_tx.send(&message)).unwrap_err();
}

#[test]
fn recv_fails_when_larger_than_frame_limit() {
    let (memsocket_tx, memsocket_rx) = MemorySocket::new_pair();
    // sender won't error b/c their max frame size is larger
    let mut message_tx = NetworkMessageSink::new(memsocket_tx, 128);
    // receiver will reject the message b/c the frame size is > 64 bytes max
    let mut message_rx = NetworkMessageStream::new(memsocket_rx, 64);

    let message = NetworkMessage::DirectSendMsg(DirectSendMsg {
        protocol_id: ProtocolId::ConsensusRpc,
        priority: 0,
        raw_msg: vec![0; 80],
    });
    let f_send = message_tx.send(&message);
    let f_recv = message_rx.next();

    let (_, res_message) = block_on(future::join(f_send, f_recv));
    res_message.unwrap().unwrap_err();
}

fn arb_rpc_request(max_frame_size: usize) -> impl Strategy<Value = RpcRequest> {
    (
        any::<ProtocolId>(),
        any::<RequestId>(),
        any::<Priority>(),
        (0..max_frame_size).prop_map(|size| vec![0u8; size]),
    )
        .prop_map(
            |(protocol_id, request_id, priority, raw_request)| RpcRequest {
                protocol_id,
                request_id,
                priority,
                raw_request,
            },
        )
}

fn arb_rpc_response(max_frame_size: usize) -> impl Strategy<Value = RpcResponse> {
    (
        any::<RequestId>(),
        any::<Priority>(),
        (0..max_frame_size).prop_map(|size| vec![0u8; size]),
    )
        .prop_map(|(request_id, priority, raw_response)| RpcResponse {
            request_id,
            priority,
            raw_response,
        })
}

fn arb_direct_send_msg(max_frame_size: usize) -> impl Strategy<Value = DirectSendMsg> {
    let args = (
        any::<ProtocolId>(),
        any::<Priority>(),
        (0..max_frame_size).prop_map(|size| vec![0u8; size]),
    );
    args.prop_map(|(protocol_id, priority, raw_msg)| DirectSendMsg {
        protocol_id,
        priority,
        raw_msg,
    })
}

fn arb_network_message(max_frame_size: usize) -> impl Strategy<Value = NetworkMessage> {
    prop_oneof![
        any::<ErrorCode>().prop_map(NetworkMessage::Error),
        arb_rpc_request(max_frame_size).prop_map(NetworkMessage::RpcRequest),
        arb_rpc_response(max_frame_size).prop_map(NetworkMessage::RpcResponse),
        arb_direct_send_msg(max_frame_size).prop_map(NetworkMessage::DirectSendMsg),
    ]
    .prop_filter("larger than max frame size", move |msg| {
        bcs::serialized_size(&msg).unwrap() <= max_frame_size
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn network_message_canonical_serialization(message in any::<NetworkMessage>()) {
        assert_canonical_encode_decode(message);
    }

    /// Test that NetworkMessageSink and NetworkMessageStream can understand each
    /// other and fully preserve the NetworkMessages being sent
    #[test]
    fn network_message_socket_roundtrip(
        messages in vec(arb_network_message(128), 1..20),
        fragmented_read in any::<bool>(),
        fragmented_write in any::<bool>(),
    ) {
        let (mut socket_tx, mut socket_rx) = ReadWriteTestSocket::new_pair();

        if fragmented_read {
            socket_rx.set_fragmented_read();
        }
        if fragmented_write {
            socket_tx.set_fragmented_write();
        }

        let mut message_tx = NetworkMessageSink::new(socket_tx, 128);
        let message_rx = NetworkMessageStream::new(socket_rx, 128);

        let f_send_all = async {
            for message in &messages {
                message_tx.send(&message).await.unwrap();
            }
            message_tx.close().await.unwrap();
        };
        let f_recv_all = message_rx.collect::<Vec<_>>();

        let (_, recv_messages) = block_on(future::join(f_send_all, f_recv_all));

        for (message, recv_message) in messages.into_iter().zip(recv_messages.into_iter()) {
            assert_eq!(message, recv_message.unwrap());
        }
    }
}
