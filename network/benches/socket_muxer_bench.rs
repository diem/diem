// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![feature(async_await)]
// Allow KiB, MiB consts
#![allow(non_upper_case_globals, non_snake_case)]
// Allow fns to take &usize, since criterion only passes parameters by ref
#![allow(clippy::trivially_copy_pass_by_ref)]
// Allow writing 1 * KiB or 1 * MiB
#![allow(clippy::identity_op)]

//! Network Benchmarks
//! ==================
//!
//! The `socket_muxer_bench` benchmarks measures the throughput of sending
//! messages over a single stream.
//!
//! # Run the benchmarks
//!
//! `cargo bench -p network`
//!
//! # View the report
//!
//! `open network/target/criterion/report/index.html`
//!
//! Note: gnuplot must be installed to generate benchmark plots.

use bytes::Bytes;
use criterion::{
    criterion_group, criterion_main, AxisScale, Bencher, Criterion, ParameterizedBenchmark,
    PlotConfiguration, Throughput,
};
use futures::{
    channel::oneshot,
    compat::Sink01CompatExt,
    executor::block_on,
    future::{Future, FutureExt, TryFutureExt},
    io::{AsyncRead, AsyncReadExt, AsyncWrite},
    sink::{Sink, SinkExt},
    stream::{self, Stream, StreamExt},
};
use memsocket::MemorySocket;
use netcore::{
    multiplexing::{yamux::Yamux, StreamMultiplexer},
    transport::{
        memory::MemoryTransport,
        tcp::{TcpSocket, TcpTransport},
        Transport, TransportExt,
    },
};
use noise::{NoiseConfig, NoiseSocket};
use parity_multiaddr::Multiaddr;
use std::{fmt::Debug, sync::Arc, time::Duration};
use tokio::{
    codec::Framed,
    runtime::{Runtime, TaskExecutor},
};
use unsigned_varint::codec::UviBytes;

const KiB: usize = 1 << 10;
const MiB: usize = 1 << 20;

// The number of messages to send per `Bencher::iter`. We also flush to ensure
// we measure all the message being sent.
const SENDS_PER_ITER: usize = 100;

/// Build a MemorySocket + Noise transport
fn build_memsocket_noise_transport() -> impl Transport<Output = NoiseSocket<MemorySocket>> {
    MemoryTransport::default().and_then(move |socket, origin| {
        async move {
            let noise_config = Arc::new(NoiseConfig::new_random());
            let (_remote_static_key, socket) =
                noise_config.upgrade_connection(socket, origin).await?;
            Ok(socket)
        }
    })
}

/// Build a MemorySocket + Muxer transport
fn build_memsocket_muxer_transport() -> impl Transport<Output = impl StreamMultiplexer> {
    MemoryTransport::default().and_then(Yamux::upgrade_connection)
}

/// Build a MemorySocket + Noise + Muxer transport
fn build_memsocket_noise_muxer_transport() -> impl Transport<Output = impl StreamMultiplexer> {
    MemoryTransport::default()
        .and_then(move |socket, origin| {
            async move {
                let noise_config = Arc::new(NoiseConfig::new_random());
                let (_remote_static_key, socket) =
                    noise_config.upgrade_connection(socket, origin).await?;
                Ok(socket)
            }
        })
        .and_then(Yamux::upgrade_connection)
}

/// Build a Tcp + Noise transport
fn build_tcp_noise_transport() -> impl Transport<Output = NoiseSocket<TcpSocket>> {
    TcpTransport::default().and_then(move |socket, origin| {
        async move {
            let noise_config = Arc::new(NoiseConfig::new_random());
            let (_remote_static_key, socket) =
                noise_config.upgrade_connection(socket, origin).await?;
            Ok(socket)
        }
    })
}

/// Build a Tcp + Muxer transport
fn build_tcp_muxer_transport() -> impl Transport<Output = impl StreamMultiplexer> {
    TcpTransport::default().and_then(Yamux::upgrade_connection)
}

/// Build a Tcp + Noise + Muxer transport
fn build_tcp_noise_muxer_transport() -> impl Transport<Output = impl StreamMultiplexer> {
    TcpTransport::default()
        .and_then(move |socket, origin| {
            async move {
                let noise_config = Arc::new(NoiseConfig::new_random());
                let (_remote_static_key, socket) =
                    noise_config.upgrade_connection(socket, origin).await?;
                Ok(socket)
            }
        })
        .and_then(Yamux::upgrade_connection)
}

/// Spawn a Future on an executor, but send the output over oneshot channel.
fn spawn_with_handle<F>(executor: &TaskExecutor, f: F) -> oneshot::Receiver<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    let (tx, rx) = oneshot::channel();
    let f_send = async move {
        let out = f.await;
        let _ = tx.send(out);
    };
    executor.spawn(f_send.boxed().unit_error().compat());
    rx
}

/// Server side handler for send throughput benchmark when the messages are sent
/// over a simple stream (tcp or in-memory).
async fn server_stream_handler<L, I, S, E>(mut server_listener: L) -> impl Stream
where
    L: Stream<Item = Result<(I, Multiaddr), E>> + Unpin,
    I: Future<Output = Result<S, E>>,
    S: AsyncRead + AsyncWrite + Unpin,
    E: ::std::error::Error,
{
    // Wait for next inbound connection
    let (f_stream, _) = server_listener.next().await.unwrap().unwrap();
    let stream = f_stream.await.unwrap();
    let mut stream = Framed::new(stream.compat(), UviBytes::<Bytes>::default()).sink_compat();

    // Drain all messages from the client.
    while let Some(_) = stream.next().await {}
    stream.close().await.unwrap();

    // Return stream so we drop after runtime shuts down to avoid race
    stream
}

/// Server side handler for send throughput benchmark when the messages are sent
/// over a muxer substream.
async fn server_muxer_handler<L, I, M, E>(mut server_listener: L) -> (M, impl Stream)
where
    L: Stream<Item = Result<(I, Multiaddr), E>> + Unpin,
    I: Future<Output = Result<M, E>>,
    M: StreamMultiplexer,
    E: ::std::error::Error,
{
    // Wait for next inbound connection
    let (f_muxer, _) = server_listener.next().await.unwrap().unwrap();
    let muxer = f_muxer.await.unwrap();

    // Wait for inbound client substream
    let mut muxer_inbounds = muxer.listen_for_inbound();
    let substream = muxer_inbounds.next().await.unwrap().unwrap();
    let mut stream = Framed::new(substream.compat(), UviBytes::<Bytes>::default()).sink_compat();

    // Drain all messages from the client.
    while let Some(_) = stream.next().await {}
    stream.close().await.unwrap();

    // Return muxer and stream so we drop after runtime shuts down to avoid race
    (muxer, stream)
}

/// The tight inner loop we're actually benchmarking. In this benchmark, we simply
/// measure the throughput of sending many messages of size `msg_len` over
/// `client_stream`.
fn bench_client_send<S>(b: &mut Bencher, msg_len: usize, client_stream: &mut S)
where
    S: Sink<Bytes> + Unpin,
    S::SinkError: Debug,
{
    // Benchmark sending over the in-memory stream.
    let data = Bytes::from(vec![0u8; msg_len]);
    b.iter(|| {
        // Create a stream of messages to send
        let mut data_stream = stream::repeat(data.clone()).take(SENDS_PER_ITER as u64);
        // Send the batch of messages. Note that `Sink::send_all` will flush the
        // sink after exhausting the `data_stream`, which is necessary to ensure
        // we measure sending all of the messages.
        block_on(client_stream.send_all(&mut data_stream)).unwrap();
    });

    // Shutdown client stream.
    block_on(client_stream.close()).unwrap();
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
        .block_on(client_transport.dial(server_addr).unwrap().boxed().compat())
        .unwrap();
    let mut client_stream =
        Framed::new(client_socket.compat(), UviBytes::<Bytes>::default()).sink_compat();

    // Benchmark client sending data to server.
    bench_client_send(b, msg_len, &mut client_stream);

    // Return the stream so we can drop it after the bench completes
    client_stream
}

/// Setup and benchmark the client side for the muxer substream case (yamux).
fn bench_client_muxer_send<T, M>(
    b: &mut Bencher,
    msg_len: usize,
    runtime: &mut Runtime,
    server_addr: Multiaddr,
    client_transport: T,
) -> (M, impl Stream)
where
    T: Transport<Output = M> + Send + 'static,
    M: StreamMultiplexer + 'static,
{
    // Client dials the server. Some of our transports have timeouts built in,
    // which means the futures must be run on a tokio Runtime.
    let f_client = async move {
        let client_muxer = client_transport.dial(server_addr).unwrap().await.unwrap();
        let client_substream = client_muxer.open_outbound().await.unwrap();
        (client_muxer, client_substream)
    };
    let (client_muxer, client_substream) = runtime
        .block_on(f_client.boxed().unit_error().compat())
        .unwrap();
    let mut client_stream =
        Framed::new(client_substream.compat(), UviBytes::<Bytes>::default()).sink_compat();

    // Benchmark client sending data to server.
    bench_client_send(b, msg_len, &mut client_stream);

    // Return the muxer and stream so we can drop them after the bench completes
    (client_muxer, client_stream)
}

/// Benchmark the throughput of sending messages of size `msg_len` over an
/// in-memory socket.
fn bench_memsocket_send(b: &mut Bencher, msg_len: &usize) {
    let mut runtime = Runtime::new().unwrap();
    let executor = runtime.executor();

    let client_transport = MemoryTransport::default();
    let server_transport = MemoryTransport::default();
    let (server_listener, server_addr) = server_transport
        .listen_on("/memory/0".parse().unwrap())
        .unwrap();

    // Server waits for client connection then reads all messages.
    let f_server = spawn_with_handle(&executor, server_stream_handler(server_listener));

    // Benchmark sending some data to the server.
    let _client_stream =
        bench_client_stream_send(b, *msg_len, &mut runtime, server_addr, client_transport);

    // Wait for server task to finish.
    let _server_stream = block_on(f_server).unwrap();
}

/// Benchmark the throughput of sending messages of size `msg_len` over an
/// in-memory socket with Noise encryption.
fn bench_memsocket_noise_send(b: &mut Bencher, msg_len: &usize) {
    let mut runtime = Runtime::new().unwrap();
    let executor = runtime.executor();

    let client_transport = build_memsocket_noise_transport();
    let server_transport = build_memsocket_noise_transport();
    let (server_listener, server_addr) = server_transport
        .listen_on("/memory/0".parse().unwrap())
        .unwrap();

    // Server waits for client connection then reads all messages.
    let f_server = spawn_with_handle(&executor, server_stream_handler(server_listener));

    // Benchmark sending some data to the server.
    let _client_stream =
        bench_client_stream_send(b, *msg_len, &mut runtime, server_addr, client_transport);

    // Wait for server task to finish.
    let _server_stream = block_on(f_server).unwrap();
}

/// Benchmark the throughput of sending messages of size `msg_len` over a muxer
/// over an in-memory socket.
fn bench_memsocket_muxer_send(b: &mut Bencher, msg_len: &usize) {
    let mut runtime = Runtime::new().unwrap();
    let executor = runtime.executor();

    let client_transport = build_memsocket_muxer_transport();
    let server_transport = build_memsocket_muxer_transport();
    let (server_listener, server_addr) = server_transport
        .listen_on("/memory/0".parse().unwrap())
        .unwrap();

    // Server waits for client connection and substream, then reads all messages.
    let f_server = spawn_with_handle(&executor, server_muxer_handler(server_listener));

    // Benchmark sending some data to the server.
    let (_client_muxer, _client_stream) =
        bench_client_muxer_send(b, *msg_len, &mut runtime, server_addr, client_transport);

    // Wait for server task to finish.
    let (_server_muxer, _server_stream) = block_on(f_server).unwrap();
}

/// Benchmark the throughput of sending messages of size`msg_len` over a muxer
/// over an in-memory socket with noise encryption
fn bench_memsocket_noise_muxer_send(b: &mut Bencher, msg_len: &usize) {
    let mut runtime = Runtime::new().unwrap();
    let executor = runtime.executor();

    let client_transport = build_memsocket_noise_muxer_transport();
    let server_transport = build_memsocket_noise_muxer_transport();
    let (server_listener, server_addr) = server_transport
        .listen_on("/memory/0".parse().unwrap())
        .unwrap();

    // Server waits for client connection and substream, then reads all messages.
    let f_server = spawn_with_handle(&executor, server_muxer_handler(server_listener));

    // Benchmark sending some data to the server.
    let (_client_muxer, _client_stream) =
        bench_client_muxer_send(b, *msg_len, &mut runtime, server_addr, client_transport);

    // Wait for server task to finish.
    let (_server_muxer, _server_stream) = block_on(f_server).unwrap();
}

/// Benchmark the throughput of sending messages of size `msg_len` over tcp
/// loopback.
fn bench_tcp_send(b: &mut Bencher, msg_len: &usize) {
    let mut runtime = Runtime::new().unwrap();
    let executor = runtime.executor();

    let client_transport = TcpTransport::default();
    let server_transport = TcpTransport::default();
    let (server_listener, server_addr) = server_transport
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();

    // Server waits for client connection then reads all messages.
    let f_server = spawn_with_handle(&executor, server_stream_handler(server_listener));

    // Benchmark sending some data to the server.
    let _client_stream =
        bench_client_stream_send(b, *msg_len, &mut runtime, server_addr, client_transport);

    // Wait for server task to finish.
    let _server_stream = block_on(f_server).unwrap();
}

/// Benchmark the throughput of sending messages of size `msg_len` over tcp
/// loopback with Noise encryption.
fn bench_tcp_noise_send(b: &mut Bencher, msg_len: &usize) {
    let mut runtime = Runtime::new().unwrap();
    let executor = runtime.executor();

    let client_transport = build_tcp_noise_transport();
    let server_transport = build_tcp_noise_transport();
    let (server_listener, server_addr) = server_transport
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();

    // Server waits for client connection then reads all messages.
    let f_server = spawn_with_handle(&executor, server_stream_handler(server_listener));

    // Benchmark sending some data to the server.
    let _client_stream =
        bench_client_stream_send(b, *msg_len, &mut runtime, server_addr, client_transport);

    // Wait for server task to finish.
    let _server_stream = block_on(f_server).unwrap();
}

/// Benchmark the throughput of sending messages of size `msg_len` over a muxer
/// over tcp loopback.
fn bench_tcp_muxer_send(b: &mut Bencher, msg_len: &usize) {
    let mut runtime = Runtime::new().unwrap();
    let executor = runtime.executor();

    let client_transport = build_tcp_muxer_transport();
    let server_transport = build_tcp_muxer_transport();
    let (server_listener, server_addr) = server_transport
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();

    // Server waits for client connection and substream, then reads all messages.
    let f_server = spawn_with_handle(&executor, server_muxer_handler(server_listener));

    // Benchmark sending some data to the server.
    let (_client_muxer, _client_stream) =
        bench_client_muxer_send(b, *msg_len, &mut runtime, server_addr, client_transport);

    // Wait for server task to finish.
    let (_server_muxer, _server_stream) = block_on(f_server).unwrap();
}

/// Benchmark the throughput of sending messages of size `msg_len` over a muxer over tcp lookback
/// with noise encryption.
fn bench_tcp_noise_muxer_send(b: &mut Bencher, msg_len: &usize) {
    let mut runtime = Runtime::new().unwrap();
    let executor = runtime.executor();

    let client_transport = build_tcp_noise_muxer_transport();
    let server_transport = build_tcp_noise_muxer_transport();
    let (server_listener, server_addr) = server_transport
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();

    // Server waits for client connection and substream, then reads all messages.
    let f_server = spawn_with_handle(&executor, server_muxer_handler(server_listener));

    // Benchmark sending some data to the server.
    let (_client_muxer, _client_stream) =
        bench_client_muxer_send(b, *msg_len, &mut runtime, server_addr, client_transport);

    // Wait for server task to finish.
    let (_server_muxer, _server_stream) = block_on(f_server).unwrap();
}

/// Measure sending messages of varying sizes over:
/// 1. in-memory transport
/// 2. in-memory transport + noise encryption
/// 3. in-memory transport + yamux
/// 4. in-memory transport + noise encryption + yamux
/// 5. tcp transport
/// 6. tcp transport + noise encryption
/// 7. tcp transport + yamux
/// 8. tcp transport + noise encryption + yamux
///
/// Important:
/// 1. Measures single-threaded send since only one sending task is used, so any
///    muxer lock contention is likely not measured.
/// 2. We use a `UviBytes` codec to frame the benchmark messages since this is
///    what we currently use in the codebase; however, this seems to add not
///    insignificant overhead and might change in the near future.
/// 3. TCP benchmarks are only over loopback.
/// 4. Socket buffer sizes and buffering strategies are not yet optimized.
fn socket_muxer_bench(c: &mut Criterion) {
    ::logger::try_init_for_testing();

    // Parameterize benchmarks over the message length.
    let msg_lens = vec![32usize, 256, 1 * KiB, 4 * KiB, 64 * KiB, 256 * KiB, 1 * MiB];

    c.bench(
        "socket_muxer_send_throughput",
        ParameterizedBenchmark::new("memsocket", bench_memsocket_send, msg_lens)
            .with_function("memsocket+noise", bench_memsocket_noise_send)
            .with_function("memsocket+muxer", bench_memsocket_muxer_send)
            .with_function("memsocket+noise+muxer", bench_memsocket_noise_muxer_send)
            .with_function("tcp", bench_tcp_send)
            .with_function("tcp+noise", bench_tcp_noise_send)
            .with_function("tcp+muxer", bench_tcp_muxer_send)
            .with_function("tcp+noise+muxer", bench_tcp_noise_muxer_send)
            .warm_up_time(Duration::from_secs(2))
            .measurement_time(Duration::from_secs(2))
            .sample_size(10)
            .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic))
            .throughput(|msg_len| {
                let msg_len = *msg_len as u32;
                let num_msgs = SENDS_PER_ITER as u32;
                Throughput::Bytes(msg_len * num_msgs)
            }),
    );
}

criterion_group!(network_benches, socket_muxer_bench);
criterion_main!(network_benches);
