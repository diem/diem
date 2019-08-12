// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Simple transport used to identify the PeerId of a remote
//
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use netcore::{
    negotiate::{negotiate_inbound, negotiate_outbound_interactive},
    transport::ConnectionOrigin,
};
use std::{convert::TryInto, io};
use types::PeerId;

const PEER_ID_EXCHANGE_PROTOCOL_NAME: &[u8] = b"/peer_id_exchange/0.1.0";

/// The protocol upgrade configuration.
#[derive(Clone)]
pub struct PeerIdExchange(PeerId);

/// A simple PeerID exchange protocol
///
/// A PeerID is sent from each side in the form of:
///     <u8 length><[u8] PeerId>
// TODO change to using u16 framing
impl PeerIdExchange {
    pub fn new(peer_id: PeerId) -> Self {
        Self(peer_id)
    }

    pub async fn exchange_peer_id<TSocket>(
        self,
        socket: TSocket,
        origin: ConnectionOrigin,
    ) -> io::Result<(PeerId, TSocket)>
    where
        TSocket: AsyncRead + AsyncWrite + Unpin,
    {
        // Perform protocol negotiation
        let (mut socket, proto) = match origin {
            ConnectionOrigin::Inbound => {
                negotiate_inbound(socket, [PEER_ID_EXCHANGE_PROTOCOL_NAME]).await?
            }
            ConnectionOrigin::Outbound => {
                negotiate_outbound_interactive(socket, [PEER_ID_EXCHANGE_PROTOCOL_NAME]).await?
            }
        };

        assert_eq!(proto, PEER_ID_EXCHANGE_PROTOCOL_NAME);

        // Now exchange your PeerIds
        let mut buf: Vec<u8> = self.0.into();
        let buf_len = buf.len();
        assert_eq!(buf_len, 32);
        let buf_len = buf_len as u8;
        buf.insert(0, buf_len);

        socket.write_all(&buf).await?;
        socket.flush().await?;
        socket.read_exact(&mut buf[0..1]).await?;
        let len = buf[0] as usize;
        buf.resize(len, 0);

        socket.read_exact(&mut buf).await?;

        Ok((buf.try_into().expect("Invalid PeerId"), socket))
    }
}

#[cfg(test)]
mod tests {
    use super::PeerIdExchange;
    use futures::{
        executor::block_on,
        future::join,
        io::{AsyncReadExt, AsyncWriteExt},
        stream::StreamExt,
    };
    use memsocket::MemorySocket;
    use netcore::transport::{
        boxed::BoxedTransport, memory::MemoryTransport, Transport, TransportExt,
    };
    use types::PeerId;

    // Build an unsecure transport
    fn test_transport(peer_id: PeerId) -> BoxedTransport<(PeerId, MemorySocket), ::std::io::Error> {
        let transport = MemoryTransport::default();
        let peer_identifier_config = PeerIdExchange::new(peer_id);

        transport
            .and_then(move |socket, origin| peer_identifier_config.exchange_peer_id(socket, origin))
            .boxed()
    }

    #[test]
    fn peer_identifier() {
        let peer_a = PeerId::random();
        let peer_b = PeerId::random();

        // Create the listener
        let (mut listener, address) = test_transport(peer_a)
            .listen_on("/memory/0".parse().unwrap())
            .unwrap();

        let server = async move {
            if let Some(result) = listener.next().await {
                let (upgrade, _addr) = result.unwrap();
                let (peer_id, mut socket) = upgrade.await.unwrap();

                assert_eq!(peer_b, peer_id);

                socket.write_all(b"hello world").await.unwrap();
                socket.flush().await.unwrap();
                socket.close().await.unwrap();
            }
        };

        let client = async move {
            let (peer_id, mut socket) =
                test_transport(peer_b).dial(address).unwrap().await.unwrap();

            assert_eq!(peer_a, peer_id);

            let mut buf = Vec::new();
            socket.read_to_end(&mut buf).await.unwrap();
            socket.close().await.unwrap();

            assert_eq!(buf, b"hello world");
        };

        block_on(join(server, client));
    }
}
