// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::network_id::{NetworkId, NodeNetworkId};
use diem_types::PeerId;
use serde::{Deserialize, Serialize};
use short_hex_str::AsShortHexStr;
use std::fmt;

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
/// Identifier of a node, represented as (network_id, peer_id)
pub struct PeerNetworkId(pub NodeNetworkId, pub PeerId);

impl PeerNetworkId {
    pub fn network_id(&self) -> NodeNetworkId {
        self.0.clone()
    }

    pub fn raw_network_id(&self) -> NetworkId {
        self.0.network_id()
    }

    pub fn peer_id(&self) -> PeerId {
        self.1
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn random() -> Self {
        Self(
            NodeNetworkId::new(NetworkId::default(), 0),
            PeerId::random(),
        )
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn random_validator() -> Self {
        Self(
            NodeNetworkId::new(NetworkId::Validator, 0),
            PeerId::random(),
        )
    }
}

impl fmt::Debug for PeerNetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for PeerNetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PeerId:{}, NodeNetworkId:({})",
            self.peer_id().short_str(),
            self.raw_network_id()
        )
    }
}
