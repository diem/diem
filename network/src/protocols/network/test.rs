// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for validator_network.
use super::*;
use crate::protocols::network::dummy::{setup_network, DummyMsg};
use futures::{future::join, StreamExt};
use std::time::Duration;

#[test]
fn test_network_builder() {
    setup_network();
}

#[test]
fn test_direct_send() {
    ::libra_logger::try_init_for_testing();
    let mut tn = setup_network();
    let dialer_peer_id = tn.dialer_peer_id;
    let mut dialer_events = tn.dialer_events;
    let mut dialer_sender = tn.dialer_sender;
    let listener_peer_id = tn.listener_peer_id;
    let mut listener_events = tn.listener_events;
    let mut listener_sender = tn.listener_sender;

    let msg = DummyMsg(vec![]);

    // The dialer sends a direct send and listener receives
    let msg_clone = msg.clone();
    let f_dialer = async move {
        dialer_sender
            .send_to(listener_peer_id, msg_clone.clone())
            .unwrap();
        match listener_events.next().await.unwrap().unwrap() {
            Event::Message((peer_id, msg)) => {
                assert_eq!(peer_id, dialer_peer_id);
                assert_eq!(msg, msg_clone);
            }
            event => panic!("Unexpected event {:?}", event),
        }
    };

    // The listener sends a direct send and the dialer receives
    let f_listener = async move {
        listener_sender
            .send_to(dialer_peer_id, msg.clone())
            .unwrap();
        match dialer_events.next().await.unwrap().unwrap() {
            Event::Message((peer_id, incoming_msg)) => {
                assert_eq!(peer_id, listener_peer_id);
                assert_eq!(incoming_msg, msg);
            }
            event => panic!("Unexpected event {:?}", event),
        }
    };

    tn.runtime.block_on(join(f_dialer, f_listener));
}

#[test]
fn test_rpc() {
    ::libra_logger::try_init_for_testing();
    let mut tn = setup_network();
    let dialer_peer_id = tn.dialer_peer_id;
    let mut dialer_events = tn.dialer_events;
    let mut dialer_sender = tn.dialer_sender;
    let listener_peer_id = tn.listener_peer_id;
    let mut listener_events = tn.listener_events;
    let mut listener_sender = tn.listener_sender;

    let msg = DummyMsg(vec![]);

    // Dialer send rpc request and receives rpc response
    let msg_clone = msg.clone();
    let f_send =
        dialer_sender.send_rpc(listener_peer_id, msg_clone.clone(), Duration::from_secs(10));
    let f_respond = async move {
        match listener_events.next().await.unwrap().unwrap() {
            Event::RpcRequest((peer_id, msg, rs)) => {
                assert_eq!(peer_id, dialer_peer_id);
                assert_eq!(msg, msg_clone);
                rs.send(Ok(lcs::to_bytes(&msg).unwrap().into())).unwrap();
            }
            event => panic!("Unexpected event: {:?}", event),
        }
    };

    let (res_msg, _) = tn.runtime.block_on(join(f_send, f_respond));
    assert_eq!(res_msg.unwrap(), msg);

    // Listener send rpc request and receives rpc response
    let msg_clone = msg.clone();
    let f_send =
        listener_sender.send_rpc(dialer_peer_id, msg_clone.clone(), Duration::from_secs(10));
    let f_respond = async move {
        match dialer_events.next().await.unwrap().unwrap() {
            Event::RpcRequest((peer_id, msg, rs)) => {
                assert_eq!(peer_id, listener_peer_id);
                assert_eq!(msg, msg_clone);
                rs.send(Ok(lcs::to_bytes(&msg).unwrap().into())).unwrap();
            }
            event => panic!("Unexpected event: {:?}", event),
        }
    };

    let (res_msg, _) = tn.runtime.block_on(join(f_send, f_respond));
    assert_eq!(res_msg.unwrap(), msg);
}
