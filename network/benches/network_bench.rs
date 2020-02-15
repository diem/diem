// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Allow KiB, MiB consts
#![allow(non_upper_case_globals, non_snake_case)]
// Allow fns to take &usize, since criterion only passes parameters by ref
#![allow(clippy::trivially_copy_pass_by_ref)]
// Allow writing 1 * KiB or 1 * MiB
#![allow(clippy::identity_op)]

use core::str::FromStr;
use criterion::{
    criterion_group, criterion_main, Bencher, Criterion, ParameterizedBenchmark, Throughput,
};
use futures::{
    channel::mpsc,
    executor::block_on,
    sink::SinkExt,
    stream::{FuturesUnordered, StreamExt},
};
use libra_config::config::RoleType;
use libra_crypto::{ed25519::compat, test_utils::TEST_SEED, x25519};
use libra_prost_ext::MessageExt;
use network::{
    proto::ConsensusMsg,
    protocols::rpc::error::RpcError,
    validator_network::{
        self,
        network_builder::{NetworkBuilder, TransportType},
        ConsensusNetworkSender, Event,
    },
    NetworkPublicKeys,
};
use parity_multiaddr::Multiaddr;

use libra_types::PeerId;
use rand::{rngs::StdRng, SeedableRng};
use std::{collections::HashMap, time::Duration};
use tokio::runtime::Runtime;

const KiB: usize = 1 << 10;
const MiB: usize = 1 << 20;
const NUM_MSGS: u32 = 100;
const TOLERANCE: u32 = 20;
const HOUR_IN_MS: u64 = 60 * 60 * 1000;

fn direct_send_bench(b: &mut Bencher, msg_len: &usize) {
    let runtime = Runtime::new().unwrap();
    let (dialer_peer_id, dialer_addr) = (
        PeerId::random(),
        Multiaddr::from_str("/ip4/127.0.0.1/tcp/0").unwrap(),
    );
    let (listener_peer_id, listener_addr) = (
        PeerId::random(),
        Multiaddr::from_str("/ip4/127.0.0.1/tcp/0").unwrap(),
    );

    // Setup keys for dialer.
    let mut rng = StdRng::from_seed(TEST_SEED);
    let (dialer_signing_private_key, dialer_signing_public_key) =
        compat::generate_keypair(&mut rng);
    let (dialer_identity_private_key, dialer_identity_public_key) =
        x25519::compat::generate_keypair(&mut rng);

    // Setup keys for listener.
    let (listener_signing_private_key, listener_signing_public_key) =
        compat::generate_keypair(&mut rng);
    let (listener_identity_private_key, listener_identity_public_key) =
        x25519::compat::generate_keypair(&mut rng);

    // Setup trusted peers.
    let trusted_peers: HashMap<_, _> = vec![
        (
            dialer_peer_id,
            NetworkPublicKeys {
                signing_public_key: dialer_signing_public_key.clone(),
                identity_public_key: dialer_identity_public_key.clone(),
            },
        ),
        (
            listener_peer_id,
            NetworkPublicKeys {
                signing_public_key: listener_signing_public_key.clone(),
                identity_public_key: listener_identity_public_key.clone(),
            },
        ),
    ]
    .into_iter()
    .collect();

    // Set up the listener network
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        listener_peer_id,
        listener_addr,
        RoleType::Validator,
    );
    network_builder
        .transport(TransportType::TcpNoise(Some((
            listener_identity_private_key,
            listener_identity_public_key,
        ))))
        .trusted_peers(trusted_peers.clone())
        .signing_keys((listener_signing_private_key, listener_signing_public_key))
        .discovery_interval_ms(HOUR_IN_MS)
        .add_discovery();
    let (_listener_sender, mut listener_events) =
        validator_network::consensus::add_to_network(&mut network_builder);
    let listen_addr = network_builder.build();

    // Set up the dialer network
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        dialer_peer_id,
        dialer_addr,
        RoleType::Validator,
    );
    network_builder
        .transport(TransportType::TcpNoise(Some((
            dialer_identity_private_key,
            dialer_identity_public_key,
        ))))
        .trusted_peers(trusted_peers)
        .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
        .seed_peers(
            [(listener_peer_id, vec![listen_addr])]
                .iter()
                .cloned()
                .collect(),
        )
        .discovery_interval_ms(HOUR_IN_MS)
        .add_discovery();
    let (mut dialer_sender, mut dialer_events) =
        validator_network::consensus::add_to_network(&mut network_builder);
    let _dialer_addr = network_builder.build();

    // Wait for establishing connection
    let first_dialer_event = block_on(dialer_events.next()).unwrap().unwrap();
    assert_eq!(first_dialer_event, Event::NewPeer(listener_peer_id));
    let first_listener_event = block_on(listener_events.next()).unwrap().unwrap();
    assert_eq!(first_listener_event, Event::NewPeer(dialer_peer_id));

    // Compose Proposal message with `msg_len` bytes payload
    let msg = compose_proposal(*msg_len);

    let (mut tx, mut rx) = mpsc::channel(0);
    // The listener side keeps receiving messages and send signal back to the bencher to finish
    // the iteration once NUM_MSGS messages are received.
    let f_listener = async move {
        let mut counter = 0u32;
        while let Some(_) = listener_events.next().await {
            counter += 1;
            // By the nature of DirectSend protocol, some messages may be lost when a connection is
            // broken temporarily.
            if counter == NUM_MSGS - TOLERANCE {
                let _ = tx.send(()).await;
                counter = 0;
            }
        }
    };
    runtime.spawn(f_listener);

    // The dialer side keeps sending messages. In each iteration of the benchmark, it sends
    // NUM_MSGS messages and wait until the listener side sends signal back.
    b.iter(|| {
        for _ in 0..NUM_MSGS {
            dialer_sender
                .send_to(listener_peer_id, msg.clone())
                .unwrap();
        }
        block_on(rx.next()).unwrap();
    });
}

fn compose_proposal(msg_len: usize) -> ConsensusMsg {
    let mut msg = ConsensusMsg::default();
    msg.message = vec![0u8; msg_len];
    msg
}

fn rpc_bench(b: &mut Bencher, msg_len: &usize) {
    let runtime = Runtime::new().unwrap();
    let (dialer_peer_id, dialer_addr) = (
        PeerId::random(),
        Multiaddr::from_str("/ip4/127.0.0.1/tcp/0").unwrap(),
    );
    let (listener_peer_id, listener_addr) = (
        PeerId::random(),
        Multiaddr::from_str("/ip4/127.0.0.1/tcp/0").unwrap(),
    );

    // Setup keys for dialer.
    let mut rng = StdRng::from_seed(TEST_SEED);
    let (dialer_signing_private_key, dialer_signing_public_key) =
        compat::generate_keypair(&mut rng);
    let (dialer_identity_private_key, dialer_identity_public_key) =
        x25519::compat::generate_keypair(&mut rng);

    // Setup keys for listener.
    let (listener_signing_private_key, listener_signing_public_key) =
        compat::generate_keypair(&mut rng);
    let (listener_identity_private_key, listener_identity_public_key) =
        x25519::compat::generate_keypair(&mut rng);

    // Setup trusted peers.
    let trusted_peers: HashMap<_, _> = vec![
        (
            dialer_peer_id,
            NetworkPublicKeys {
                signing_public_key: dialer_signing_public_key.clone(),
                identity_public_key: dialer_identity_public_key.clone(),
            },
        ),
        (
            listener_peer_id,
            NetworkPublicKeys {
                signing_public_key: listener_signing_public_key.clone(),
                identity_public_key: listener_identity_public_key.clone(),
            },
        ),
    ]
    .into_iter()
    .collect();

    // Set up the listener network
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        listener_peer_id,
        listener_addr,
        RoleType::Validator,
    );
    network_builder
        .transport(TransportType::TcpNoise(Some((
            listener_identity_private_key,
            listener_identity_public_key,
        ))))
        .trusted_peers(trusted_peers.clone())
        .signing_keys((listener_signing_private_key, listener_signing_public_key))
        .discovery_interval_ms(HOUR_IN_MS)
        .add_discovery();
    let (_listener_sender, mut listener_events) =
        validator_network::consensus::add_to_network(&mut network_builder);
    let listen_addr = network_builder.build();

    // Set up the dialer network
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        dialer_peer_id,
        dialer_addr,
        RoleType::Validator,
    );
    network_builder
        .transport(TransportType::TcpNoise(Some((
            dialer_identity_private_key,
            dialer_identity_public_key,
        ))))
        .trusted_peers(trusted_peers)
        .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
        .seed_peers(
            [(listener_peer_id, vec![listen_addr])]
                .iter()
                .cloned()
                .collect(),
        )
        .discovery_interval_ms(HOUR_IN_MS)
        .add_discovery();
    let (dialer_sender, mut dialer_events) =
        validator_network::consensus::add_to_network(&mut network_builder);
    let _dialer_addr = network_builder.build();

    // Wait for establishing connection
    let first_dialer_event = block_on(dialer_events.next()).unwrap().unwrap();
    assert_eq!(first_dialer_event, Event::NewPeer(listener_peer_id));
    let first_listener_event = block_on(listener_events.next()).unwrap().unwrap();
    assert_eq!(first_listener_event, Event::NewPeer(dialer_peer_id));

    // Compose RequestBlock message and RespondBlock message with `msg_len` bytes payload
    let req = compose_send_rpc();
    let res = compose_respond_block(*msg_len);

    // The listener side keeps receiving RPC requests and sending responses back
    let f_listener = async move {
        while let Some(Ok(event)) = listener_events.next().await {
            match event {
                Event::RpcRequest((_, _, res_tx)) => res_tx
                    .send(Ok(res.clone().to_bytes().expect("fail to serialize proto")))
                    .expect("fail to send rpc response to network"),
                event => panic!("Unexpected event: {:?}", event),
            }
        }
    };
    runtime.spawn(f_listener);

    // The dialer side keeps sending RPC requests. In each iteration of the benchmark, it sends
    // NUM_MSGS requests and blocks on getting the responses.
    b.iter(|| {
        let mut requests = FuturesUnordered::new();
        for _ in 0..NUM_MSGS {
            requests.push(send_rpc(
                dialer_sender.clone(),
                listener_peer_id,
                req.clone(),
            ));
        }
        while let Some(res) = block_on(requests.next()) {
            let _ = res.unwrap();
        }
    });
}

async fn send_rpc(
    mut sender: ConsensusNetworkSender,
    recipient: PeerId,
    req_msg: ConsensusMsg,
) -> Result<ConsensusMsg, RpcError> {
    sender
        .send_rpc(recipient, req_msg, Duration::from_secs(15))
        .await
}

fn compose_send_rpc() -> ConsensusMsg {
    ConsensusMsg::default()
}

fn compose_respond_block(msg_len: usize) -> ConsensusMsg {
    let mut msg = ConsensusMsg::default();
    msg.message = vec![0u8; msg_len];
    msg
}

fn network_crate_benchmark(c: &mut Criterion) {
    ::libra_logger::try_init_for_testing();

    // Parameterize benchmarks over the message length.
    let msg_lens = vec![32usize, 256, 1 * KiB, 4 * KiB, 64 * KiB, 256 * KiB, 1 * MiB];

    c.bench(
        "network_crate_benchmark",
        ParameterizedBenchmark::new("direct_send", direct_send_bench, msg_lens)
            .with_function("rpc", rpc_bench)
            .sample_size(10)
            .throughput(|msg_len| Throughput::Bytes(((*msg_len as u32) * NUM_MSGS).into())),
    );
}

criterion_group!(benches, network_crate_benchmark);
criterion_main!(benches);
