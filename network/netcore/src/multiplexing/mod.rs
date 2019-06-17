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

use futures::{
    future::Future,
    io::{AsyncRead, AsyncWrite},
    stream::Stream,
};
use std::{fmt::Debug, io};

pub mod yamux;

/// A StreamMultiplexer is responsible for multiplexing multiple [`AsyncRead`]/[`AsyncWrite`]
/// streams over a single underlying [`AsyncRead`]/[`AsyncWrite`] stream.
///
/// New substreams are opened either by [listening](StreamMultiplexer::listen_for_inbound) for
/// inbound substreams opened by the remote side or by [opening](StreamMultiplexer::open_outbound)
/// and outbound substream locally.
pub trait StreamMultiplexer: Debug + Send + Sync {
    /// The type of substreams opened by this Multiplexer.
    ///
    /// Must implement both AsyncRead and AsyncWrite.
    type Substream: AsyncRead + AsyncWrite + Send + Debug + Unpin;

    /// A stream of new [`Substreams`](StreamMultiplexer::Substream) opened by the remote side.
    type Listener: Stream<Item = io::Result<Self::Substream>> + Send + Unpin;

    /// A pending [`Substream`](StreamMultiplexer::Substream) to be opened on the underlying
    /// connection, obtained from [requesting a new substream](StreamMultiplexer::open_outbound).
    type Outbound: Future<Output = io::Result<Self::Substream>> + Send;

    /// A pending request to shut down the underlying connection, obtained from
    /// [closing](StreamMultiplexer::close).
    type Close: Future<Output = io::Result<()>> + Send;

    /// Returns a stream of new Substreams opened by the remote side.
    fn listen_for_inbound(&self) -> Self::Listener;

    /// Requests that a new Substream be opened.
    fn open_outbound(&self) -> Self::Outbound;

    /// Close and shutdown this [`StreamMultiplexer`].
    ///
    /// After the returned future has resolved this multiplexer will be shutdown.  All subsequent
    /// reads or writes to any still existing handles to substreams opened through this multiplexer
    /// must return EOF (in the case of a read), or an error.
    fn close(&self) -> Self::Close;
}
