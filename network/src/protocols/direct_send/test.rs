// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::NegotiatedSubstream,
    peer_manager::{
        PeerManagerError, PeerManagerNotification, PeerManagerRequest, PeerManagerRequestSender,
    },
    protocols::direct_send::{DirectSend, DirectSendNotification, DirectSendRequest, Message},
    ProtocolId,
};
use bytes::Bytes;
use channel;
use futures::{sink::SinkExt, stream::StreamExt};
use libra_types::PeerId;
use memsocket::MemorySocket;
use netcore::compat::IoCompat;
use tokio::runtime::{Handle, Runtime};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

const PROTOCOL_1: &[u8] = b"/direct_send/1.0.0";
const PROTOCOL_2: &[u8] = b"/direct_send/2.0.0";
const MESSAGE_1: &[u8] = b"Direct Send 1";
const MESSAGE_2: &[u8] = b"Direct Send 2";
const MESSAGE_3: &[u8] = b"Direct Send 3";

fn start_direct_send_actor(
    executor: Handle,
) -> (
    channel::Sender<DirectSendRequest>,
    channel::Receiver<DirectSendNotification>,
    channel::Sender<PeerManagerNotification<MemorySocket>>,
    channel::Receiver<PeerManagerRequest<MemorySocket>>,
) {
    let (ds_requests_tx, ds_requests_rx) = channel::new_test(8);
    let (ds_notifs_tx, ds_notifs_rx) = channel::new_test(8);
    let (peer_mgr_notifs_tx, peer_mgr_notifs_rx) = channel::new_test(8);
    let (peer_mgr_reqs_tx, peer_mgr_reqs_rx) = channel::new_test(8);
    let direct_send = DirectSend::new(
        executor.clone(),
        ds_requests_rx,
        ds_notifs_tx,
        peer_mgr_notifs_rx,
        PeerManagerRequestSender::new(peer_mgr_reqs_tx),
    );
    executor.spawn(direct_send.start());

    (
        ds_requests_tx,
        ds_notifs_rx,
        peer_mgr_notifs_tx,
        peer_mgr_reqs_rx,
    )
}

async fn expect_network_provider_recv_message(
    ds_notifs_rx: &mut channel::Receiver<DirectSendNotification>,
    expected_peer_id: PeerId,
    expected_protocol: &'static [u8],
    expected_message: &'static [u8],
) {
    match ds_notifs_rx.next().await.unwrap() {
        DirectSendNotification::RecvMessage(peer_id, msg) => {
            assert_eq!(peer_id, expected_peer_id);
            assert_eq!(msg.protocol.as_ref(), expected_protocol);
            assert_eq!(msg.mdata, Bytes::from_static(expected_message));
        }
    }
}

async fn expect_open_substream_request<TSubstream>(
    peer_mgr_reqs_rx: &mut channel::Receiver<PeerManagerRequest<TSubstream>>,
    expected_peer_id: PeerId,
    expected_protocol: &'static [u8],
    response: Result<TSubstream, PeerManagerError>,
) where
    TSubstream: std::fmt::Debug,
{
    match peer_mgr_reqs_rx.next().await.unwrap() {
        PeerManagerRequest::OpenSubstream(peer_id, protocol, substream_tx) => {
            assert_eq!(peer_id, expected_peer_id);
            assert_eq!(protocol.as_ref(), expected_protocol);
            substream_tx.send(response).unwrap();
        }
        _ => panic!("Unexpected event"),
    }
}

#[test]
fn test_inbound_substream() {
    let mut rt = Runtime::new().unwrap();

    let (_ds_requests_tx, mut ds_notifs_rx, mut peer_mgr_notifs_tx, _peer_mgr_reqs_rx) =
        start_direct_send_actor(rt.handle().clone());

    let peer_id = PeerId::random();
    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    // The dialer sends two messages to the listener.
    let f_substream = async move {
        let mut dialer_substream =
            Framed::new(IoCompat::new(dialer_substream), LengthDelimitedCodec::new());
        dialer_substream
            .send(Bytes::from_static(MESSAGE_1))
            .await
            .unwrap();
        dialer_substream
            .send(Bytes::from_static(MESSAGE_2))
            .await
            .unwrap();
    };

    // Fake the listener NetworkProvider to notify DirectSend of the inbound substream.
    let f_network_provider = async move {
        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewInboundSubstream(
                peer_id,
                NegotiatedSubstream {
                    protocol: ProtocolId::from_static(&PROTOCOL_1[..]),
                    substream: listener_substream,
                },
            ))
            .await
            .unwrap();

        // The listener should receive these two messages
        expect_network_provider_recv_message(&mut ds_notifs_rx, peer_id, PROTOCOL_1, MESSAGE_1)
            .await;
        expect_network_provider_recv_message(&mut ds_notifs_rx, peer_id, PROTOCOL_1, MESSAGE_2)
            .await;
    };

    rt.spawn(f_substream);
    rt.block_on(f_network_provider);
}

#[test]
fn test_inbound_substream_closed() {
    let mut rt = Runtime::new().unwrap();

    let (_ds_requests_tx, mut ds_notifs_rx, mut peer_mgr_notifs_tx, _peer_mgr_reqs_rx) =
        start_direct_send_actor(rt.handle().clone());

    let peer_id = PeerId::random();
    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    // The dialer sends a message to the listener.
    let f_substream = async move {
        let mut dialer_substream =
            Framed::new(IoCompat::new(dialer_substream), LengthDelimitedCodec::new());
        dialer_substream
            .send(Bytes::from_static(MESSAGE_1))
            .await
            .unwrap();
        // close the substream on the dialer side
        drop(dialer_substream);
    };

    // Fake the listener NetworkProvider
    let f_network_provider = async move {
        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewInboundSubstream(
                peer_id,
                NegotiatedSubstream {
                    protocol: ProtocolId::from_static(&PROTOCOL_1[..]),
                    substream: listener_substream,
                },
            ))
            .await
            .unwrap();

        expect_network_provider_recv_message(&mut ds_notifs_rx, peer_id, PROTOCOL_1, MESSAGE_1)
            .await;
    };

    rt.spawn(f_substream);
    rt.block_on(f_network_provider);
}

#[test]
fn test_outbound_single_protocol() {
    let mut rt = Runtime::new().unwrap();

    let (mut ds_requests_tx, _ds_notifs_rx, _peer_mgr_notifs_tx, mut peer_mgr_reqs_rx) =
        start_direct_send_actor(rt.handle().clone());

    let peer_id = PeerId::random();
    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    // Fake the dialer NetworkProvider
    let f_network_provider = async move {
        // Send 2 messages with the same protocol
        ds_requests_tx
            .send(DirectSendRequest::SendMessage(
                peer_id,
                Message {
                    protocol: Bytes::from_static(&PROTOCOL_1[..]),
                    mdata: Bytes::from_static(MESSAGE_1),
                },
            ))
            .await
            .unwrap();
        ds_requests_tx
            .send(DirectSendRequest::SendMessage(
                peer_id,
                Message {
                    protocol: Bytes::from_static(&PROTOCOL_1[..]),
                    mdata: Bytes::from_static(MESSAGE_2),
                },
            ))
            .await
            .unwrap();

        // DirectSend actor should request to open a substream with the same protocol
        expect_open_substream_request(
            &mut peer_mgr_reqs_rx,
            peer_id,
            PROTOCOL_1,
            Ok(dialer_substream),
        )
        .await;
    };

    // The listener should receive these two messages.
    let f_substream = async move {
        let mut listener_substream = Framed::new(
            IoCompat::new(listener_substream),
            LengthDelimitedCodec::new(),
        );
        let msg = listener_substream.next().await.unwrap().unwrap();
        assert_eq!(msg.as_ref(), MESSAGE_1);
        let msg = listener_substream.next().await.unwrap().unwrap();
        assert_eq!(msg.as_ref(), MESSAGE_2);
    };

    rt.spawn(f_network_provider);
    rt.block_on(f_substream);
}

#[test]
fn test_outbound_multiple_protocols() {
    let mut rt = Runtime::new().unwrap();

    let (mut ds_requests_tx, _ds_notifs_rx, _peer_mgr_notifs_tx, mut peer_mgr_reqs_rx) =
        start_direct_send_actor(rt.handle().clone());

    let peer_id = PeerId::random();
    let (dialer_substream_1, listener_substream_1) = MemorySocket::new_pair();
    let (dialer_substream_2, listener_substream_2) = MemorySocket::new_pair();

    // Fake the dialer NetworkProvider
    let f_network_provider = async move {
        // Send 2 messages with different protocols to the same peer
        ds_requests_tx
            .send(DirectSendRequest::SendMessage(
                peer_id,
                Message {
                    protocol: Bytes::from_static(&PROTOCOL_1[..]),
                    mdata: Bytes::from_static(MESSAGE_1),
                },
            ))
            .await
            .unwrap();
        ds_requests_tx
            .send(DirectSendRequest::SendMessage(
                peer_id,
                Message {
                    protocol: Bytes::from_static(&PROTOCOL_2[..]),
                    mdata: Bytes::from_static(MESSAGE_2),
                },
            ))
            .await
            .unwrap();

        // DirectSend actor should request to open 2 different substreams.
        expect_open_substream_request(
            &mut peer_mgr_reqs_rx,
            peer_id,
            PROTOCOL_1,
            Ok(dialer_substream_1),
        )
        .await;
        expect_open_substream_request(
            &mut peer_mgr_reqs_rx,
            peer_id,
            PROTOCOL_2,
            Ok(dialer_substream_2),
        )
        .await;
    };

    // The listener should receive 1 message on each substream.
    let f_substream = async move {
        let mut listener_substream_1 = Framed::new(
            IoCompat::new(listener_substream_1),
            LengthDelimitedCodec::new(),
        );
        let msg = listener_substream_1.next().await.unwrap().unwrap();
        assert_eq!(msg.as_ref(), MESSAGE_1);
        let mut listener_substream_2 = Framed::new(
            IoCompat::new(listener_substream_2),
            LengthDelimitedCodec::new(),
        );
        let msg = listener_substream_2.next().await.unwrap().unwrap();
        assert_eq!(msg.as_ref(), MESSAGE_2);
    };

    rt.spawn(f_network_provider);
    rt.block_on(f_substream);
}

#[test]
fn test_outbound_not_connected() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();

    let (mut ds_requests_tx, _ds_notifs_rx, _peer_mgr_notifs_tx, mut peer_mgr_reqs_rx) =
        start_direct_send_actor(rt.handle().clone());

    let peer_id = PeerId::random();
    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    // Fake the dialer NetworkProvider
    let f_network_provider = async move {
        // Request DirectSend to send the first message
        ds_requests_tx
            .send(DirectSendRequest::SendMessage(
                peer_id,
                Message {
                    protocol: Bytes::from_static(&PROTOCOL_1[..]),
                    mdata: Bytes::from_static(MESSAGE_1),
                },
            ))
            .await
            .unwrap();

        // PeerManager returns the NotConnected error
        expect_open_substream_request(
            &mut peer_mgr_reqs_rx,
            peer_id,
            PROTOCOL_1,
            Err(PeerManagerError::NotConnected(peer_id)),
        )
        .await;

        // Request DirectSend to send the second message
        ds_requests_tx
            .send(DirectSendRequest::SendMessage(
                peer_id,
                Message {
                    protocol: Bytes::from_static(&PROTOCOL_1[..]),
                    mdata: Bytes::from_static(MESSAGE_2),
                },
            ))
            .await
            .unwrap();

        // PeerManager returns the substream
        expect_open_substream_request(
            &mut peer_mgr_reqs_rx,
            peer_id,
            PROTOCOL_1,
            Ok(dialer_substream),
        )
        .await;
    };

    // The listener should receive the message.
    let f_substream = async move {
        let mut listener_substream = Framed::new(
            IoCompat::new(listener_substream),
            LengthDelimitedCodec::new(),
        );
        let msg = listener_substream.next().await.unwrap().unwrap();
        // Only the second message should be received, because when the first message is sent,
        // the peer isn't connected.
        assert_eq!(msg.as_ref(), MESSAGE_2);
    };

    rt.spawn(f_network_provider);
    rt.block_on(f_substream);
}

#[test]
fn test_outbound_connection_closed() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();

    let (mut ds_requests_tx, _ds_notifs_rx, _peer_mgr_notifs_tx, mut peer_mgr_reqs_rx) =
        start_direct_send_actor(rt.handle().clone());

    let peer_id = PeerId::random();
    let (dialer_substream_1, listener_substream_1) = MemorySocket::new_pair();
    let (dialer_substream_2, listener_substream_2) = MemorySocket::new_pair();

    // Send the first message and open the first substream
    let f_first_message = async move {
        // Request DirectSend to send the first message
        ds_requests_tx
            .send(DirectSendRequest::SendMessage(
                peer_id,
                Message {
                    protocol: Bytes::from_static(&PROTOCOL_1[..]),
                    mdata: Bytes::from_static(MESSAGE_1),
                },
            ))
            .await
            .unwrap();

        // PeerManager returns the first substream
        expect_open_substream_request(
            &mut peer_mgr_reqs_rx,
            peer_id,
            PROTOCOL_1,
            Ok(dialer_substream_1),
        )
        .await;

        (ds_requests_tx, peer_mgr_reqs_rx)
    };
    let (mut ds_requests_tx, mut peer_mgr_reqs_rx) = rt.block_on(f_first_message);

    // Receive the first message and close the first substream
    let f_close_first_substream = async move {
        let mut listener_substream = Framed::new(
            IoCompat::new(listener_substream_1),
            LengthDelimitedCodec::new(),
        );
        let msg = listener_substream.next().await.unwrap().unwrap();
        // The listener should receive the first message.
        assert_eq!(msg.as_ref(), MESSAGE_1);
        // Close the substream by dropping it on the listener side
        drop(listener_substream);
    };
    rt.block_on(f_close_first_substream);

    // Send the second message while the connection is still lost.
    let f_second_message = async move {
        // Request DirectSend to send the second message
        ds_requests_tx
            .send(DirectSendRequest::SendMessage(
                peer_id,
                Message {
                    protocol: Bytes::from_static(&PROTOCOL_1[..]),
                    mdata: Bytes::from_static(MESSAGE_2),
                },
            ))
            .await
            .unwrap();

        ds_requests_tx
    };
    let mut ds_requests_tx = rt.block_on(f_second_message);

    // Keep sending the third message and open the second substream
    let f_third_message = async move {
        // Request DirectSend to send the third message
        loop {
            ds_requests_tx
                .send(DirectSendRequest::SendMessage(
                    peer_id,
                    Message {
                        protocol: Bytes::from_static(&PROTOCOL_1[..]),
                        mdata: Bytes::from_static(MESSAGE_3),
                    },
                ))
                .await
                .unwrap();
        }
    };
    rt.spawn(f_third_message);

    let f_open_second_substream = async move {
        // PeerManager returns the second substream
        expect_open_substream_request(
            &mut peer_mgr_reqs_rx,
            peer_id,
            PROTOCOL_1,
            Ok(dialer_substream_2),
        )
        .await;

        peer_mgr_reqs_rx
    };
    let mut peer_mgr_reqs_rx = rt.block_on(f_open_second_substream);

    // Fake peer manager to keep the PeerManagerRequest receiver
    let f_peer_manager = async move {
        loop {
            expect_open_substream_request(
                &mut peer_mgr_reqs_rx,
                peer_id,
                PROTOCOL_1,
                Err(PeerManagerError::NotConnected(peer_id)),
            )
            .await;
        }
    };
    rt.spawn(f_peer_manager);

    // The listener should only receive the third message through the second substream.
    let f_second_substream = async move {
        let mut listener_substream = Framed::new(
            IoCompat::new(listener_substream_2),
            LengthDelimitedCodec::new(),
        );
        let msg = listener_substream.next().await.unwrap().unwrap();
        assert_eq!(msg.as_ref(), MESSAGE_3);
    };
    rt.block_on(f_second_substream);
}
