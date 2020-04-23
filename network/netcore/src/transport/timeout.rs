// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Timeout Transport

use crate::transport::Transport;
use futures::{future::Future, stream::Stream};
use libra_network_address::NetworkAddress;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::{delay_for, Delay};

/// A [`TimeoutTransport`] is a transport which wraps another transport with a timeout on all
/// inbound and outbound connection setup.
///
/// Note: The [Listener](Transport::Listener) stream is not subject to the provided timeout.
#[derive(Debug)]
pub struct TimeoutTransport<T> {
    transport: T,
    timeout: Duration,
}

impl<T> TimeoutTransport<T> {
    /// Wraps around a [`Transport`] and adds timeouts to all inbound and outbound connections
    /// created by it.
    pub(crate) fn new(transport: T, timeout: Duration) -> Self {
        Self { transport, timeout }
    }
}

impl<T> Transport for TimeoutTransport<T>
where
    T: Transport,
    T::Error: 'static,
{
    type Output = T::Output;
    type Error = TimeoutTransportError<T::Error>;
    type Listener = TimeoutStream<T::Listener>;
    type Inbound = TimeoutFuture<T::Inbound>;
    type Outbound = TimeoutFuture<T::Outbound>;

    fn listen_on(
        &self,
        addr: NetworkAddress,
    ) -> Result<(Self::Listener, NetworkAddress), Self::Error> {
        let (listener, addr) = self.transport.listen_on(addr)?;
        let listener = TimeoutStream::new(listener, self.timeout);

        Ok((listener, addr))
    }

    fn dial(&self, addr: NetworkAddress) -> Result<Self::Outbound, Self::Error> {
        let fut = self.transport.dial(addr)?;

        Ok(TimeoutFuture::new(fut, self.timeout))
    }
}

/// Listener stream returned by [listen_on](Transport::listen_on) on a TimeoutTransport.
#[pin_project]
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct TimeoutStream<St> {
    #[pin]
    inner: St,
    timeout: Duration,
}

impl<St> TimeoutStream<St>
where
    St: Stream,
{
    fn new(stream: St, timeout: Duration) -> Self {
        Self {
            inner: stream,
            timeout,
        }
    }
}

impl<St, Fut, O, E> Stream for TimeoutStream<St>
where
    St: Stream<Item = Result<(Fut, NetworkAddress), E>>,
    Fut: Future<Output = Result<O, E>>,
    E: ::std::error::Error,
{
    type Item = Result<(TimeoutFuture<Fut>, NetworkAddress), TimeoutTransportError<E>>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        match self.as_mut().project().inner.poll_next(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Err(TimeoutTransportError::TransportError(e))))
            }
            Poll::Ready(Some(Ok((fut, addr)))) => {
                let fut = TimeoutFuture::new(fut, self.timeout);
                Poll::Ready(Some(Ok((fut, addr))))
            }
        }
    }
}

/// Future which wraps an inner Future with a timeout.
#[pin_project]
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct TimeoutFuture<F> {
    #[pin]
    future: F,
    timeout: Delay,
}

impl<F> TimeoutFuture<F>
where
    F: Future,
{
    fn new(future: F, timeout: Duration) -> Self {
        Self {
            future,
            timeout: delay_for(timeout),
        }
    }
}

impl<F, O, E> Future for TimeoutFuture<F>
where
    F: Future<Output = Result<O, E>>,
    E: ::std::error::Error,
{
    type Output = Result<O, TimeoutTransportError<E>>;

    fn poll(mut self: Pin<&mut Self>, mut context: &mut Context) -> Poll<Self::Output> {
        // check to see if we've overshot the timeout first
        match Pin::new(self.as_mut().project().timeout).poll(&mut context) {
            Poll::Pending => {}
            Poll::Ready(()) => return Poll::Ready(Err(TimeoutTransportError::Timeout)),
        };

        // All good, try polling the inner future now
        match self.as_mut().project().future.poll(&mut context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(TimeoutTransportError::TransportError(e))),
            Poll::Ready(Ok(output)) => Poll::Ready(Ok(output)),
        }
    }
}

#[derive(Debug)]
pub enum TimeoutTransportError<E> {
    Timeout,
    TimerError(::tokio::time::Error),
    TransportError(E),
}

impl<E> ::std::convert::From<E> for TimeoutTransportError<E> {
    fn from(error: E) -> Self {
        TimeoutTransportError::TransportError(error)
    }
}

impl<E> ::std::fmt::Display for TimeoutTransportError<E>
where
    E: ::std::fmt::Display,
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            TimeoutTransportError::Timeout => write!(f, "Timeout has been reached"),
            TimeoutTransportError::TimerError(err) => write!(f, "Error in the timer: '{}'", err),
            TimeoutTransportError::TransportError(err) => write!(f, "{}", err),
        }
    }
}

impl<E> ::std::error::Error for TimeoutTransportError<E>
where
    E: ::std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
        match self {
            TimeoutTransportError::Timeout => None,
            TimeoutTransportError::TimerError(err) => Some(err),
            TimeoutTransportError::TransportError(err) => Some(err),
        }
    }
}
