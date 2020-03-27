// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use futures::{
    future::{self, Future},
    io::{AsyncRead, AsyncWrite},
    sink::SinkExt,
    stream::{Stream, StreamExt},
};
use memsocket::MemorySocket;
use netcore::{
    compat::IoCompat,
    multiplexing::{yamux::Yamux, Control, StreamMultiplexer},
    transport::{
        memory::MemoryTransport,
        tcp::{TcpSocket, TcpTransport},
        ConnectionOrigin, Transport, TransportExt,
    },
};
use noise::{NoiseConfig, NoiseSocket};
use parity_multiaddr::Multiaddr;
use std::{convert::TryInto, env, ffi::OsString, sync::Arc};
use tokio::runtime::Handle;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Debug)]
pub struct Args {
    pub tcp_addr: Option<Multiaddr>,
    pub tcp_noise_addr: Option<Multiaddr>,
    pub tcp_muxer_addr: Option<Multiaddr>,
    pub tcp_noise_muxer_addr: Option<Multiaddr>,
    pub msg_lens: Option<Vec<usize>>,
}

fn parse_addr(s: OsString) -> Multiaddr {
    s.into_string()
        .expect("Error: Address should be valid Unicode")
        .try_into()
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
            tcp_muxer_addr: env::var_os("TCP_MUXER_ADDR").map(parse_addr),
            tcp_noise_muxer_addr: env::var_os("TCP_NOISE_MUXER_ADDR").map(parse_addr),
            msg_lens: env::var_os("MSG_LENS").map(parse_msg_lens),
        }
    }
}

/// Build a MemorySocket + Noise transport
pub fn build_memsocket_noise_transport() -> impl Transport<Output = NoiseSocket<MemorySocket>> {
    MemoryTransport::default().and_then(move |socket, origin| async move {
        let noise_config = Arc::new(NoiseConfig::new_random());
        let (_remote_static_key, socket) = noise_config.upgrade_connection(socket, origin).await?;
        Ok(socket)
    })
}

/// Build a MemorySocket + Muxer transport
pub fn build_memsocket_muxer_transport() -> impl Transport<Output = impl StreamMultiplexer> {
    MemoryTransport::default().and_then(Yamux::upgrade_connection)
}

/// Build a MemorySocket + Muxer + Muxer transport
pub fn build_memsocket_dual_muxed_transport() -> impl Transport<Output = impl StreamMultiplexer> {
    MemoryTransport::default()
        .and_then(Yamux::upgrade_connection)
        .and_then(move |socket, origin| {
            async move {
                let substream;
                let (mut listener, mut control) = socket.start().await;
                if origin == ConnectionOrigin::Outbound {
                    // Open outbound substream.
                    substream = control.open_stream().await.unwrap();
                } else {
                    // Wait for inbound client substream.
                    substream = listener.next().await.unwrap().unwrap();
                }
                // Spawn listener to avoid closing the underlying connection.
                tokio::spawn(listener.for_each(|_| future::ready(())));
                Yamux::upgrade_connection(substream, origin).await
            }
        })
}

/// Build a MemorySocket + Noise + Muxer transport
pub fn build_memsocket_noise_muxer_transport() -> impl Transport<Output = impl StreamMultiplexer> {
    MemoryTransport::default()
        .and_then(move |socket, origin| async move {
            let noise_config = Arc::new(NoiseConfig::new_random());
            let (_remote_static_key, socket) =
                noise_config.upgrade_connection(socket, origin).await?;
            Ok(socket)
        })
        .and_then(Yamux::upgrade_connection)
}

/// Build a Tcp + Noise transport
pub fn build_tcp_noise_transport() -> impl Transport<Output = NoiseSocket<TcpSocket>> {
    TcpTransport::default().and_then(move |socket, origin| async move {
        let noise_config = Arc::new(NoiseConfig::new_random());
        let (_remote_static_key, socket) = noise_config.upgrade_connection(socket, origin).await?;
        Ok(socket)
    })
}

/// Build a Tcp + Muxer transport
pub fn build_tcp_muxer_transport() -> impl Transport<Output = impl StreamMultiplexer> {
    TcpTransport::default().and_then(Yamux::upgrade_connection)
}

/// Build a Tcp + Noise + Muxer transport
pub fn build_tcp_noise_muxer_transport() -> impl Transport<Output = impl StreamMultiplexer> {
    TcpTransport::default()
        .and_then(move |socket, origin| async move {
            let noise_config = Arc::new(NoiseConfig::new_random());
            let (_remote_static_key, socket) =
                noise_config.upgrade_connection(socket, origin).await?;
            Ok(socket)
        })
        .and_then(Yamux::upgrade_connection)
}

/// Server side handler for send throughput benchmark when the messages are sent
/// over a simple stream (tcp or in-memory).
pub async fn server_stream_handler<L, I, S, E>(mut server_listener: L)
where
    L: Stream<Item = Result<(I, Multiaddr), E>> + Unpin,
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

/// Server side handler for send throughput benchmark when the messages are sent
/// over a muxer substream.
pub async fn server_muxer_handler<L, I, M, E>(mut server_listener: L)
where
    L: Stream<Item = Result<(I, Multiaddr), E>> + Unpin,
    I: Future<Output = Result<M, E>>,
    M: StreamMultiplexer,
    E: ::std::error::Error,
{
    // Wait for next inbound connection
    while let Some(Ok((f_muxer, _client_addr))) = server_listener.next().await {
        let muxer = f_muxer.await.unwrap();
        let (mut listener, _control) = muxer.start().await;
        // Wait for inbound client substream
        let substream = listener.next().await.unwrap().unwrap();
        let mut stream = Framed::new(IoCompat::new(substream), LengthDelimitedCodec::new());

        // Drain all messages from the client.
        while let Some(_) = stream.next().await {}
        stream.close().await.unwrap();
    }
}

pub fn start_stream_server<T, L, I, S, E>(
    executor: &Handle,
    transport: T,
    listen_addr: Multiaddr,
) -> Multiaddr
where
    T: Transport<Output = S, Error = E, Listener = L, Inbound = I>,
    L: Stream<Item = Result<(I, Multiaddr), E>> + Unpin + Send + 'static,
    I: Future<Output = Result<S, E>> + Send + 'static,
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    E: ::std::error::Error + Send + Sync + 'static,
{
    let (listener, server_addr) = executor.enter(move || transport.listen_on(listen_addr).unwrap());
    executor.spawn(server_stream_handler(listener));
    server_addr
}

pub fn start_muxer_server<T, L, I, M, E>(
    executor: &Handle,
    transport: T,
    listen_addr: Multiaddr,
) -> Multiaddr
where
    T: Transport<Output = M, Error = E, Listener = L, Inbound = I>,
    L: Stream<Item = Result<(I, Multiaddr), E>> + Unpin + Send + 'static,
    I: Future<Output = Result<M, E>> + Send + 'static,
    M: StreamMultiplexer + 'static,
    E: ::std::error::Error + Send + Sync + 'static,
{
    let (listener, server_addr) = executor.enter(move || transport.listen_on(listen_addr).unwrap());
    executor.spawn(server_muxer_handler(listener));
    server_addr
}
