// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::trusted_peers::{deserialize_key, serialize_key};
use libra_crypto::{
    ed25519::*,
    test_utils::TEST_SEED,
    x25519::{self, X25519StaticPrivateKey, X25519StaticPublicKey},
    PrivateKey, ValidKeyStringExt,
};
use mirai_annotations::verify_unreachable;
use rand::{rngs::StdRng, SeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
enum PrivateKeyContainer<T> {
    Present(T),
    Removed,
    Absent,
}

impl<T> PrivateKeyContainer<T>
where
    T: PrivateKey,
{
    pub fn take(&mut self) -> Option<T> {
        match self {
            PrivateKeyContainer::Present(_) => {
                let key = std::mem::replace(self, PrivateKeyContainer::Removed);
                match key {
                    PrivateKeyContainer::Present(priv_key) => Some(priv_key),
                    _ => verify_unreachable!(
                        "mem::replace returned value for PrivateKeyContainer different from the one observed right before the call"
                    ),
                }
            }
            _ => None,
        }
    }
}

impl<T> Serialize for PrivateKeyContainer<T>
where
    T: Serialize + ValidKeyStringExt,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PrivateKeyContainer::Present(key) => serialize_key(key, serializer),
            _ => serializer.serialize_str(""),
        }
    }
}

impl<'de, T> Deserialize<'de> for PrivateKeyContainer<T>
where
    T: ValidKeyStringExt + DeserializeOwned + 'static,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<PrivateKeyContainer<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Note: Any error in parsing is assumed to be due to the private key being absent.
        deserialize_key(deserializer)
            .map(PrivateKeyContainer::Present)
            .or_else(|_| Ok(PrivateKeyContainer::Absent))
    }
}

// NetworkKeyPairs is used to store a node's Network specific keypairs.
// It is filled via a config file at the moment.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
pub struct NetworkKeyPairs {
    network_signing_private_key: PrivateKeyContainer<Ed25519PrivateKey>,
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    network_signing_public_key: Ed25519PublicKey,

    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    network_identity_private_key: X25519StaticPrivateKey,
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    network_identity_public_key: X25519StaticPublicKey,
}

// required for serialization
impl Default for NetworkKeyPairs {
    fn default() -> Self {
        let mut rng = StdRng::from_seed(TEST_SEED);
        let (net_private_sig, net_public_sig) = compat::generate_keypair(&mut rng);
        let (private_kex, public_kex) = x25519::compat::generate_keypair(&mut rng);
        Self {
            network_signing_private_key: PrivateKeyContainer::Present(net_private_sig),
            network_signing_public_key: net_public_sig,
            network_identity_private_key: private_kex,
            network_identity_public_key: public_kex,
        }
    }
}

impl NetworkKeyPairs {
    // used in testing to fill the structure with test keypairs
    pub fn load(
        network_signing_private_key: Ed25519PrivateKey,
        network_identity_private_key: X25519StaticPrivateKey,
    ) -> Self {
        let network_signing_public_key = (&network_signing_private_key).into();
        let network_identity_public_key = (&network_identity_private_key).into();
        Self {
            network_signing_private_key: PrivateKeyContainer::Present(network_signing_private_key),
            network_signing_public_key,
            network_identity_private_key,
            network_identity_public_key,
        }
    }

    /// Beware, this destroys the private key from this NodeConfig
    pub fn take_network_signing_private(&mut self) -> Option<Ed25519PrivateKey> {
        self.network_signing_private_key.take()
    }

    pub fn get_network_identity_private(&self) -> X25519StaticPrivateKey {
        self.network_identity_private_key.clone()
    }

    pub fn get_network_identity_public(&self) -> &X25519StaticPublicKey {
        &self.network_identity_public_key
    }

    // getters for keypairs
    pub fn get_network_identity_keypair(&self) -> (X25519StaticPrivateKey, X25519StaticPublicKey) {
        (
            self.get_network_identity_private(),
            self.get_network_identity_public().clone(),
        )
    }
}

// ConsensusKeyPair is used to store a validator's consensus keypair.
// It is filled via a config file at the moment.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
pub struct ConsensusKeyPair {
    consensus_private_key: PrivateKeyContainer<Ed25519PrivateKey>,
    #[serde(serialize_with = "serialize_opt_key")]
    #[serde(deserialize_with = "deserialize_opt_key")]
    consensus_public_key: Option<Ed25519PublicKey>,
}

// required for serialization
impl Default for ConsensusKeyPair {
    fn default() -> Self {
        let mut rng = StdRng::from_seed(TEST_SEED);
        let (consensus_private_sig, consensus_public_sig) = compat::generate_keypair(&mut rng);
        Self {
            consensus_private_key: PrivateKeyContainer::Present(consensus_private_sig),
            consensus_public_key: Some(consensus_public_sig),
        }
    }
}

impl ConsensusKeyPair {
    // used in testing to fill the structure with test keypairs
    pub fn load(consensus_private_key: Option<Ed25519PrivateKey>) -> Self {
        let (consensus_private_key, consensus_public_key) = {
            match consensus_private_key {
                Some(private_key) => {
                    let public_key = (&private_key).into();
                    (PrivateKeyContainer::Present(private_key), Some(public_key))
                }
                None => (PrivateKeyContainer::Absent, None),
            }
        };
        Self {
            consensus_private_key,
            consensus_public_key,
        }
    }

    pub fn public(&self) -> Option<&Ed25519PublicKey> {
        self.consensus_public_key.as_ref()
    }

    /// Beware, this destroys the private key from this NodeConfig
    pub fn take_consensus_private(&mut self) -> Option<Ed25519PrivateKey> {
        self.consensus_private_key.take()
    }

    pub fn is_present(&self) -> bool {
        match self.consensus_private_key {
            PrivateKeyContainer::Present(_) => true,
            _ => false,
        }
    }
}

pub fn serialize_opt_key<S, K>(opt_key: &Option<K>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Serialize + ValidKeyStringExt,
{
    opt_key
        .as_ref()
        .map_or(Ok("".to_string()), |key| {
            key.to_encoded_string()
                .map_err(<S::Error as serde::ser::Error>::custom)
        })
        .and_then(|str| serializer.serialize_str(&str[..]))
}

pub fn deserialize_opt_key<'de, D, K>(deserializer: D) -> Result<Option<K>, D::Error>
where
    D: Deserializer<'de>,
    K: ValidKeyStringExt + DeserializeOwned + 'static,
{
    let encoded_key: String = Deserialize::deserialize(deserializer)?;

    ValidKeyStringExt::from_encoded_string(&encoded_key)
        .map_err(<D::Error as serde::de::Error>::custom)
        .map(Some)
}
