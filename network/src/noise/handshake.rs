// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The handshake module implements the handshake part of the protocol.
//! This module also implements additional anti-DoS mitigation,
//! by including a timestamp in each handshake initialization message.
//! Refer to the module's documentation for more information.
//! A successful handshake returns a `NoiseStream` which is defined in the
//! [stream] module.
//!
//! [stream]: network::noise::stream

use crate::{logging::NetworkSchema, noise::stream::NoiseStream};
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libra_config::network_id::NetworkContext;
use libra_crypto::{noise, x25519};
use libra_logger::debug;
use libra_time::duration_since_epoch;
use libra_types::PeerId;
use netcore::transport::ConnectionOrigin;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom as _,
    fmt::Debug,
    io,
    sync::{Arc, RwLock},
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
    /// The timestamp is sent as a payload, so that it is encrypted.
    /// Note that a millisecond value is a 16-byte value in rust,
    /// but as we use it to store a duration since UNIX_EPOCH we will never use more than 8 bytes.
    pub const TIMESTAMP_SIZE: usize = 8;

    /// obtain the current timestamp
    pub fn now() -> [u8; Self::TIMESTAMP_SIZE] {
        let now: u64 = duration_since_epoch().as_millis() as u64; // (TIMESTAMP_SIZE)

        // e.g. [157, 126, 253, 97, 114, 1, 0, 0]
        now.to_le_bytes()
    }

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

/// Noise handshake authentication mode.
pub enum HandshakeAuthMode {
    /// In `Mutual` mode, both sides will authenticate each other with their
    /// `trusted_peers` set. We also include replay attack mitigation in this mode.
    ///
    /// For example, in the Libra validator network, validator peers will only
    /// allow connections from other validator peers. They will use this mode to
    /// check that inbound connections authenticate to a network public key
    /// actually contained in the current validator set.
    Mutual {
        // Only use anti replay protection in mutual-auth scenarios. In theory,
        // this is applicable everywhere; however, we would need to spend some
        // time making this more sophisticated so it garbage collects old
        // timestamps and doesn't use unbounded space. These are not problems in
        // mutual-auth scenarios because we have a bounded set of trusted peers
        // that rarely changes.
        anti_replay_timestamps: RwLock<AntiReplayTimestamps>,
        trusted_peers: Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>,
    },
    /// In `ServerOnly` mode, the dialer authenticates the server. However, the
    /// server does not care who connects to them and will allow inbound connections
    /// from any peer.
    ServerOnly,
}

impl HandshakeAuthMode {
    pub fn mutual(trusted_peers: Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>) -> Self {
        HandshakeAuthMode::Mutual {
            anti_replay_timestamps: RwLock::new(AntiReplayTimestamps::default()),
            trusted_peers,
        }
    }

    fn anti_replay_timestamps(&self) -> Option<&RwLock<AntiReplayTimestamps>> {
        match &self {
            HandshakeAuthMode::Mutual {
                anti_replay_timestamps,
                ..
            } => Some(&anti_replay_timestamps),
            HandshakeAuthMode::ServerOnly => None,
        }
    }

    fn trusted_peers(&self) -> Option<&RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>> {
        match &self {
            HandshakeAuthMode::Mutual { trusted_peers, .. } => Some(&trusted_peers),
            HandshakeAuthMode::ServerOnly => None,
        }
    }
}

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
pub struct NoiseUpgrader {
    /// The validator's network context
    network_context: Arc<NetworkContext>,
    /// Config for executing Noise handshakes. Includes our static private key.
    noise_config: noise::NoiseConfig,
    /// Handshake authentication can be either mutual or server-only authentication.
    auth_mode: HandshakeAuthMode,
}

impl NoiseUpgrader {
    /// Create a new NoiseConfig with the provided keypair and authentication mode.
    pub fn new(
        network_context: Arc<NetworkContext>,
        key: x25519::PrivateKey,
        auth_mode: HandshakeAuthMode,
    ) -> Self {
        Self {
            network_context,
            noise_config: noise::NoiseConfig::new(key),
            auth_mode,
        }
    }

    /// Perform a protocol upgrade on an underlying connection. In addition perform the noise IK
    /// handshake to establish a noise stream and exchange static public keys. Upon success,
    /// returns the static public key of the remote as well as a NoiseStream.
    // TODO(philiphayes): rework socket-bench-server so we can remove this function
    #[allow(dead_code)]
    pub async fn upgrade_with_noise<TSocket>(
        &self,
        socket: TSocket,
        origin: ConnectionOrigin,
        remote_public_key: Option<x25519::PublicKey>,
    ) -> io::Result<(x25519::PublicKey, NoiseStream<TSocket>)>
    where
        TSocket: AsyncRead + AsyncWrite + Debug + Unpin,
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
                self.upgrade_outbound(socket, remote_public_key, AntiReplayTimestamps::now)
                    .await?
            }
            ConnectionOrigin::Inbound => {
                let (socket, _peer_id) = self.upgrade_inbound(socket).await?;
                socket
            }
        };

        // return remote public key with a socket including the noise stream
        let remote_public_key = socket.get_remote_static();
        Ok((remote_public_key, socket))
    }

    /// The prologue is the client's peer_id and the remote's expected public key.
    const PROLOGUE_SIZE: usize = PeerId::LENGTH + x25519::PUBLIC_KEY_SIZE;

    /// The client message consist of the prologue + a noise message with a timestamp as payload.
    const CLIENT_MESSAGE_SIZE: usize =
        Self::PROLOGUE_SIZE + noise::handshake_init_msg_len(AntiReplayTimestamps::TIMESTAMP_SIZE);

    /// The server's message contains no payload.
    const SERVER_MESSAGE_SIZE: usize = noise::handshake_resp_msg_len(0);

    /// Perform an outbound protocol upgrade on this connection.
    ///
    /// This runs the "client" side of the Noise IK handshake to establish a
    /// secure Noise stream and send its static public key to the server.
    /// In mutual auth scenarios, we will also include an anti replay attack counter in the
    /// Noise handshake payload. Currently this counter is always a millisecond-
    /// granularity unix epoch timestamp.
    pub async fn upgrade_outbound<TSocket, F>(
        &self,
        mut socket: TSocket,
        remote_public_key: x25519::PublicKey,
        time_provider: F,
    ) -> io::Result<NoiseStream<TSocket>>
    where
        TSocket: AsyncRead + AsyncWrite + Debug + Unpin,
        F: Fn() -> [u8; AntiReplayTimestamps::TIMESTAMP_SIZE],
    {
        // buffer to hold prologue + first noise handshake message
        let mut client_message = [0; Self::CLIENT_MESSAGE_SIZE];

        // craft prologue = self_peer_id | expected_public_key
        client_message[..PeerId::LENGTH].copy_from_slice(self.network_context.peer_id().as_ref());
        client_message[PeerId::LENGTH..Self::PROLOGUE_SIZE]
            .copy_from_slice(remote_public_key.as_slice());

        let (prologue_msg, mut client_noise_msg) = client_message.split_at_mut(Self::PROLOGUE_SIZE);

        // craft 8-byte payload as current timestamp (in milliseconds)
        let payload = time_provider();

        // craft first handshake message  (-> e, es, s, ss)
        let mut rng = rand::rngs::OsRng;
        let initiator_state = self
            .noise_config
            .initiate_connection(
                &mut rng,
                &prologue_msg,
                remote_public_key,
                Some(&payload),
                &mut client_noise_msg,
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // send the first handshake message
        debug!(
            NetworkSchema::new(&self.network_context),
            noise_role = "client",
            step = "writing_data",
            remote_public_key = remote_public_key,
            "{} noise client writing_data: {}",
            self.network_context,
            remote_public_key
        );
        socket.write_all(&client_message).await?;
        socket.flush().await?;

        // receive the server's response (<- e, ee, se)
        debug!(
            NetworkSchema::new(&self.network_context),
            noise_role = "client",
            step = "reading_data",
            remote_public_key = remote_public_key,
            "{} noise client reading_data: {}",
            self.network_context,
            remote_public_key
        );
        let mut server_response = [0u8; Self::SERVER_MESSAGE_SIZE];
        socket.read_exact(&mut server_response).await?;

        // parse the server's response
        // TODO: security logging here? (mimoo)
        debug!(
            NetworkSchema::new(&self.network_context),
            noise_role = "client",
            step = "finalizing",
            remote_public_key = remote_public_key,
            "{} noise client finalize: {}",
            self.network_context,
            remote_public_key
        );
        let (_, session) = self
            .noise_config
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
    ) -> io::Result<(NoiseStream<TSocket>, PeerId)>
    where
        TSocket: AsyncRead + AsyncWrite + Debug + Unpin,
    {
        // buffer to contain the client first message
        let mut client_message = [0; Self::CLIENT_MESSAGE_SIZE];

        // receive the prologue + first noise handshake message
        debug!(
            NetworkSchema::new(&self.network_context),
            noise_role = "server",
            step = "reading_data",
            "{} noise client finalize: {:?}",
            self.network_context,
            socket
        );
        socket.read_exact(&mut client_message).await?;

        // extract prologue (remote_peer_id | self_public_key)
        let (remote_peer_id, self_expected_public_key) =
            client_message[..Self::PROLOGUE_SIZE].split_at(PeerId::LENGTH);

        // parse the client's peer id
        // note: in mutual authenticated network, we could verify that their peer_id is in the trust peer set now.
        // We do this later in this function instead (to batch a number of checks) as there is no known attack here.
        let remote_peer_id = PeerId::try_from(remote_peer_id).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "noise: client sent invalid peer id: {}",
                    hex::encode(remote_peer_id)
                ),
            )
        })?;

        // prevent accidental self-dials
        // this situation could occur either as a result of our own discovery
        // mis-configuration or a potentially malicious discovery peer advertising
        // a (loopback ip or mirror proxy) and our public key.
        if remote_peer_id == self.network_context.peer_id() {
            // TODO(philiphayes): security logging. someone should investigate
            // on-chain reconfiguration history to see if someone is misbehaving.
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "noise: detected accidental self-dial: we have the same peer id as the client",
            ));
        }

        // verify that this is indeed our public key
        if self_expected_public_key != self.noise_config.public_key().as_slice() {
            // TODO: security logging (mimoo)
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "noise: client expecting us to have incorrect public key: {}",
                    hex::encode(self_expected_public_key)
                ),
            ));
        }

        // parse it
        let (prologue, client_init_message) = client_message.split_at(Self::PROLOGUE_SIZE);
        let (remote_public_key, handshake_state, payload) = self
            .noise_config
            .parse_client_init_message(&prologue, &client_init_message)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // if mutual auth mode, verify the remote pubkey is in our set of trusted peers
        if let Some(trusted_peers) = self.auth_mode.trusted_peers() {
            match trusted_peers
                .read()
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        "noise: unable to read trusted_peers lock",
                    )
                })?
                .get(&remote_peer_id)
            {
                Some(remote_pubkey_set) => {
                    if !remote_pubkey_set.contains(&remote_public_key) {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "noise: peer id {} connecting to us with an unknown public key: {}, expected one of: {:?}",
                                remote_peer_id, remote_public_key, remote_pubkey_set,
                            ),
                        ));
                    }
                }
                None => {
                    // TODO: security logging (mimoo)
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "noise: client connecting to us with an unknown peer id: {}",
                            remote_peer_id
                        ),
                    ));
                }
            };
        } else {
            // if not, verify that their peerid is constructed correctly from their public key
            let expected_remote_peer_id = PeerId::from_identity_public_key(remote_public_key);
            if expected_remote_peer_id != remote_peer_id {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "noise: peer id expected: {}, received: {}",
                        hex::encode(expected_remote_peer_id),
                        hex::encode(remote_peer_id),
                    ),
                ));
            }
        }

        // if on a mutually authenticated network,
        // the payload should contain a u64 client timestamp
        if let Some(anti_replay_timestamps) = self.auth_mode.anti_replay_timestamps() {
            // check that the payload received as the client timestamp (in seconds)
            if payload.len() != AntiReplayTimestamps::TIMESTAMP_SIZE {
                // TODO: security logging (mimoo)
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "noise: client initiated connection without an 8-byte timestamp",
                ));
            }
            let mut client_timestamp = [0u8; AntiReplayTimestamps::TIMESTAMP_SIZE];
            client_timestamp.copy_from_slice(&payload);
            let client_timestamp = u64::from_le_bytes(client_timestamp);

            // check the timestamp is not a replay
            let mut anti_replay_timestamps = anti_replay_timestamps.write().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "noise: unable to read anti_replay_timestamps lock",
                )
            })?;
            if anti_replay_timestamps.is_replay(remote_public_key, client_timestamp) {
                // TODO: security logging
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "noise: client initiated connection with a timestamp already seen before: {}",
                        client_timestamp
                    ),
                ));
            }

            // store the timestamp
            anti_replay_timestamps.store_timestamp(remote_public_key, client_timestamp);
        }

        // construct the response
        let mut rng = rand::rngs::OsRng;
        let mut server_response = [0u8; Self::SERVER_MESSAGE_SIZE];
        let session = self
            .noise_config
            .respond_to_client(&mut rng, handshake_state, None, &mut server_response)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // send the response
        debug!(
            NetworkSchema::new(&self.network_context).remote_peer(&remote_peer_id),
            noise_role = "server",
            step = "writing_data",
            "{} noise server writing_data: {}",
            self.network_context,
            remote_peer_id.short_str()
        );
        socket.write_all(&server_response).await?;

        // finalize the connection
        debug!(
            NetworkSchema::new(&self.network_context).remote_peer(&remote_peer_id),
            noise_role = "server",
            step = "finalize",
            "{} noise server finalize: {}",
            self.network_context,
            remote_peer_id.short_str()
        );
        Ok((NoiseStream::new(socket, session), remote_peer_id))
    }
}

//
// Tests
// -----
//

#[cfg(test)]
mod test {
    use super::*;
    use crate::testutils::fake_socket::ReadWriteTestSocket;
    use futures::{executor::block_on, future::join};
    use libra_crypto::{test_utils::TEST_SEED, traits::Uniform as _};
    use memsocket::MemorySocket;
    use rand::SeedableRng as _;
    use std::{
        io,
        sync::{Arc, RwLock},
    };

    /// helper to setup two testing peers
    fn build_peers(
        is_mutual_auth: bool,
    ) -> (
        (NoiseUpgrader, x25519::PublicKey),
        (NoiseUpgrader, x25519::PublicKey),
    ) {
        let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);

        let client_private_key = x25519::PrivateKey::generate(&mut rng);
        let client_public_key = client_private_key.public_key();

        let server_private_key = x25519::PrivateKey::generate(&mut rng);
        let server_public_key = server_private_key.public_key();

        let (client_auth, server_auth, client_peer_id, server_peer_id) = if is_mutual_auth {
            let client_peer_id = PeerId::random();
            let client_pubkey_set = [client_public_key].iter().copied().collect();
            let server_peer_id = PeerId::random();
            let server_pubkey_set = [server_public_key].iter().copied().collect();
            let trusted_peers = Arc::new(RwLock::new(
                vec![
                    (client_peer_id, client_pubkey_set),
                    (server_peer_id, server_pubkey_set),
                ]
                .into_iter()
                .collect(),
            ));
            let client_auth = HandshakeAuthMode::mutual(trusted_peers.clone());
            let server_auth = HandshakeAuthMode::mutual(trusted_peers);
            (client_auth, server_auth, client_peer_id, server_peer_id)
        } else {
            let client_peer_id = PeerId::from_identity_public_key(client_public_key);
            let server_peer_id = PeerId::from_identity_public_key(server_public_key);
            (
                HandshakeAuthMode::ServerOnly,
                HandshakeAuthMode::ServerOnly,
                client_peer_id,
                server_peer_id,
            )
        };

        let client = NoiseUpgrader::new(
            NetworkContext::mock_with_peer_id(client_peer_id),
            client_private_key,
            client_auth,
        );
        let server = NoiseUpgrader::new(
            NetworkContext::mock_with_peer_id(server_peer_id),
            server_private_key,
            server_auth,
        );

        ((client, client_public_key), (server, server_public_key))
    }

    /// helper to perform a noise handshake with two peers
    fn perform_handshake(
        client: &NoiseUpgrader,
        server: &NoiseUpgrader,
        server_public_key: x25519::PublicKey,
    ) -> (
        io::Result<NoiseStream<MemorySocket>>,
        io::Result<(NoiseStream<MemorySocket>, PeerId)>,
    ) {
        // create an in-memory socket for testing
        let (dialer_socket, listener_socket) = MemorySocket::new_pair();

        // perform the handshake
        block_on(join(
            client.upgrade_outbound(dialer_socket, server_public_key, AntiReplayTimestamps::now),
            server.upgrade_inbound(listener_socket),
        ))
    }

    /// provide a function that will return the same given value as a timestamp
    fn bad_timestamp(value: u64) -> impl Fn() -> [u8; AntiReplayTimestamps::TIMESTAMP_SIZE] {
        move || value.to_le_bytes()
    }

    #[test]
    fn test_timestamp_replay() {
        // 1. generate peers
        let ((client, _), (server, server_public_key)) =
            build_peers(true /* is_mutual_auth */);

        // 2. perform the handshake with some timestamp, it should work
        let (dialer_socket, listener_socket) = MemorySocket::new_pair();
        let (client_session, server_session) = block_on(join(
            client.upgrade_outbound(dialer_socket, server_public_key, bad_timestamp(1)),
            server.upgrade_inbound(listener_socket),
        ));

        client_session.unwrap();
        server_session.unwrap();

        // 3. perform the handshake again with timestamp in the past, it should fail
        let (dialer_socket, listener_socket) = MemorySocket::new_pair();
        let (client_session, server_session) = block_on(join(
            client.upgrade_outbound(dialer_socket, server_public_key, bad_timestamp(0)),
            server.upgrade_inbound(listener_socket),
        ));

        client_session.unwrap_err();
        server_session.unwrap_err();

        // 4. perform the handshake again with the same timestamp, it should fail
        let (dialer_socket, listener_socket) = MemorySocket::new_pair();
        let (client_session, server_session) = block_on(join(
            client.upgrade_outbound(dialer_socket, server_public_key, bad_timestamp(1)),
            server.upgrade_inbound(listener_socket),
        ));

        client_session.unwrap_err();
        server_session.unwrap_err();

        // 5. perform the handshake again with a valid timestamp in the future, it should work
        let (dialer_socket, listener_socket) = MemorySocket::new_pair();
        let (client_session, server_session) = block_on(join(
            client.upgrade_outbound(dialer_socket, server_public_key, bad_timestamp(2)),
            server.upgrade_inbound(listener_socket),
        ));

        client_session.unwrap();
        server_session.unwrap();
    }

    fn test_handshake_success(is_mutual_auth: bool) {
        // perform handshake with two testing peers
        let ((client, client_public_key), (server, server_public_key)) =
            build_peers(is_mutual_auth);

        let (client_res, server_res) = perform_handshake(&client, &server, server_public_key);
        let client_stream = client_res.unwrap();
        let (server_stream, _) = server_res.unwrap();

        assert_eq!(client_stream.get_remote_static(), server_public_key);
        assert_eq!(server_stream.get_remote_static(), client_public_key);
    }

    #[test]
    fn test_handshake_success_server_only_auth() {
        test_handshake_success(false /* is_mutual_auth */);
    }

    #[test]
    fn test_handshake_success_mutual_auth() {
        test_handshake_success(true /* is_mutual_auth */);
    }

    fn test_handshake_self_fails(is_mutual_auth: bool) {
        let (_, (server, server_public_key)) = build_peers(is_mutual_auth);

        let (client_res, server_res) = perform_handshake(&server, &server, server_public_key);
        // Both sides should error
        client_res.unwrap_err();
        server_res.unwrap_err();
    }

    #[test]
    fn test_handshake_self_fails_server_only_auth() {
        test_handshake_self_fails(false /* is_mutual_auth */);
    }

    #[test]
    fn test_handshake_self_fails_mutual_auth() {
        test_handshake_self_fails(true /* is_mutual_auth */);
    }

    #[test]
    fn test_handshake_fragmented_reads() {
        // create an in-memory socket for testing
        let (mut dialer_socket, mut listener_socket) = ReadWriteTestSocket::new_pair();

        // fragment reads
        dialer_socket.set_fragmented_read();
        listener_socket.set_fragmented_read();

        // get peers
        let ((client, _client_public_key), (server, server_public_key)) = build_peers(false);

        // perform the handshake
        let (client_session, server_session) = block_on(join(
            client.upgrade_outbound(dialer_socket, server_public_key, AntiReplayTimestamps::now),
            server.upgrade_inbound(listener_socket),
        ));

        client_session.unwrap();
        server_session.unwrap();
    }
}
