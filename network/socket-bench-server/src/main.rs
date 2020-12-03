// Copyright (c) The Diem Core Contributors
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
//! `RUSTFLAGS="-Ctarget-cpu=skylake -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3" TCP_ADDR=/ip6/::1/tcp/12345 cargo x bench -p network remote_tcp`

use diem_logger::info;
use netcore::transport::tcp::TcpTransport;
use socket_bench_server::{build_tcp_noise_transport, start_stream_server, Args};
use tokio::runtime::Builder;

fn main() {
    ::diem_logger::Logger::new().init();

    let args = Args::from_env();

    let rt = Builder::new()
        .threaded_scheduler()
        .core_threads(32)
        .enable_all()
        .build()
        .unwrap();
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
