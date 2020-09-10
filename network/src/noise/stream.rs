// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The socket module implements the post-handshake part of the protocol.
//! Its main type (`NoiseStream`) is returned after a successful [handshake].
//! functions in this module enables encrypting and decrypting messages from a socket.
//! Note that since noise is length-unaware, we have to prefix every noise message with its length
//!
//! [handshake]: network::noise::handshake

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
// NoiseStream
// ------------
//

/// A Noise stream with a remote peer.
///
/// Encrypts data to be written to and decrypts data that is read from the underlying socket using
/// the noise protocol. This is done by prefixing noise payloads with a u16 (big endian) length field.
#[derive(Debug)]
pub struct NoiseStream<TSocket> {
    /// the socket we write to and read from
    socket: TSocket,
    /// the noise session used to encrypt and decrypt messages
    session: noise::NoiseSession,
    /// handy buffers to write/read
    buffers: Box<NoiseBuffers>,
    /// an enum used for progressively reading a noise payload
    read_state: ReadState,
    /// an enum used for progressively writing a noise payload
    write_state: WriteState,
}

impl<TSocket> NoiseStream<TSocket> {
    /// Create a NoiseStream from a socket and a noise post-handshake session
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
}

//
// Reading a stream
// ----------------
//

/// Possible read states for a [NoiseStream]
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

impl<TSocket> NoiseStream<TSocket>
where
    TSocket: AsyncRead + Unpin,
{
    fn poll_read(&mut self, mut context: &mut Context, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        loop {
            trace!("NoiseStream ReadState::{:?}", self.read_state);
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
                                // 0-length messages are not expected
                                self.read_state = ReadState::Eof(Err(()));
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
                        &mut self.buffers.read_buffer[..(frame_len as usize)],
                        offset
                    )) {
                        Ok(()) => {
                            match self.session.read_message_in_place(
                                &mut self.buffers.read_buffer[..(frame_len as usize)],
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
                        &self.buffers.read_buffer[*offset..(*offset + bytes_to_copy)],
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
// Writing a stream
// ----------------
//

/// Possible write states for a [NoiseStream]
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

impl<TSocket> NoiseStream<TSocket>
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
                "NoiseStream {} WriteState::{:?}",
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
                        self.buffers.write_buffer[*offset..(*offset + bytes_to_copy)]
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
                            .write_message_in_place(&mut self.buffers.write_buffer[..*offset])
                        {
                            Ok(authentication_tag) => {
                                // append the authentication tag
                                self.buffers.write_buffer[*offset..*offset + noise::AES_GCM_TAGLEN]
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
                        &self.buffers.write_buffer[..(frame_len as usize)],
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

//
// Trait implementations
// ---------------------
//

impl<TSocket> AsyncRead for NoiseStream<TSocket>
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

impl<TSocket> AsyncWrite for NoiseStream<TSocket>
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
/// NoiseStream
struct NoiseBuffers {
    /// A read buffer, used for both a received ciphertext and then for its decrypted content.
    read_buffer: [u8; noise::MAX_SIZE_NOISE_MSG],
    /// A write buffer, used for both a plaintext to send, and then its encrypted version.
    write_buffer: [u8; noise::MAX_SIZE_NOISE_MSG],
}

impl NoiseBuffers {
    fn new() -> Self {
        Self {
            read_buffer: [0; noise::MAX_SIZE_NOISE_MSG],
            write_buffer: [0; noise::MAX_SIZE_NOISE_MSG],
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
// ------------------------------------------------
//

/// Write an offset of a buffer to a socket, only returns Ready once done.
fn poll_write_all<TSocket>(
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
fn poll_read_exact<TSocket>(
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
    use crate::{
        noise::{AntiReplayTimestamps, HandshakeAuthMode, NoiseUpgrader},
        testutils::fake_socket::{ReadOnlyTestSocket, ReadWriteTestSocket},
    };
    use futures::{
        executor::block_on,
        future::join,
        io::{AsyncReadExt, AsyncWriteExt},
    };
    use libra_config::network_id::NetworkContext;
    use libra_crypto::{test_utils::TEST_SEED, traits::Uniform as _, x25519};
    use libra_types::PeerId;
    use memsocket::MemorySocket;
    use rand::SeedableRng as _;
    use std::io;

    /// helper to setup two testing peers
    fn build_peers() -> (
        (NoiseUpgrader, x25519::PublicKey),
        (NoiseUpgrader, x25519::PublicKey),
    ) {
        let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);

        let client_private = x25519::PrivateKey::generate(&mut rng);
        let client_public = client_private.public_key();
        let client_peer_id = PeerId::from_identity_public_key(client_public);

        let server_private = x25519::PrivateKey::generate(&mut rng);
        let server_public = server_private.public_key();
        let server_peer_id = PeerId::from_identity_public_key(server_public);

        let client = NoiseUpgrader::new(
            NetworkContext::mock_with_peer_id(client_peer_id),
            client_private,
            HandshakeAuthMode::ServerOnly,
        );
        let server = NoiseUpgrader::new(
            NetworkContext::mock_with_peer_id(server_peer_id),
            server_private,
            HandshakeAuthMode::ServerOnly,
        );

        ((client, client_public), (server, server_public))
    }

    /// helper to perform a noise handshake with two peers
    fn perform_handshake(
        client: NoiseUpgrader,
        server_public_key: x25519::PublicKey,
        server: NoiseUpgrader,
    ) -> (NoiseStream<MemorySocket>, NoiseStream<MemorySocket>) {
        // create an in-memory socket for testing
        let (dialer_socket, listener_socket) = MemorySocket::new_pair();

        // perform the handshake
        let (client_session, server_session) = block_on(join(
            client.upgrade_outbound(dialer_socket, server_public_key, AntiReplayTimestamps::now),
            server.upgrade_inbound(listener_socket),
        ));

        //
        let client_session = client_session.unwrap();
        let (server_session, _) = server_session.unwrap();
        (client_session, server_session)
    }

    #[test]
    fn simple_test() -> io::Result<()> {
        // perform handshake with two testing peers
        let ((client, _client_public), (server, server_public)) = build_peers();
        let (mut client, mut server) = perform_handshake(client, server_public, server);

        block_on(client.write_all(b"stormlight"))?;
        block_on(client.write_all(b" "))?;
        block_on(client.write_all(b"archive"))?;
        block_on(client.flush())?;
        block_on(client.close())?;

        let mut buf = Vec::new();
        block_on(server.read_to_end(&mut buf))?;

        assert_eq!(buf, b"stormlight archive");

        Ok(())
    }

    // we used to time out when given a stream of all zeros, now we want an EOF
    #[test]
    fn dont_read_forever() {
        // setup fake socket
        let mut fake_socket = ReadOnlyTestSocket::new(&[0u8]);

        // the socket will read a continuous streams of zeros
        fake_socket.set_trailing();

        // setup a NoiseStream with a dummy state
        let noise_session = noise::NoiseSession::new_for_testing();
        let mut peer = NoiseStream::new(fake_socket, noise_session);

        // make sure we error and we don't continuously read
        block_on(async move {
            let mut buffer = [0u8; 128];
            let res = peer.read(&mut buffer).await;
            assert!(res.is_err());
        });
    }

    #[test]
    fn interleaved_writes() {
        // perform handshake with two testing peers
        let ((client, _client_public), (server, server_public)) = build_peers();
        let (mut client, mut server) = perform_handshake(client, server_public, server);

        block_on(client.write_all(b"The Name of the Wind")).unwrap();
        block_on(client.flush()).unwrap();
        block_on(client.write_all(b"The Wise Man's Fear")).unwrap();
        block_on(client.flush()).unwrap();

        block_on(server.write_all(b"The Doors of Stone")).unwrap();
        block_on(server.flush()).unwrap();

        let mut buf = [0; 20];
        block_on(server.read_exact(&mut buf)).unwrap();
        assert_eq!(&buf, b"The Name of the Wind");
        let mut buf = [0; 19];
        block_on(server.read_exact(&mut buf)).unwrap();
        assert_eq!(&buf, b"The Wise Man's Fear");

        let mut buf = [0; 18];
        block_on(client.read_exact(&mut buf)).unwrap();
        assert_eq!(&buf, b"The Doors of Stone");
    }

    #[test]
    fn u16_max_writes() {
        // perform handshake with two testing peers
        let ((client, _client_public), (server, server_public)) = build_peers();
        let (mut client, mut server) = perform_handshake(client, server_public, server);

        let buf_send = [1; noise::MAX_SIZE_NOISE_MSG];
        block_on(client.write_all(&buf_send)).unwrap();
        block_on(client.flush()).unwrap();

        let mut buf_receive = [0; noise::MAX_SIZE_NOISE_MSG];
        block_on(server.read_exact(&mut buf_receive)).unwrap();
        assert_eq!(&buf_receive[..], &buf_send[..]);
    }

    #[test]
    fn fragmented_stream() {
        // create an in-memory socket for testing
        let (mut dialer_socket, mut listener_socket) = ReadWriteTestSocket::new_pair();

        // fragment reads
        dialer_socket.set_fragmented_read();
        listener_socket.set_fragmented_read();

        // get peers
        let ((client, _client_public_key), (server, server_public_key)) = build_peers();

        // perform the handshake
        let (client, server) = block_on(join(
            client.upgrade_outbound(dialer_socket, server_public_key, AntiReplayTimestamps::now),
            server.upgrade_inbound(listener_socket),
        ));

        // get session
        let mut client = client.unwrap();
        let (mut server, _) = server.unwrap();

        // test send and receive
        block_on(client.write_all(b"The Name of the Wind")).unwrap();
        block_on(client.flush()).unwrap();
        block_on(client.write_all(b"The Wise Man's Fear")).unwrap();
        block_on(client.flush()).unwrap();

        block_on(server.write_all(b"The Doors of Stone")).unwrap();
        block_on(server.flush()).unwrap();

        let mut buf = [0; 20];
        block_on(server.read_exact(&mut buf)).unwrap();
        assert_eq!(&buf, b"The Name of the Wind");
        let mut buf = [0; 19];
        block_on(server.read_exact(&mut buf)).unwrap();
        assert_eq!(&buf, b"The Wise Man's Fear");

        let mut buf = [0; 18];
        block_on(client.read_exact(&mut buf)).unwrap();
        assert_eq!(&buf, b"The Doors of Stone");
    }
}
