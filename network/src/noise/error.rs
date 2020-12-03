// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::noise::NoiseError;
use diem_types::PeerId;
use short_hex_str::ShortHexStr;
use std::io;
use thiserror::Error;

/// Different errors than can be raised when negotiating a Noise handshake.
#[derive(Debug, Error)]
pub enum NoiseHandshakeError {
    #[error("noise client: MUST_FIX: missing remote server's public key when dialing")]
    MissingServerPublicKey,

    #[error("noise client: MUST_FIX: error building handshake init message: {0}")]
    BuildClientHandshakeMessageFailed(NoiseError),

    #[error("noise client: error sending client handshake init message: {0}")]
    ClientWriteFailed(io::Error),

    #[error(
        "noise client: error reading server handshake response message, server \
         probably rejected our handshake message: {0}"
    )]
    ClientReadFailed(io::Error),

    #[error("noise client: error flushing socket after writing: {0}")]
    ClientFlushFailed(io::Error),

    #[error("noise client: error finalizing secure connection: {0}")]
    ClientFinalizeFailed(NoiseError),

    #[error("noise server: error reading client handshake init message: {0}")]
    ServerReadFailed(io::Error),

    #[error("noise server: client peer id is malformed: {0}")]
    InvalidClientPeerId(String),

    #[error("noise server: detected self-dial: we're trying to connect to ourselves")]
    SelfDialDetected,

    #[error(
        "noise server: client {0}: client is expecting us to have a different \
         public key: {1}"
    )]
    ClientExpectingDifferentPubkey(ShortHexStr, String),

    #[error("noise server: client {0}: error parsing handshake init message: {1}")]
    ServerParseClient(ShortHexStr, NoiseError),

    #[error(
        "noise server: client {0}: known client peer id connecting to us with \
         unauthenticated public key: {1}"
    )]
    UnauthenticatedClientPubkey(ShortHexStr, String),

    #[error("noise server: client {0}: client connecting with unauthenticated peer id: {1}")]
    UnauthenticatedClient(ShortHexStr, PeerId),

    #[error(
        "noise server: client {0}: client's self-reported peer id and pubkey-derived peer \
         id don't match: self-reported: {1}, derived: {2}"
    )]
    ClientPeerIdMismatch(ShortHexStr, PeerId, PeerId),

    #[error("noise server: client {0}: handshake message is missing the anti-replay timestamp")]
    MissingAntiReplayTimestamp(ShortHexStr),

    #[error(
        "noise server: client {0}: detected a replayed handshake message, we've \
         seen this timestamp before: {1}"
    )]
    ServerReplayDetected(ShortHexStr, u64),

    #[error("noise server: client {0}: error building handshake response message: {1}")]
    BuildServerHandshakeMessageFailed(ShortHexStr, NoiseError),

    #[error("noise server: client {0}: error sending server handshake response message: {1}")]
    ServerWriteFailed(ShortHexStr, io::Error),
}

impl NoiseHandshakeError {
    /// Errors that are either clear bugs or indicate some security issue. Should
    /// immediately alert an engineer if we hit one of these errors.
    pub fn should_security_log(&self) -> bool {
        use NoiseHandshakeError::*;
        matches!(self, ServerReplayDetected(_, _))
    }
}
