// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protocol used to identify key information about a remote
//!
//! Currently, the information shared as part of this protocol includes the peer identity and a
//! list of protocols supported by the peer.
use crate::ProtocolId;
use bytes::BytesMut;
use futures::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use libra_types::PeerId;
use netcore::framing::{read_u16frame, write_u16frame};
use serde::{Deserialize, Serialize};
use std::io;

/// The Identity of a node
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

    pub fn is_protocol_supported(&self, protocol: ProtocolId) -> bool {
        self.supported_protocols
            .iter()
            .any(|proto| *proto == protocol)
    }

    pub fn supported_protocols(&self) -> &[ProtocolId] {
        &self.supported_protocols
    }
}

/// The Identity exchange protocol
pub async fn exchange_identity<T>(own_identity: &Identity, socket: &mut T) -> io::Result<Identity>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    // Send serialized message to peer.
    let msg = lcs::to_bytes(own_identity).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize identity msg: {}", e),
        )
    })?;
    write_u16frame(socket, &msg).await?;
    socket.flush().await?;

    // Read an IdentityMsg from the Remote
    let mut response = BytesMut::new();
    read_u16frame(socket, &mut response).await?;
    let identity = lcs::from_bytes(&response).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse identity msg: {}", e),
        )
    })?;
    Ok(identity)
}

#[cfg(test)]
mod tests {
    use crate::{
        protocols::identity::{exchange_identity, Identity},
        ProtocolId,
    };
    use futures::{executor::block_on, future::join};
    use libra_types::PeerId;
    use memsocket::MemorySocket;

    fn build_test_connection() -> (MemorySocket, MemorySocket) {
        MemorySocket::new_pair()
    }

    #[test]
    fn simple_identify() {
        let (mut outbound, mut inbound) = build_test_connection();
        let server_identity = Identity::new(
            PeerId::random(),
            vec![
                ProtocolId::ConsensusDirectSend,
                ProtocolId::MempoolDirectSend,
            ],
        );
        let client_identity = Identity::new(
            PeerId::random(),
            vec![ProtocolId::ConsensusRpc, ProtocolId::ConsensusDirectSend],
        );
        let server_identity_config = server_identity.clone();
        let client_identity_config = client_identity.clone();

        let server = async move {
            let identity = exchange_identity(&server_identity_config, &mut inbound)
                .await
                .expect("Identity exchange fails");

            assert_eq!(identity, client_identity);
        };

        let client = async move {
            let identity = exchange_identity(&client_identity_config, &mut outbound)
                .await
                .expect("Identity exchange fails");

            assert_eq!(identity, server_identity);
        };

        block_on(join(server, client));
    }
}
