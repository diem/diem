// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use futures::{
    future::Future,
    io::{AsyncRead, AsyncWrite},
    sink::SinkExt,
    stream::{Stream, StreamExt},
};
use libra_crypto::test_utils::TEST_SEED;
use libra_network_address::NetworkAddress;
use memsocket::MemorySocket;
use netcore::{
    compat::IoCompat,
    transport::{
        memory::MemoryTransport,
        tcp::{TcpSocket, TcpTransport},
        Transport, TransportExt,
    },
};
use noise::{NoiseConfig, NoiseSocket};
use rand::prelude::*;
use std::{env, ffi::OsString, sync::Arc};
use tokio::runtime::Handle;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Debug)]
pub struct Args {
    pub tcp_addr: Option<NetworkAddress>,
    pub tcp_noise_addr: Option<NetworkAddress>,
    pub msg_lens: Option<Vec<usize>>,
}

fn parse_addr(s: OsString) -> NetworkAddress {
    s.to_str()
        .expect("Error: Address should be valid Unicode")
        .parse()
        .expect("Error: Address should be a multiaddr")
}

fn parse_msg_lens(s: OsString) -> Vec<usize> {
    let s = s
        .into_string()
        .expect("Error: $MSG_LENS should be valid Unicode");

    // check for surrounding array brackets
    if &s[..1] != "[" || &s[s.len() - 1..] != "]" {
        panic!(
            "Error: Malformed $MSG_LENS: \"{}\": Should be formatted like an array \"[123, 456]\"",
            s
        );
    }

    // parse Vec<usize> from comma-delimited string
    s[1..s.len() - 1]
        .split(',')
        .map(|ss| {
            ss.trim()
                .parse::<usize>()
                .expect("Error: Malformed $MSG_LENS: Failed to parse usize")
        })
        .collect()
}

impl Args {
    pub fn from_env() -> Self {
        Self {
            tcp_addr: env::var_os("TCP_ADDR").map(parse_addr),
            tcp_noise_addr: env::var_os("TCP_NOISE_ADDR").map(parse_addr),
            msg_lens: env::var_os("MSG_LENS").map(parse_msg_lens),
        }
    }
}

/// Build a MemorySocket + Noise transport
pub fn build_memsocket_noise_transport() -> impl Transport<Output = NoiseSocket<MemorySocket>> {
    MemoryTransport::default().and_then(move |socket, _addr, origin| async move {
        let mut rng: StdRng = SeedableRng::from_seed(TEST_SEED);
        let noise_config = Arc::new(NoiseConfig::new_random(&mut rng));
        let (_remote_static_key, socket) = noise_config.upgrade_connection(socket, origin).await?;
        Ok(socket)
    })
}

/// Build a Tcp + Noise transport
pub fn build_tcp_noise_transport() -> impl Transport<Output = NoiseSocket<TcpSocket>> {
    TcpTransport::default().and_then(move |socket, _addr, origin| async move {
        let mut rng: StdRng = SeedableRng::from_seed(TEST_SEED);
        let noise_config = Arc::new(NoiseConfig::new_random(&mut rng));
        let (_remote_static_key, socket) = noise_config.upgrade_connection(socket, origin).await?;
        Ok(socket)
    })
}

/// Server side handler for send throughput benchmark when the messages are sent
/// over a simple stream (tcp or in-memory).
pub async fn server_stream_handler<L, I, S, E>(mut server_listener: L)
where
    L: Stream<Item = Result<(I, NetworkAddress), E>> + Unpin,
    I: Future<Output = Result<S, E>>,
    S: AsyncRead + AsyncWrite + Unpin,
    E: ::std::error::Error,
{
    // Wait for next inbound connection
    while let Some(Ok((f_stream, _client_addr))) = server_listener.next().await {
        let stream = f_stream.await.unwrap();
        let mut stream = Framed::new(IoCompat::new(stream), LengthDelimitedCodec::new());

        // Drain all messages from the client.
        while let Some(_) = stream.next().await {}
        stream.close().await.unwrap();
    }
}

pub fn start_stream_server<T, L, I, S, E>(
    executor: &Handle,
    transport: T,
    listen_addr: NetworkAddress,
) -> NetworkAddress
where
    T: Transport<Output = S, Error = E, Listener = L, Inbound = I>,
    L: Stream<Item = Result<(I, NetworkAddress), E>> + Unpin + Send + 'static,
    I: Future<Output = Result<S, E>> + Send + 'static,
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    E: ::std::error::Error + Send + Sync + 'static,
{
    let (listener, server_addr) = executor.enter(move || transport.listen_on(listen_addr).unwrap());
    executor.spawn(server_stream_handler(listener));
    server_addr
}
