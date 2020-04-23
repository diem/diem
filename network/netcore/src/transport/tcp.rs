// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! TCP Transport
use crate::{compat::IoCompat, transport::Transport};
use futures::{
    future::{self, Future},
    io::{AsyncRead, AsyncWrite},
    ready,
    stream::Stream,
};
use libra_network_address::{NetworkAddress, Protocol};
use std::{
    convert::TryFrom,
    fmt::Debug,
    io,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::net::{TcpListener, TcpStream};

/// Transport to build TCP connections
#[derive(Debug, Clone, Default)]
pub struct TcpTransport {
    /// Size of the recv buffer size to set for opened sockets, or `None` to keep default.
    pub recv_buffer_size: Option<usize>,
    /// Size of the send buffer size to set for opened sockets, or `None` to keep default.
    pub send_buffer_size: Option<usize>,
    /// TTL to set for opened sockets, or `None` to keep default.
    pub ttl: Option<u32>,
    /// Keep alive duration to set for opened sockets, or `None` to keep default.
    #[allow(clippy::option_option)]
    pub keepalive: Option<Option<Duration>>,
    /// `TCP_NODELAY` to set for opened sockets, or `None` to keep default.
    pub nodelay: Option<bool>,
}

impl TcpTransport {
    fn apply_config(&self, stream: &TcpStream) -> ::std::io::Result<()> {
        if let Some(size) = self.recv_buffer_size {
            stream.set_recv_buffer_size(size)?;
        }

        if let Some(size) = self.send_buffer_size {
            stream.set_send_buffer_size(size)?;
        }

        if let Some(ttl) = self.ttl {
            stream.set_ttl(ttl)?;
        }

        if let Some(keepalive) = self.keepalive {
            stream.set_keepalive(keepalive)?;
        }

        if let Some(nodelay) = self.nodelay {
            stream.set_nodelay(nodelay)?;
        }

        Ok(())
    }
}

impl Transport for TcpTransport {
    type Output = TcpSocket;
    type Error = ::std::io::Error;
    type Listener = TcpListenerStream;
    type Inbound = future::Ready<io::Result<TcpSocket>>;
    type Outbound = TcpOutbound;

    fn listen_on(
        &self,
        addr: NetworkAddress,
    ) -> Result<(Self::Listener, NetworkAddress), Self::Error> {
        let socket_addr = multiaddr_to_socketaddr(&addr)?;
        let config = self.clone();
        let listener = ::std::net::TcpListener::bind(&socket_addr)?;
        let local_addr = NetworkAddress::from(listener.local_addr()?);
        let listener = TcpListener::try_from(listener)?;

        Ok((
            TcpListenerStream {
                inner: listener,
                config,
            },
            local_addr,
        ))
    }

    fn dial(&self, addr: NetworkAddress) -> Result<Self::Outbound, Self::Error> {
        let socket_addr = multiaddr_to_string(&addr)?;
        let config = self.clone();
        let f: Pin<Box<dyn Future<Output = io::Result<TcpStream>> + Send + 'static>> =
            Box::pin(TcpStream::connect(socket_addr));
        Ok(TcpOutbound { inner: f, config })
    }
}

#[must_use = "streams do nothing unless polled"]
pub struct TcpListenerStream {
    inner: TcpListener,
    config: TcpTransport,
}

impl Stream for TcpListenerStream {
    type Item = io::Result<(future::Ready<io::Result<TcpSocket>>, NetworkAddress)>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner.incoming()).poll_next(context) {
            Poll::Ready(Some(Ok(socket))) => {
                if let Err(e) = self.config.apply_config(&socket) {
                    return Poll::Ready(Some(Err(e)));
                }
                let dialer_addr = match socket.peer_addr() {
                    Ok(addr) => NetworkAddress::from(addr),
                    Err(e) => return Poll::Ready(Some(Err(e))),
                };
                Poll::Ready(Some(Ok((
                    future::ready(Ok(TcpSocket::new(socket))),
                    dialer_addr,
                ))))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[must_use = "futures do nothing unless polled"]
pub struct TcpOutbound {
    inner: Pin<Box<dyn Future<Output = io::Result<TcpStream>> + Send + 'static>>,
    config: TcpTransport,
}

impl Future for TcpOutbound {
    type Output = io::Result<TcpSocket>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let socket = ready!(Pin::new(&mut self.inner).poll(context))?;
        self.config.apply_config(&socket)?;
        Poll::Ready(Ok(TcpSocket::new(socket)))
    }
}

/// A wrapper around a tokio TcpStream
///
/// In order to properly implement the AsyncRead/AsyncWrite traits we need to wrap a TcpStream to
/// ensure that the "close" method actually closes the write half of the TcpStream.  This is
/// because the "close" method on a TcpStream just performs a no-op instead of actually shutting
/// down the write side of the TcpStream.
//TODO Probably should add some tests for this
#[derive(Debug)]
pub struct TcpSocket {
    inner: IoCompat<TcpStream>,
}

impl TcpSocket {
    fn new(socket: TcpStream) -> Self {
        Self {
            inner: IoCompat::new(socket),
        }
    }
}

impl AsyncRead for TcpSocket {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_read(context, buf)
    }
}

impl AsyncWrite for TcpSocket {
    fn poll_write(
        mut self: Pin<&mut Self>,
        context: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(context, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(context)
    }

    fn poll_close(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_close(context)
    }
}

fn multiaddr_to_socketaddr(addr: &NetworkAddress) -> ::std::io::Result<SocketAddr> {
    let mut iter = addr.as_slice().iter();
    let proto1 = iter.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid NetworkAddress '{:?}'", addr),
        )
    })?;
    let proto2 = iter.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid NetworkAddress '{:?}'", addr),
        )
    })?;

    if iter.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid NetworkAddress '{:?}'", addr),
        ));
    }

    match (proto1, proto2) {
        (Protocol::Ip4(ip), Protocol::Tcp(port)) => Ok(SocketAddr::new((*ip).into(), *port)),
        (Protocol::Ip6(ip), Protocol::Tcp(port)) => Ok(SocketAddr::new((*ip).into(), *port)),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid NetworkAddress '{:?}'", addr),
        )),
    }
}

fn multiaddr_to_string(addr: &NetworkAddress) -> ::std::io::Result<String> {
    let mut iter = addr.as_slice().iter();
    let proto1 = iter.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid NetworkAddress '{:?}'", addr),
        )
    })?;
    let proto2 = iter.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid NetworkAddress '{:?}'", addr),
        )
    })?;

    if iter.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid NetworkAddress '{:?}'", addr),
        ));
    }

    match (proto1, proto2) {
        (Protocol::Ip4(ip), Protocol::Tcp(port)) => Ok(format!("{}:{}", ip, port)),
        (Protocol::Ip6(ip), Protocol::Tcp(port)) => Ok(format!("{}:{}", ip, port)),
        (Protocol::Dns4(host), Protocol::Tcp(port))
        | (Protocol::Dns6(host), Protocol::Tcp(port)) => Ok(format!("{}:{}", host, port)),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid NetworkAddress '{:?}'", addr),
        )),
    }
}

#[cfg(test)]
mod test {
    use crate::transport::{tcp::TcpTransport, ConnectionOrigin, Transport, TransportExt};
    use futures::{
        future::{join, FutureExt},
        io::{AsyncReadExt, AsyncWriteExt},
        stream::StreamExt,
    };

    #[tokio::test]
    async fn simple_listen_and_dial() -> Result<(), ::std::io::Error> {
        let t = TcpTransport::default().and_then(|mut out, _addr, origin| async move {
            match origin {
                ConnectionOrigin::Inbound => {
                    out.write_all(b"Earth").await?;
                    let mut buf = [0; 3];
                    out.read_exact(&mut buf).await?;
                    assert_eq!(&buf, b"Air");
                }
                ConnectionOrigin::Outbound => {
                    let mut buf = [0; 5];
                    out.read_exact(&mut buf).await?;
                    assert_eq!(&buf, b"Earth");
                    out.write_all(b"Air").await?;
                }
            }
            Ok(())
        });

        let (listener, addr) = t.listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())?;

        let dial = t.dial(addr)?;
        let listener = listener.into_future().then(|(maybe_result, _stream)| {
            let (incoming, _addr) = maybe_result.unwrap().unwrap();
            incoming.map(Result::unwrap)
        });

        let (outgoing, _incoming) = join(dial, listener).await;
        assert!(outgoing.is_ok());
        Ok(())
    }

    #[test]
    fn unsupported_multiaddrs() {
        let t = TcpTransport::default();

        let result = t.listen_on("/memory/0".parse().unwrap());
        assert!(result.is_err());

        let result = t.dial("/memory/22".parse().unwrap());
        assert!(result.is_err());
    }
}
