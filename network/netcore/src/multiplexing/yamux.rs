// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of [`StreamMultiplexer`] using the [`yamux`] protocol over TCP
//!
//! [`StreamMultiplexer`]: crate::multiplexing::StreamMultiplexer
//! [`yamux`]: https://github.com/hashicorp/yamux/blob/master/spec.md

use crate::{
    multiplexing::StreamMultiplexer,
    negotiate::{negotiate_inbound, negotiate_outbound_interactive},
    transport::ConnectionOrigin,
};
use futures::{
    compat::{Compat, Compat01As03},
    future::{self, Future},
    io::{AsyncRead, AsyncWrite},
    stream::Stream,
};
use futures_01::future::poll_fn as poll_fn_01;
use std::{
    fmt::Debug,
    io,
    pin::Pin,
    task::{Context, Poll},
};
use yamux;

/// Re-export `Mode` from the yamux crate
pub use yamux::Mode;

/// The substream type produced by the `Yamux` multiplexer
pub type StreamHandle<T> = Compat01As03<yamux::StreamHandle<Compat<T>>>;

const YAMUX_PROTOCOL_NAME: &[u8] = b"/yamux/1.0.0";

#[derive(Debug)]
pub struct Yamux<TSocket> {
    inner: yamux::Connection<Compat<TSocket>>,
}

// Receive window is kept at 16MB. This ensures that the window size is large enough to accomodate
// the full RPC request or response which is sent as a framed message using length delimited codec.
// The default LengthDelimitedCodec accomodates a message of size 8 MiB and the frame header.
const RECEIVE_WINDOW: u32 = 16 * 1024 * 1024; // 16 MiB
const MAX_BUFFER_SIZE: usize = 32 * 1024 * 1024; // 32 MiB
const MAX_CONCURRENT_STREAMS: usize = 100;

impl<TSocket> Yamux<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + Debug + Unpin,
{
    pub fn new(socket: TSocket, mode: Mode) -> Self {
        let mut config = yamux::Config::default();
        // OnReceive window update mode ensures that sender is unblocked as long as we have space
        // in our receive buffer to accept more data.
        config.set_window_update_mode(yamux::WindowUpdateMode::OnReceive);
        config.set_max_num_streams(MAX_CONCURRENT_STREAMS);
        config.set_max_buffer_size(MAX_BUFFER_SIZE);
        config
            .set_receive_window(RECEIVE_WINDOW)
            .expect("Invalid receive window size");
        let socket = Compat::new(socket);
        Self {
            inner: yamux::Connection::new(socket, config, mode),
        }
    }

    pub async fn upgrade_connection(socket: TSocket, origin: ConnectionOrigin) -> io::Result<Self> {
        // Perform protocol negotiation
        let (socket, proto) = match origin {
            ConnectionOrigin::Inbound => negotiate_inbound(socket, [YAMUX_PROTOCOL_NAME]).await?,
            ConnectionOrigin::Outbound => {
                negotiate_outbound_interactive(socket, [YAMUX_PROTOCOL_NAME]).await?
            }
        };

        assert_eq!(proto, YAMUX_PROTOCOL_NAME);

        let mode = match origin {
            ConnectionOrigin::Inbound => Mode::Server,
            ConnectionOrigin::Outbound => Mode::Client,
        };

        Ok(Yamux::new(socket, mode))
    }
}

impl<TSocket> StreamMultiplexer for Yamux<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + Debug + Unpin,
{
    type Substream = StreamHandle<TSocket>;
    type Listener = Listener<TSocket>;
    type Outbound = future::Ready<io::Result<Self::Substream>>;
    type Close = Close<TSocket>;

    fn listen_for_inbound(&self) -> Self::Listener {
        Listener::new(self.inner.clone())
    }

    fn open_outbound(&self) -> Self::Outbound {
        let output = match self.inner.open_stream() {
            Ok(Some(substream)) => Ok(Compat01As03::new(substream)),
            Ok(None) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unable to open substream; underlying connection is dead",
            )),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        };

        future::ready(output)
    }

    fn close(&self) -> Self::Close {
        Close::new(self.inner.clone())
    }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct Close<TSocket> {
    inner: yamux::Connection<Compat<TSocket>>,
}

impl<TSocket> Close<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(connection: yamux::Connection<Compat<TSocket>>) -> Self {
        Self { inner: connection }
    }
}

impl<TSocket> Future for Close<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Unpin,
{
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let mut close_fut = Compat01As03::new(poll_fn_01(|| {
            self.inner
                .close()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        }));
        Pin::new(&mut close_fut).poll(context)
    }
}

#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Listener<TSocket> {
    inner: Compat01As03<yamux::Connection<Compat<TSocket>>>,
}

impl<TSocket> Listener<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(connection: yamux::Connection<Compat<TSocket>>) -> Self {
        Self {
            inner: Compat01As03::new(connection),
        }
    }
}

impl<TSocket> Stream for Listener<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Unpin,
{
    type Item = io::Result<Compat01As03<yamux::StreamHandle<Compat<TSocket>>>>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Err(io::Error::new(io::ErrorKind::Other, e))))
            }
            Poll::Ready(Some(Ok(substream))) => Poll::Ready(Some(Ok(Compat01As03::new(substream)))),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::multiplexing::{
        yamux::{Mode, Yamux},
        StreamMultiplexer,
    };
    use futures::{
        executor::block_on,
        future::join,
        io::{AsyncReadExt, AsyncWriteExt},
        stream::StreamExt,
    };
    use memsocket::MemorySocket;
    use std::io;

    #[test]
    fn substream_within_substream() -> io::Result<()> {
        let (dialer, listener) = MemorySocket::new_pair();
        let msg = b"Too fast too furious";
        let msg2 = b"Mission Impossible";

        let dialer = async move {
            let muxer = Yamux::new(dialer, Mode::Client);
            // Open outer substream.
            let substream = muxer.open_outbound().await?;
            // Create multiplexer over substream.
            let muxer = Yamux::new(substream, Mode::Client);
            // Open new inner outbound substream over outer substream.
            let mut substream = muxer.open_outbound().await?;
            // Send data over inner substream.
            substream.write_all(msg).await?;
            substream.flush().await?;
            // We have to close the substream to unblock the read end if it uses `read_to_end`.
            substream.close().await?;
            // Listen for new inbound inner substreams over outer substream.
            let (maybe_substream, _) = muxer.listen_for_inbound().into_future().await;
            let mut substream = maybe_substream
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no substream"))??;
            // Receive data over inner substream.
            let mut buf = Vec::new();
            substream.read_to_end(&mut buf).await?;
            assert_eq!(buf, msg2);
            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let listener = async move {
            let muxer = Yamux::new(listener, Mode::Server);
            // Listen for inbound outer substream.
            let (maybe_substream, _) = muxer.listen_for_inbound().into_future().await;
            let substream = maybe_substream
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no substream"))??;
            // Create multiplexer over substream.
            let muxer = Yamux::new(substream, Mode::Server);
            // Listen for inbound inner substream over outer substream.
            let (maybe_substream, _listener) = muxer.listen_for_inbound().into_future().await;
            let mut substream = maybe_substream
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no substream"))??;
            // Receive data over inner substream.
            let mut buf = Vec::new();
            substream.read_to_end(&mut buf).await?;
            assert_eq!(buf, msg);
            // Open new inner outbound substream over outer substream.
            let mut substream = muxer.open_outbound().await?;
            // Send data over inner substream.
            substream.write_all(msg2).await?;
            substream.flush().await?;
            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let (dialer_result, listener_result) = block_on(join(dialer, listener));
        dialer_result?;
        listener_result?;
        Ok(())
    }

    #[test]
    fn open_substream() -> io::Result<()> {
        let (dialer, listener) = MemorySocket::new_pair();
        let msg = b"The Way of Kings";

        let dialer = async move {
            let muxer = Yamux::new(dialer, Mode::Client);

            let mut substream = muxer.open_outbound().await?;

            substream.write_all(msg).await?;
            substream.flush().await?;

            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let listener = async move {
            let muxer = Yamux::new(listener, Mode::Server);

            let (maybe_substream, _listener) = muxer.listen_for_inbound().into_future().await;
            let mut substream = maybe_substream
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no substream"))??;

            let mut buf = Vec::new();
            substream.read_to_end(&mut buf).await?;
            assert_eq!(buf, msg);

            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let (dialer_result, listener_result) = block_on(join(dialer, listener));
        dialer_result?;
        listener_result?;
        Ok(())
    }

    #[test]
    fn close() -> io::Result<()> {
        let (dialer, listener) = MemorySocket::new_pair();
        let msg = b"Words of Radiance";

        let dialer = async move {
            let muxer = Yamux::new(dialer, Mode::Client);

            let mut substream = muxer.open_outbound().await?;

            substream.write_all(msg).await?;
            substream.flush().await?;

            let mut buf = Vec::new();
            substream.read_to_end(&mut buf).await?;
            assert_eq!(buf, b"");

            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let listener = async move {
            let muxer = Yamux::new(listener, Mode::Server);

            let (maybe_substream, _listener) = muxer.listen_for_inbound().into_future().await;
            let mut substream = maybe_substream
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no substream"))??;

            let mut buf = vec![0; msg.len()];
            substream.read_exact(&mut buf).await?;
            assert_eq!(buf, msg);

            // Close the muxer and then try to write to it
            muxer.close().await?;

            let result = substream.write_all(b"ignored message").await;
            match result {
                Ok(()) => panic!("Write should have failed"),
                Err(e) => assert_eq!(e.kind(), io::ErrorKind::WriteZero),
            }

            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let (dialer_result, listener_result) = block_on(join(dialer, listener));
        dialer_result?;
        listener_result?;
        Ok(())
    }

    #[test]
    fn send_big_message() -> io::Result<()> {
        #[allow(non_snake_case)]
        let MiB: usize = 1 << 20;
        let msg_len = 16 * MiB;

        let (dialer, listener) = MemorySocket::new_pair();

        let dialer = async move {
            let muxer = Yamux::new(dialer, Mode::Client);
            let mut substream = muxer.open_outbound().await?;

            let msg = vec![0x55u8; msg_len];
            substream.write_all(msg.as_slice()).await?;

            let mut buf = vec![0u8; msg_len];
            substream.read_exact(&mut buf).await?;
            substream.close().await?;

            assert_eq!(buf.len(), msg_len);
            assert_eq!(buf, vec![0xAAu8; msg_len]);

            // Force return type of the async block
            let result: io::Result<Yamux<_>> = Ok(muxer);
            result
        };

        let listener = async move {
            let muxer = Yamux::new(listener, Mode::Server);
            let (maybe_substream, _listener) = muxer.listen_for_inbound().into_future().await;
            let mut substream = maybe_substream
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no substream"))??;

            let mut buf = vec![0u8; msg_len];
            substream.read_exact(&mut buf).await?;
            assert_eq!(buf, vec![0x55u8; msg_len]);

            let msg = vec![0xAAu8; msg_len];
            substream.write_all(msg.as_slice()).await?;
            substream.close().await?;

            // Force return type of the async block
            let result: io::Result<Yamux<_>> = Ok(muxer);
            result
        };

        let (dialer_result, listener_result) = block_on(join(dialer, listener));
        let _ = dialer_result?;
        let _ = listener_result?;
        Ok(())
    }
}
