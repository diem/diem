// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Low-level module for establishing connections with peers
//!
//! The main component of this module is the [`Transport`] trait, which provides an interface for
//! establishing both inbound and outbound connections with remote peers. The [`TransportExt`]
//! trait contains a variety of combinators for modifying a transport allowing composability and
//! layering of additional transports or protocols.
//!
//! [`Transport`]: crate::transport::Transport
//! [`TransportExt`]: crate::transport::TransportExt

use futures::{future::Future, stream::Stream};
use parity_multiaddr::Multiaddr;
use std::time::Duration;

pub mod and_then;
pub mod boxed;
pub mod memory;
pub mod tcp;
pub mod timeout;

/// Origin of how a Connection was established.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionOrigin {
    /// `Inbound` indicates that we are the listener for this connection.
    Inbound,
    /// `Outbound` indicates that we are the dialer for this connection.
    Outbound,
}

/// A Transport is responsible for establishing connections with remote Peers.
///
/// Connections are established either by [listening](Transport::listen_on)
/// or [dialing](Transport::dial) on a [`Transport`]. A peer that
/// obtains a connection by listening is often referred to as the *listener* and the
/// peer that initiated the connection through dialing as the *dialer*.
///
/// Additional protocols can be layered on top of the connections established
/// by a [`Transport`] through utilizing the combinators in the [`TransportExt`] trait.
pub trait Transport {
    /// The result of establishing a connection.
    ///
    /// Generally this would include a socket-like streams which allows for sending and receiving
    /// of data through the connection.
    type Output;

    /// The Error type of errors which can happen while establishing a connection.
    type Error: ::std::error::Error + Send + Sync + 'static;

    /// A stream of [`Inbound`](Transport::Inbound) connections and the address of the dialer.
    ///
    /// An item should be produced whenever a connection is received at the lowest level of the
    /// transport stack. Each item is an [`Inbound`](Transport::Inbound) future
    /// that resolves to an [`Output`](Transport::Output) value once all protocol upgrades
    /// have been applied.
    type Listener: Stream<Item = Result<(Self::Inbound, Multiaddr), Self::Error>> + Send + Unpin;

    /// A pending [`Output`](Transport::Output) for an inbound connection,
    /// obtained from the [`Listener`](Transport::Listener) stream.
    ///
    /// After a connection has been accepted by the transport, it may need to go through
    /// asynchronous post-processing (i.e. protocol upgrade negotiations). Such
    /// post-processing should not block the `Listener` from producing the next
    /// connection, hence further connection setup proceeds asynchronously.
    /// Once a `Inbound` future resolves it yields the [`Output`](Transport::Output)
    /// of the connection setup process.
    type Inbound: Future<Output = Result<Self::Output, Self::Error>> + Send;

    /// A pending [`Output`](Transport::Output) for an outbound connection,
    /// obtained from [dialing](Transport::dial) stream.
    type Outbound: Future<Output = Result<Self::Output, Self::Error>> + Send;

    /// Listens on the given [`Multiaddr`], returning a stream of incoming connections.
    ///
    /// The returned [`Multiaddr`] is the actual listening address, this is done to take into
    /// account OS-assigned port numbers (e.g. listening on port 0).
    fn listen_on(&self, addr: Multiaddr) -> Result<(Self::Listener, Multiaddr), Self::Error>
    where
        Self: Sized;

    /// Dials the given [`Multiaddr`], returning a future for a pending outbound connection.
    fn dial(&self, addr: Multiaddr) -> Result<Self::Outbound, Self::Error>
    where
        Self: Sized;
}

impl<T: ?Sized> TransportExt for T where T: Transport {}

/// An extension trait for [`Transport`]s that provides a variety of convenient
/// combinators.
///
/// Additional protocols or functionality can be layered on top of an existing
/// [`Transport`] by using this extension trait. For example, one might want to
/// take a raw connection and upgrade it to a secure transport followed by a
/// stream multiplexer by chaining calls to [`and_then`](TransportExt::and_then).
/// Each method yields a new [`Transport`] whose connection setup incorporates
/// all earlier upgrades followed by the new upgrade, i.e. the order of the
/// upgrades is significant.
pub trait TransportExt: Transport {
    /// Turns a [`Transport`] into an abstract boxed transport.
    fn boxed(self) -> boxed::BoxedTransport<Self::Output, Self::Error>
    where
        Self: Sized + Send + 'static,
        Self::Listener: Send + 'static,
        Self::Inbound: Send + 'static,
        Self::Outbound: Send + 'static,
    {
        boxed::BoxedTransport::new(self)
    }

    /// Applies a function producing an asynchronous result to every connection
    /// created by this transport.
    ///
    /// This function can be used for ad-hoc protocol upgrades on a transport
    /// or for processing or adapting the output of an earlier upgrade.  The
    /// provided function must take as input the output from the existing
    /// transport and a [`ConnectionOrigin`] which can be used to identify the
    /// origin of the connection (inbound vs outbound).
    fn and_then<F, Fut, O>(self, f: F) -> and_then::AndThen<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Output, Multiaddr, ConnectionOrigin) -> Fut + Clone,
        // Pin the error types to be the same for now
        // TODO don't require the error types to be the same
        Fut: Future<Output = Result<O, Self::Error>>,
    {
        and_then::AndThen::new(self, f)
    }

    /// Wraps a [`Transport`] with a timeout to the
    /// [Inbound](Transport::Inbound) and [Outbound](Transport::Outbound)
    /// connection futures.
    ///
    /// Note: The timeout does not apply to the [Listener](Transport::Listener) stream.
    fn with_timeout(self, timeout: Duration) -> timeout::TimeoutTransport<Self>
    where
        Self: Sized,
    {
        timeout::TimeoutTransport::new(self, timeout)
    }
}
