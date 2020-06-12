// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::config::RoleType;
use libra_types::PeerId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A grouping of common information between all networking code for logging.
/// This should greatly reduce the groupings between these given everywhere, and will allow
/// for logging accordingly.  TODO: Figure out how to split these as structured logging
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkContext {
    network_id: NetworkId,
    role: RoleType,
    peer_id: PeerId,
}

impl fmt::Display for NetworkContext {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "[{},{},{}]",
            self.network_id,
            self.role,
            self.peer_id.short_str()
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
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }
    pub fn role(&self) -> RoleType {
        self.role
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct NetworkInfo {
    name: String,
}

impl fmt::Display for NetworkInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.name)
    }
}

/// A representation of the network being used in communication.
/// There should only be one of each NetworkId used for a single node, and handshakes should verify
/// that the NetworkId being used is the same during a handshake, to effectively ensure communication
/// is restricted to a network.  Network should be checked that it is not the `DEFAULT_NETWORK`
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NetworkId {
    Validator,
    Public,
    Private(NetworkInfo),
}

/// Default needed to handle downstream structs that use `Default`
impl Default for NetworkId {
    fn default() -> NetworkId {
        NetworkId::Public
    }
}

impl fmt::Display for NetworkId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NetworkId::Validator => write!(formatter, "Validator"),
            NetworkId::Public => write!(formatter, "Public"),
            NetworkId::Private(info) => write!(formatter, "Private({})", info),
        }
    }
}

impl NetworkId {
    /// Convenience function to specify the VFN network
    pub fn vfn_network() -> NetworkId {
        NetworkId::private_network("vfn")
    }

    /// Creates a private network so we don't have to keep track of `NetworkInfo` outside of here.
    pub fn private_network(name: &str) -> NetworkId {
        NetworkId::Private(NetworkInfo {
            name: name.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "NetworkId", rename_all = "snake_case", tag = "type")]
enum HumanReadableNetworkId {
    Validator,
    Public,
    Private(NetworkInfo),
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "NetworkId")]
enum NonHumanReadableNetworkId {
    Validator,
    Public,
    Private(NetworkInfo),
}

impl Serialize for NetworkId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            match self {
                NetworkId::Validator => HumanReadableNetworkId::Validator,
                NetworkId::Public => HumanReadableNetworkId::Public,
                NetworkId::Private(info) => HumanReadableNetworkId::Private(info.clone()),
            }
            .serialize(serializer)
        } else {
            match self {
                NetworkId::Validator => NonHumanReadableNetworkId::Validator,
                NetworkId::Public => NonHumanReadableNetworkId::Public,
                NetworkId::Private(info) => NonHumanReadableNetworkId::Private(info.clone()),
            }
            .serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for NetworkId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            Ok(match HumanReadableNetworkId::deserialize(deserializer)? {
                HumanReadableNetworkId::Validator => NetworkId::Validator,
                HumanReadableNetworkId::Public => NetworkId::Public,
                HumanReadableNetworkId::Private(info) => NetworkId::Private(info),
            })
        } else {
            Ok(
                match NonHumanReadableNetworkId::deserialize(deserializer)? {
                    NonHumanReadableNetworkId::Validator => NetworkId::Validator,
                    NonHumanReadableNetworkId::Public => NetworkId::Public,
                    NonHumanReadableNetworkId::Private(info) => NetworkId::Private(info),
                },
            )
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialization() {
        let id = NetworkId::private_network("fooo");
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
}
