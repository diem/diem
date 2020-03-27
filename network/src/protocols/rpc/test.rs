// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{error::RpcError, *};
use crate::{
    common::NegotiatedSubstream,
    peer::{PeerNotification, PeerRequest},
};
use futures::future::{join, join3, join4};
use libra_types::PeerId;
use memsocket::MemorySocket;
use parity_multiaddr::Multiaddr;
use std::str::FromStr;
use tokio::runtime::Runtime;

static RPC_PROTOCOL: ProtocolId = ProtocolId::ConsensusRpc;

async fn do_outbound_rpc_req<TSubstream>(
    peer_tx: PeerHandle<TSubstream>,
    protocol: ProtocolId,
    data: Bytes,
    timeout: Duration,
) -> Result<Bytes, RpcError>
where
    TSubstream: AsyncRead + AsyncWrite + Send + Unpin,
{
    let (res_tx, res_rx) = oneshot::channel();
    let outbound_req = OutboundRpcRequest {
        protocol,
        data,
        res_tx,
        timeout,
    };
    let rpc_req = RpcRequest::SendRpc(outbound_req);
    handle_outbound_rpc(peer_tx, rpc_req).await;
    res_rx.await.unwrap()
}

// On the next OpenSubstream event, return the given substream.
async fn mock_peer<TSubstream: Debug>(
    mut peer_rx: channel::Receiver<PeerRequest<TSubstream>>,
    substream: TSubstream,
) {
    // Return a mocked substream on the next OpenSubstream request
    match peer_rx.next().await.unwrap() {
        PeerRequest::OpenSubstream(_protocol, substream_tx) => {
            substream_tx.send(Ok(substream)).unwrap();
        }
        req => panic!("Unexpected PeerRequest: {:?}, expected OpenSubstream", req),
    }
}

// Test the rpc substream upgrades.
#[test]
fn upgrades() {
    ::libra_logger::Logger::new().environment_only(true).init();

    let listener_peer_id = PeerId::random();
    let dialer_peer_id = PeerId::random();
    let protocol_id = RPC_PROTOCOL;
    let req_data = b"hello";
    let res_data = b"goodbye";

    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    // Fake the listener Peer created at the dialer.
    let (listener_peer_reqs_tx, listener_peer_reqs_rx) = channel::new_test(8);
    let listener_peer_reqs_tx = PeerHandle::new(
        listener_peer_id,
        Multiaddr::from_str("/ip6/::1/tcp/8081").unwrap(),
        listener_peer_reqs_tx,
    );
    let f_listener_peer = mock_peer(listener_peer_reqs_rx, dialer_substream);

    // Fake the dialer Peer created at the listener
    let (dialer_rpc_notifs_tx, mut dialer_rpc_notifs_rx) = channel::new_test(8);
    let f_diler_peer = async move {
        // Handle the inbound rpc request
        match dialer_rpc_notifs_rx.select_next_some().await {
            RpcNotification::RecvRpc(req) => {
                assert_eq!(req.protocol, protocol_id);
                assert_eq!(req.data.as_ref(), req_data);
                req.res_tx.send(Ok(Bytes::from_static(res_data))).unwrap();
            }
        }
    };

    let substream = NegotiatedSubstream {
        protocol: protocol_id,
        substream: listener_substream,
    };
    let inbound_notif = PeerNotification::NewSubstream(dialer_peer_id, substream);

    // Handle the inbound substream
    let f_dialer_upgrade = handle_inbound_substream(
        dialer_rpc_notifs_tx,
        inbound_notif,
        Duration::from_millis(500),
    );

    // Make an outbound substream request
    let f_listener_upgrade = async move {
        let res = do_outbound_rpc_req(
            listener_peer_reqs_tx,
            protocol_id,
            Bytes::from_static(req_data),
            Duration::from_secs(1),
        )
        .await;

        // Check the rpc response data
        let data = res.unwrap();
        assert_eq!(data.as_ref(), res_data);
    };

    let f = join4(
        f_listener_peer,
        f_diler_peer,
        f_listener_upgrade,
        f_dialer_upgrade,
    );
    Runtime::new().unwrap().block_on(f);
}

// An outbound rpc request should fail if the listener drops the connection after
// receiving the request.
#[test]
fn listener_close_before_response() {
    ::libra_logger::Logger::new().environment_only(true).init();

    let listener_peer_id = PeerId::random();
    let protocol_id = RPC_PROTOCOL;
    let req_data = b"hello";

    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    // Fake the listener Peer created at the dialer.
    let (listener_peer_reqs_tx, listener_peer_reqs_rx) = channel::new_test(8);
    let listener_peer_reqs_tx = PeerHandle::new(
        listener_peer_id,
        Multiaddr::from_str("/ip6/::1/tcp/8081").unwrap(),
        listener_peer_reqs_tx,
    );
    let f_listener_peer = mock_peer(listener_peer_reqs_rx, dialer_substream);

    // Make an outbound rpc request
    let f_dialer_upgrade = async move {
        let res = do_outbound_rpc_req(
            listener_peer_reqs_tx,
            protocol_id,
            Bytes::from_static(req_data),
            Duration::from_secs(1),
        )
        .await;

        // Check the error
        let err = res.expect_err("Dialer's rpc request should fail");
        match err {
            RpcError::IoError(err) => assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof),
            err => panic!("Unexpected error: {:?}, expected IoError", err),
        };
    };

    // Listener reads the request but then drops the connection
    let f_listener = async move {
        // rpc messages are length-prefixed
        let mut substream = Framed::new(
            IoCompat::new(listener_substream),
            LengthDelimitedCodec::new(),
        );

        // read the rpc request data
        let data = match substream.next().await {
            Some(data) => data.unwrap().freeze(),
            None => panic!("listener: expected rpc request from dialer"),
        };
        assert_eq!(data.as_ref(), req_data);

        // Listener then suddenly drops the connection
        substream.close().await.unwrap();
    };

    let f = join3(f_listener_peer, f_listener, f_dialer_upgrade);
    Runtime::new().unwrap().block_on(f);
}

// An outbound rpc request should fail if the listener drops the connection after
// negotiation but before the dialer sends their request.
#[test]
fn listener_close_before_dialer_send() {
    ::libra_logger::Logger::new().environment_only(true).init();

    let listener_peer_id = PeerId::random();
    let protocol_id = RPC_PROTOCOL;
    let req_data = b"hello";

    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    // Listener immediately drops connection
    drop(listener_substream);

    // Fake the listener Peer created at the dialer.
    let (listener_peer_reqs_tx, listener_peer_reqs_rx) = channel::new_test(8);
    let listener_peer_reqs_tx = PeerHandle::new(
        listener_peer_id,
        Multiaddr::from_str("/ip6/::1/tcp/8081").unwrap(),
        listener_peer_reqs_tx,
    );
    let f_listener_peer = mock_peer(listener_peer_reqs_rx, dialer_substream);

    // Make an outbound substream request
    let f_listener_upgrade = async move {
        let res = do_outbound_rpc_req(
            listener_peer_reqs_tx,
            protocol_id,
            Bytes::from_static(req_data),
            Duration::from_secs(1),
        )
        .await;

        // Check the error
        let err = res.expect_err("Dialer's rpc request should fail");
        match err {
            RpcError::IoError(err) => assert_eq!(err.kind(), io::ErrorKind::BrokenPipe),
            err => panic!("Unexpected error: {:?}, expected IoError", err),
        };
    };

    let f = join(f_listener_peer, f_listener_upgrade);
    Runtime::new().unwrap().block_on(f);
}

// An inbound rpc request should fail if the dialer drops the connection after
// negotiation but before sending their request.
#[test]
fn dialer_close_before_listener_recv() {
    ::libra_logger::Logger::new().environment_only(true).init();

    let protocol_id = RPC_PROTOCOL;

    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    // Dialer immediately drops connection after negotiation
    drop(dialer_substream);

    // Listener handles the inbound substream, but should get an EOF error
    let f_listener_upgrade = async move {
        let (notification_tx, _notification_rx) = channel::new_test(8);
        // use inner to get Result
        let res =
            handle_inbound_substream_inner(notification_tx, protocol_id, listener_substream).await;

        // Check the error
        let err = res.expect_err("Listener's rpc handler should fail");
        match err {
            RpcError::IoError(err) => assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof),
            err => panic!("Unexpected error: {:?}, expected IoError", err),
        };
    };

    Runtime::new().unwrap().block_on(f_listener_upgrade);
}

// An inbound rpc request should fail if the dialer drops the connection before
// reading out the response.
#[test]
fn dialer_close_before_listener_send() {
    ::libra_logger::Logger::new().environment_only(true).init();

    let protocol_id = RPC_PROTOCOL;
    let req_data = b"hello";
    let res_data = b"goodbye";

    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    // Fake the listener NetworkProvider
    let (dialer_rpc_notifs_tx, mut dialer_rpc_notifs_rx) = channel::new_test(8);
    let f_listener_network = async move {
        // Handle the inbound rpc request
        match dialer_rpc_notifs_rx.next().await.unwrap() {
            RpcNotification::RecvRpc(req) => {
                assert_eq!(req.protocol, protocol_id);
                assert_eq!(req.data.as_ref(), req_data);
                req.res_tx.send(Ok(Bytes::from_static(res_data))).unwrap();
            }
        }
    };

    // Listener handles the inbound substream, but should get a broken pipe error
    let f_listener_upgrade = async move {
        // use inner to get Result
        let res =
            handle_inbound_substream_inner(dialer_rpc_notifs_tx, protocol_id, listener_substream)
                .await;

        // Check the error
        let err = res.expect_err("Listener's rpc handler should fail");
        match err {
            RpcError::IoError(err) => assert_eq!(err.kind(), io::ErrorKind::BrokenPipe),
            err => panic!("Unexpected error: {:?}, expected IoError", err),
        };
    };

    let f_dialer_upgrade = async move {
        // Rpc messages are length-prefixed.
        let mut substream =
            Framed::new(IoCompat::new(dialer_substream), LengthDelimitedCodec::new());
        // Send the rpc request data.
        substream
            .buffered_send(bytes::Bytes::from_static(req_data))
            .await
            .unwrap();
        // Dialer then suddenly drops the connection
        substream.close().await.unwrap();
    };

    let f = join3(f_listener_network, f_listener_upgrade, f_dialer_upgrade);
    Runtime::new().unwrap().block_on(f);
}

// Sending two requests should fail
#[test]
fn dialer_sends_two_requests_err() {
    ::libra_logger::Logger::new().environment_only(true).init();

    let protocol_id = RPC_PROTOCOL;
    let req_data = b"hello";

    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    // Listener handles the inbound substream, but should get an EOF error
    let f_listener_upgrade = async move {
        let (notification_tx, _notification_rx) = channel::new_test(8);
        // use inner to get Result
        let res =
            handle_inbound_substream_inner(notification_tx, protocol_id, listener_substream).await;

        // Check the error
        let err = res.expect_err("Listener's rpc handler should fail");
        match err {
            RpcError::UnexpectedRpcRequest => {}
            err => panic!("Unexpected error: {:?}, expected UnexpectedRpcRequest", err),
        };
    };

    let f_dialer_upgrade = async move {
        // Rpc messages are length-prefixed.
        let mut substream =
            Framed::new(IoCompat::new(dialer_substream), LengthDelimitedCodec::new());
        // Send the rpc request data.
        substream
            .buffered_send(bytes::Bytes::from_static(req_data))
            .await
            .unwrap();
        // ERROR: Send _another_ rpc request data in the same substream.
        substream
            .buffered_send(bytes::Bytes::from_static(req_data))
            .await
            .unwrap();
        // Dialer half-closes
        substream.close().await.unwrap();
        // Listener should RST substream
        if let Some(res) = substream.next().await {
            panic!("Unexpected response; expected None: {:?}", res);
        }
    };

    let f = join(f_listener_upgrade, f_dialer_upgrade);

    Runtime::new().unwrap().block_on(f);
}

// Test that outbound rpc calls will timeout.
#[test]
fn outbound_rpc_timeout() {
    ::libra_logger::Logger::new().environment_only(true).init();

    let listener_peer_id = PeerId::random();
    let protocol_id = RPC_PROTOCOL;
    let req_data = b"hello";

    // Listener hangs after negotiation
    let (dialer_substream, _listener_substream) = MemorySocket::new_pair();

    // Fake the dialer NetworkProvider
    let (listener_peer_reqs_tx, listener_peer_reqs_rx) = channel::new_test(8);
    let listener_peer_reqs_tx = PeerHandle::new(
        listener_peer_id,
        Multiaddr::from_str("/ip6/::1/tcp/8081").unwrap(),
        listener_peer_reqs_tx,
    );
    let f_dialer_peer = mock_peer(listener_peer_reqs_rx, dialer_substream);

    // Make an outbound substream request; listener hangs so this should timeout.
    let f_dialer_upgrade = async move {
        let res = do_outbound_rpc_req(
            listener_peer_reqs_tx,
            protocol_id,
            Bytes::from_static(req_data),
            Duration::from_millis(100),
        )
        .await;

        // Check error is timeout error
        let err = res.expect_err("Dialer's rpc request should fail");
        match err {
            RpcError::TimedOut => {}
            err => panic!("Unexpected error: {:?}, expected TimedOut", err),
        };
    };

    let f = join(f_dialer_peer, f_dialer_upgrade);
    Runtime::new().unwrap().block_on(f);
}

// Test that inbound rpc calls will timeout.
#[test]
fn inbound_rpc_timeout() {
    ::libra_logger::Logger::new().environment_only(true).init();

    let dialer_peer_id = PeerId::random();
    let protocol_id = RPC_PROTOCOL;

    // Dialer hangs after negotiation
    let (_dialer_substream, listener_substream) = MemorySocket::new_pair();
    let (listener_rpc_notifs_tx, _listener_rpc_notifs_rx) = channel::new_test(8);

    // Handle the inbound substream
    let substream = NegotiatedSubstream {
        protocol: protocol_id,
        substream: listener_substream,
    };
    let inbound_notif = PeerNotification::NewSubstream(dialer_peer_id, substream);
    let f_listener_upgrade = handle_inbound_substream(
        listener_rpc_notifs_tx,
        inbound_notif,
        Duration::from_millis(100),
    );

    // The listener future should complete (with a timeout) despite the dialer
    // hanging.
    Runtime::new().unwrap().block_on(f_listener_upgrade);
}

// Test that outbound rpcs can be canceled before sending
#[test]
fn outbound_cancellation_before_send() {
    ::libra_logger::Logger::new().environment_only(true).init();

    let listener_peer_id = PeerId::random();
    let protocol_id = RPC_PROTOCOL;
    let req_data = b"hello";

    // Fake the dialer NetworkProvider channels
    let (listener_peer_reqs_tx, _listener_peer_reqs_rx) = channel::new_test(8);
    let listener_peer_reqs_tx = PeerHandle::<MemorySocket>::new(
        listener_peer_id,
        Multiaddr::from_str("/ip6/::1/tcp/8081").unwrap(),
        listener_peer_reqs_tx,
    );

    // build the rpc request future
    let (res_tx, res_rx) = oneshot::channel();
    let outbound_req = OutboundRpcRequest {
        protocol: protocol_id,
        data: Bytes::from_static(req_data),
        res_tx,
        timeout: Duration::from_secs(1),
    };
    let rpc_req = RpcRequest::SendRpc(outbound_req);
    let f_rpc = handle_outbound_rpc(listener_peer_reqs_tx, rpc_req);

    // drop res_rx to cancel the rpc request
    drop(res_rx);

    // the rpc request should finish (from the cancellation) even though there is
    // no remote peer
    Runtime::new().unwrap().block_on(f_rpc);
}

// Test that outbound rpcs can be canceled while receiving response data.
#[test]
fn outbound_cancellation_recv() {
    ::libra_logger::Logger::new().environment_only(true).init();

    let mut rt = Runtime::new().unwrap();
    let executor = rt.handle().clone();

    let listener_peer_id = PeerId::random();
    let protocol_id = RPC_PROTOCOL;
    let req_data = b"hello";
    let res_data = b"goodbye";

    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    // Fake the dialer NetworkProvider
    let (dialer_peer_reqs_tx, dialer_peer_reqs_rx) = channel::new_test(8);
    let dialer_peer_reqs_tx = PeerHandle::new(
        listener_peer_id,
        Multiaddr::from_str("/ip6/::1/tcp/8081").unwrap(),
        dialer_peer_reqs_tx,
    );
    let f_dialer_peer = mock_peer(dialer_peer_reqs_rx, dialer_substream);

    // triggered when listener finishes reading response to notify dialer to cancel
    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
    // triggered when dialer finishes canceling the request to notify listener to
    // try sending.
    let (cancel_done_tx, cancel_done_rx) = oneshot::channel::<()>();

    // Make an outbound rpc request but then cancel it after sending
    let f_dialer_upgrade = async move {
        let (res_tx, res_rx) = oneshot::channel();
        let mut res_rx = res_rx.fuse();

        let outbound_req = OutboundRpcRequest {
            protocol: protocol_id,
            data: Bytes::from_static(req_data),
            res_tx,
            timeout: Duration::from_secs(1),
        };
        let rpc_req = RpcRequest::SendRpc(outbound_req);
        let (f_rpc, f_rpc_done) = handle_outbound_rpc(dialer_peer_reqs_tx, rpc_req).remote_handle();
        executor.spawn(f_rpc);

        futures::select! {
            res = res_rx => panic!("dialer: expected cancellation signal, rpc call finished unexpectedly: {:?}", res),
            _ = cancel_rx.fuse() => {
                // drop res_rx to cancel rpc call
                drop(res_rx);

                // wait for rpc to finish cancellation
                f_rpc_done.await;

                // notify listener that cancel is finished so it can try sending
                cancel_done_tx.send(()).unwrap();
            }
        }
    };

    // Listener reads the request but then fails to send because the dialer canceled
    let f_listener = async move {
        // rpc messages are length-prefixed
        let mut substream = Framed::new(
            IoCompat::new(listener_substream),
            LengthDelimitedCodec::new(),
        );
        // read the rpc request data
        let data = match substream.next().await {
            Some(data) => data.unwrap().freeze(),
            None => panic!("listener: Expected rpc request from dialer"),
        };
        assert_eq!(data.as_ref(), req_data);
        // wait for dialer's half-close
        match substream.next().await {
            None => {}
            res => panic!("listener: Expected half-close: {:?}", res),
        }

        // trigger dialer cancel
        drop(cancel_tx);

        // wait for dialer to finish cancelling
        cancel_done_rx.await.unwrap();

        // should get an error when trying to send
        match substream.send(bytes::Bytes::from_static(res_data)).await {
            Err(err) => assert_eq!(io::ErrorKind::BrokenPipe, err.kind()),
            res => panic!("listener: Unexpected result: {:?}", res),
        }
    };

    let f = join3(f_dialer_peer, f_dialer_upgrade, f_listener);
    rt.block_on(f);
}

// Test the full rpc protocol actor.
#[test]
fn rpc_protocol() {
    ::libra_logger::Logger::new().environment_only(true).init();

    let listener_peer_id = PeerId::random();
    let dialer_peer_id = PeerId::random();
    let protocol_id = RPC_PROTOCOL;
    let req_data = b"hello";
    let res_data = b"goodbye";

    let mut rt = Runtime::new().unwrap();

    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    // Set up the dialer Rpc protocol actor
    let (mut dialer_rpc_tx, dialer_rpc_rx) = channel::new_test(8);
    let (_, dialer_peer_notifs_rx) = channel::new_test(8);
    let (dialer_peer_reqs_tx, mut dialer_peer_reqs_rx) = channel::new_test(8);
    let dialer_peer_reqs_tx = PeerHandle::new(
        listener_peer_id,
        Multiaddr::from_str("/ip6/::1/tcp/8080").unwrap(),
        dialer_peer_reqs_tx,
    );
    let (rpc_handler_tx, _) = channel::new_test(8);
    let dialer_rpc = Rpc::new(
        rt.handle().clone(),
        dialer_peer_reqs_tx,
        dialer_rpc_rx,
        dialer_peer_notifs_rx,
        rpc_handler_tx,
        Duration::from_millis(500),
        10,
        10,
    );

    // Fake the dialer NetworkProvider
    let f_dialer_network = async move {
        let (res_tx, res_rx) = oneshot::channel();

        let req = OutboundRpcRequest {
            protocol: protocol_id,
            data: Bytes::from_static(req_data),
            res_tx,
            timeout: Duration::from_secs(1),
        };

        // Tell Rpc to send an rpc request
        dialer_rpc_tx.send(RpcRequest::SendRpc(req)).await.unwrap();

        // Fulfill the open substream request
        match dialer_peer_reqs_rx.next().await.unwrap() {
            PeerRequest::OpenSubstream(protocol, substream_tx) => {
                assert_eq!(protocol, protocol_id);
                substream_tx.send(Ok(dialer_substream)).unwrap();
            }
            _ => {
                unreachable!();
            }
        }

        // Check the rpc response data
        let data = res_rx.await.unwrap().unwrap();
        assert_eq!(data.as_ref(), res_data);
    };

    // Set up the listener Rpc protocol actor
    let (_, listener_rpc_reqs_rx) = channel::new_test(8);
    let (mut listener_peer_notifs_tx, listener_peer_notifs_rx) = channel::new_test(8);
    let (listener_peer_reqs_tx, _) = channel::new_test(8);
    let listener_peer_reqs_tx = PeerHandle::new(
        dialer_peer_id,
        Multiaddr::from_str("/ip6/::1/tcp/8081").unwrap(),
        listener_peer_reqs_tx,
    );
    let (listener_rpc_notifs_tx, mut listener_rpc_notifs_rx) = channel::new_test(8);
    let listener_rpc = Rpc::new(
        rt.handle().clone(),
        listener_peer_reqs_tx,
        listener_rpc_reqs_rx,
        listener_peer_notifs_rx,
        listener_rpc_notifs_tx,
        Duration::from_millis(500),
        10,
        10,
    );

    // Fake the listener NetworkProvider
    let f_listener_network = async move {
        // Notify Rpc of a new inbound substream

        listener_peer_notifs_tx
            .send(PeerNotification::NewSubstream(
                dialer_peer_id,
                NegotiatedSubstream {
                    protocol: protocol_id,
                    substream: listener_substream,
                },
            ))
            .await
            .unwrap();

        // Handle the inbound rpc request
        match listener_rpc_notifs_rx.next().await.unwrap() {
            RpcNotification::RecvRpc(req) => {
                assert_eq!(req.protocol, protocol_id);
                assert_eq!(req.data.as_ref(), req_data);
                req.res_tx.send(Ok(Bytes::from_static(res_data))).unwrap();
            }
        }
    };

    let f = join4(
        f_listener_network,
        listener_rpc.start(),
        f_dialer_network,
        dialer_rpc.start(),
    );
    rt.block_on(f);
}
