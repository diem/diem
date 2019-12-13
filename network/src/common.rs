// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ProtocolId;
use libra_config::config::NetworkPeerInfo;
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
pub type NetworkPublicKeys = NetworkPeerInfo;
