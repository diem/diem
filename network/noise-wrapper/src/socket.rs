// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Noise Socket
//!
//! This code is helpful to read and write Noise messages on a socket.
//! Since Noise messages are variable-length, we prefix them with
//!

use futures::{
    io::{AsyncRead, AsyncWrite},
    ready,
};
use std::{
    convert::TryInto,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use libra_crypto::{noise, x25519};
use libra_logger::prelude::*;

//
// NoiseSocket
// -----------
//

/// A Noise session with a remote peer.
///
/// Encrypts data to be written to and decrypts data that is read from the underlying socket using
/// the noise protocol. This is done by wrapping noise payloads in u16 (big endian) length prefix
/// frames.
#[derive(Debug)]
pub struct NoiseSocket<TSocket> {
    /// the socket we write to and read from
    socket: TSocket,
    /// the noise stack
    session: noise::NoiseSession,
    /// handy buffers to write/read
    buffers: Box<NoiseBuffers>,
    ///
    read_state: ReadState,
    ///
    write_state: WriteState,
}

impl<TSocket> NoiseSocket<TSocket> {
    /// Create a NoiseSocket from a socket and a noise post-handshake session
    pub fn new(socket: TSocket, session: noise::NoiseSession) -> Self {
        Self {
            socket,
            session,
            buffers: Box::new(NoiseBuffers::new()),
            read_state: ReadState::Init,
            write_state: WriteState::Init,
        }
    }

    /// Pull out the static public key of the remote
    pub fn get_remote_static(&self) -> x25519::PublicKey {
        self.session.get_remote_static()
    }

    pub fn read_message_in_place<'a>(
        &mut self,
        message: &'a mut [u8],
    ) -> Result<&'a [u8], noise::NoiseError> {
        self.session.read_message_in_place(message)
    }

    pub fn write_message_in_place(
        &mut self,
        message: &mut [u8],
    ) -> Result<Vec<u8>, noise::NoiseError> {
        self.session.write_message_in_place(message)
    }
}

// read parts
// ----------

/// Possible read states for a [NoiseSocket]
#[derive(Debug)]
enum ReadState {
    /// Initial State
    Init,
    /// Read frame length
    ReadFrameLen { buf: [u8; 2], offset: usize },
    /// Read encrypted frame
    ReadFrame { frame_len: u16, offset: usize },
    /// Copy decrypted frame to provided buffer
    CopyDecryptedFrame { decrypted_len: usize, offset: usize },
    /// End of file reached, result indicated if EOF was expected or not
    Eof(Result<(), ()>),
    /// Decryption Error
    DecryptionError(noise::NoiseError),
}

impl<TSocket> NoiseSocket<TSocket>
where
    TSocket: AsyncRead + Unpin,
{
    fn poll_read(&mut self, mut context: &mut Context, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        loop {
            trace!("NoiseSocket ReadState::{:?}", self.read_state);
            match self.read_state {
                ReadState::Init => {
                    self.read_state = ReadState::ReadFrameLen {
                        buf: [0, 0],
                        offset: 0,
                    };
                }
                ReadState::ReadFrameLen {
                    ref mut buf,
                    ref mut offset,
                } => {
                    match ready!(poll_read_u16frame_len(
                        &mut context,
                        Pin::new(&mut self.socket),
                        buf,
                        offset
                    )) {
                        Ok(Some(frame_len)) => {
                            // Empty Frame
                            if frame_len == 0 {
                                self.read_state = ReadState::Init;
                            } else {
                                self.read_state = ReadState::ReadFrame {
                                    frame_len,
                                    offset: 0,
                                };
                            }
                        }
                        Ok(None) => {
                            self.read_state = ReadState::Eof(Ok(()));
                        }
                        Err(e) => {
                            if e.kind() == io::ErrorKind::UnexpectedEof {
                                self.read_state = ReadState::Eof(Err(()));
                            }
                            return Poll::Ready(Err(e));
                        }
                    }
                }
                ReadState::ReadFrame {
                    frame_len,
                    ref mut offset,
                } => {
                    match ready!(poll_read_exact(
                        &mut context,
                        Pin::new(&mut self.socket),
                        &mut self.buffers.read_encrypted[..(frame_len as usize)],
                        offset
                    )) {
                        Ok(()) => {
                            match self.session.read_message_in_place(
                                &mut self.buffers.read_encrypted[..(frame_len as usize)],
                            ) {
                                Ok(decrypted) => {
                                    self.read_state = ReadState::CopyDecryptedFrame {
                                        decrypted_len: decrypted.len(),
                                        offset: 0,
                                    };
                                }
                                Err(e) => {
                                    error!("Decryption Error: {}", e);
                                    self.read_state = ReadState::DecryptionError(e);
                                }
                            }
                        }
                        Err(e) => {
                            if e.kind() == io::ErrorKind::UnexpectedEof {
                                self.read_state = ReadState::Eof(Err(()));
                            }
                            return Poll::Ready(Err(e));
                        }
                    }
                }
                ReadState::CopyDecryptedFrame {
                    decrypted_len,
                    ref mut offset,
                } => {
                    let bytes_to_copy =
                        ::std::cmp::min(decrypted_len as usize - *offset, buf.len());
                    buf[..bytes_to_copy].copy_from_slice(
                        &self.buffers.read_encrypted[*offset..(*offset + bytes_to_copy)],
                    );
                    trace!(
                        "CopyDecryptedFrame: copied {}/{} bytes",
                        *offset + bytes_to_copy,
                        decrypted_len
                    );
                    *offset += bytes_to_copy;
                    if *offset == decrypted_len as usize {
                        self.read_state = ReadState::Init;
                    }
                    return Poll::Ready(Ok(bytes_to_copy));
                }
                ReadState::Eof(Ok(())) => return Poll::Ready(Ok(0)),
                ReadState::Eof(Err(())) => {
                    return Poll::Ready(Err(io::ErrorKind::UnexpectedEof.into()))
                }
                ReadState::DecryptionError(ref e) => {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("DecryptionError: {}", e),
                    )))
                }
            }
        }
    }
}

//
// write parts
// -----------
//

/// Possible write states for a [NoiseSocket]
#[derive(Debug)]
enum WriteState {
    /// Initial State
    Init,
    /// Buffer provided data
    BufferData { offset: usize },
    /// Write frame length to the wire
    WriteFrameLen {
        frame_len: u16,
        buf: [u8; 2],
        offset: usize,
    },
    /// Write encrypted frame to the wire
    WriteEncryptedFrame { frame_len: u16, offset: usize },
    /// Flush the underlying socket
    Flush,
    /// End of file reached
    Eof,
    /// Encryption Error
    EncryptionError(noise::NoiseError),
}

impl<TSocket> NoiseSocket<TSocket>
where
    TSocket: AsyncWrite + Unpin,
{
    fn poll_write_or_flush(
        &mut self,
        mut context: &mut Context,
        buf: Option<&[u8]>,
    ) -> Poll<io::Result<Option<usize>>> {
        loop {
            trace!(
                "NoiseSocket {} WriteState::{:?}",
                if buf.is_some() {
                    "poll_write"
                } else {
                    "poll_flush"
                },
                self.write_state,
            );
            match self.write_state {
                WriteState::Init => {
                    if buf.is_some() {
                        self.write_state = WriteState::BufferData { offset: 0 };
                    } else {
                        return Poll::Ready(Ok(None));
                    }
                }
                WriteState::BufferData { ref mut offset } => {
                    let bytes_buffered = if let Some(buf) = buf {
                        let bytes_to_copy =
                            ::std::cmp::min(MAX_WRITE_BUFFER_LENGTH - *offset, buf.len());
                        self.buffers.write_decrypted[*offset..(*offset + bytes_to_copy)]
                            .copy_from_slice(&buf[..bytes_to_copy]);
                        trace!("BufferData: buffered {}/{} bytes", bytes_to_copy, buf.len());
                        *offset += bytes_to_copy;
                        Some(bytes_to_copy)
                    } else {
                        None
                    };

                    if buf.is_none() || *offset == MAX_WRITE_BUFFER_LENGTH {
                        match self
                            .session
                            .write_message_in_place(&mut self.buffers.write_decrypted[..*offset])
                        {
                            Ok(authentication_tag) => {
                                // append the authentication tag
                                self.buffers.write_decrypted
                                    [*offset..*offset + noise::AES_GCM_TAGLEN]
                                    .copy_from_slice(&authentication_tag);
                                // calculate frame length
                                let frame_len = noise::encrypted_len(*offset);
                                let frame_len = frame_len
                                    .try_into()
                                    .expect("offset should be able to fit in u16");
                                self.write_state = WriteState::WriteFrameLen {
                                    frame_len,
                                    buf: u16::to_be_bytes(frame_len),
                                    offset: 0,
                                };
                            }
                            Err(e) => {
                                error!("Encryption Error: {}", e);
                                let err = io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("EncryptionError: {}", e),
                                );
                                self.write_state = WriteState::EncryptionError(e);
                                return Poll::Ready(Err(err));
                            }
                        }
                    }

                    if let Some(bytes_buffered) = bytes_buffered {
                        return Poll::Ready(Ok(Some(bytes_buffered)));
                    }
                }
                WriteState::WriteFrameLen {
                    frame_len,
                    ref buf,
                    ref mut offset,
                } => {
                    match ready!(poll_write_all(
                        &mut context,
                        Pin::new(&mut self.socket),
                        buf,
                        offset
                    )) {
                        Ok(()) => {
                            self.write_state = WriteState::WriteEncryptedFrame {
                                frame_len,
                                offset: 0,
                            };
                        }
                        Err(e) => {
                            if e.kind() == io::ErrorKind::WriteZero {
                                self.write_state = WriteState::Eof;
                            }
                            return Poll::Ready(Err(e));
                        }
                    }
                }
                WriteState::WriteEncryptedFrame {
                    frame_len,
                    ref mut offset,
                } => {
                    match ready!(poll_write_all(
                        &mut context,
                        Pin::new(&mut self.socket),
                        &self.buffers.write_decrypted[..(frame_len as usize)],
                        offset
                    )) {
                        Ok(()) => {
                            self.write_state = WriteState::Flush;
                        }
                        Err(e) => {
                            if e.kind() == io::ErrorKind::WriteZero {
                                self.write_state = WriteState::Eof;
                            }
                            return Poll::Ready(Err(e));
                        }
                    }
                }
                WriteState::Flush => {
                    ready!(Pin::new(&mut self.socket).poll_flush(&mut context))?;
                    self.write_state = WriteState::Init;
                }
                WriteState::Eof => return Poll::Ready(Err(io::ErrorKind::WriteZero.into())),
                WriteState::EncryptionError(ref e) => {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("EncryptionError: {}", e),
                    )))
                }
            }
        }
    }

    fn poll_write(&mut self, context: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
        if let Some(bytes_written) = ready!(self.poll_write_or_flush(context, Some(buf)))? {
            Poll::Ready(Ok(bytes_written))
        } else {
            unreachable!();
        }
    }

    fn poll_flush(&mut self, context: &mut Context) -> Poll<io::Result<()>> {
        if ready!(self.poll_write_or_flush(context, None))?.is_none() {
            Poll::Ready(Ok(()))
        } else {
            unreachable!();
        }
    }
}

// trait implementations for NoiseSocket

impl<TSocket> AsyncRead for NoiseSocket<TSocket>
where
    TSocket: AsyncRead + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.get_mut().poll_read(context, buf)
    }
}

impl<TSocket> AsyncWrite for NoiseSocket<TSocket>
where
    TSocket: AsyncWrite + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        context: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.get_mut().poll_write(context, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, context: &mut Context) -> Poll<io::Result<()>> {
        self.get_mut().poll_flush(context)
    }

    fn poll_close(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.socket).poll_close(context)
    }
}

//
// NoiseBuffers
// ------------
//

// encrypted messages include a tag along with the payload.
const MAX_WRITE_BUFFER_LENGTH: usize = noise::decrypted_len(noise::MAX_SIZE_NOISE_MSG);

/// Collection of buffers used for buffering data during the various read/write states of a
/// NoiseSocket
struct NoiseBuffers {
    /// TODO: doc
    read_encrypted: [u8; noise::MAX_SIZE_NOISE_MSG],
    /// TODO: doc
    write_decrypted: [u8; MAX_WRITE_BUFFER_LENGTH],
}

impl NoiseBuffers {
    fn new() -> Self {
        Self {
            read_encrypted: [0; noise::MAX_SIZE_NOISE_MSG],
            write_decrypted: [0; MAX_WRITE_BUFFER_LENGTH],
        }
    }
}

/// Hand written Debug implementation in order to omit the printing of huge buffers of data
impl ::std::fmt::Debug for NoiseBuffers {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        f.debug_struct("NoiseBuffers").finish()
    }
}

//
// Helpers for writing to and reading from a socket
//

/// TODO: doc
pub fn poll_write_all<TSocket>(
    mut context: &mut Context,
    mut socket: Pin<&mut TSocket>,
    buf: &[u8],
    offset: &mut usize,
) -> Poll<io::Result<()>>
where
    TSocket: AsyncWrite,
{
    loop {
        let n = ready!(socket.as_mut().poll_write(&mut context, &buf[*offset..]))?;
        trace!("poll_write_all: wrote {}/{} bytes", *offset + n, buf.len());
        if n == 0 {
            return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
        }
        *offset += n;
        assert!(*offset <= buf.len());

        if *offset == buf.len() {
            return Poll::Ready(Ok(()));
        }
    }
}

/// Read a u16 frame length from `socket`.
///
/// Can result in the following output:
/// 1) Ok(None) => EOF; remote graceful shutdown
/// 2) Err(UnexpectedEOF) => read 1 byte then hit EOF; remote died
/// 3) Ok(Some(n)) => new frame of length n
fn poll_read_u16frame_len<TSocket>(
    context: &mut Context,
    socket: Pin<&mut TSocket>,
    buf: &mut [u8; 2],
    offset: &mut usize,
) -> Poll<io::Result<Option<u16>>>
where
    TSocket: AsyncRead,
{
    match ready!(poll_read_exact(context, socket, buf, offset)) {
        Ok(()) => Poll::Ready(Ok(Some(u16::from_be_bytes(*buf)))),
        Err(e) => {
            if *offset == 0 && e.kind() == io::ErrorKind::UnexpectedEof {
                return Poll::Ready(Ok(None));
            }
            Poll::Ready(Err(e))
        }
    }
}

/// As data can be fragmented over multiple TCP packets, poll_read_exact
/// continuously calls poll_read on the socket until enough data is read.
/// It is possible that this function never completes,
/// so a timeout needs to be set on the caller side.
pub fn poll_read_exact<TSocket>(
    mut context: &mut Context,
    mut socket: Pin<&mut TSocket>,
    buf: &mut [u8],
    offset: &mut usize,
) -> Poll<io::Result<()>>
where
    TSocket: AsyncRead,
{
    loop {
        let n = ready!(socket.as_mut().poll_read(&mut context, &mut buf[*offset..]))?;
        trace!("poll_read_exact: read {}/{} bytes", *offset + n, buf.len());
        if n == 0 {
            return Poll::Ready(Err(io::ErrorKind::UnexpectedEof.into()));
        }
        *offset += n;
        assert!(*offset <= buf.len());
        if *offset == buf.len() {
            return Poll::Ready(Ok(()));
        }
    }
}

//
// Tests
// -----
//

#[cfg(test)]
mod test {
    use super::*;

    use crate::handshake::NoiseWrapper;
    use futures::{
        executor::block_on,
        future::join,
        io::{AsyncReadExt, AsyncWriteExt},
    };
    use libra_crypto::{test_utils::TEST_SEED, x25519};
    use memsocket::MemorySocket;
    use std::io;
    use std::sync::{Arc, RwLock};
    use std::collections::HashMap;
    use libra_types::PeerId;
    use libra_config::config::NetworkPeerInfo;

    use rand::SeedableRng as _;
    use libra_crypto::traits::Uniform as _;

    /// helper to setup two peers
    fn build_peers() -> (
        (NoiseWrapper, x25519::PublicKey),
        (NoiseWrapper, x25519::PublicKey),
    ) {
        let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);

        let client_private = x25519::PrivateKey::generate(&mut rng);
        let client_public = client_private.public_key();

        let server_private = x25519::PrivateKey::generate(&mut rng);
        let server_public = server_private.public_key();

        let client = NoiseWrapper::new(client_private);
        let server = NoiseWrapper::new(server_private);

        (
            (client, client_public),
            (server, server_public),
        )
    }

    /// helper to perform a noise handshake with two peers
    fn perform_handshake(
        client: NoiseWrapper,
        server_public_key: x25519::PublicKey,
        server: NoiseWrapper,
        trusted_peers: Option<&Arc<RwLock<HashMap<PeerId, NetworkPeerInfo>>>>,
    ) -> io::Result<(NoiseSocket<MemorySocket>, NoiseSocket<MemorySocket>)> {
        // create an in-memory socket for testing
        let (dialer_socket, listener_socket) = MemorySocket::new_pair();

        // perform the handshake
        let (client_session, server_session) = block_on(join(
            client.dial(dialer_socket, server_public_key),
            server.accept(listener_socket, trusted_peers),
        ));

        //
        Ok((client_session?, server_session?))
    }

    #[test]
    fn test_handshake() {
        let ((client, client_public), (server, server_public)) =
            build_peers();

        let (client, server) = perform_handshake(client, server_public, server, None).unwrap();

        assert_eq!(client.get_remote_static(), server_public,);
        assert_eq!(server.get_remote_static(), client_public,);
    }
    /*

    #[test]
    fn simple_test() -> io::Result<()> {
        let ((_dialer_keypair, dialer), (_listener_keypair, listener)) =
            build_peers();

        let (mut dialer_socket, mut listener_socket) = perform_handshake(dialer, listener)?;

        block_on(dialer_socket.write_all(b"stormlight"))?;
        block_on(dialer_socket.write_all(b" "))?;
        block_on(dialer_socket.write_all(b"archive"))?;
        block_on(dialer_socket.flush())?;
        block_on(dialer_socket.close())?;

        let mut buf = Vec::new();
        block_on(listener_socket.read_to_end(&mut buf))?;

        assert_eq!(buf, b"stormlight archive");

        Ok(())
    }

    #[test]
    fn interleaved_writes() -> io::Result<()> {
        let ((_dialer_keypair, dialer), (_listener_keypair, listener)) =
            build_peers();

        let (mut a, mut b) = perform_handshake(dialer, listener)?;

        block_on(a.write_all(b"The Name of the Wind"))?;
        block_on(a.flush())?;
        block_on(a.write_all(b"The Wise Man's Fear"))?;
        block_on(a.flush())?;

        block_on(b.write_all(b"The Doors of Stone"))?;
        block_on(b.flush())?;

        let mut buf = [0; 20];
        block_on(b.read_exact(&mut buf))?;
        assert_eq!(&buf, b"The Name of the Wind");
        let mut buf = [0; 19];
        block_on(b.read_exact(&mut buf))?;
        assert_eq!(&buf, b"The Wise Man's Fear");

        let mut buf = [0; 18];
        block_on(a.read_exact(&mut buf))?;
        assert_eq!(&buf, b"The Doors of Stone");

        Ok(())
    }

    #[test]
    fn u16_max_writes() -> io::Result<()> {
        let ((_dialer_keypair, dialer), (_listener_keypair, listener)) =
            build_peers();

        let (mut a, mut b) = perform_handshake(dialer, listener)?;

        let buf_send = [1; noise::MAX_SIZE_NOISE_MSG];
        block_on(a.write_all(&buf_send))?;
        block_on(a.flush())?;

        let mut buf_receive = [0; noise::MAX_SIZE_NOISE_MSG];
        block_on(b.read_exact(&mut buf_receive))?;
        assert_eq!(&buf_receive[..], &buf_send[..]);

        Ok(())
    }
    */
}
