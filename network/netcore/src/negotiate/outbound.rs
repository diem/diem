// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    framing::{read_u16frame, write_u16frame},
    negotiate::{PROTOCOL_INTERACTIVE, PROTOCOL_NOT_SUPPORTED, PROTOCOL_SELECT},
};
use bytes::BytesMut;
use futures::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use std::io::Result;

/// Perform protocol negotiation on an outbound `stream` using interactive
/// negotiation, waiting for the remote end to send back and ACK on the agreed
/// upon protocol. The protocols provided in `supported_protocols` are tried in
/// order, preferring protocols first in the provided list.
pub async fn negotiate_outbound_interactive<TSocket, TProto, TProtocols>(
    mut stream: TSocket,
    supported_protocols: TProtocols,
) -> Result<(TSocket, TProto)>
where
    TSocket: AsyncRead + AsyncWrite + Unpin,
    TProto: AsRef<[u8]> + Clone,
    TProtocols: AsRef<[TProto]>,
{
    write_u16frame(&mut stream, PROTOCOL_INTERACTIVE).await?;

    let mut buf = BytesMut::new();
    let mut received_header_ack = false;
    for proto in supported_protocols.as_ref() {
        write_u16frame(&mut stream, proto.as_ref()).await?;
        stream.flush().await?;

        // Read the ACK that we're speaking PROTOCOL_INTERACTIVE if we still haven't done so.
        // Note that we do this after sending the first protocol id, allowing for the negotiation to
        // happen in a single round trip in case the remote node indeed speaks our preferred
        // protocol.
        if !received_header_ack {
            read_u16frame(&mut stream, &mut buf).await?;
            if buf.as_ref() != PROTOCOL_INTERACTIVE {
                // Remote side doesn't understand us, give up
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unable to negotiate protocol - PROTOCOL_INTERACTIVE not supported",
                ));
            }

            received_header_ack = true;
        }

        read_u16frame(&mut stream, &mut buf).await?;

        if buf.as_ref() == proto.as_ref() {
            // We received an ACK on the protocol!
            return Ok((stream, proto.clone()));
        } else if buf.as_ref() != PROTOCOL_NOT_SUPPORTED {
            // We received an unexpected message from the remote, give up
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unable to negotiate protocol - unexpected interactive response",
            ));
        }
    }

    // We weren't able to find a matching protocol, give up
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Unable to negotiate protocol - no matching protocol",
    ))
}

/// Perform an optimistic protocol negotiation on `stream` using the provided
/// `protocol`.
///
/// The negotiation frames are only enqueued and not yet flushed (assuming the
/// underlying transport is buffered). It's up to the protocol that handles this
/// new outbound substream to decide when it should flush these frames.
pub async fn negotiate_outbound_select<TSocket, TProto>(
    mut stream: TSocket,
    protocol: TProto,
) -> Result<TSocket>
where
    TSocket: AsyncWrite + Unpin,
    TProto: AsRef<[u8]> + Clone,
{
    write_u16frame(&mut stream, PROTOCOL_SELECT).await?;
    // Write out the protocol we're optimistically selecting and return
    write_u16frame(&mut stream, protocol.as_ref()).await?;
    // We do not wait for any ACK from the listener. This is OK because in case the listener does
    // not want to speak this protocol, it can simply close the stream leading to an upstream
    // failure in the dialer when it tries to read/write to the stream.
    Ok(stream)
}

#[cfg(test)]
mod test {
    use crate::{
        framing::{read_u16frame, write_u16frame},
        negotiate::{
            outbound::{negotiate_outbound_interactive, negotiate_outbound_select},
            PROTOCOL_INTERACTIVE, PROTOCOL_NOT_SUPPORTED, PROTOCOL_SELECT,
        },
    };
    use bytes::BytesMut;
    use futures::{executor::block_on, future::join, io::AsyncWriteExt};
    use memsocket::MemorySocket;
    use std::io::Result;

    #[test]
    fn test_negotiate_outbound_interactive() -> Result<()> {
        let (a, mut b) = MemorySocket::new_pair();
        let test_protocol = b"/hello/1.0.0";

        let outbound = async move {
            let (_stream, proto) = negotiate_outbound_interactive(a, [test_protocol]).await?;

            assert_eq!(proto, test_protocol);

            // Force return type of the async block
            let result: Result<()> = Ok(());
            result
        };

        let inbound = async move {
            let mut buf = BytesMut::new();
            read_u16frame(&mut b, &mut buf).await?;
            assert_eq!(buf.as_ref(), PROTOCOL_INTERACTIVE);
            read_u16frame(&mut b, &mut buf).await?;
            assert_eq!(buf.as_ref(), test_protocol);

            write_u16frame(&mut b, PROTOCOL_INTERACTIVE).await?;
            write_u16frame(&mut b, test_protocol).await?;
            b.flush().await?;

            // Force return type of the async block
            let result: Result<()> = Ok(());
            result
        };

        let (result_outbound, result_inbound) = block_on(join(outbound, inbound));
        assert_eq!(result_outbound.is_ok(), true);
        assert_eq!(result_inbound.is_ok(), true);

        Ok(())
    }

    #[test]
    fn test_negotiate_outbound_interactive_unsupported() -> Result<()> {
        let (a, mut b) = MemorySocket::new_pair();
        let protocol_supported = b"/hello/1.0.0";
        let protocol_unsupported = b"/hello/2.0.0";

        let outbound = async move {
            let (_stream, proto) =
                negotiate_outbound_interactive(a, [protocol_unsupported, protocol_supported])
                    .await?;

            assert_eq!(proto, protocol_supported);

            // Force return type of the async block
            let result: Result<()> = Ok(());
            result
        };

        let inbound = async move {
            let mut buf = BytesMut::new();
            read_u16frame(&mut b, &mut buf).await?;
            assert_eq!(buf.as_ref(), PROTOCOL_INTERACTIVE);
            read_u16frame(&mut b, &mut buf).await?;
            assert_eq!(buf.as_ref(), protocol_unsupported);

            write_u16frame(&mut b, PROTOCOL_INTERACTIVE).await?;
            write_u16frame(&mut b, PROTOCOL_NOT_SUPPORTED).await?;
            b.flush().await?;

            read_u16frame(&mut b, &mut buf).await?;
            assert_eq!(buf.as_ref(), protocol_supported);

            write_u16frame(&mut b, protocol_supported).await?;
            b.flush().await?;

            // Force return type of the async block
            let result: Result<()> = Ok(());
            result
        };

        let (result_outbound, result_inbound) = block_on(join(outbound, inbound));
        assert_eq!(result_outbound.is_ok(), true);
        assert_eq!(result_inbound.is_ok(), true);

        Ok(())
    }

    #[test]
    fn test_negotiate_outbound_select() -> Result<()> {
        let (a, mut b) = MemorySocket::new_pair();
        let test_protocol = b"/hello/1.0.0";
        let hello_request = b"Hello World!";

        let outbound = async move {
            let mut stream = negotiate_outbound_select(a, test_protocol).await?;

            write_u16frame(&mut stream, hello_request).await?;
            stream.flush().await?;

            // Force return type of the async block
            let result: Result<()> = Ok(());
            result
        };

        let inbound = async move {
            let mut buf = BytesMut::new();
            read_u16frame(&mut b, &mut buf).await?;
            assert_eq!(buf.as_ref(), PROTOCOL_SELECT);
            read_u16frame(&mut b, &mut buf).await?;
            assert_eq!(buf.as_ref(), test_protocol);

            read_u16frame(&mut b, &mut buf).await?;
            assert_eq!(buf.as_ref(), hello_request);

            // Force return type of the async block
            let result: Result<()> = Ok(());
            result
        };

        let (result_outbound, result_inbound) = block_on(join(outbound, inbound));
        assert!(result_outbound.is_ok());
        assert!(result_inbound.is_ok());

        Ok(())
    }

    #[test]
    fn test_negotiate_outbound_select_unsupported() -> Result<()> {
        let (a, mut b) = MemorySocket::new_pair();
        let protocol_unsupported = b"/hello/2.0.0";

        let outbound = async move {
            let mut stream = negotiate_outbound_select(a, protocol_unsupported).await?;
            stream.flush().await?;

            let mut buf = BytesMut::new();
            read_u16frame(&mut stream, &mut buf).await
        };

        let inbound = async move {
            let mut buf = BytesMut::new();
            read_u16frame(&mut b, &mut buf).await?;
            assert_eq!(buf.as_ref(), PROTOCOL_SELECT);
            read_u16frame(&mut b, &mut buf).await?;
            assert_eq!(buf.as_ref(), protocol_unsupported);

            // Just drop b to signal that the upgrade failed
            drop(b);

            // Force return type of the async block
            let result: Result<()> = Ok(());
            result
        };

        let (result_outbound, result_inbound) = block_on(join(outbound, inbound));
        assert_eq!(result_outbound.is_err(), true);
        assert_eq!(result_inbound.is_ok(), true);

        Ok(())
    }
}
