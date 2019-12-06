// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::{test_utils::TEST_SEED, PrivateKey, Uniform, ValidKeyStringExt};
use mirai_annotations::verify_unreachable;
use rand::{rngs::StdRng, SeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
pub enum PrivateKeyContainer<T> {
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

#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct KeyPair<T>
where
    T: PrivateKey + Serialize + ValidKeyStringExt,
    T::PublicKeyMaterial: DeserializeOwned + 'static + Serialize + ValidKeyStringExt,
{
    #[serde(bound(deserialize = "PrivateKeyContainer<T>: Deserialize<'de>"))]
    private_key: PrivateKeyContainer<T>,
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    public_key: T::PublicKeyMaterial,
}

impl<T> Default for KeyPair<T>
where
    T: PrivateKey + Serialize + Uniform + ValidKeyStringExt,
    T::PublicKeyMaterial: DeserializeOwned + 'static + Serialize + ValidKeyStringExt,
{
    fn default() -> Self {
        let mut rng = StdRng::from_seed(TEST_SEED);
        let private_key = T::generate_for_testing(&mut rng);
        let public_key = private_key.public_key();
        Self {
            private_key: PrivateKeyContainer::Present(private_key),
            public_key,
        }
    }
}

impl<T> KeyPair<T>
where
    T: PrivateKey + Serialize + ValidKeyStringExt,
    T::PublicKeyMaterial: DeserializeOwned + 'static + Serialize + ValidKeyStringExt,
{
    pub fn load(private_key: T) -> Self {
        let public_key = private_key.public_key();
        Self {
            private_key: PrivateKeyContainer::Present(private_key),
            public_key,
        }
    }

    pub fn public(&self) -> &T::PublicKeyMaterial {
        &self.public_key
    }

    /// Beware, this destroys the private key from this NodeConfig
    pub fn take_private(&mut self) -> Option<T> {
        self.private_key.take()
    }
}

pub fn serialize_key<S, K>(key: &K, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Serialize + ValidKeyStringExt,
{
    key.to_encoded_string()
        .map_err(<S::Error as serde::ser::Error>::custom)
        .and_then(|str| serializer.serialize_str(&str[..]))
}

pub fn deserialize_key<'de, D, K>(deserializer: D) -> Result<K, D::Error>
where
    D: Deserializer<'de>,
    K: ValidKeyStringExt + DeserializeOwned + 'static,
{
    let encoded_key: &str = Deserialize::deserialize(deserializer)?;
    ValidKeyStringExt::from_encoded_string(encoded_key)
        .map_err(<D::Error as serde::de::Error>::custom)
}
