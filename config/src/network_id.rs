// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::config::RoleType;
use diem_types::PeerId;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

/// A grouping of common information between all networking code for logging.
/// This should greatly reduce the groupings between these given everywhere, and will allow
/// for logging accordingly.
#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct NetworkContext {
    #[serde(serialize_with = "NetworkId::serialize_str")]
    network_id: NetworkId,
    role: RoleType,
    peer_id: PeerId,
}

impl fmt::Debug for NetworkContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for NetworkContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{},{},{}]",
            self.network_id.as_str(),
            self.role,
            self.peer_id.short_str(),
        )
    }
}

impl NetworkContext {
    pub fn new(network_id: NetworkId, role: RoleType, peer_id: PeerId) -> NetworkContext {
        NetworkContext {
            network_id,
            role,
            peer_id,
        }
    }

    pub fn network_id(&self) -> &NetworkId {
        &self.network_id
    }

    pub fn role(&self) -> RoleType {
        self.role
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    #[cfg(any(test, feature = "testing", feature = "fuzzing"))]
    pub fn mock_with_peer_id(peer_id: PeerId) -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self::new(
            NetworkId::Validator,
            RoleType::Validator,
            peer_id,
        ))
    }

    #[cfg(any(test, feature = "testing", feature = "fuzzing"))]
    pub fn mock() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self::new(
            NetworkId::Validator,
            RoleType::Validator,
            PeerId::random(),
        ))
    }
}

/// A representation of the network being used in communication.
/// There should only be one of each NetworkId used for a single node (except for NetworkId::Public),
/// and handshakes should verify that the NetworkId being used is the same during a handshake,
/// to effectively ensure communication is restricted to a network.  Network should be checked that
/// it is not the `DEFAULT_NETWORK`
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename = "NetworkId", rename_all = "snake_case")]
pub enum NetworkId {
    Validator,
    Public,
    Private(String),
}

/// An intra-node identifier for a network of a node unique for a network
/// This extra layer on top of `NetworkId` mainly exists for the application-layer (e.g. mempool,
/// state sync) to differentiate between multiple public
/// networks that a node may belong to
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct NodeNetworkId(NetworkId, usize);

impl NodeNetworkId {
    pub fn new(network_id: NetworkId, num_id: usize) -> Self {
        Self(network_id, num_id)
    }

    pub fn network_id(&self) -> NetworkId {
        self.0.clone()
    }
}

impl fmt::Debug for NodeNetworkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for NodeNetworkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

/// Default needed to handle downstream structs that use `Default`
impl Default for NetworkId {
    fn default() -> NetworkId {
        NetworkId::Public
    }
}

impl fmt::Debug for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl NetworkId {
    /// Convenience function to specify the VFN network
    pub fn vfn_network() -> NetworkId {
        NetworkId::Private("vfn".to_string())
    }

    pub fn as_str(&self) -> &str {
        match self {
            NetworkId::Validator => "Validator",
            NetworkId::Public => "Public",
            // We used to return "Private({info})" here; however, it's important
            // to get the network id str without temp allocs, as this is in the
            // metrics/logging hot path. In theory, someone could set their
            // network id as `Private("Validator")`, which would result in
            // confusing metrics/logging output for them, but would not affect
            // correctness.
            NetworkId::Private(info) => info.as_ref(),
        }
    }

    fn serialize_str<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialization() {
        let id = NetworkId::Private("fooo".to_string());
        let encoded = serde_yaml::to_string(&id).unwrap();
        let decoded: NetworkId = serde_yaml::from_str(encoded.as_str()).unwrap();
        assert_eq!(id, decoded);
        let encoded = serde_yaml::to_vec(&id).unwrap();
        let decoded: NetworkId = serde_yaml::from_slice(encoded.as_slice()).unwrap();
        assert_eq!(id, decoded);

        let id = NetworkId::Validator;
        let encoded = serde_yaml::to_string(&id).unwrap();
        let decoded: NetworkId = serde_yaml::from_str(encoded.as_str()).unwrap();
        assert_eq!(id, decoded);
        let encoded = serde_yaml::to_vec(&id).unwrap();
        let decoded: NetworkId = serde_yaml::from_slice(encoded.as_slice()).unwrap();
        assert_eq!(id, decoded);
    }

    #[test]
    fn test_network_context_serialization() {
        let network_name = "Awesome".to_string();
        let role = RoleType::Validator;
        let peer_id = PeerId::random();
        let context = NetworkContext::new(NetworkId::Private(network_name.clone()), role, peer_id);
        let expected = format!(
            "---\nnetwork_id: {}\nrole: {}\npeer_id: {}",
            network_name, role, peer_id
        );
        assert_eq!(expected, serde_yaml::to_string(&context).unwrap());
    }
}
