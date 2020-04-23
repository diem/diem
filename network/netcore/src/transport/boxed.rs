// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transport::Transport;
use futures::{
    future::{Future, FutureExt},
    stream::{Stream, StreamExt},
};
use libra_network_address::NetworkAddress;
use std::pin::Pin;

pub type Listener<O, E> =
    Pin<Box<dyn Stream<Item = Result<(Inbound<O, E>, NetworkAddress), E>> + Send>>;
pub type Inbound<O, E> = Pin<Box<dyn Future<Output = Result<O, E>> + Send>>;
pub type Outbound<O, E> = Pin<Box<dyn Future<Output = Result<O, E>> + Send>>;

trait AbstractBoxedTransport<O, E> {
    fn listen_on(&self, addr: NetworkAddress) -> Result<(Listener<O, E>, NetworkAddress), E>;
    fn dial(&self, addr: NetworkAddress) -> Result<Outbound<O, E>, E>;
}

impl<T, O, E> AbstractBoxedTransport<O, E> for T
where
    T: Transport<Output = O, Error = E> + Send + 'static,
    T::Listener: Send + 'static,
    T::Inbound: Send + 'static,
    T::Outbound: Send + 'static,
    E: ::std::error::Error + Send + Sync + 'static,
{
    fn listen_on(&self, addr: NetworkAddress) -> Result<(Listener<O, E>, NetworkAddress), E> {
        let (listener, addr) = self.listen_on(addr)?;
        let listener = listener
            .map(|result| result.map(|(incoming, addr)| (incoming.boxed() as Inbound<O, E>, addr)));
        Ok((listener.boxed() as Listener<O, E>, addr))
    }

    fn dial(&self, addr: NetworkAddress) -> Result<Outbound<O, E>, E> {
        let outgoing = self.dial(addr)?;
        Ok(outgoing.boxed() as Outbound<O, E>)
    }
}

/// See the [boxed](crate::transport::TransportExt::boxed) method for more information.
pub struct BoxedTransport<O, E> {
    inner: Box<dyn AbstractBoxedTransport<O, E> + Send + 'static>,
}

impl<O, E> BoxedTransport<O, E>
where
    E: ::std::error::Error + Send + Sync + 'static,
{
    pub(crate) fn new<T>(transport: T) -> Self
    where
        T: Transport<Output = O, Error = E> + Send + 'static,
        T::Listener: Send + 'static,
        T::Inbound: Send + 'static,
        T::Outbound: Send + 'static,
    {
        Self {
            inner: Box::new(transport) as Box<_>,
        }
    }
}

impl<O, E> Transport for BoxedTransport<O, E>
where
    E: ::std::error::Error + Send + Sync + 'static,
{
    type Output = O;
    type Error = E;
    type Listener = Listener<O, E>;
    type Inbound = Inbound<O, E>;
    type Outbound = Outbound<O, E>;

    fn listen_on(
        &self,
        addr: NetworkAddress,
    ) -> Result<(Self::Listener, NetworkAddress), Self::Error> {
        self.inner.listen_on(addr)
    }

    fn dial(&self, addr: NetworkAddress) -> Result<Self::Outbound, Self::Error> {
        self.inner.dial(addr)
    }
}
