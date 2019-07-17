// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![feature(async_await)]
// Allow KiB, MiB consts
#![allow(non_upper_case_globals, non_snake_case)]
// Allow fns to take &usize, since criterion only passes parameters by ref
#![allow(clippy::trivially_copy_pass_by_ref)]
// Allow writing 1 * KiB or 1 * MiB
#![allow(clippy::identity_op)]

use bytes::Bytes;
use core::str::FromStr;
use criterion::{
    criterion_group, criterion_main, AxisScale, Bencher, Criterion, ParameterizedBenchmark,
    PlotConfiguration, Throughput,
};
use crypto::x25519;
use futures::{
    channel::mpsc,
    compat::Future01CompatExt,
    executor::block_on,
    future::{FutureExt, TryFutureExt},
    sink::SinkExt,
    stream::{FuturesUnordered, StreamExt},
};
use network::{
    proto::{Block, ConsensusMsg, RequestBlock, RespondBlock},
    protocols::rpc::error::RpcError,
    validator_network::{
        network_builder::{NetworkBuilder, TransportType},
        ConsensusNetworkSender, Event, CONSENSUS_DIRECT_SEND_PROTOCOL, CONSENSUS_RPC_PROTOCOL,
    },
    NetworkPublicKeys, ProtocolId,
};
use nextgen_crypto::{ed25519::compat, test_utils::TEST_SEED};
use parity_multiaddr::Multiaddr;
use protobuf::Message;
use rand::{rngs::StdRng, SeedableRng};
use std::{collections::HashMap, time::Duration};
use tokio::runtime::Runtime;
use types::PeerId;

const KiB: usize = 1 << 10;
const MiB: usize = 1 << 20;
const NUM_MSGS: u32 = 100;
const TOLERANCE: u32 = 5;
const HOUR_IN_MS: u64 = 60 * 60 * 1000;

fn direct_send_bench(b: &mut Bencher, msg_len: &usize) {
    let mut runtime = Runtime::new().unwrap();
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
    let (dialer_identity_private_key, dialer_identity_public_key) = x25519::generate_keypair();

    // Setup keys for listener.
    let (listener_signing_private_key, listener_signing_public_key) =
        compat::generate_keypair(&mut rng);
    let (listener_identity_private_key, listener_identity_public_key) = x25519::generate_keypair();

    // Setup trusted peers.
    let trusted_peers: HashMap<_, _> = vec![
        (
            dialer_peer_id,
            NetworkPublicKeys {
                signing_public_key: dialer_signing_public_key.clone().into(),
                identity_public_key: dialer_identity_public_key,
            },
        ),
        (
            listener_peer_id,
            NetworkPublicKeys {
                signing_public_key: listener_signing_public_key.clone().into(),
                identity_public_key: listener_identity_public_key,
            },
        ),
    ]
    .into_iter()
    .collect();

    // Set up the listener network
    let ((_, _), (_listener_sender, mut listener_events), listen_addr) =
        NetworkBuilder::new(runtime.executor(), listener_peer_id, listener_addr)
            .transport(TransportType::TcpNoise)
            .trusted_peers(trusted_peers.clone())
            .identity_keys((listener_identity_private_key, listener_identity_public_key))
            .signing_keys((listener_signing_private_key, listener_signing_public_key))
            .discovery_interval_ms(HOUR_IN_MS)
            .consensus_protocols(vec![ProtocolId::from_static(
                CONSENSUS_DIRECT_SEND_PROTOCOL,
            )])
            .direct_send_protocols(vec![ProtocolId::from_static(
                CONSENSUS_DIRECT_SEND_PROTOCOL,
            )])
            .build();

    // Set up the dialer network
    let ((_, _), (mut dialer_sender, mut dialer_events), _) =
        NetworkBuilder::new(runtime.executor(), dialer_peer_id, dialer_addr)
            .transport(TransportType::TcpNoise)
            .trusted_peers(trusted_peers.clone())
            .identity_keys((dialer_identity_private_key, dialer_identity_public_key))
            .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
            .seed_peers(
                [(listener_peer_id, vec![listen_addr])]
                    .iter()
                    .cloned()
                    .collect(),
            )
            .discovery_interval_ms(HOUR_IN_MS)
            .consensus_protocols(vec![ProtocolId::from_static(
                CONSENSUS_DIRECT_SEND_PROTOCOL,
            )])
            .direct_send_protocols(vec![ProtocolId::from_static(
                CONSENSUS_DIRECT_SEND_PROTOCOL,
            )])
            .build();

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
                tx.send(()).await.unwrap();
                counter = 0;
            }
        }
    };
    runtime.spawn(f_listener.boxed().unit_error().compat());

    // The dialer side keeps sending messages. In each iteration of the benchmark, it sends
    // NUM_MSGS messages and wait until the listener side sends signal back.
    b.iter(|| {
        for _ in 0..NUM_MSGS {
            block_on(dialer_sender.send_to(listener_peer_id, msg.clone())).unwrap();
        }
        block_on(rx.next()).unwrap();
    });
    block_on(runtime.shutdown_now().compat()).unwrap();
}

fn compose_proposal(msg_len: usize) -> ConsensusMsg {
    let mut msg = ConsensusMsg::new();
    let proposal = msg.mut_proposal();
    proposal.set_proposer(PeerId::random().into());
    let block = proposal.mut_proposed_block();
    block.set_payload(vec![0u8; msg_len].into());
    msg
}

fn rpc_bench(b: &mut Bencher, msg_len: &usize) {
    let mut runtime = Runtime::new().unwrap();
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
    let (dialer_identity_private_key, dialer_identity_public_key) = x25519::generate_keypair();

    // Setup keys for listener.
    let (listener_signing_private_key, listener_signing_public_key) =
        compat::generate_keypair(&mut rng);
    let (listener_identity_private_key, listener_identity_public_key) = x25519::generate_keypair();

    // Setup trusted peers.
    let trusted_peers: HashMap<_, _> = vec![
        (
            dialer_peer_id,
            NetworkPublicKeys {
                signing_public_key: dialer_signing_public_key.clone().into(),
                identity_public_key: dialer_identity_public_key,
            },
        ),
        (
            listener_peer_id,
            NetworkPublicKeys {
                signing_public_key: listener_signing_public_key.clone().into(),
                identity_public_key: listener_identity_public_key,
            },
        ),
    ]
    .into_iter()
    .collect();

    // Set up the listener network
    let ((_, _), (_listener_sender, mut listener_events), listen_addr) =
        NetworkBuilder::new(runtime.executor(), listener_peer_id, listener_addr)
            .transport(TransportType::TcpNoise)
            .trusted_peers(trusted_peers.clone())
            .identity_keys((listener_identity_private_key, listener_identity_public_key))
            .signing_keys((listener_signing_private_key, listener_signing_public_key))
            .discovery_interval_ms(HOUR_IN_MS)
            .consensus_protocols(vec![ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL)])
            .rpc_protocols(vec![ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL)])
            .build();

    // Set up the dialer network
    let ((_, _), (dialer_sender, mut dialer_events), _) =
        NetworkBuilder::new(runtime.executor(), dialer_peer_id, dialer_addr)
            .transport(TransportType::TcpNoise)
            .trusted_peers(trusted_peers.clone())
            .identity_keys((dialer_identity_private_key, dialer_identity_public_key))
            .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
            .seed_peers(
                [(listener_peer_id, vec![listen_addr])]
                    .iter()
                    .cloned()
                    .collect(),
            )
            .discovery_interval_ms(HOUR_IN_MS)
            .consensus_protocols(vec![ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL)])
            .rpc_protocols(vec![ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL)])
            .build();

    // Wait for establishing connection
    let first_dialer_event = block_on(dialer_events.next()).unwrap().unwrap();
    assert_eq!(first_dialer_event, Event::NewPeer(listener_peer_id));
    let first_listener_event = block_on(listener_events.next()).unwrap().unwrap();
    assert_eq!(first_listener_event, Event::NewPeer(dialer_peer_id));

    // Compose RequestBlock message and RespondBlock message with `msg_len` bytes payload
    let req = compose_request_block();
    let res = compose_respond_block(*msg_len);

    // The listener side keeps receiving RPC requests and sending responses back
    let f_listener = async move {
        while let Some(Ok(event)) = listener_events.next().await {
            match event {
                Event::RpcRequest((_, _, res_tx)) => res_tx
                    .send(Ok(Bytes::from(
                        res.clone()
                            .write_to_bytes()
                            .expect("fail to serialize proto"),
                    )))
                    .expect("fail to send rpc response to network"),
                event => panic!("Unexpected event: {:?}", event),
            }
        }
    };
    runtime.spawn(f_listener.boxed().unit_error().compat());

    // The dialer side keeps sending RPC requests. In each iteration of the benchmark, it sends
    // NUM_MSGS requests and blocks on getting the responses.
    b.iter(|| {
        let mut requests = FuturesUnordered::new();
        for _ in 0..NUM_MSGS {
            requests.push(request_block(
                dialer_sender.clone(),
                listener_peer_id,
                req.clone(),
            ));
        }
        while let Some(res) = block_on(requests.next()) {
            let _ = res.unwrap();
        }
    });
    block_on(runtime.shutdown_now().compat()).unwrap();
}

async fn request_block(
    mut sender: ConsensusNetworkSender,
    recipient: PeerId,
    req_msg: RequestBlock,
) -> Result<RespondBlock, RpcError> {
    sender
        .request_block(recipient, req_msg, Duration::from_secs(15))
        .await
}

fn compose_request_block() -> RequestBlock {
    let mut req = RequestBlock::new();
    req.set_block_id(vec![0u8; 32].into());
    req
}

fn compose_respond_block(msg_len: usize) -> ConsensusMsg {
    let mut msg = ConsensusMsg::new();
    let res = msg.mut_respond_block();
    let mut block = Block::new();
    block.set_payload(vec![0u8; msg_len].into());
    res.mut_blocks().push(block);
    msg
}

fn network_crate_benchmark(c: &mut Criterion) {
    ::logger::try_init_for_testing();

    // Parameterize benchmarks over the message length.
    let msg_lens = vec![32usize, 256, 1 * KiB, 4 * KiB, 64 * KiB, 256 * KiB, 1 * MiB];

    c.bench(
        "network_crate_benchmark",
        ParameterizedBenchmark::new("direct_send", direct_send_bench, msg_lens)
            .with_function("rpc", rpc_bench)
            .sample_size(10)
            .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic))
            .throughput(|msg_len| Throughput::Bytes((*msg_len as u32) * NUM_MSGS)),
    );
}

criterion_group!(benches, network_crate_benchmark);
criterion_main!(benches);
