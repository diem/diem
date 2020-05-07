// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Standalone server for socket_bench
//! ========================================
//!
//! You can run `socket_bench` across a real network by running this bench
//! server remotely. For example,
//!
//! `TCP_ADDR=/ip6/::1/tcp/12345 cargo run --release --bin socket-bench-server`
//!
//! will run the socket bench server handling the remote_tcp benchmark. A
//! corresponding client would exercise this benchmark using
//!
//! `TCP_ADDR=/ip6/::1/tcp/12345 cargo x bench -p network remote_tcp`

use libra_logger::info;
use netcore::transport::tcp::TcpTransport;
use socket_bench_server::{build_tcp_noise_transport, start_stream_server, Args};
use tokio::runtime::Runtime;

fn main() {
    ::libra_logger::Logger::new().init();

    let args = Args::from_env();

    let rt = Runtime::new().unwrap();
    let executor = rt.handle();

    if let Some(addr) = args.tcp_addr {
        let addr = start_stream_server(&executor, TcpTransport::default(), addr);
        info!("bench: tcp: listening on: {}", addr);
    }

    if let Some(addr) = args.tcp_noise_addr {
        let addr = start_stream_server(&executor, build_tcp_noise_transport(), addr);
        info!("bench: tcp+noise: listening on: {}", addr);
    }
    std::thread::park();
}
