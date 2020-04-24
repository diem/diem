// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! [Noise protocol framework][noise] support for use in Libra.
//!
//! The main feature of this module is [`NoiseSocket`](crate::socket::NoiseSocket) which
//! provides wire-framing for noise payloads.  Currently the only handshake pattern supported is IX.
//!
//! [noise]: http://noiseprotocol.org/

use futures::io::{AsyncRead, AsyncWrite};
use netcore::{
    negotiate::{negotiate_inbound, negotiate_outbound_interactive},
    transport::ConnectionOrigin,
};
use snow::{self, params::NoiseParams};
use std::io;

mod socket;
#[cfg(any(feature = "fuzzing", test))]
pub use self::socket::noise_fuzzing;

pub use self::socket::NoiseSocket;
use libra_crypto::{x25519, ValidCryptoMaterial};

const NOISE_IX_25519_AESGCM_SHA256_PROTOCOL_NAME: &[u8] = b"/noise_ix_25519_aesgcm_sha256/1.0.0";
const NOISE_PARAMETER: &str = "Noise_IX_25519_AESGCM_SHA256";

/// The Noise protocol configuration to be used to perform a protocol upgrade on an underlying
/// socket.
pub struct NoiseConfig {
    key: x25519::PrivateKey,
    parameters: NoiseParams,
}

impl NoiseConfig {
    /// Create a new NoiseConfig with the provided keypair
    pub fn new(key: x25519::PrivateKey) -> Self {
        let parameters: NoiseParams = NOISE_PARAMETER.parse().expect("Invalid protocol name");
        Self { key, parameters }
    }

    /// Create a new NoiseConfig with an ephemeral static key.
    #[cfg(feature = "testing")]
    pub fn new_random(rng: &mut (impl rand_core::RngCore + rand_core::CryptoRng)) -> Self {
        let parameters: NoiseParams = NOISE_PARAMETER.parse().expect("Invalid protocol name");
        use libra_crypto::Uniform;
        let key = x25519::PrivateKey::generate(rng);
        Self { key, parameters }
    }

    /// Perform a protocol upgrade on an underlying connection. In addition perform the noise IX
    /// handshake to establish a noise session and exchange static public keys. Upon success,
    /// returns the static public key of the remote as well as a NoiseSocket.
    pub async fn upgrade_connection<TSocket>(
        &self,
        socket: TSocket,
        origin: ConnectionOrigin,
    ) -> io::Result<(Vec<u8>, NoiseSocket<TSocket>)>
    where
        TSocket: AsyncRead + AsyncWrite + Unpin,
    {
        // Perform protocol negotiation
        let (socket, proto) = match origin {
            ConnectionOrigin::Inbound => {
                negotiate_inbound(socket, [NOISE_IX_25519_AESGCM_SHA256_PROTOCOL_NAME]).await?
            }
            ConnectionOrigin::Outbound => {
                negotiate_outbound_interactive(socket, [NOISE_IX_25519_AESGCM_SHA256_PROTOCOL_NAME])
                    .await?
            }
        };

        assert_eq!(proto, NOISE_IX_25519_AESGCM_SHA256_PROTOCOL_NAME);

        // Instantiate the snow session
        // Note: We need to scope the Builder struct so that the compiler doesn't over eagerly
        // capture it into the Async State-machine.
        let session = {
            let key = self.key.to_bytes();
            let builder = snow::Builder::new(self.parameters.clone()).local_private_key(&key);
            match origin {
                ConnectionOrigin::Inbound => builder.build_responder(),
                ConnectionOrigin::Outbound => builder.build_initiator(),
            }
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?
        };

        let handshake = socket::Handshake::new(socket, session);

        let socket = handshake.handshake_1rt().await?;
        let remote_static_key = socket
            .get_remote_static()
            .expect("Noise remote static key already taken")
            .to_owned();
        Ok((remote_static_key, socket))
    }
}
