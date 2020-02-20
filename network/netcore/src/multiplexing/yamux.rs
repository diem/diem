// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of [`StreamMultiplexer`] using the [`yamux`] protocol over TCP
//!
//! [`StreamMultiplexer`]: crate::multiplexing::StreamMultiplexer
//! [`yamux`]: https://github.com/hashicorp/yamux/blob/master/spec.md

use crate::{
    multiplexing::{Control, StreamMultiplexer},
    negotiate::{negotiate_inbound, negotiate_outbound_interactive},
    transport::ConnectionOrigin,
};
use async_trait::async_trait;
use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    io::{AsyncRead, AsyncWrite},
    sink::SinkExt,
    stream::Stream,
};
use std::{
    fmt::Debug,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// Re-export `Mode` and `Stream` from the yamux crate
pub use yamux::Mode;
pub use yamux::Stream as StreamHandle;

const YAMUX_PROTOCOL_NAME: &[u8] = b"/yamux/1.0.0";

#[derive(Debug)]
pub struct Yamux<TSocket> {
    connection: yamux::Connection<TSocket>,
}

const MAX_BUFFER_SIZE: u32 = 8 * 1024 * 1024; // 8MB
const RECEIVE_WINDOW: u32 = 4 * 1024 * 1024; // 4MB

impl<TSocket> Yamux<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + Debug + Unpin + 'static,
{
    pub fn new(socket: TSocket, mode: Mode) -> Self {
        let mut config = yamux::Config::default();
        // Use OnRead mode instead of OnReceive mode to provide back pressure to the sending side.
        // Caveat: the OnRead mode has the risk of deadlock, where both sides send data larger than
        // receive window and don't read before finishing writes. But it doesn't apply to our use
        // cases. Some of our streams are unidirectional, where only one end writes data, e.g.,
        // Direct Send. Some of our streams are bidirectional, but only one end write data at a
        // time, e.g., RPC. Some of our streams may write at the same time, but the frames are
        // shorter than the receive window, e.g., protocol negotiation.
        config.set_window_update_mode(yamux::WindowUpdateMode::OnRead);
        // Because OnRead mode increases the RTT of window update, bigger buffer size and receive
        // window size perform better.
        config.set_max_buffer_size(MAX_BUFFER_SIZE as usize);
        config.set_receive_window(RECEIVE_WINDOW);
        let connection = yamux::Connection::new(socket, config, mode);
        Self { connection }
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

#[derive(Clone)]
pub struct YamuxControl {
    inner: yamux::Control,
    _drop_tx: Arc<oneshot::Sender<()>>,
}

#[async_trait]
impl Control for YamuxControl {
    type Substream = StreamHandle;

    async fn open_stream(&mut self) -> io::Result<Self::Substream> {
        self.inner
            .open_stream()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))
    }

    async fn close(&mut self) -> io::Result<()> {
        self.inner
            .close()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

pub struct StreamListener<Substream: Send> {
    listener: mpsc::UnboundedReceiver<io::Result<Substream>>,
    _drop_tx: Arc<oneshot::Sender<()>>,
}

impl<Substream> Stream for StreamListener<Substream>
where
    Substream: Send,
{
    type Item = io::Result<Substream>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.listener).poll_next(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Err(io::Error::new(io::ErrorKind::Other, e))))
            }
            Poll::Ready(Some(Ok(substream))) => Poll::Ready(Some(Ok(substream))),
        }
    }
}

#[async_trait]
impl<TSocket> StreamMultiplexer for Yamux<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + Debug + Unpin + Sync + 'static,
{
    type Substream = StreamHandle;
    type Control = YamuxControl;
    type Listener = StreamListener<Self::Substream>;

    async fn start(self) -> (Self::Listener, Self::Control) {
        let control = self.connection.control();
        let (mut tx, rx) = mpsc::unbounded();
        // Our library wraps calls to `next_stream` in a future and runs the future on the current
        // tokio runtime. That way, users of our yamux library don't need to worry about the
        // unintuitive need for polling for new inbound substreams even if they only need to open
        // outbound substreams or perform IO on existing susbstreams.

        // We create a oneshot channel to decide when to forcefully terminate the connection. This
        // is triggered if the user drops both the Listener and Control handles returned by this
        // function.
        let (drop_tx, drop_rx): (oneshot::Sender<()>, _) = oneshot::channel();
        let drop_tx = Arc::new(drop_tx);
        let stream_listener = async move {
            let mut drop_rx = drop_rx.fuse();
            let mut connection = self.connection;
            loop {
                // We use `select_biased` to ensure all inbound messages are processed before we
                // close our end of the connection.
                futures::select_biased! {
                  maybe_substream = connection.next_stream().fuse() => {
                    let maybe_substream = maybe_substream.transpose();
                    match maybe_substream {
                        None | Some(Err(_)) => {
                            // Ignore error.
                            let _ = tx
                                .send(Err(io::Error::new(
                                    io::ErrorKind::BrokenPipe,
                                    "Connection closed",
                                )))
                                .await;
                            return;
                        }
                        Some(Ok(substream)) => {
                            // Failure to notify about new inbound substream could happen because
                            // our end of the connection is not interested in receiving inbound
                            // substreams. Instead of breaking out and forcefully dropping the
                            // connection, we simply respect the wishes of the client and fail
                            // silently.
                            // TODO: Add logging through a logging facade.
                            let _ = tx.send(Ok(substream)).await;
                        }
                    }
                  },
                  _ = drop_rx => {
                    // Both listener and control handles must have been dropped if we have reached
                    // here. In this case, we forcefully terminate the connection by dropping.
                    drop(connection);
                    return;
                  }
                }
            }
        };
        // Run the IO future on the executor which drives this connection.
        tokio::spawn(stream_listener);
        (
            StreamListener {
                listener: rx,
                _drop_tx: drop_tx.clone(),
            },
            YamuxControl {
                inner: control,
                _drop_tx: drop_tx,
            },
        )
    }
}

#[cfg(test)]
mod test {
    use crate::multiplexing::{
        yamux::{Mode, Yamux},
        Control, StreamMultiplexer,
    };
    use futures::{
        future::join,
        io::{AsyncReadExt, AsyncWriteExt},
        stream::StreamExt,
    };
    use libra_logger::prelude::*;
    use memsocket::MemorySocket;
    use std::io;

    #[tokio::test]
    async fn substream_within_substream() -> io::Result<()> {
        let (dialer, listener) = MemorySocket::new_pair();
        let msg = b"Too fast too furious";
        let msg2 = b"Mission Impossible";

        let dialer = async move {
            let muxer = Yamux::new(dialer, Mode::Client);
            let (_listener_outer, mut control_outer) = muxer.start().await;
            // Open outer substream.
            let substream = control_outer.open_stream().await?;
            // Create multiplexer over substream.
            let muxer = Yamux::new(substream, Mode::Client);
            let (mut listener_inner, mut control_inner) = muxer.start().await;
            // Open new inner outbound substream over outer substream.
            let mut substream = control_inner.open_stream().await?;
            // Send data over inner substream.
            substream.write_all(msg).await?;
            substream.flush().await?;
            // We have to close the substream to unblock the read end if it uses `read_to_end`.
            substream.close().await?;
            // Listen for new inbound inner substreams over outer substream.
            let mut substream = listener_inner
                .next()
                .await
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
            let (mut listener_outer, _control_outer) = muxer.start().await;
            // Listen for inbound outer substream.
            let substream = listener_outer
                .next()
                .await
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no substream"))??;
            // Create multiplexer over substream.
            let muxer = Yamux::new(substream, Mode::Server);
            let (mut listener_inner, mut control_inner) = muxer.start().await;
            // Listen for inbound inner substream over outer substream.
            let mut substream = listener_inner
                .next()
                .await
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no substream"))??;
            // Receive data over inner substream.
            let mut buf = Vec::new();
            substream.read_to_end(&mut buf).await?;
            assert_eq!(buf, msg);
            // Open new inner outbound substream over outer substream.
            let mut substream = control_inner.open_stream().await?;
            // Send data over inner substream.
            substream.write_all(msg2).await?;
            substream.flush().await?;
            // We have to close the substream to unblock the read end if it uses `read_to_end`.
            substream.close().await?;
            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let (dialer_result, listener_result) = join(dialer, listener).await;
        dialer_result?;
        listener_result?;
        Ok(())
    }

    #[tokio::test]
    async fn open_substream() -> io::Result<()> {
        ::libra_logger::try_init_for_testing();
        let (dialer, listener) = MemorySocket::new_pair();
        let msg = b"The Way of Kings";

        let dialer = async move {
            let muxer = Yamux::new(dialer, Mode::Client);
            let (_stream_listener, mut control) = muxer.start().await;

            info!("[dialer] Opening outbound substream");
            let mut substream = control.open_stream().await?;
            info!("[dialer] Opened outbound substream");

            substream.write_all(msg).await?;
            substream.flush().await?;
            // We have to close the substream to unblock the read end if it uses `read_to_end`.
            substream.close().await?;
            info!("[dialer] Done writing message");

            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let listener = async move {
            let muxer = Yamux::new(listener, Mode::Server);
            let (mut stream_listener, _control) = muxer.start().await;

            info!("[listener] Listening for inbound substream");
            let mut substream = stream_listener
                .next()
                .await
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no substream"))??;
            info!("[listener] Opened inbound substream");

            let mut buf = Vec::new();
            substream.read_to_end(&mut buf).await?;
            assert_eq!(buf, msg);
            info!("[listneer] Done reading message");

            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let (dialer_result, listener_result) = join(dialer, listener).await;
        dialer_result?;
        listener_result?;
        Ok(())
    }

    #[tokio::test]
    async fn close() -> io::Result<()> {
        ::libra_logger::try_init_for_testing();
        let (dialer, listener) = MemorySocket::new_pair();
        let msg = b"Words of Radiance";

        let dialer = async move {
            let muxer = Yamux::new(dialer, Mode::Client);
            let (_stream_listener, mut control) = muxer.start().await;

            info!("[dialer] Opening outbound substream");
            let mut substream = control.open_stream().await.map_err(|e| {
                error!("{:?}", e);
                e
            })?;
            info!("[dialer] Opened outbound substream");

            substream.write_all(msg).await?;
            substream.flush().await?;
            // We have to close the substream to unblock the read end if it uses `read_to_end`.
            substream.close().await?;
            info!("[dialer] Done writing message");

            let mut buf = Vec::new();
            substream.read_to_end(&mut buf).await?;
            info!("[dialer] Done reading empty message");
            assert_eq!(buf, b"");

            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let listener = async move {
            let muxer = Yamux::new(listener, Mode::Server);
            let (mut stream_listener, mut control) = muxer.start().await;

            info!("[listener] Listening for inbound substream");
            let mut substream = stream_listener
                .next()
                .await
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no substream"))??;
            info!("[listener] Opened inbound substream");

            let mut buf = vec![0; msg.len()];
            substream.read_exact(&mut buf).await?;
            assert_eq!(buf, msg);
            info!("[listneer] Done reading message");

            // Close the muxer and then try to write to it
            control.close().await?;
            info!("[listener] Done closing connection");

            let result = substream.write_all(b"ignored message").await;
            match result {
                Ok(()) => panic!("Write should have failed"),
                Err(e) => assert_eq!(e.kind(), io::ErrorKind::WriteZero),
            }

            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let (dialer_result, listener_result) = join(dialer, listener).await;
        dialer_result?;
        listener_result?;
        Ok(())
    }

    #[tokio::test]
    async fn close_connection() -> io::Result<()> {
        ::libra_logger::try_init_for_testing();
        let (dialer, listener) = MemorySocket::new_pair();

        let dialer = async move {
            let muxer = Yamux::new(dialer, Mode::Client);
            let (_stream_listener, mut control) = muxer.start().await;
            control.close().await.map_err(|e| {
                error!("{:?}", e);
                e
            })?;
            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let listener = async move {
            let muxer = Yamux::new(listener, Mode::Server);
            let (mut stream_listener, _control) = muxer.start().await;
            assert!(stream_listener.next().await.unwrap().is_err());
            let result: io::Result<()> = Ok(());
            result
        };

        let (dialer_result, listener_result) = join(dialer, listener).await;
        dialer_result?;
        listener_result?;
        Ok(())
    }

    #[tokio::test]
    async fn drop_connection() -> io::Result<()> {
        let (dialer, listener) = MemorySocket::new_pair();
        let msg = b"Words of Radiance";

        let dialer = async move {
            let muxer = Yamux::new(dialer, Mode::Client);
            // Dropping the listener. We should still be able to open outbound substreams.
            let (_, mut control) = muxer.start().await;
            let mut substream = control.open_stream().await.map_err(|e| {
                error!("{:?}", e);
                e
            })?;
            substream.write_all(msg).await?;
            substream.flush().await?;
            // We have to close the substream to unblock the read end if it uses `read_to_end`.
            substream.close().await?;
            // Drop the control. This should forcefully terminate the connection.
            drop(control);
            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let listener = async move {
            let muxer = Yamux::new(listener, Mode::Server);
            let (mut stream_listener, _control) = muxer.start().await;
            let mut substream = stream_listener
                .next()
                .await
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no substream"))??;
            let mut buf = Vec::new();
            substream.read_to_end(&mut buf).await?;
            assert_eq!(buf, msg);
            assert!(stream_listener.next().await.unwrap().is_err());
            let result: io::Result<()> = Ok(());
            result
        };
        let (dialer_result, listener_result) = join(dialer, listener).await;
        dialer_result?;
        listener_result?;
        Ok(())
    }

    #[tokio::test]
    async fn send_big_message() -> io::Result<()> {
        #[allow(non_snake_case)]
        let MiB: usize = 1 << 20;
        let msg_len = 16 * MiB;

        let (dialer, listener) = MemorySocket::new_pair();

        let dialer = async move {
            let muxer = Yamux::new(dialer, Mode::Client);
            let (_stream_listener, mut control) = muxer.start().await;
            let mut substream = control.open_stream().await?;

            let msg = vec![0x55u8; msg_len];
            substream.write_all(msg.as_slice()).await?;

            let mut buf = vec![0u8; msg_len];
            substream.read_exact(&mut buf).await?;
            substream.close().await?;

            assert_eq!(buf.len(), msg_len);
            assert_eq!(buf, vec![0xAAu8; msg_len]);

            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let listener = async move {
            let muxer = Yamux::new(listener, Mode::Server);
            let (mut stream_listener, _control) = muxer.start().await;
            let mut substream = stream_listener
                .next()
                .await
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no substream"))??;

            let mut buf = vec![0u8; msg_len];
            substream.read_exact(&mut buf).await?;
            assert_eq!(buf, vec![0x55u8; msg_len]);

            let msg = vec![0xAAu8; msg_len];
            substream.write_all(msg.as_slice()).await?;
            substream.close().await?;

            // Force return type of the async block
            let result: io::Result<()> = Ok(());
            result
        };

        let (dialer_result, listener_result) = join(dialer, listener).await;
        dialer_result?;
        listener_result?;
        Ok(())
    }
}
