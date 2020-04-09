// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the structs transported during the network handshake protocol v1.
//! These should serialize as per [link](TODO: Add ref).
//!
//! During the v1 Handshake protocol, both end-points of a connection send a serialized and
//! length-prefixed `HandshakeMsg` to each other. The handshake message contains a map from
//! supported messaging protocol versions to a bit vector representing application protocols
//! supported over that messaging protocol. On receipt, both ends will determine the highest
//! intersecting messaging protocol version and use that for the remainder of the session.

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, convert::TryInto, iter::Iterator};

#[cfg(test)]
mod test;

/// Unique identifier associated with each application protocol.
/// New application protocols can be added without bumping up the MessagingProtocolVersion.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum ProtocolId {
    ConsensusRpc = 0,
    ConsensusDirectSend = 1,
    MempoolDirectSend = 2,
    StateSynchronizerDirectSend = 3,
    DiscoveryDirectSend = 4,
    HealthCheckerRpc = 5,
    IdentityDirectSend = 6,
    OnchainDiscoveryRpc = 7,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct SupportedProtocols(bitvec::BitVec);

/// The HandshakeMsg contains a mapping from MessagingProtocolVersion suppported by the node to a
/// bit-vector specifying application-level protocols supported over that version.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HandshakeMsg {
    pub supported_protocols: BTreeMap<MessagingProtocolVersion, SupportedProtocols>,
}

/// Enum representing different versions of the Libra network protocol. These should be listed from
/// old to new, old having the smallest value.
/// We derive `PartialOrd` since nodes need to find highest intersecting protocol version.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug, Hash, Deserialize, Serialize)]
pub enum MessagingProtocolVersion {
    V1 = 0,
}

impl TryInto<Vec<ProtocolId>> for SupportedProtocols {
    type Error = lcs::Error;

    fn try_into(self) -> lcs::Result<Vec<ProtocolId>> {
        let mut protocols = Vec::with_capacity(self.0.count_ones() as usize);
        if let Some(last_bit) = self.0.last_set_bit() {
            for i in 0..=last_bit {
                if self.0.is_set(i) {
                    let protocol: ProtocolId = lcs::from_bytes(&[i])?;
                    protocols.push(protocol);
                }
            }
        }
        Ok(protocols)
    }
}

impl<'a, T: Iterator<Item = &'a ProtocolId>> From<T> for SupportedProtocols {
    fn from(protocols: T) -> Self {
        let mut bv = bitvec::BitVec::default();
        protocols.for_each(|p| bv.set(*p as u8));
        Self(bv)
    }
}

impl SupportedProtocols {
    /// Returns a new SupportedProtocols struct that is an intersection.
    fn intersection(self, other: SupportedProtocols) -> SupportedProtocols {
        SupportedProtocols(self.0 & other.0)
    }
}

impl HandshakeMsg {
    pub fn new() -> Self {
        HandshakeMsg {
            supported_protocols: BTreeMap::default(),
        }
    }

    pub fn add(
        &mut self,
        messaging_protocol: MessagingProtocolVersion,
        application_protocols: SupportedProtocols,
    ) {
        self.supported_protocols
            .insert(messaging_protocol, application_protocols);
    }

    pub fn find_common_protocols(
        &self,
        other: &HandshakeMsg,
    ) -> Option<(MessagingProtocolVersion, SupportedProtocols)> {
        // First, find the highest MessagingProtocolVersion supported by both nodes.
        let mut inner = other.supported_protocols.iter().rev().peekable();
        // Iterate over all supported protocol versions in decreasing order.
        for (k_outer, _) in self.supported_protocols.iter().rev() {
            // Remove all elements from inner iterator that are larger than the current head of the
            // outer iterator.
            match inner.by_ref().find(|(k_inner, _)| *k_inner <= k_outer) {
                None => {
                    return None;
                }
                Some((k_inner, _)) if k_inner == k_outer => {
                    // Find all protocols supported by both nodes for the above protocol version.
                    // Both `self` and `other` shold have entry in map for `key`.
                    let protocols_self = self.supported_protocols.get(k_inner).unwrap();
                    let protocols_other = other.supported_protocols.get(k_inner).unwrap();
                    return Some((
                        *k_inner,
                        protocols_self.clone().intersection(protocols_other.clone()),
                    ));
                }
                _ => {}
            }
        }
        None
    }
}
