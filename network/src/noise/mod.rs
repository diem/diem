// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crate implements wrappers around our [Noise][noise] implementation.
//! Noise is a protocol framework to encrypt and authentication connections.
//! We use Noise to secure connections between peers in Libra.
//! Specifically, we use the [Noise IK][ik] handshake which is a one round-trip protocol
//! (the client sends one message, then the server responds).
//! For more information about Noise and our implementation, refer to the [crypto] crate.
//!
//! Usage example:
//!
//! ```
//! use network::{
//!     noise::{HandshakeAuthMode, NoiseUpgrader},
//!     NetworkPublicKeys,
//! };
//! use futures::{executor, future, io::{AsyncReadExt, AsyncWriteExt}};
//! use memsocket::MemorySocket;
//! use libra_crypto::{x25519, ed25519, Uniform, PrivateKey, test_utils::TEST_SEED};
//! use rand::{rngs::StdRng, SeedableRng};
//! use libra_types::PeerId;
//! use libra_config::config::NetworkPeerInfo;
//! use std::{collections::HashMap, sync::{Arc, RwLock}};
//!
//! fn example() -> std::io::Result<()> {
//! // create client and server NoiseUpgrader
//! let mut rng = StdRng::from_seed(TEST_SEED);
//! let client_private = x25519::PrivateKey::generate(&mut rng);
//! let client_public = client_private.public_key();
//! let client_peer_id = PeerId::random();
//!
//! let server_private = x25519::PrivateKey::generate(&mut rng);
//! let server_public = server_private.public_key();
//! let server_peer_id = PeerId::random();
//!
//! // create list of trusted peers
//! let mut trusted_peers = Arc::new(RwLock::new(HashMap::new()));
//! {
//!     trusted_peers.write().unwrap().insert(client_peer_id, NetworkPublicKeys {
//!        identity_public_key: client_public,
//!     });
//!     trusted_peers.write().unwrap().insert(server_peer_id, NetworkPublicKeys {
//!        identity_public_key: server_public,
//!     });
//! }
//!
//! let client_auth = HandshakeAuthMode::mutual(trusted_peers.clone());
//! let client = NoiseUpgrader::new(client_peer_id, client_private, client_auth);
//!
//! let server_auth = HandshakeAuthMode::mutual(trusted_peers);
//! let server = NoiseUpgrader::new(server_peer_id, server_private, server_auth);
//!
//! // use an in-memory socket as example
//! let (dialer_socket, listener_socket) = MemorySocket::new_pair();
//!
//! // perform the handshake
//! let (client_session, server_session) = executor::block_on(future::join(
//!    client.upgrade_outbound(dialer_socket, server_public),
//!    server.upgrade_inbound(listener_socket),
//! ));
//!
//! let (mut client_session, _server_peer_id) = client_session?;
//! let (mut server_session, _client_peer_id) = server_session?;
//!
//! // client -> server
//! executor::block_on(client_session.write_all(b"client hello"))?;
//! executor::block_on(client_session.flush())?;
//!
//! let mut buf = [0; 12];
//! executor::block_on(server_session.read_exact(&mut buf))?;
//! assert_eq!(&buf, b"client hello");
//!
//! // client <- server
//! executor::block_on(server_session.write_all(b"server hello"))?;
//! executor::block_on(server_session.flush())?;
//!
//! let mut buf = [0; 12];
//! executor::block_on(client_session.read_exact(&mut buf))?;
//! assert_eq!(&buf, b"server hello");
//!
//! Ok(())
//! }
//!
//! example().unwrap();
//! ```
//!
//! [noise]: http://noiseprotocol.org/
//! [ik]: https://noiseexplorer.com/patterns/IK
//! [crypto]: ../libra_crypto/noise/index.html

pub mod handshake;
pub mod stream;

#[cfg(any(test, feature = "fuzzing"))]
pub mod fuzzing;

pub use handshake::{AntiReplayTimestamps, HandshakeAuthMode, NoiseUpgrader};
