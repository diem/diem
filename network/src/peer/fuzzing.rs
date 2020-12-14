// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants::MAX_FRAME_SIZE,
    interface::NetworkProvider,
    protocols::wire::{
        handshake::v1::{MessagingProtocolVersion, SupportedProtocols},
        messaging::v1::{NetworkMessage, NetworkMessageSink},
    },
    testutils::fake_socket::ReadOnlyTestSocketVec,
    transport::{Connection, ConnectionId, ConnectionMetadata},
    ProtocolId,
};
use diem_config::network_id::NetworkContext;
use diem_network_address::NetworkAddress;
use diem_proptest_helpers::ValueGenerator;
use diem_types::PeerId;
use futures::{executor::block_on, future, io::AsyncReadExt, sink::SinkExt, stream::StreamExt};
use memsocket::MemorySocket;
use netcore::transport::ConnectionOrigin;
use proptest::{arbitrary::any, collection::vec};

/// Generate a sequence of `NetworkMessage`, bcs serialize them, and write them
/// out to a buffer using our length-prefixed message codec.
pub fn generate_corpus(gen: &mut ValueGenerator) -> Vec<u8> {
    let network_msgs = gen.generate(vec(any::<NetworkMessage>(), 1..20));

    let (write_socket, mut read_socket) = MemorySocket::new_pair();
    let mut writer = NetworkMessageSink::new(write_socket, MAX_FRAME_SIZE);

    // Write the `NetworkMessage`s to a fake socket
    let f_send = async move {
        for network_msg in &network_msgs {
            writer.send(network_msg).await.unwrap();
        }
    };
    // Read the serialized `NetworkMessage`s from the fake socket
    let f_recv = async move {
        let mut buf = Vec::new();
        read_socket.read_to_end(&mut buf).await.unwrap();
        buf
    };

    let (_, buf) = block_on(future::join(f_send, f_recv));
    buf
}

/// Fuzz the `Peer` actor's inbound message handling.
///
/// For each fuzzer iteration, we spin up a new `Peer` actor and pipe the raw
/// fuzzer data into it. This mostly tests that the `Peer` inbound message handling
/// doesn't panic or leak memory when reading, deserializing, and handling messages
/// from remote peers.
pub fn fuzz(data: &[u8]) {
    // Use the basic single-threaded runtime, since our current tokio version has
    // a chance to leak memory and/or thread handles when using the threaded
    // runtime and sometimes blocks when trying to shutdown the runtime.
    //
    // https://github.com/tokio-rs/tokio/pull/2649
    let mut rt = tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();
    let executor = rt.handle().clone();

    // We want to choose a constant peer id for _our_ peer id, since we will
    // generate unbounded metrics otherwise and OOM during fuzzing.
    let peer_id = PeerId::ZERO;
    // However, we want to choose a random _remote_ peer id to ensure we _don't_
    // have metrics logging the remote peer id (which would eventually OOM in
    // production for public-facing nodes).
    let remote_peer_id = PeerId::random();

    // Mock data
    let network_context = NetworkContext::mock_with_peer_id(peer_id);
    let socket = ReadOnlyTestSocketVec::new(data.to_vec());
    let metadata = ConnectionMetadata::new(
        remote_peer_id,
        ConnectionId::from(123),
        NetworkAddress::mock(),
        ConnectionOrigin::Inbound,
        MessagingProtocolVersion::V1,
        SupportedProtocols::from(
            [
                ProtocolId::ConsensusRpc,
                ProtocolId::ConsensusDirectSend,
                ProtocolId::MempoolDirectSend,
                ProtocolId::StateSynchronizerDirectSend,
                ProtocolId::DiscoveryDirectSend,
                ProtocolId::HealthCheckerRpc,
            ]
            .iter(),
        ),
    );
    let connection = Connection { socket, metadata };

    let (connection_notifs_tx, connection_notifs_rx) = channel::new_test(8);
    let max_concurrent_reqs = 8;
    let max_concurrent_notifs = 8;
    let channel_size = 8;

    // Spin up a new `Peer` actor through the `NetworkProvider` interface.
    let (network_reqs_tx, network_notifs_rx) = NetworkProvider::start(
        network_context,
        executor,
        connection,
        connection_notifs_tx,
        max_concurrent_reqs,
        max_concurrent_notifs,
        channel_size,
        MAX_FRAME_SIZE,
    );

    rt.block_on(async move {
        // Wait for "remote" to disconnect (we read all data and socket read
        // returns EOF), we read a disconnect request, or we fail to deserialize
        // something.
        connection_notifs_rx.collect::<Vec<_>>().await;

        // ACK the "remote" d/c and drop our handle to the Peer actor. Then wait
        // for all network notifs to drain out and finish.
        drop(network_reqs_tx);
        network_notifs_rx.collect::<Vec<_>>().await;
    });
}

#[test]
fn test_peer_fuzzers() {
    let mut value_gen = ValueGenerator::deterministic();
    for _ in 0..50 {
        let corpus = generate_corpus(&mut value_gen);
        fuzz(&corpus);
    }
}
