// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Module responsible for defining and implementing Stream Multiplexing
//!
//! The main component of this module is the [`StreamMultiplexer`] trait, which
//! provides an interface for multiplexing multiple [`AsyncRead`]/[`AsyncWrite`] substreams over a
//! single underlying [`AsyncRead`]/[`AsyncWrite`] stream. [`Yamux`], an implementation of this
//! trait over [`TcpStream`], is also provided.
//!
//! [`StreamMultiplexer`]: crate::multiplexing::StreamMultiplexer
//! [`AsyncRead`]: futures::io::AsyncRead
//! [`AsyncWrite`]: futures::io::AsyncWrite
//! [`TcpStream`]: tokio::net::tcp::TcpStream
//! [`Yamux`]: crate::multiplexing::yamux::Yamux

use async_trait::async_trait;
use futures::{
    io::{AsyncRead, AsyncWrite},
    stream::Stream,
};
use std::{fmt::Debug, io};

pub mod yamux;

#[async_trait]
pub trait Control {
    /// The type of substreams opened by this Multiplexer.
    ///
    /// Must implement both AsyncRead and AsyncWrite.
    type Substream: AsyncRead + AsyncWrite + Send + Debug + Unpin;

    /// Requests that a new Substream be opened on the underlying connection.
    async fn open_stream(&mut self) -> io::Result<Self::Substream>;

    /// Close and shutdown this connection.
    ///
    /// After the returned future has resolved this multiplexer will be shutdown.  All subsequent
    /// reads or writes to any still existing handles to substreams opened through this multiplexer
    /// must return EOF (in the case of a read), or an error.
    async fn close(&mut self) -> io::Result<()>;
}

/// A StreamMultiplexer is responsible for multiplexing multiple [`AsyncRead`]/[`AsyncWrite`]
/// streams over a single underlying [`AsyncRead`]/[`AsyncWrite`] stream.
///
/// New substreams are opened either by [listening](StreamMultiplexer::listen_for_inbound) for
/// inbound substreams opened by the remote side or by [opening](StreamMultiplexer::open_outbound)
/// and outbound substream locally.
#[async_trait]
pub trait StreamMultiplexer: Debug + Send + Sync {
    /// The type of substreams opened by this Multiplexer.
    ///
    /// Must implement both AsyncRead and AsyncWrite.
    type Substream: AsyncRead + AsyncWrite + Send + Debug + Unpin;

    /// Controller for the connection. This can be used for opening outbound substreams or for
    /// closing the connection.
    type Control: Control<Substream = Self::Substream> + Clone + Send + Sync;

    /// A stream of new [`Substreams`](StreamMultiplexer::Substream) opened by the remote side.
    type Listener: Stream<Item = io::Result<<Self::Control as Control>::Substream>> + Send + Unpin;

    async fn start(self) -> (Self::Listener, Self::Control);
}
