// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::{self, FAILED_LABEL},
    peer::{PeerHandle, PeerNotification, PeerRequest},
    peer_manager::PeerManagerError,
    protocols::{
        direct_send::{DirectSend, DirectSendNotification, DirectSendRequest, Message},
        wire::messaging::v1::{DirectSendMsg, NetworkMessage, Priority},
    },
    ProtocolId,
};
use bytes::Bytes;
use futures::{sink::SinkExt, stream::StreamExt};
use libra_config::network_id::NetworkContext;
use libra_logger::debug;
use libra_types::PeerId;
use once_cell::sync::Lazy;
use serial_test::serial;
use std::sync::Arc;
use tokio::runtime::{Handle, Runtime};

const PROTOCOL_1: ProtocolId = ProtocolId::ConsensusDirectSend;
const PROTOCOL_2: ProtocolId = ProtocolId::MempoolDirectSend;
static MESSAGE_1: Lazy<Vec<u8>> = Lazy::new(|| Vec::from("Direct Send 1"));
static MESSAGE_2: Lazy<Vec<u8>> = Lazy::new(|| Vec::from("Direct Send 2"));

// counters are static and therefore shared across tests. This can sometimes lead to
// surprising counter readings if tests are run in parallel. Since we use counter values in some
// test cases, we run tests serially and reset counters in each test.
fn reset_counters() {
    counters::LIBRA_NETWORK_DIRECT_SEND_BYTES.reset();
    counters::LIBRA_NETWORK_DIRECT_SEND_MESSAGES.reset();
}

fn start_direct_send_actor(
    executor: Handle,
) -> (
    Arc<NetworkContext>,
    channel::Sender<DirectSendRequest>,
    channel::Receiver<DirectSendNotification>,
    channel::Sender<PeerNotification>,
    channel::Receiver<PeerRequest>,
) {
    let network_context = NetworkContext::mock();
    let (ds_requests_tx, ds_requests_rx) = channel::new_test(8);
    let (ds_notifs_tx, ds_notifs_rx) = channel::new_test(8);
    let (peer_notifs_tx, peer_notifs_rx) = channel::new_test(8);
    let (peer_reqs_tx, peer_reqs_rx) = channel::new_test(8);
    // Reset counters before starting actor.
    reset_counters();
    let direct_send = DirectSend::new(
        Arc::clone(&network_context),
        PeerHandle::new(PeerId::random(), peer_reqs_tx),
        ds_requests_rx,
        ds_notifs_tx,
        peer_notifs_rx,
    );
    executor.spawn(direct_send.start());

    (
        network_context,
        ds_requests_tx,
        ds_notifs_rx,
        peer_notifs_tx,
        peer_reqs_rx,
    )
}

async fn expect_network_provider_recv_message(
    ds_notifs_rx: &mut channel::Receiver<DirectSendNotification>,
    expected_protocol_id: ProtocolId,
    expected_message: Vec<u8>,
) {
    match ds_notifs_rx.next().await.unwrap() {
        DirectSendNotification::RecvMessage(msg) => {
            assert_eq!(msg.protocol_id, expected_protocol_id);
            assert_eq!(msg.mdata, expected_message);
        }
    }
}

async fn expect_send_message_request(
    peer_reqs_rx: &mut channel::Receiver<PeerRequest>,
    expected_protocol_id: ProtocolId,
    expected_message: DirectSendMsg,
    expected_result: Result<(), PeerManagerError>,
) {
    match peer_reqs_rx.next().await.unwrap() {
        PeerRequest::SendMessage(message, protocol_id, result_tx) => {
            assert_eq!(NetworkMessage::DirectSendMsg(expected_message), message);
            assert_eq!(protocol_id, expected_protocol_id);
            result_tx.send(expected_result).unwrap();
        }
        _ => panic!("Unexpected event"),
    }
}

#[test]
#[serial]
fn test_inbound_msg() {
    ::libra_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();

    let (_network_context, _ds_requests_tx, mut ds_notifs_rx, mut peer_notifs_tx, _peer_reqs_rx) =
        start_direct_send_actor(rt.handle().clone());

    // The dialer sends two messages to the listener.
    let f_substream = async move {
        debug!("Sending first message");
        peer_notifs_tx
            .send(PeerNotification::NewMessage(NetworkMessage::DirectSendMsg(
                DirectSendMsg {
                    protocol_id: PROTOCOL_1,
                    priority: Priority::default(),
                    raw_msg: MESSAGE_1.clone(),
                },
            )))
            .await
            .unwrap();
        debug!("Sending second message");
        peer_notifs_tx
            .send(PeerNotification::NewMessage(NetworkMessage::DirectSendMsg(
                DirectSendMsg {
                    protocol_id: PROTOCOL_2,
                    priority: Priority::default(),
                    raw_msg: MESSAGE_2.clone(),
                },
            )))
            .await
            .unwrap();
    };

    // The listener should receive these two messages
    let f_network_provider = async move {
        expect_network_provider_recv_message(&mut ds_notifs_rx, PROTOCOL_1, MESSAGE_1.clone())
            .await;
        expect_network_provider_recv_message(&mut ds_notifs_rx, PROTOCOL_2, MESSAGE_2.clone())
            .await;
    };

    rt.spawn(f_substream);
    rt.block_on(f_network_provider);
}

#[test]
#[serial]
fn test_outbound_msg() {
    let mut rt = Runtime::new().unwrap();

    let (_network_context, mut ds_requests_tx, _ds_notifs_rx, _peer_notifs_tx, mut peer_reqs_rx) =
        start_direct_send_actor(rt.handle().clone());

    // Fake the dialer NetworkProvider
    let f_network_provider = async move {
        let msg_sent = DirectSendRequest::SendMessage(Message {
            protocol_id: PROTOCOL_1,
            mdata: Bytes::from(MESSAGE_1.clone()),
        });
        debug!("Sending message");
        ds_requests_tx.send(msg_sent).await.unwrap();
    };

    // The listener should receive a request to send that message over the wire as a
    // NetworkMessage::DirectSendMsg, and return success.
    let f_substream = async move {
        let msg_received = DirectSendMsg {
            protocol_id: PROTOCOL_1,
            priority: Priority::default(),
            raw_msg: MESSAGE_1.clone(),
        };
        expect_send_message_request(&mut peer_reqs_rx, PROTOCOL_1, msg_received, Ok(())).await;
    };

    rt.spawn(f_network_provider);
    rt.block_on(f_substream);
}

#[test]
#[serial]
fn test_send_failure() {
    ::libra_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();

    let (network_context, mut ds_requests_tx, _ds_notifs_rx, _peer_notifs_tx, mut peer_reqs_rx) =
        start_direct_send_actor(rt.handle().clone());

    let peer_id = PeerId::random();

    // Fake the dialer NetworkProvider
    let f_network_provider = async move {
        // Request DirectSend to send the first message
        ds_requests_tx
            .send(DirectSendRequest::SendMessage(Message {
                protocol_id: PROTOCOL_1,
                mdata: Bytes::from(MESSAGE_1.clone()),
            }))
            .await
            .unwrap();
        // Request DirectSend to send the second message
        ds_requests_tx
            .send(DirectSendRequest::SendMessage(Message {
                protocol_id: PROTOCOL_1,
                mdata: Bytes::from(MESSAGE_2.clone()),
            }))
            .await
            .unwrap();
    };

    // The listener should receive the message.
    let f_substream = async move {
        // Peer returns the NotConnected error.
        expect_send_message_request(
            &mut peer_reqs_rx,
            PROTOCOL_1,
            DirectSendMsg {
                protocol_id: PROTOCOL_1,
                priority: Priority::default(),
                raw_msg: MESSAGE_1.clone(),
            },
            Err(PeerManagerError::NotConnected(peer_id)),
        )
        .await;
        // Peer returns Ok(()).
        expect_send_message_request(
            &mut peer_reqs_rx,
            PROTOCOL_1,
            DirectSendMsg {
                protocol_id: PROTOCOL_1,
                priority: Priority::default(),
                raw_msg: MESSAGE_2.clone(),
            },
            Ok(()),
        )
        .await;
        // Ensure failure counter has been incremented to 1.
        // NB: The fact that we check the counter after receiving the second request is due an
        // implementation detail, because after receiving the first request, we cannot be immediately
        // sure that it's result has been processed and the counter updated.
        assert_eq!(
            counters::direct_send_messages(&network_context, FAILED_LABEL).get() as u64,
            1
        );
    };

    rt.spawn(f_network_provider);
    rt.block_on(f_substream);
}
