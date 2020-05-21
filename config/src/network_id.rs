// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct NetworkInfo {
    name: String,
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

impl NetworkId {
    /// Convenience function to specify the VFN network
    pub fn vfn_network() -> NetworkId {
        NetworkId::private_network("VFN")
    }

    /// Creates a private network so we don't have to keep track of `NetworkInfo` outside of here.
    pub fn private_network(name: &str) -> NetworkId {
        NetworkId::Private(NetworkInfo {
            name: name.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename = "NetworkId")]
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
        let encoded = toml::to_string(&id).unwrap();
        let decoded: NetworkId = toml::from_str(encoded.as_str()).unwrap();
        assert_eq!(id, decoded);
        let encoded = toml::to_vec(&id).unwrap();
        let decoded: NetworkId = toml::from_slice(encoded.as_slice()).unwrap();
        assert_eq!(id, decoded);

        let id = NetworkId::Validator;
        let encoded = toml::to_string(&id).unwrap();
        let decoded: NetworkId = toml::from_str(encoded.as_str()).unwrap();
        assert_eq!(id, decoded);
        let encoded = toml::to_vec(&id).unwrap();
        let decoded: NetworkId = toml::from_slice(encoded.as_slice()).unwrap();
        assert_eq!(id, decoded);
    }
}
