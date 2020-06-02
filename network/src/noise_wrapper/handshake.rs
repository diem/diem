// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The handshake module implements the handshake part of the protocol.
//! This module also implements additional anti-DoS mitigation,
//! by including a timestamp in each handshake initialization message.
//! Refer to the module's documentation for more information.
//! A successful handshake returns a `NoiseStream` which is defined in [socket] module.
//!
//! [socket]: network::noise_wrapper::socket

use crate::noise_wrapper::stream::NoiseStream;
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libra_config::config::NetworkPeerInfo;
use libra_crypto::{noise, x25519};
use libra_types::PeerId;
use netcore::transport::ConnectionOrigin;
use std::{
    collections::HashMap,
    io,
    sync::{Arc, RwLock},
    time,
};

/// In a mutually authenticated network, a client message is accompanied with a timestamp.
/// This is in order to prevent replay attacks, where the attacker does not know the client's static key,
/// but can still replay a handshake message in order to force a peer into performing a few Diffie-Hellman key exchange operations.
///
/// Thus, to prevent replay attacks a responder will always check if the timestamp is strictly increasing,
/// effectively considering it as a stateful counter.
///
/// If the client timestamp has been seen before, or is not strictly increasing,
/// we can abort the handshake early and avoid heavy Diffie-Hellman computations.
/// If the client timestamp is valid, we store it.
#[derive(Default)]
pub struct AntiReplayTimestamps(HashMap<x25519::PublicKey, u64>);

impl AntiReplayTimestamps {
    /// Returns true if the timestamp has already been observed for this peer
    /// or if it's an old timestamp
    pub fn is_replay(&self, pubkey: x25519::PublicKey, timestamp: u64) -> bool {
        if let Some(last_timestamp) = self.0.get(&pubkey) {
            &timestamp <= last_timestamp
        } else {
            false
        }
    }

    /// Stores the timestamp
    pub fn store_timestamp(&mut self, pubkey: x25519::PublicKey, timestamp: u64) {
        self.0
            .entry(pubkey)
            .and_modify(|last_timestamp| *last_timestamp = timestamp)
            .or_insert(timestamp);
    }
}

/// The timestamp is sent as a payload, so that it is encrypted.
/// Note that a millisecond value is a 16-byte value in rust,
/// but as we use it to store a duration since UNIX_EPOCH we will never use more than 8 bytes.
const PAYLOAD_SIZE: usize = 8;

// Noise Upgrader
// --------------
// Noise by default is not aware of the above or lower protocol layers,
// We thus need to build this wrapper around Noise to both:
//
// - fragment messages that need to be encrypted by noise (due to its maximum 65535-byte messages)
// - understand how long noise messages we send and receive are,
//   in order to pass them to the noise implementaiton
//

/// The Noise configuration to be used to perform a protocol upgrade on an underlying socket.
pub struct NoiseUpgrader(noise::NoiseConfig);

impl NoiseUpgrader {
    /// Create a new NoiseConfig with the provided keypair
    pub fn new(key: x25519::PrivateKey) -> Self {
        Self(noise::NoiseConfig::new(key))
    }

    /// Perform a protocol upgrade on an underlying connection. In addition perform the noise IX
    /// handshake to establish a noise stream and exchange static public keys. Upon success,
    /// returns the static public key of the remote as well as a NoiseStream.
    // TODO(philiphayes): rework socket-bench-server so we can remove this function
    #[allow(dead_code)]
    pub async fn upgrade<TSocket>(
        &self,
        socket: TSocket,
        origin: ConnectionOrigin,
        anti_replay_timestamps: Option<&Arc<RwLock<AntiReplayTimestamps>>>,
        remote_public_key: Option<x25519::PublicKey>,
        trusted_peers: Option<&Arc<RwLock<HashMap<PeerId, NetworkPeerInfo>>>>,
    ) -> io::Result<(x25519::PublicKey, NoiseStream<TSocket>)>
    where
        TSocket: AsyncRead + AsyncWrite + Unpin,
    {
        // perform the noise handshake
        let socket = match origin {
            ConnectionOrigin::Outbound => {
                let remote_public_key = match remote_public_key {
                    Some(key) => key,
                    None if cfg!(any(test, feature = "fuzzing")) => unreachable!(),
                    None => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "noise: SHOULD NOT HAPPEN: missing server's key when dialing",
                        ));
                    }
                };
                self.upgrade_outbound(socket, anti_replay_timestamps.is_some(), remote_public_key)
                    .await?
            }
            ConnectionOrigin::Inbound => {
                self.upgrade_inbound(socket, anti_replay_timestamps, trusted_peers)
                    .await?
            }
        };

        // return remote public key with a socket including the noise stream
        let remote_public_key = socket.get_remote_static();
        Ok((remote_public_key, socket))
    }

    /// Perform an outbound protocol upgrade on this connection.
    ///
    /// This runs the "client" side of the Noise IK handshake to establish a
    /// secure Noise stream and exchange static public keys. In mutual auth
    /// scenarios, we will also include an anti replay attack counter in the
    /// Noise handshake payload. Currently this counter is always a millisecond-
    /// granularity unix epoch timestamp.
    pub async fn upgrade_outbound<TSocket>(
        &self,
        mut socket: TSocket,
        mutual_authentication: bool,
        remote_public_key: x25519::PublicKey,
    ) -> io::Result<NoiseStream<TSocket>>
    where
        TSocket: AsyncRead + AsyncWrite + Unpin,
    {
        // in mutual authenticated networks, send a payload of the current timestamp (in milliseconds)
        let payload = if mutual_authentication {
            let now: u64 = time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .expect("system clock should work")
                .as_millis() as u64;
            // e.g. [157, 126, 253, 97, 114, 1, 0, 0]
            let now = now.to_le_bytes().to_vec();
            Some(now)
        } else {
            None
        };

        // create first handshake message  (-> e, es, s, ss)
        let mut rng = rand::rngs::OsRng;
        let mut first_message = [0u8; noise::handshake_init_msg_len(PAYLOAD_SIZE)];
        let initiator_state = self
            .0
            .initiate_connection(
                &mut rng,
                &[],
                remote_public_key,
                payload.as_ref().map(|x| &x[..]),
                &mut first_message,
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // write the first handshake message
        socket.write_all(&first_message).await?;

        // flush
        socket.flush().await?;

        // receive the server's response (<- e, ee, se)
        let mut server_response = [0u8; noise::handshake_resp_msg_len(0)];
        socket.read_exact(&mut server_response).await?;

        // parse the server's response
        // TODO: security logging here? (mimoo)
        let (_, session) = self
            .0
            .finalize_connection(initiator_state, &server_response)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // finalize the connection
        Ok(NoiseStream::new(socket, session))
    }

    /// Perform an inbound protocol upgrade on this connection.
    ///
    /// This runs the "server" side of the Noise IK handshake to establish a
    /// secure Noise stream and exchange static public keys. If the configuration
    /// requires mutual authentication, we will only allow connections from peers
    /// that successfully authenticate to a public key in our `trusted_peers` set.
    /// In addition, we will expect the client to include an anti replay attack
    /// counter in the Noise handshake payload in mutual auth scenarios.
    pub async fn upgrade_inbound<TSocket>(
        &self,
        mut socket: TSocket,
        anti_replay_timestamps: Option<&Arc<RwLock<AntiReplayTimestamps>>>,
        trusted_peers: Option<&Arc<RwLock<HashMap<PeerId, NetworkPeerInfo>>>>,
    ) -> io::Result<NoiseStream<TSocket>>
    where
        TSocket: AsyncRead + AsyncWrite + Unpin,
    {
        // receive the initiation message
        let mut client_init_message = [0u8; noise::handshake_init_msg_len(PAYLOAD_SIZE)];
        socket.read_exact(&mut client_init_message).await?;

        // parse it
        let (their_public_key, handshake_state, payload) = self
            .0
            .parse_client_init_message(&[], &client_init_message)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // make sure the public key is a validator before continuing (if we're in the validator network)
        if let Some(trusted_peers) = trusted_peers {
            let found = trusted_peers
                .read()
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        "noise: unable to read trusted_peers lock",
                    )
                })?
                .iter()
                .any(|(_peer_id, public_keys)| public_keys.identity_public_key == their_public_key);
            if !found {
                // TODO: security logging (mimoo)
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "noise: client connecting to us with an unknown public key: {}",
                        their_public_key
                    ),
                ));
            }
        }

        // if on a mutually authenticated network
        if let Some(anti_replay_timestamps) = anti_replay_timestamps {
            // check that the payload received as the client timestamp (in seconds)
            if payload.len() != PAYLOAD_SIZE {
                // TODO: security logging (mimoo)
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "noise: client initiated connection without an 8-byte timestamp",
                ));
            }
            let mut client_timestamp = [0u8; PAYLOAD_SIZE];
            client_timestamp.copy_from_slice(&payload);
            let client_timestamp = u64::from_le_bytes(client_timestamp);

            // check the timestamp is not a replay
            let mut anti_replay_timestamps = anti_replay_timestamps.write().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "noise: unable to read anti_replay_timestamps lock",
                )
            })?;
            if anti_replay_timestamps.is_replay(their_public_key, client_timestamp) {
                // TODO: security logging the ip + blocking the ip? (mimoo)
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "noise: client initiated connection with a timestamp already seen before: {}",
                        client_timestamp
                    ),
                ));
            }

            // store the timestamp
            anti_replay_timestamps.store_timestamp(their_public_key, client_timestamp);
        }

        // construct the response
        let mut rng = rand::rngs::OsRng;
        let mut server_response = [0u8; noise::handshake_resp_msg_len(0)];
        let session = self
            .0
            .respond_to_client(&mut rng, handshake_state, None, &mut server_response)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // send the response
        socket.write_all(&server_response).await?;

        // finalize the connection
        Ok(NoiseStream::new(socket, session))
    }
}

//
// Tests
// -----
//

#[cfg(test)]
mod test {
    use super::*;

    use futures::{executor::block_on, future::join};
    use libra_crypto::test_utils::TEST_SEED;
    use memsocket::MemorySocket;
    use std::{
        collections::HashMap,
        io,
        sync::{Arc, RwLock},
    };

    use libra_crypto::traits::Uniform as _;
    use rand::SeedableRng as _;

    /// helper to setup two testing peers
    fn build_peers() -> (
        (NoiseUpgrader, x25519::PublicKey),
        (NoiseUpgrader, x25519::PublicKey),
    ) {
        let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);

        let client_private = x25519::PrivateKey::generate(&mut rng);
        let client_public = client_private.public_key();

        let server_private = x25519::PrivateKey::generate(&mut rng);
        let server_public = server_private.public_key();

        let client = NoiseUpgrader::new(client_private);
        let server = NoiseUpgrader::new(server_private);

        ((client, client_public), (server, server_public))
    }

    /// helper to perform a noise handshake with two peers
    fn perform_handshake(
        client: NoiseUpgrader,
        server_public_key: x25519::PublicKey,
        server: NoiseUpgrader,
        trusted_peers: Option<&Arc<RwLock<HashMap<PeerId, NetworkPeerInfo>>>>,
    ) -> io::Result<(NoiseStream<MemorySocket>, NoiseStream<MemorySocket>)> {
        // create an in-memory socket for testing
        let (dialer_socket, listener_socket) = MemorySocket::new_pair();
        let anti_replay_timestamps = Arc::new(RwLock::new(AntiReplayTimestamps::default()));

        // perform the handshake
        let (client_session, server_session) = block_on(join(
            client.upgrade_outbound(dialer_socket, true, server_public_key),
            server.upgrade_inbound(
                listener_socket,
                Some(&anti_replay_timestamps),
                trusted_peers,
            ),
        ));

        //
        Ok((client_session?, server_session?))
    }

    #[test]
    fn test_handshake() {
        // perform handshake with two testing peers
        let ((client, client_public), (server, server_public)) = build_peers();
        let (client, server) = perform_handshake(client, server_public, server, None).unwrap();

        assert_eq!(client.get_remote_static(), server_public,);
        assert_eq!(server.get_remote_static(), client_public,);
    }
}
