// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Standalone server for socket_bench
//! ========================================
//!
//! You can run `socket_bench` across a real network by running this bench
//! server remotely. For example,
//!
//! `RUSTFLAGS="-Ctarget-cpu=skylake -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3" TCP_ADDR=/ip6/::1/tcp/12345 cargo run --release -p socket-bench-server`
//!
//! will run the socket bench server handling the remote_tcp benchmark. A
//! corresponding client would exercise this benchmark using
//!
//! `RUSTFLAGS="-Ctarget-cpu=skylake -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3" TCP_ADDR=/ip6/::1/tcp/12345 cargo x bench -p network -- connection_throughput`

use libra_crypto::{test_utils::TEST_SEED, x25519, Uniform as _};
use libra_logger::info;
use netcore::transport::tcp::TcpTransport;
use rand::prelude::*;
use socket_bench_server::{build_tcp_noise_transport, start_stream_server, Args};
use tokio::runtime::Builder;

fn main() {
    ::libra_logger::Logger::new().init();

    let args = Args::from_env();

    let rt = Builder::new()
        .threaded_scheduler()
        .core_threads(32)
        .enable_all()
        .build()
        .unwrap();
    let executor = rt.handle();

    // passed as TCP_ADDR=/ip6/::1/tcp/12345
    if let Some(addr) = args.tcp_addr {
        let addr = start_stream_server(&executor, TcpTransport::default(), addr);
        info!("bench: tcp: listening on: {}", addr);
    }

    // passed as TCP_NOISE_ADDR=/ip4/1.2.3.4/tcp/1234
    if let Some(addr) = args.tcp_noise_addr {
        // generate this peer's keypair
        let mut rng: StdRng = SeedableRng::from_seed(TEST_SEED);
        let private_key = x25519::PrivateKey::generate(&mut rng);
        let public_key = private_key.public_key();

        let noise_transport = build_tcp_noise_transport(Some(private_key), true);
        let addr = start_stream_server(&executor, noise_transport, addr);

        // display full networkaddr
        let handshake_version = 0;
        let addr = addr.append_prod_protos(public_key, handshake_version);
        info!("bench: tcp+noise: listening on: {}", addr);
    }
    std::thread::park();
}
