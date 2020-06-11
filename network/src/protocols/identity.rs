// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protocol used to exchange supported protocol information with a remote.

use crate::protocols::wire::handshake::v1::HandshakeMsg;
use bytes::BytesMut;
use futures::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use netcore::framing::{read_u16frame, write_u16frame};
use std::io;

/// The Handshake exchange protocol.
pub async fn exchange_handshake<T>(
    own_handshake: &HandshakeMsg,
    socket: &mut T,
) -> io::Result<HandshakeMsg>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    // Send serialized handshake message to remote peer.
    let msg = lcs::to_bytes(own_handshake).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize identity msg: {}", e),
        )
    })?;
    write_u16frame(socket, &msg).await?;
    socket.flush().await?;

    // Read handshake message from the Remote
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
        protocols::{
            identity::exchange_handshake,
            wire::handshake::v1::{HandshakeMsg, MessagingProtocolVersion},
        },
        ProtocolId,
    };
    use futures::{executor::block_on, future::join};
    use libra_config::{chain_id::ChainId, network_id::NetworkId};
    use memsocket::MemorySocket;

    fn build_test_connection() -> (MemorySocket, MemorySocket) {
        MemorySocket::new_pair()
    }

    #[test]
    fn simple_handshake() {
        let network_id = NetworkId::Validator;
        let chain_id = ChainId::default();
        let (mut outbound, mut inbound) = build_test_connection();

        // Create client and server handshake messages.
        let mut server_handshake = HandshakeMsg::new(chain_id.clone(), network_id.clone());
        server_handshake.add(
            MessagingProtocolVersion::V1,
            [
                ProtocolId::ConsensusDirectSend,
                ProtocolId::MempoolDirectSend,
            ]
            .iter()
            .into(),
        );
        let mut client_handshake = HandshakeMsg::new(chain_id, network_id);
        client_handshake.add(
            MessagingProtocolVersion::V1,
            [ProtocolId::ConsensusRpc, ProtocolId::ConsensusDirectSend]
                .iter()
                .into(),
        );

        let server_handshake_clone = server_handshake.clone();
        let client_handshake_clone = client_handshake.clone();

        let server = async move {
            let handshake = exchange_handshake(&server_handshake, &mut inbound)
                .await
                .expect("Handshake fails");

            assert_eq!(
                lcs::to_bytes(&handshake).unwrap(),
                lcs::to_bytes(&client_handshake_clone).unwrap()
            );
        };

        let client = async move {
            let handshake = exchange_handshake(&client_handshake, &mut outbound)
                .await
                .expect("Handshake fails");

            assert_eq!(
                lcs::to_bytes(&handshake).unwrap(),
                lcs::to_bytes(&server_handshake_clone).unwrap()
            );
        };

        block_on(join(server, client));
    }
}
