// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//!
//! This module exposes two types of sockets useful for tests:
//! - ReadOnlyTestSocket: a socket that can be read from in different ways.
//! - ReadWriteTestSocket: a similar wrapper but around MemorySocket to retrieve handshake messages being sent as well.
//!

use futures::{
    io::{AsyncRead, AsyncWrite},
    ready,
    task::{Context, Poll},
};
use memsocket::MemorySocket;
use std::{io, pin::Pin};

//
// ReadOnlyTestSocket
// ==================
//

pub struct ReadOnlyTestSocket<'a> {
    /// the content
    content: &'a [u8],
    /// the socket will read byte-by-byte
    fragmented: bool,
    /// continue to read 0s once content has been fully read
    trailing: bool,
}

impl<'a> ReadOnlyTestSocket<'a> {
    pub fn new(content: &'a [u8]) -> Self {
        Self {
            content,
            fragmented: false,
            trailing: false,
        }
    }

    /// reads will have to be done byte by byte
    #[allow(dead_code)]
    pub fn set_fragmented_read(&mut self) {
        self.fragmented = true;
    }

    /// reads will never return pending, but 0s
    #[allow(dead_code)]
    pub fn set_trailing(&mut self) {
        self.trailing = true;
    }
}

/// Does nothing, but looks to the caller as if write worked
impl<'a> AsyncWrite for ReadOnlyTestSocket<'a> {
    fn poll_write(
        self: Pin<&mut Self>,
        _context: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _context: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    /// Attempt to close the channel. Cannot Fail.
    fn poll_close(self: Pin<&mut Self>, _context: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

/// Read based on the mode set
impl<'a> AsyncRead for ReadOnlyTestSocket<'a> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        mut _context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        // nothing left to read
        if self.content.is_empty() {
            if self.trailing {
                // [0, 0, 0, 0, ...]
                for val in buf.iter_mut() {
                    *val = 0;
                }
                return Poll::Ready(Ok(buf.len()));
            } else {
                // EOF
                return Poll::Ready(Ok(0));
            }
        }

        // if something left return what's asked
        if self.fragmented {
            // read only one byte
            let (read, content) = self.content.split_at(1);
            buf[0] = read[0];

            // update internal state
            self.content = content;

            // return 1 byte
            Poll::Ready(Ok(1))
        } else {
            // read as much as we can
            let to_read = std::cmp::min(buf.len(), self.content.len());
            buf[..to_read].copy_from_slice(&self.content[..to_read]);

            // update internal state
            self.content = &self.content[to_read..self.content.len()];

            // return length read
            Poll::Ready(Ok(to_read))
        }
    }
}

//
// ReadWriteTestSocket
// ==================
//

pub struct ReadWriteTestSocket<'a> {
    /// an in-memory socket
    inner: MemorySocket,
    /// useful to save what was written on the socket
    written: Option<&'a mut Vec<u8>>,
    /// fragment reads byte by byte
    fragmented: bool,
}

impl<'a> ReadWriteTestSocket<'a> {
    fn new(memory_socket: MemorySocket) -> Self {
        Self {
            inner: memory_socket,
            written: None,
            fragmented: false,
        }
    }

    /// the vec passed as argument will expand to store any writes on the socket
    pub fn save_writing(&mut self, buf: &'a mut Vec<u8>) {
        self.written = Some(buf);
    }

    /// reads will have to be done byte by byte
    #[allow(dead_code)]
    pub fn set_fragmented_read(&mut self) {
        self.fragmented = true;
    }

    /// Creates a new pair of sockets
    pub fn new_pair() -> (Self, Self) {
        let (dialer_socket, listener_socket) = MemorySocket::new_pair();
        (
            ReadWriteTestSocket::new(dialer_socket),
            ReadWriteTestSocket::new(listener_socket),
        )
    }
}

impl<'a> AsyncWrite for ReadWriteTestSocket<'a> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        context: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let bytes_written = ready!(Pin::new(&mut self.inner).poll_write(context, buf))?;
        if let Some(v) = self.written.as_mut() {
            v.extend_from_slice(&buf[..bytes_written])
        }
        Poll::Ready(Ok(bytes_written))
    }

    fn poll_flush(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(context)
    }

    /// Attempt to close the channel. Cannot Fail.
    fn poll_close(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_close(context)
    }
}

impl<'a> AsyncRead for ReadWriteTestSocket<'a> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        // read from inner sockets
        if self.fragmented {
            // fragmented: read only one byte
            let mut buf_byte = [0u8];
            let poll_res = Pin::new(&mut self.inner).poll_read(context, &mut buf_byte);
            if poll_res.is_ready() {
                buf[0] = buf_byte[0];
            }
            poll_res
        } else {
            // read normally
            Pin::new(&mut self.inner).poll_read(context, buf)
        }
    }
}

//
// Tests
// =====
//

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{
        executor::block_on,
        io::{AsyncReadExt, AsyncWriteExt},
    };

    #[test]
    fn test_normal_reads() {
        let a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut socket = ReadOnlyTestSocket::new(&a);

        block_on(async {
            let mut buf = [0u8; 10];
            socket.read_exact(&mut buf).await.unwrap();
            assert!(buf == a);
        });
    }

    #[test]
    fn test_fragmented_reads() {
        let a = [1, 2, 3, 4, 5];
        let mut socket = ReadOnlyTestSocket::new(&a);
        socket.set_fragmented_read();

        block_on(async {
            let mut buf = [9u8; 10];
            for i in 1..=5 {
                let read = socket.read(&mut buf).await.unwrap();
                assert!(read == 1);
                assert!(buf[0] == i);
            }
        });
    }

    #[test]
    fn test_trailing_reads() {
        let a = [1, 2, 3, 4, 5];
        let mut socket = ReadOnlyTestSocket::new(&a);
        socket.set_trailing();

        block_on(async {
            let mut buf = [0u8; 10];
            socket.read_exact(&mut buf).await.unwrap();
            assert!(buf[..5] == a);
            assert!(buf[5..] == [0u8, 0, 0, 0, 0]);
        });
    }

    #[test]
    fn test_fragmented_exposed_socket() {
        block_on(async {
            let (mut dialer, mut listener) = ReadWriteTestSocket::new_pair();
            let mut init_msg = Vec::new();
            let mut resp_msg = Vec::new();

            // save writes
            dialer.save_writing(&mut init_msg);
            listener.save_writing(&mut resp_msg);

            // fragment reads
            dialer.set_fragmented_read();
            listener.set_fragmented_read();

            let first_message = [1u8, 2, 3, 4, 5];
            let second_message = [6u8, 7, 8, 9, 10];

            // dialer sends first message
            let written = dialer.write(&first_message).await.unwrap();
            assert_eq!(written, first_message.len());

            // listener reads byte-by-byte
            let mut buf = [0u8; 5];
            for i in 1..=5 {
                let read = listener.read(&mut buf).await.unwrap();
                assert!(read == 1);
                assert!(buf[0] == i);
            }

            // listener responds with second message
            let written = listener.write(&second_message).await.unwrap();
            assert_eq!(written, second_message.len());

            // dialer reads byte-by-byte
            for i in 6..=10 {
                let read = dialer.read(&mut buf).await.unwrap();
                assert!(read == 1);
                assert!(buf[0] == i);
            }

            // did we record the writes correctly?
            assert!(init_msg.as_slice() == first_message);
            assert!(resp_msg.as_slice() == second_message);
        });
    }
}
