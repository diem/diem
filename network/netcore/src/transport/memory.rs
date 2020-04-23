// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transport::Transport;
use futures::{future, stream::Stream};
use libra_network_address::{NetworkAddress, Protocol};
use memsocket::{MemoryListener, MemorySocket};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

/// Transport to build in-memory connections
#[derive(Debug, Default)]
pub struct MemoryTransport;

impl Transport for MemoryTransport {
    type Output = MemorySocket;
    type Error = io::Error;
    type Listener = Listener;
    type Inbound = future::Ready<Result<Self::Output, Self::Error>>;
    type Outbound = future::Ready<Result<Self::Output, Self::Error>>;

    fn listen_on(
        &self,
        addr: NetworkAddress,
    ) -> Result<(Self::Listener, NetworkAddress), Self::Error> {
        let port = parse_addr(&addr)?;
        let listener = MemoryListener::bind(port)?;
        let actual_port = listener.local_addr();
        let actual_addr = NetworkAddress::from(Protocol::Memory(actual_port));

        Ok((Listener { inner: listener }, actual_addr))
    }

    fn dial(&self, addr: NetworkAddress) -> Result<Self::Outbound, Self::Error> {
        let port = parse_addr(&addr)?;
        let socket = MemorySocket::connect(port)?;
        Ok(future::ready(Ok(socket)))
    }
}

fn parse_addr(addr: &NetworkAddress) -> io::Result<u16> {
    let mut iter = addr.as_slice().iter();

    let port = if let Some(Protocol::Memory(port)) = iter.next() {
        *port
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid NetworkAddress '{:?}'", addr),
        ));
    };

    if iter.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid NetworkAddress '{:?}'", addr),
        ));
    }

    Ok(port)
}

#[must_use = "streams do nothing unless polled"]
#[derive(Debug)]
pub struct Listener {
    inner: MemoryListener,
}

impl Stream for Listener {
    type Item = io::Result<(future::Ready<io::Result<MemorySocket>>, NetworkAddress)>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        let mut incoming = self.inner.incoming();
        match Pin::new(&mut incoming).poll_next(context) {
            Poll::Ready(Some(Ok(socket))) => {
                // Dialer addresses for MemoryTransport don't make a ton of sense,
                // so use port 0 to ensure they aren't used as an address to dial.
                let dialer_addr = NetworkAddress::from(Protocol::Memory(0));
                Poll::Ready(Some(Ok((future::ready(Ok(socket)), dialer_addr))))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::transport::{memory::MemoryTransport, Transport};
    use futures::{
        executor::block_on,
        future::join,
        io::{AsyncReadExt, AsyncWriteExt},
        stream::StreamExt,
    };

    #[test]
    fn simple_listen_and_dial() -> Result<(), ::std::io::Error> {
        let t = MemoryTransport::default();

        let (listener, addr) = t.listen_on("/memory/0".parse().unwrap())?;

        let listener = async move {
            let (item, _listener) = listener.into_future().await;
            let (inbound, _addr) = item.unwrap().unwrap();
            let mut socket = inbound.await.unwrap();

            let mut buf = Vec::new();
            socket.read_to_end(&mut buf).await.unwrap();
            assert_eq!(buf, b"hello world");
        };
        let outbound = t.dial(addr)?;

        let dialer = async move {
            let mut socket = outbound.await.unwrap();
            socket.write_all(b"hello world").await.unwrap();
            socket.flush().await.unwrap();
        };

        block_on(join(dialer, listener));
        Ok(())
    }

    #[test]
    fn unsupported_multiaddrs() {
        let t = MemoryTransport::default();

        let result = t.listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap());
        assert!(result.is_err());

        let result = t.dial("/ip4/127.0.0.1/tcp/22".parse().unwrap());
        assert!(result.is_err());
    }
}
