// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Allow KiB, MiB consts
#![allow(non_upper_case_globals, non_snake_case)]
// Allow fns to take &usize, since criterion only passes parameters by ref
#![allow(clippy::trivially_copy_pass_by_ref)]
// Allow writing 1 * KiB or 1 * MiB
#![allow(clippy::identity_op)]

use criterion::{
    criterion_group, criterion_main, AxisScale, Bencher, Criterion, ParameterizedBenchmark,
    PlotConfiguration, Throughput,
};
use futures::{
    channel::mpsc,
    executor::block_on,
    sink::SinkExt,
    stream::{FuturesUnordered, StreamExt},
};
use libra_prost_ext::MessageExt;
use libra_types::PeerId;
use network::{
    proto::ConsensusMsg,
    protocols::rpc::error::RpcError,
    validator_network::{
        test_network::{setup_network, TestNetworkSender},
        Event,
    },
};
use std::time::Duration;

const KiB: usize = 1 << 10;
const MiB: usize = 1 << 20;
const NUM_MSGS: u32 = 100;
const TOLERANCE: u32 = 20;

fn direct_send_bench(b: &mut Bencher, msg_len: &usize) {
    let tn = setup_network();
    let runtime = tn.runtime;
    let mut dialer_sender = tn.dialer_sender;
    let listener_peer_id = tn.listener_peer_id;
    let mut listener_events = tn.listener_events;

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
    let tn = setup_network();
    let runtime = tn.runtime;
    let dialer_sender = tn.dialer_sender;
    let listener_peer_id = tn.listener_peer_id;
    let mut listener_events = tn.listener_events;

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
    mut sender: TestNetworkSender,
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
            .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic))
            .throughput(|msg_len| Throughput::Bytes(((*msg_len as u32) * NUM_MSGS).into())),
    );
}

criterion_group!(benches, network_crate_benchmark);
criterion_main!(benches);
