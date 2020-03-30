// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Allow KiB, MiB consts
#![allow(non_upper_case_globals, non_snake_case)]
// Allow fns to take &usize, since criterion only passes parameters by ref
#![allow(clippy::trivially_copy_pass_by_ref)]
// Allow writing 1 * KiB or 1 * MiB
#![allow(clippy::identity_op)]

//! Network Benchmarks
//! ==================
//!
//! The `socket_bench` benchmarks measures the throughput of sending
//! messages over a single socket.
//!
//! # Run the benchmarks
//!
//! `cargo bench -p network`
//!
//! # View the report
//!
//! `open network/target/criterion/report/index.html`
//!
//! # Remote benchmarks
//!
//! The `socket_bench` can also act as a client to the corresponding
//! `socket-bench-server`. Simply pass in one or more of the following env vars
//! which correspond to different remote benchmarks, e.g.,
//!
//! `TCP_ADDR=/ip4/12.34.56.78/tcp/1234 cargo bench -p network remote_tcp`
//!
//! The message lengths (in bytes) we benchmark can be changed using the
//! `$MSG_LENS` environment variable.
//!
//! `MSG_LENS="[123, 456]" cargo bench -p network local_tcp`
//!
//! Note: gnuplot must be installed to generate benchmark plots.

use bytes::{Bytes, BytesMut};
use criterion::{
    criterion_group, criterion_main, AxisScale, Bencher, Criterion, ParameterizedBenchmark,
    PlotConfiguration, Throughput,
};
use futures::{
    executor::block_on,
    io::{AsyncRead, AsyncWrite},
    sink::{Sink, SinkExt},
    stream::{self, Stream, StreamExt},
};
use netcore::{
    compat::IoCompat,
    transport::{memory::MemoryTransport, tcp::TcpTransport, Transport},
};
use parity_multiaddr::Multiaddr;
use socket_bench_server::{
    build_memsocket_noise_transport, build_tcp_noise_transport, start_stream_server, Args,
};
use std::{fmt::Debug, io, time::Duration};
use tokio::runtime::Runtime;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

const KiB: usize = 1 << 10;
const MiB: usize = 1 << 20;

// The number of messages to send per `Bencher::iter`. We also flush to ensure
// we measure all the message being sent.
const SENDS_PER_ITER: usize = 100;

/// The tight inner loop we're actually benchmarking. In this benchmark, we simply
/// measure the throughput of sending many messages of size `msg_len` over
/// `client_stream`.
fn bench_client_send<S>(b: &mut Bencher, msg_len: usize, client_stream: &mut S)
where
    S: Sink<Bytes> + Stream<Item = Result<BytesMut, io::Error>> + Unpin,
    S::Error: Debug,
{
    // Benchmark sending over the in-memory stream.
    let data = Bytes::from(vec![0u8; msg_len]);
    b.iter(|| {
        // Create a stream of messages to send
        let mut data_stream = stream::repeat(data.clone()).take(SENDS_PER_ITER).map(Ok);
        // Send the batch of messages. Note that `Sink::send_all` will flush the
        // sink after exhausting the `data_stream`, which is necessary to ensure
        // we measure sending all of the messages.
        block_on(client_stream.send_all(&mut data_stream)).unwrap();
    });

    // Client half-closes their side of the stream
    block_on(client_stream.close()).unwrap();

    // Wait for server to half-close to complete the shutdown
    assert!(block_on(client_stream.next()).is_none());
}

/// Setup and benchmark the client side for the simple stream case
/// (tcp or in-memory).
fn bench_client_stream_send<T, S>(
    b: &mut Bencher,
    msg_len: usize,
    runtime: &mut Runtime,
    server_addr: Multiaddr,
    client_transport: T,
) -> impl Stream
where
    T: Transport<Output = S> + 'static,
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    // Client dials the server. Some of our transports have timeouts built in,
    // which means the futures must be run on a tokio Runtime.
    let client_socket = runtime
        .block_on(client_transport.dial(server_addr).unwrap())
        .unwrap();
    let mut client_stream = Framed::new(IoCompat::new(client_socket), LengthDelimitedCodec::new());

    // Benchmark client sending data to server.
    bench_client_send(b, msg_len, &mut client_stream);

    // Return the stream so we can drop it after the bench completes
    client_stream
}

/// Benchmark the throughput of sending messages of size `msg_len` over an
/// in-memory socket.
fn bench_memsocket_send(b: &mut Bencher, msg_len: &usize, server_addr: Multiaddr) {
    let mut runtime = Runtime::new().unwrap();

    let client_transport = MemoryTransport::default();

    // Benchmark sending some data to the server.
    let _client_stream =
        bench_client_stream_send(b, *msg_len, &mut runtime, server_addr, client_transport);
}

/// Benchmark the throughput of sending messages of size `msg_len` over an
/// in-memory socket with Noise encryption.
fn bench_memsocket_noise_send(b: &mut Bencher, msg_len: &usize, server_addr: Multiaddr) {
    let mut runtime = Runtime::new().unwrap();

    let client_transport = build_memsocket_noise_transport();

    // Benchmark sending some data to the server.
    let _client_stream =
        bench_client_stream_send(b, *msg_len, &mut runtime, server_addr, client_transport);
}

/// Benchmark the throughput of sending messages of size `msg_len` over tcp to
/// server at multiaddr `server_addr`.
fn bench_tcp_send(b: &mut Bencher, msg_len: &usize, server_addr: Multiaddr) {
    let mut runtime = Runtime::new().unwrap();

    let client_transport = TcpTransport::default();

    // Benchmark sending some data to the server.
    let _client_stream =
        bench_client_stream_send(b, *msg_len, &mut runtime, server_addr, client_transport);
}

/// Benchmark the throughput of sending messages of size `msg_len` over tcp to
/// server at multiaddr `server_addr` with TCP_NODELAY set.
fn bench_tcp_send_with_nodelay(b: &mut Bencher, msg_len: &usize, server_addr: Multiaddr) {
    let mut runtime = Runtime::new().unwrap();

    let client_transport = TcpTransport {
        nodelay: Some(true),
        ..TcpTransport::default()
    };

    // Benchmark sending some data to the server.
    let _client_stream =
        bench_client_stream_send(b, *msg_len, &mut runtime, server_addr, client_transport);
}

/// Benchmark the throughput of sending messages of size `msg_len` over tcp with
/// Noise encryption to server at multiaddr `server_addr`.
fn bench_tcp_noise_send(b: &mut Bencher, msg_len: &usize, server_addr: Multiaddr) {
    let mut runtime = Runtime::new().unwrap();

    let client_transport = build_tcp_noise_transport();

    // Benchmark sending some data to the server.
    let _client_stream =
        bench_client_stream_send(b, *msg_len, &mut runtime, server_addr, client_transport);
}

/// Measure sending messages of varying sizes over varying transports, where
///
/// base transport := {in-memory, loopback tcp, remote tcp}
/// encryption := {none, noise}
/// transports := base transport × encryption
///
/// listed explicitly,
///
///  1. in-memory transport
///  2. in-memory transport + noise encryption
///  3. loopback tcp transport
///  4. loopback tcp transport + noise encryption
///  5. remote tcp transport
///  6. remote tcp transport + noise encryption
///  7. remote tcp transport + nodelay
///
/// Important:
/// 1. We use a `UviBytes` codec to frame the benchmark messages since this is
///    what we currently use in the codebase; however, this seems to add not
///    insignificant overhead and might change in the near future.
/// 2. Socket buffer sizes and buffering strategies are not yet optimized.
/// 3. local_tcp benchmarks are only over loopback.
/// 4. remote_tcp benchmarks connect to a `socket_bench_server` instance running
///    running remotely.
/// 6. The remote benchmarks connect to env-defined multiaddrs `$TCP_ADDR` and
///    `$TCP_NOISE_ADDR`  for
///    benchmarks `remote_tcp`, `remote_tcp+noise` and `remote_tcp+nodelay` respectively.
fn socket_bench(c: &mut Criterion) {
    ::libra_logger::Logger::new().environment_only(true).init();

    let rt = Runtime::new().unwrap();
    let executor = rt.handle().clone();

    let args = Args::from_env();

    let remote_tcp_addr = args.tcp_addr;
    let remote_tcp_noise_addr = args.tcp_noise_addr;

    // Parameterize benchmarks over the message length.
    let default_msg_lens = vec![32usize, 256, 1 * KiB, 4 * KiB, 64 * KiB, 256 * KiB, 1 * MiB];
    let msg_lens = args.msg_lens.unwrap_or(default_msg_lens);

    // start local bench servers

    let memsocket_addr = start_stream_server(
        &executor,
        MemoryTransport::default(),
        "/memory/0".parse().unwrap(),
    );
    let memsocket_noise_addr = start_stream_server(
        &executor,
        build_memsocket_noise_transport(),
        "/memory/0".parse().unwrap(),
    );

    let local_tcp_addr = start_stream_server(
        &executor,
        TcpTransport::default(),
        "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
    );
    let local_tcp_nodelay_addr = start_stream_server(
        &executor,
        TcpTransport {
            nodelay: Some(true),
            ..TcpTransport::default()
        },
        "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
    );
    let local_tcp_noise_addr = start_stream_server(
        &executor,
        build_tcp_noise_transport(),
        "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
    );

    // add the memsocket and tcp loopback socket benches

    let mut bench = ParameterizedBenchmark::new(
        "memsocket",
        move |b, msg_len| bench_memsocket_send(b, msg_len, memsocket_addr.clone()),
        msg_lens,
    )
    .with_function("memsocket+noise", move |b, msg_len| {
        bench_memsocket_noise_send(b, msg_len, memsocket_noise_addr.clone())
    })
    .with_function("local_tcp", move |b, msg_len| {
        bench_tcp_send(b, msg_len, local_tcp_addr.clone())
    })
    .with_function("local_tcp+noise", move |b, msg_len| {
        bench_tcp_noise_send(b, msg_len, local_tcp_noise_addr.clone())
    })
    .with_function("local_tcp_nodelay", move |b, msg_len| {
        bench_tcp_send_with_nodelay(b, msg_len, local_tcp_nodelay_addr.clone())
    });

    // optionally enable remote benches if the env variables are set

    if let Some(remote_tcp_addr) = remote_tcp_addr {
        bench = bench.with_function("remote_tcp", move |b, msg_len| {
            bench_tcp_send(b, msg_len, remote_tcp_addr.clone())
        });
    }
    if let Some(remote_tcp_noise_addr) = remote_tcp_noise_addr {
        bench = bench.with_function("remote_tcp+noise", move |b, msg_len| {
            bench_tcp_noise_send(b, msg_len, remote_tcp_noise_addr.clone())
        });
    }

    // set bench configuration

    bench = bench
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(2))
        .sample_size(10)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic))
        .throughput(|msg_len| {
            let msg_len = *msg_len as u32;
            let num_msgs = SENDS_PER_ITER as u32;
            Throughput::Bytes(u64::from(msg_len * num_msgs))
        });

    c.bench("socket_send_throughput", bench);
}

criterion_group!(network_benches, socket_bench);
criterion_main!(network_benches);
