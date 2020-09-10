// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use futures::{
    future::Future,
    io::{AsyncRead, AsyncWrite},
    sink::SinkExt,
    stream::{Stream, StreamExt},
};
use libra_config::network_id::NetworkContext;
use libra_crypto::{test_utils::TEST_SEED, x25519, Uniform as _};
use libra_logger::prelude::*;
use libra_network_address::NetworkAddress;
use libra_types::PeerId;
use memsocket::MemorySocket;
use netcore::{
    compat::IoCompat,
    transport::{
        memory::MemoryTransport,
        tcp::{TcpSocket, TcpTransport},
        Transport, TransportExt,
    },
};
use network::noise::{stream::NoiseStream, HandshakeAuthMode, NoiseUpgrader};
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
pub fn build_memsocket_noise_transport() -> impl Transport<Output = NoiseStream<MemorySocket>> {
    MemoryTransport::default().and_then(move |socket, addr, origin| async move {
        let mut rng: StdRng = SeedableRng::from_seed(TEST_SEED);
        let private = x25519::PrivateKey::generate(&mut rng);
        let public = private.public_key();
        let peer_id = PeerId::from_identity_public_key(public);
        let noise_config = Arc::new(NoiseUpgrader::new(
            NetworkContext::mock_with_peer_id(peer_id),
            private,
            HandshakeAuthMode::ServerOnly,
        ));
        let remote_public_key = addr.find_noise_proto();
        let (_remote_static_key, socket) = noise_config
            .upgrade_with_noise(socket, origin, remote_public_key)
            .await?;
        Ok(socket)
    })
}

/// Build a Tcp + Noise transport
pub fn build_tcp_noise_transport() -> impl Transport<Output = NoiseStream<TcpSocket>> {
    TcpTransport::default().and_then(move |socket, addr, origin| async move {
        let mut rng: StdRng = SeedableRng::from_seed(TEST_SEED);
        let private = x25519::PrivateKey::generate(&mut rng);
        let public = private.public_key();
        let peer_id = PeerId::from_identity_public_key(public);
        let noise_config = Arc::new(NoiseUpgrader::new(
            NetworkContext::mock_with_peer_id(peer_id),
            private,
            HandshakeAuthMode::ServerOnly,
        ));
        let remote_public_key = addr.find_noise_proto();
        let (_remote_static_key, socket) = noise_config
            .upgrade_with_noise(socket, origin, remote_public_key)
            .await?;
        Ok(socket)
    })
}

/// Server side handler for send throughput benchmark when the messages are sent
/// over a simple stream (tcp or in-memory).
pub async fn server_stream_handler<L, I, S, E>(server_listener: L)
where
    L: Stream<Item = Result<(I, NetworkAddress), E>> + Unpin,
    I: Future<Output = Result<S, E>> + Send + 'static,
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    E: ::std::error::Error + Send,
{
    // Wait for next inbound connection, this simulated the TransportHandler
    // which is single-threaded asynchronous accepting new connections.
    server_listener
        .for_each_concurrent(None, |result| async {
            match result {
                Ok((f_stream, _)) => {
                    match f_stream.await {
                        Ok(stream) => {
                            let mut stream =
                                Framed::new(IoCompat::new(stream), LengthDelimitedCodec::new());

                            tokio::task::spawn(async move {
                                // Drain all messages from the client.
                                while stream.next().await.is_some() {}
                                stream.close().await.unwrap();
                            });
                        }
                        Err(e) => error!("Connection upgrade failed {:?}", e),
                    };
                }
                Err(e) => error!("Stream failed {:?}", e),
            }
        })
        .await
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
