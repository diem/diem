// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protocol used to identify key information about a remote
//!
//! Currently, the information shared as part of this protocol includes the peer identity and a
//! list of protocols supported by the peer.
use crate::{proto::IdentityMsg, ProtocolId};
use bytes::Bytes;
use futures::{
    compat::{Compat, Sink01CompatExt},
    sink::SinkExt,
    stream::StreamExt,
};
use netcore::{
    multiplexing::StreamMultiplexer,
    negotiate::{negotiate_inbound, negotiate_outbound_interactive},
    transport::ConnectionOrigin,
};
use protobuf::{self, Message};
use std::{convert::TryFrom, io};
use tokio::codec::Framed;
use types::PeerId;
use unsigned_varint::codec::UviBytes;

const IDENTITY_PROTOCOL_NAME: &[u8] = b"/identity/0.1.0";

/// The Identity of a node
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    peer_id: PeerId,
    supported_protocols: Vec<ProtocolId>,
}

impl Identity {
    pub fn new(peer_id: PeerId, supported_protocols: Vec<ProtocolId>) -> Self {
        Self {
            peer_id,
            supported_protocols,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn is_protocol_supported(&self, protocol: &ProtocolId) -> bool {
        self.supported_protocols
            .iter()
            .any(|proto| proto == protocol)
    }

    pub fn supported_protocols(&self) -> &[ProtocolId] {
        &self.supported_protocols
    }
}

/// The Identity exchange protocol
pub async fn exchange_identity<TMuxer>(
    own_identity: &Identity,
    connection: TMuxer,
    origin: ConnectionOrigin,
) -> io::Result<(Identity, TMuxer)>
where
    TMuxer: StreamMultiplexer,
{
    // Perform protocol negotiation on a substream on the connection. The dialer is responsible
    // for opening the substream, while the listener is responsible for listening for that
    // incoming substream.
    let (substream, proto) = match origin {
        ConnectionOrigin::Inbound => {
            let mut listener = connection.listen_for_inbound();
            let substream = listener.next().await.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Connection closed by remote",
                )
            })??;
            negotiate_inbound(substream, [IDENTITY_PROTOCOL_NAME]).await?
        }
        ConnectionOrigin::Outbound => {
            let substream = connection.open_outbound().await?;
            negotiate_outbound_interactive(substream, [IDENTITY_PROTOCOL_NAME]).await?
        }
    };

    assert_eq!(proto, IDENTITY_PROTOCOL_NAME);

    // Create the Framed Sink/Stream
    let mut framed_substream =
        Framed::new(Compat::new(substream), UviBytes::default()).sink_compat();

    // Build Identity Message
    let mut msg = IdentityMsg::new();
    msg.set_supported_protocols(own_identity.supported_protocols().to_vec());
    msg.set_peer_id(own_identity.peer_id().into());

    // Send serialized message to peer.
    let bytes = msg
        .write_to_bytes()
        .expect("writing protobuf failed; should never happen");
    framed_substream.send(Bytes::from(bytes)).await?;
    framed_substream.close().await?;

    // Read an IdentityMsg from the Remote
    let response = framed_substream.next().await.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::ConnectionAborted,
            "Connection closed by remote",
        )
    })??;
    let mut response = ::protobuf::parse_from_bytes::<IdentityMsg>(&response).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse identity msg: {}", e),
        )
    })?;
    let peer_id = PeerId::try_from(response.take_peer_id()).expect("Invalid PeerId");
    let identity = Identity::new(peer_id, response.take_supported_protocols());

    Ok((identity, connection))
}

#[cfg(test)]
mod tests {
    use crate::{
        protocols::identity::{exchange_identity, Identity},
        ProtocolId,
    };
    use futures::{executor::block_on, future::join};
    use memsocket::MemorySocket;
    use netcore::{
        multiplexing::yamux::{Mode, Yamux},
        transport::ConnectionOrigin,
    };
    use types::PeerId;

    fn build_test_connection() -> (Yamux<MemorySocket>, Yamux<MemorySocket>) {
        let (dialer, listener) = MemorySocket::new_pair();

        (
            Yamux::new(dialer, Mode::Client),
            Yamux::new(listener, Mode::Server),
        )
    }

    #[test]
    fn simple_identify() {
        let (outbound, inbound) = build_test_connection();
        let server_identity = Identity::new(
            PeerId::random(),
            vec![
                ProtocolId::from_static(b"/proto/1.0.0"),
                ProtocolId::from_static(b"/proto/2.0.0"),
            ],
        );
        let client_identity = Identity::new(
            PeerId::random(),
            vec![
                ProtocolId::from_static(b"/proto/1.0.0"),
                ProtocolId::from_static(b"/proto/2.0.0"),
                ProtocolId::from_static(b"/proto/3.0.0"),
            ],
        );
        let server_identity_config = server_identity.clone();
        let client_identity_config = client_identity.clone();

        let server = async move {
            let (identity, _connection) =
                exchange_identity(&server_identity_config, inbound, ConnectionOrigin::Inbound)
                    .await
                    .unwrap();

            assert_eq!(identity, client_identity);
        };

        let client = async move {
            let (identity, _connection) = exchange_identity(
                &client_identity_config,
                outbound,
                ConnectionOrigin::Outbound,
            )
            .await
            .unwrap();

            assert_eq!(identity, server_identity);
        };

        block_on(join(server, client));
    }
}
