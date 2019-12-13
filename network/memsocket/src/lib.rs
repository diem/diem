// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytes::{buf::BufExt, Buf, Bytes};
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    io::{AsyncRead, AsyncWrite, Error, ErrorKind, Result},
    ready,
    stream::{FusedStream, Stream},
    task::{Context, Poll},
};
use once_cell::sync::Lazy;
use std::{collections::HashMap, num::NonZeroU16, pin::Pin, sync::Mutex};

static SWITCHBOARD: Lazy<Mutex<SwitchBoard>> =
    Lazy::new(|| Mutex::new(SwitchBoard(HashMap::default(), 1)));

struct SwitchBoard(HashMap<NonZeroU16, UnboundedSender<MemorySocket>>, u16);

/// An in-memory socket server, listening for connections.
///
/// After creating a `MemoryListener` by [`bind`]ing it to a socket address, it listens
/// for incoming connections. These can be accepted by awaiting elements from the
/// async stream of incoming connections, [`incoming`][`MemoryListener::incoming`].
///
/// The socket will be closed when the value is dropped.
///
/// [`bind`]: #method.bind
/// [`MemoryListener::incoming`]: #method.incoming
///
/// # Examples
///
/// ```rust,no_run
/// use std::io::Result;
///
/// use memsocket::{MemoryListener, MemorySocket};
/// use futures::prelude::*;
///
/// async fn write_stormlight(mut stream: MemorySocket) -> Result<()> {
///     let msg = b"The most important step a person can take is always the next one.";
///     stream.write_all(msg).await?;
///     stream.flush().await
/// }
///
/// async fn listen() -> Result<()> {
///     let mut listener = MemoryListener::bind(16)?;
///     let mut incoming = listener.incoming();
///
///     // accept connections and process them serially
///     while let Some(stream) = incoming.next().await {
///         write_stormlight(stream?).await?;
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct MemoryListener {
    incoming: UnboundedReceiver<MemorySocket>,
    port: NonZeroU16,
}

impl Drop for MemoryListener {
    fn drop(&mut self) {
        let mut switchboard = (&*SWITCHBOARD).lock().unwrap();
        // Remove the Sending side of the channel in the switchboard when
        // MemoryListener is dropped
        switchboard.0.remove(&self.port);
    }
}

impl MemoryListener {
    /// Creates a new `MemoryListener` which will be bound to the specified
    /// port.
    ///
    /// The returned listener is ready for accepting connections.
    ///
    /// Binding with a port number of 0 will request that a port be assigned
    /// to this listener. The port allocated can be queried via the
    /// [`local_addr`] method.
    ///
    /// # Examples
    /// Create a MemoryListener bound to port 16:
    ///
    /// ```rust,no_run
    /// use memsocket::MemoryListener;
    ///
    /// # fn main () -> ::std::io::Result<()> {
    /// let listener = MemoryListener::bind(16)?;
    /// # Ok(())}
    /// ```
    ///
    /// [`local_addr`]: #method.local_addr
    pub fn bind(port: u16) -> Result<Self> {
        let mut switchboard = (&*SWITCHBOARD).lock().unwrap();

        // Get the port we should bind to.  If 0 was given, use a random port
        let port = if let Some(port) = NonZeroU16::new(port) {
            if switchboard.0.contains_key(&port) {
                return Err(ErrorKind::AddrInUse.into());
            }
            port
        } else {
            loop {
                let port = NonZeroU16::new(switchboard.1).unwrap_or_else(|| unreachable!());

                // The switchboard is full and all ports are in use
                if switchboard.0.len() == (std::u16::MAX - 1) as usize {
                    return Err(ErrorKind::AddrInUse.into());
                }

                // Instead of overflowing to 0, resume searching at port 1 since port 0 isn't a
                // valid port to bind to.
                if switchboard.1 == std::u16::MAX {
                    switchboard.1 = 1;
                } else {
                    switchboard.1 += 1;
                }

                if !switchboard.0.contains_key(&port) {
                    break port;
                }
            }
        };

        let (sender, receiver) = mpsc::unbounded();
        switchboard.0.insert(port, sender);

        Ok(Self {
            incoming: receiver,
            port,
        })
    }

    /// Returns the local address that this listener is bound to.
    ///
    /// This can be useful, for example, when binding to port 0 to figure out
    /// which port was actually bound.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use memsocket::MemoryListener;
    ///
    /// # fn main () -> ::std::io::Result<()> {
    /// let listener = MemoryListener::bind(16)?;
    ///
    /// assert_eq!(listener.local_addr(), 16);
    /// # Ok(())}
    /// ```
    pub fn local_addr(&self) -> u16 {
        self.port.get()
    }

    /// Consumes this listener, returning a stream of the sockets this listener
    /// accepts.
    ///
    /// This method returns an implementation of the `Stream` trait which
    /// resolves to the sockets the are accepted on this listener.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use futures::prelude::*;
    /// use memsocket::MemoryListener;
    ///
    /// # async fn work () -> ::std::io::Result<()> {
    /// let mut listener = MemoryListener::bind(16)?;
    /// let mut incoming = listener.incoming();
    ///
    /// // accept connections and process them serially
    /// while let Some(stream) = incoming.next().await {
    ///     match stream {
    ///         Ok(stream) => {
    ///             println!("new connection!");
    ///         },
    ///         Err(e) => { /* connection failed */ }
    ///     }
    /// }
    /// # Ok(())}
    /// ```
    pub fn incoming(&mut self) -> Incoming<'_> {
        Incoming { inner: self }
    }

    fn poll_accept(&mut self, context: &mut Context) -> Poll<Result<MemorySocket>> {
        match Pin::new(&mut self.incoming).poll_next(context) {
            Poll::Ready(Some(socket)) => Poll::Ready(Ok(socket)),
            Poll::Ready(None) => {
                let err = Error::new(ErrorKind::Other, "MemoryListener unknown error");
                Poll::Ready(Err(err))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream returned by the `MemoryListener::incoming` function representing the
/// stream of sockets received from a listener.
#[must_use = "streams do nothing unless polled"]
#[derive(Debug)]
pub struct Incoming<'a> {
    inner: &'a mut MemoryListener,
}

impl<'a> Stream for Incoming<'a> {
    type Item = Result<MemorySocket>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        let socket = ready!(self.inner.poll_accept(context)?);
        Poll::Ready(Some(Ok(socket)))
    }
}

/// An in-memory stream between two local sockets.
///
/// A `MemorySocket` can either be created by connecting to an endpoint, via the
/// [`connect`] method, or by [accepting] a connection from a [listener].
/// It can be read or written to using the `AsyncRead`, `AsyncWrite`, and related
/// extension traits in `futures::io`.
///
/// # Examples
///
/// ```rust, no_run
/// use futures::prelude::*;
/// use memsocket::MemorySocket;
///
/// # async fn run() -> ::std::io::Result<()> {
/// let (mut socket_a, mut socket_b) = MemorySocket::new_pair();
///
/// socket_a.write_all(b"stormlight").await?;
/// socket_a.flush().await?;
///
/// let mut buf = [0; 10];
/// socket_b.read_exact(&mut buf).await?;
/// assert_eq!(&buf, b"stormlight");
///
/// # Ok(())}
/// ```
///
/// [`connect`]: struct.MemorySocket.html#method.connect
/// [accepting]: struct.MemoryListener.html#method.accept
/// [listener]: struct.MemoryListener.html
#[derive(Debug)]
pub struct MemorySocket {
    incoming: UnboundedReceiver<Bytes>,
    outgoing: UnboundedSender<Bytes>,
    current_buffer: Option<Bytes>,
    seen_eof: bool,
}

impl MemorySocket {
    /// Construct both sides of an in-memory socket.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use memsocket::MemorySocket;
    ///
    /// let (socket_a, socket_b) = MemorySocket::new_pair();
    /// ```
    pub fn new_pair() -> (Self, Self) {
        let (a_tx, a_rx) = mpsc::unbounded();
        let (b_tx, b_rx) = mpsc::unbounded();
        let a = Self {
            incoming: a_rx,
            outgoing: b_tx,
            current_buffer: None,
            seen_eof: false,
        };
        let b = Self {
            incoming: b_rx,
            outgoing: a_tx,
            current_buffer: None,
            seen_eof: false,
        };

        (a, b)
    }

    /// Create a new in-memory Socket connected to the specified port.
    ///
    /// This function will create a new MemorySocket socket and attempt to connect it to
    /// the `port` provided.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use memsocket::MemorySocket;
    ///
    /// # fn main () -> ::std::io::Result<()> {
    /// let socket = MemorySocket::connect(16)?;
    /// # Ok(())}
    /// ```
    pub fn connect(port: u16) -> Result<MemorySocket> {
        let mut switchboard = (&*SWITCHBOARD).lock().unwrap();

        // Find port to connect to
        let port = NonZeroU16::new(port).ok_or_else(|| ErrorKind::AddrNotAvailable)?;

        let sender = switchboard
            .0
            .get_mut(&port)
            .ok_or_else(|| ErrorKind::AddrNotAvailable)?;

        let (socket_a, socket_b) = Self::new_pair();
        // Send the socket to the listener
        if let Err(e) = sender.unbounded_send(socket_a) {
            if e.is_disconnected() {
                return Err(ErrorKind::AddrNotAvailable.into());
            }

            unreachable!();
        }

        Ok(socket_b)
    }
}

impl AsyncRead for MemorySocket {
    /// Attempt to read from the `AsyncRead` into `buf`.
    fn poll_read(
        mut self: Pin<&mut Self>,
        mut context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        if self.incoming.is_terminated() {
            if self.seen_eof {
                return Poll::Ready(Err(ErrorKind::UnexpectedEof.into()));
            } else {
                self.seen_eof = true;
                return Poll::Ready(Ok(0));
            }
        }

        let mut bytes_read = 0;

        loop {
            // If we're already filled up the buffer then we can return
            if bytes_read == buf.len() {
                return Poll::Ready(Ok(bytes_read));
            }

            match self.current_buffer {
                // We have data to copy to buf
                Some(ref mut current_buffer) if current_buffer.has_remaining() => {
                    let bytes_to_read =
                        ::std::cmp::min(buf.len() - bytes_read, current_buffer.remaining());
                    debug_assert!(bytes_to_read > 0);

                    current_buffer
                        .take(bytes_to_read)
                        .copy_to_slice(&mut buf[bytes_read..(bytes_read + bytes_to_read)]);
                    bytes_read += bytes_to_read;
                }

                // Either we've exhausted our current buffer or don't have one
                _ => {
                    self.current_buffer = {
                        match Pin::new(&mut self.incoming).poll_next(&mut context) {
                            Poll::Pending => {
                                // If we've read anything up to this point return the bytes read
                                if bytes_read > 0 {
                                    return Poll::Ready(Ok(bytes_read));
                                } else {
                                    return Poll::Pending;
                                }
                            }
                            Poll::Ready(Some(buf)) => Some(buf),
                            Poll::Ready(None) => return Poll::Ready(Ok(bytes_read)),
                        }
                    };
                }
            }
        }
    }
}

impl AsyncWrite for MemorySocket {
    /// Attempt to write bytes from `buf` into the outgoing channel.
    fn poll_write(
        mut self: Pin<&mut Self>,
        context: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        let len = buf.len();

        match self.outgoing.poll_ready(context) {
            Poll::Ready(Ok(())) => {
                if let Err(e) = self.outgoing.start_send(Bytes::copy_from_slice(buf)) {
                    if e.is_disconnected() {
                        return Poll::Ready(Err(Error::new(ErrorKind::BrokenPipe, e)));
                    }

                    // Unbounded channels should only ever have "Disconnected" errors
                    unreachable!();
                }
            }
            Poll::Ready(Err(e)) => {
                if e.is_disconnected() {
                    return Poll::Ready(Err(Error::new(ErrorKind::BrokenPipe, e)));
                }

                // Unbounded channels should only ever have "Disconnected" errors
                unreachable!();
            }
            Poll::Pending => return Poll::Pending,
        }

        Poll::Ready(Ok(len))
    }

    /// Attempt to flush the channel. Cannot Fail.
    fn poll_flush(self: Pin<&mut Self>, _context: &mut Context) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    /// Attempt to close the channel. Cannot Fail.
    fn poll_close(self: Pin<&mut Self>, _context: &mut Context) -> Poll<Result<()>> {
        self.outgoing.close_channel();

        Poll::Ready(Ok(()))
    }
}
