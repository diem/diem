// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Noise Socket

use futures::{
    future::poll_fn,
    io::{AsyncRead, AsyncWrite},
    ready,
};
use libra_logger::prelude::*;
use std::{
    convert::TryInto,
    io,
    pin::Pin,
    task::{Context, Poll},
};

const MAX_PAYLOAD_LENGTH: usize = u16::max_value() as usize; // 65535

// The maximum number of bytes that we can buffer is 16 bytes less than u16::max_value() because
// encrypted messages include a tag along with the payload.
const MAX_WRITE_BUFFER_LENGTH: usize = u16::max_value() as usize - 16; // 65519

/// Collection of buffers used for buffering data during the various read/write states of a
/// NoiseSocket
struct NoiseBuffers {
    /// Encrypted frame read from the wire
    read_encrypted: [u8; MAX_PAYLOAD_LENGTH],
    /// Decrypted data read from the wire (produced by having snow decrypt the `read_encrypted`
    /// buffer)
    read_decrypted: [u8; MAX_PAYLOAD_LENGTH],
    /// Unencrypted data intended to be written to the wire
    write_decrypted: [u8; MAX_WRITE_BUFFER_LENGTH],
    /// Encrypted data to write to the wire (produced by having snow encrypt the `write_decrypted`
    /// buffer)
    write_encrypted: [u8; MAX_PAYLOAD_LENGTH],
}

impl NoiseBuffers {
    fn new() -> Self {
        Self {
            read_encrypted: [0; MAX_PAYLOAD_LENGTH],
            read_decrypted: [0; MAX_PAYLOAD_LENGTH],
            write_decrypted: [0; MAX_WRITE_BUFFER_LENGTH],
            write_encrypted: [0; MAX_PAYLOAD_LENGTH],
        }
    }
}

/// Hand written Debug implementation in order to omit the printing of huge buffers of data
impl ::std::fmt::Debug for NoiseBuffers {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        f.debug_struct("NoiseBuffers").finish()
    }
}

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
    DecryptionError(snow::error::Error),
}

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
    EncryptionError(snow::error::Error),
}

/// Session mode for snow
#[derive(Debug)]
enum Session {
    Handshake(Box<snow::HandshakeState>),
    Transport(Box<snow::TransportState>),
}

impl Session {
    pub fn is_initiator(&self) -> bool {
        match self {
            Session::Handshake(ref session) => session.is_initiator(),
            Session::Transport(ref session) => session.is_initiator(),
        }
    }

    pub fn read_message(
        &mut self,
        message: &[u8],
        payload: &mut [u8],
    ) -> Result<usize, snow::error::Error> {
        match self {
            Session::Handshake(ref mut session) => session.read_message(message, payload),
            Session::Transport(ref mut session) => session.read_message(message, payload),
        }
    }

    pub fn write_message(
        &mut self,
        message: &[u8],
        payload: &mut [u8],
    ) -> Result<usize, snow::error::Error> {
        match self {
            Session::Handshake(ref mut session) => session.write_message(message, payload),
            Session::Transport(ref mut session) => session.write_message(message, payload),
        }
    }

    pub fn into_transport_mode(self) -> Result<snow::TransportState, io::Error> {
        match self {
            Session::Handshake(session) => session
                .into_transport_mode()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Noise error: {}", e))),
            Session::Transport(_) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Session not in handshake state".to_string(),
            )),
        }
    }
}

/// A Noise session with a remote
///
/// Encrypts data to be written to and decrypts data that is read from the underlying socket using
/// the noise protocol. This is done by wrapping noise payloads in u16 (big endian) length prefix
/// frames.
#[derive(Debug)]
pub struct NoiseSocket<TSocket> {
    socket: TSocket,
    session: Session,
    buffers: Box<NoiseBuffers>,
    read_state: ReadState,
    write_state: WriteState,
}

impl<TSocket> NoiseSocket<TSocket> {
    fn new(socket: TSocket, session: snow::HandshakeState) -> Self {
        Self {
            socket,
            session: Session::Handshake(Box::new(session)),
            buffers: Box::new(NoiseBuffers::new()),
            read_state: ReadState::Init,
            write_state: WriteState::Init,
        }
    }

    /// Pull out the static public key of the remote
    pub fn get_remote_static(&self) -> Option<&[u8]> {
        match self.session {
            Session::Handshake(ref session) => session.get_remote_static(),
            Session::Transport(ref session) => session.get_remote_static(),
        }
    }
}

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
                            match self.session.read_message(
                                &self.buffers.read_encrypted[..(frame_len as usize)],
                                &mut self.buffers.read_decrypted,
                            ) {
                                Ok(decrypted_len) => {
                                    self.read_state = ReadState::CopyDecryptedFrame {
                                        decrypted_len,
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
                        &self.buffers.read_decrypted[*offset..(*offset + bytes_to_copy)],
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
                        match self.session.write_message(
                            &self.buffers.write_decrypted[..*offset],
                            &mut self.buffers.write_encrypted,
                        ) {
                            Ok(encrypted_len) => {
                                let frame_len = encrypted_len
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
                        &self.buffers.write_encrypted[..(frame_len as usize)],
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

/// Represents a noise session which still needs to have a handshake performed.
pub(super) struct Handshake<TSocket>(NoiseSocket<TSocket>);

impl<TSocket> Handshake<TSocket> {
    /// Build a new `Handshake` struct given a socket and a new snow HandshakeState
    pub fn new(socket: TSocket, session: snow::HandshakeState) -> Self {
        let noise_socket = NoiseSocket::new(socket, session);
        Self(noise_socket)
    }
}

impl<TSocket> Handshake<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Unpin,
{
    /// Perform a Single Round-Trip noise IX handshake returning the underlying [NoiseSocket]
    /// (switched to transport mode) upon success.
    pub async fn handshake_1rt(mut self) -> io::Result<NoiseSocket<TSocket>> {
        // The Dialer
        if self.0.session.is_initiator() {
            // -> e, s
            self.send().await?;
            self.flush().await?;

            // <- e, ee, se, s, es
            self.receive().await?;
        } else {
            // -> e, s
            self.receive().await?;

            // <- e, ee, se, s, es
            self.send().await?;
            self.flush().await?;
        }

        self.finish()
    }

    /// Send handshake message to remote.
    async fn send(&mut self) -> io::Result<()> {
        poll_fn(|context| self.0.poll_write(context, &[]))
            .await
            .map(|_| ())
    }

    /// Flush handshake message to remote.
    async fn flush(&mut self) -> io::Result<()> {
        poll_fn(|context| self.0.poll_flush(context)).await
    }

    /// Receive handshake message from remote.
    async fn receive(&mut self) -> io::Result<()> {
        poll_fn(|context| self.0.poll_read(context, &mut []))
            .await
            .map(|_| ())
    }

    /// Finish the handshake.
    ///
    /// Converts the noise session into transport mode and returns the NoiseSocket.
    fn finish(self) -> io::Result<NoiseSocket<TSocket>> {
        let session = self.0.session.into_transport_mode()?;
        Ok(NoiseSocket {
            session: Session::Transport(Box::new(session)),
            ..self.0
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        socket::{Handshake, NoiseSocket, MAX_PAYLOAD_LENGTH},
        NOISE_PARAMETER,
    };
    use futures::{
        executor::block_on,
        future::join,
        io::{AsyncReadExt, AsyncWriteExt},
    };
    use memsocket::MemorySocket;
    use snow::{params::NoiseParams, Builder, Keypair};
    use std::io;

    fn build_test_connection() -> Result<
        (
            (Keypair, Handshake<MemorySocket>),
            (Keypair, Handshake<MemorySocket>),
        ),
        snow::error::Error,
    > {
        let parameters: NoiseParams = NOISE_PARAMETER.parse().expect("Invalid protocol name");

        let dialer_keypair = Builder::new(parameters.clone()).generate_keypair()?;
        let listener_keypair = Builder::new(parameters.clone()).generate_keypair()?;

        let dialer_session = Builder::new(parameters.clone())
            .local_private_key(&dialer_keypair.private)
            .build_initiator()?;
        let listener_session = Builder::new(parameters)
            .local_private_key(&listener_keypair.private)
            .build_responder()?;

        let (dialer_socket, listener_socket) = MemorySocket::new_pair();
        let (dialer, listener) = (
            NoiseSocket::new(dialer_socket, dialer_session),
            NoiseSocket::new(listener_socket, listener_session),
        );

        Ok((
            (dialer_keypair, Handshake(dialer)),
            (listener_keypair, Handshake(listener)),
        ))
    }

    fn perform_handshake(
        dialer: Handshake<MemorySocket>,
        listener: Handshake<MemorySocket>,
    ) -> io::Result<(NoiseSocket<MemorySocket>, NoiseSocket<MemorySocket>)> {
        let (dialer_result, listener_result) =
            block_on(join(dialer.handshake_1rt(), listener.handshake_1rt()));

        Ok((dialer_result?, listener_result?))
    }

    #[test]
    fn test_handshake() {
        let ((dialer_keypair, dialer), (listener_keypair, listener)) =
            build_test_connection().unwrap();

        let (dialer_socket, listener_socket) = perform_handshake(dialer, listener).unwrap();

        assert_eq!(
            dialer_socket.get_remote_static(),
            Some(listener_keypair.public.as_ref())
        );
        assert_eq!(
            listener_socket.get_remote_static(),
            Some(dialer_keypair.public.as_ref())
        );
    }

    #[test]
    fn simple_test() -> io::Result<()> {
        let ((_dialer_keypair, dialer), (_listener_keypair, listener)) =
            build_test_connection().unwrap();

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
            build_test_connection().unwrap();

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
            build_test_connection().unwrap();

        let (mut a, mut b) = perform_handshake(dialer, listener)?;

        let buf_send = [1; MAX_PAYLOAD_LENGTH];
        block_on(a.write_all(&buf_send))?;
        block_on(a.flush())?;

        let mut buf_receive = [0; MAX_PAYLOAD_LENGTH];
        block_on(b.read_exact(&mut buf_receive))?;
        assert_eq!(&buf_receive[..], &buf_send[..]);

        Ok(())
    }
}
