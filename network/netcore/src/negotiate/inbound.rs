// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::negotiate::{
    framing::{read_u16frame, write_u16frame},
    PROTOCOL_INTERACTIVE, PROTOCOL_NOT_SUPPORTED, PROTOCOL_SELECT,
};
use bytes::BytesMut;
use futures::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use std::io::Result;

/// Perform protocol negotiation on an inbound `stream` attempting to match
/// against the provided `supported_protocols`. Protocol negotiation is done
/// using either an interactive or optimistic negotiation, selected by the
/// remote end.
pub async fn negotiate_inbound<TSocket, TProto, TProtocols>(
    mut stream: TSocket,
    supported_protocols: TProtocols,
) -> Result<(TSocket, TProto)>
where
    TSocket: AsyncRead + AsyncWrite + Unpin,
    TProto: AsRef<[u8]> + Clone,
    TProtocols: AsRef<[TProto]>,
{
    let mut buf = BytesMut::new();
    read_u16frame(&mut stream, &mut buf).await?;

    if buf.as_ref() == PROTOCOL_INTERACTIVE {
        let selected_proto =
            negotiate_inbound_interactive(&mut stream, supported_protocols, buf).await?;

        Ok((stream, selected_proto))
    } else if buf.as_ref() == PROTOCOL_SELECT {
        let selected_proto =
            negotiate_inbound_select(&mut stream, supported_protocols, buf).await?;

        Ok((stream, selected_proto))
    } else {
        // TODO Maybe have our own Error type here?
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unable to negotiate protocol - unexpected inbound message",
        ))
    }
}

async fn negotiate_inbound_interactive<'stream, 'c, TSocket, TProto, TProtocols>(
    mut stream: &'stream mut TSocket,
    supported_protocols: TProtocols,
    mut buf: BytesMut,
) -> Result<TProto>
where
    'stream: 'c,
    TSocket: AsyncRead + AsyncWrite + Unpin,
    TProto: AsRef<[u8]> + Clone,
    TProtocols: AsRef<[TProto]> + 'c,
{
    // ACK that we are speaking PROTOCOL_INTERACTIVE
    write_u16frame(&mut stream, PROTOCOL_INTERACTIVE).await?;
    stream.flush().await?;

    // We make up to 10 attempts to negotiate a protocol.
    for _ in 0..10 {
        // Read in the Protocol they want to speak and attempt to match
        // it against our supported protocols
        read_u16frame(&mut stream, &mut buf).await?;
        for proto in supported_protocols.as_ref() {
            // Found a match!
            if buf.as_ref() == proto.as_ref() {
                // Echo back the selected protocol
                write_u16frame(&mut stream, proto.as_ref()).await?;
                stream.flush().await?;
                return Ok(proto.clone());
            }
        }
        // If the desired protocol doesn't match any of our supported
        // ones then send PROTOCOL_NOT_SUPPORTED
        write_u16frame(&mut stream, PROTOCOL_NOT_SUPPORTED).await?;
        stream.flush().await?;
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Unable to negotiate protocol - all attempts failed",
    ))
}

async fn negotiate_inbound_select<'stream, 'c, TSocket, TProto, TProtocols>(
    mut stream: &'stream mut TSocket,
    supported_protocols: TProtocols,
    mut buf: BytesMut,
) -> Result<TProto>
where
    'stream: 'c,
    TSocket: AsyncRead + Unpin,
    TProto: AsRef<[u8]> + Clone,
    TProtocols: AsRef<[TProto]> + 'c,
{
    // Read in the Protocol they want to speak and attempt to match
    // it against our supported protocols
    read_u16frame(&mut stream, &mut buf).await?;
    for proto in supported_protocols.as_ref() {
        // Found a match!
        if buf.as_ref() == proto.as_ref() {
            return Ok(proto.clone());
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Unable to negotiate Protocol - protocol not supported",
    ))
}

#[cfg(test)]
mod test {
    use crate::negotiate::{
        framing::{read_u16frame, write_u16frame},
        inbound::{negotiate_inbound_interactive, negotiate_inbound_select},
        PROTOCOL_INTERACTIVE, PROTOCOL_NOT_SUPPORTED,
    };
    use bytes::BytesMut;
    use futures::{executor::block_on, future::join, io::AsyncWriteExt};
    use memsocket::MemorySocket;
    use std::io::Result;

    #[test]
    fn test_negotiate_inbound_interactive() -> Result<()> {
        let (mut a, mut b) = MemorySocket::new_pair();
        let test_protocol = b"/hello/1.0.0";

        let outbound = async move {
            write_u16frame(&mut a, test_protocol).await?;
            a.flush().await?;

            let mut buf = BytesMut::new();
            read_u16frame(&mut a, &mut buf).await?;
            assert_eq!(buf.as_ref(), PROTOCOL_INTERACTIVE);
            read_u16frame(&mut a, &mut buf).await?;
            assert_eq!(buf.as_ref(), test_protocol);

            // Force return type of the async block
            let result: Result<()> = Ok(());
            result
        };

        let buf = BytesMut::new();
        let inbound = negotiate_inbound_interactive(&mut b, [test_protocol], buf);

        let (result_outbound, result_inbound) = block_on(join(outbound, inbound));
        assert_eq!(result_outbound.is_ok(), true);
        assert_eq!(result_inbound?, test_protocol);

        Ok(())
    }

    #[test]
    fn test_negotiate_inbound_interactive_unsupported() -> Result<()> {
        let (mut a, mut b) = MemorySocket::new_pair();
        let protocol_supported = b"/hello/1.0.0";
        let protocol_unsupported = b"/hello/2.0.0";

        let outbound = async move {
            write_u16frame(&mut a, protocol_unsupported).await?;
            a.flush().await?;

            let mut buf = BytesMut::new();
            read_u16frame(&mut a, &mut buf).await?;
            assert_eq!(buf.as_ref(), PROTOCOL_INTERACTIVE);
            read_u16frame(&mut a, &mut buf).await?;
            assert_eq!(buf.as_ref(), PROTOCOL_NOT_SUPPORTED);

            write_u16frame(&mut a, protocol_supported).await?;
            a.flush().await?;

            read_u16frame(&mut a, &mut buf).await?;
            assert_eq!(buf.as_ref(), protocol_supported);

            // Force return type of the async block
            let result: Result<()> = Ok(());
            result
        };

        let buf = BytesMut::new();
        let inbound = negotiate_inbound_interactive(&mut b, [protocol_supported], buf);

        let (result_outbound, result_inbound) = block_on(join(outbound, inbound));
        assert_eq!(result_outbound.is_ok(), true);
        assert_eq!(result_inbound?, protocol_supported);

        Ok(())
    }

    #[test]
    fn test_negotiate_inbound_select() -> Result<()> {
        let (mut a, mut b) = MemorySocket::new_pair();
        let test_protocol = b"/hello/1.0.0";
        let hello_request = b"Hello World!";

        let outbound = async move {
            write_u16frame(&mut a, test_protocol).await?;
            a.flush().await?;

            let mut buf = BytesMut::new();
            read_u16frame(&mut a, &mut buf).await?;
            assert_eq!(buf.as_ref(), hello_request);

            // Force return type of the async block
            let result: Result<()> = Ok(());
            result
        };

        let inbound = async move {
            let buf = BytesMut::new();
            let selected_proto = negotiate_inbound_select(&mut b, [test_protocol], buf).await?;
            assert_eq!(selected_proto, test_protocol);

            write_u16frame(&mut b, hello_request).await?;
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
    fn test_negotiate_inbound_select_unsupported() -> Result<()> {
        let (mut a, mut b) = MemorySocket::new_pair();
        let protocol_supported = b"/hello/1.0.0";
        let protocol_unsupported = b"/hello/2.0.0";

        let outbound = async move {
            write_u16frame(&mut a, protocol_unsupported).await?;
            a.flush().await?;

            let mut buf = BytesMut::new();
            read_u16frame(&mut a, &mut buf).await
        };

        let inbound = async move {
            let buf = BytesMut::new();
            negotiate_inbound_select(&mut b, [protocol_supported], buf).await
        };

        let (result_outbound, result_inbound) = block_on(join(outbound, inbound));
        assert_eq!(result_outbound.is_err(), true);
        assert_eq!(result_inbound.is_err(), true);

        Ok(())
    }
}
