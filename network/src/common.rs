// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ProtocolId;
use crypto::x25519::X25519PublicKey;
use nextgen_crypto::ed25519::*;
use std::fmt;

/// A Negotiated substream encapsulates a protocol and a substream for which that protocol has been
/// negotiated.
pub struct NegotiatedSubstream<TSubstream> {
    /// Protocol we have negotiated to use on the substream.
    pub protocol: ProtocolId,
    /// Opened substream.
    pub substream: TSubstream,
}

impl<TSubstream> fmt::Debug for NegotiatedSubstream<TSubstream> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NegotiatedSubstream {{ protocol: {:?}, substream: ... }}",
            self.protocol,
        )
    }
}

/// Public keys used at the network layer
#[derive(Debug, Clone)]
pub struct NetworkPublicKeys {
    /// This key can validate signed messages at the network layer.
    pub signing_public_key: Ed25519PublicKey,
    /// This key establishes a node's identity in the p2p network.
    pub identity_public_key: X25519PublicKey,
}
