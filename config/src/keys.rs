// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::{PrivateKey, Uniform, ValidKeyStringExt};
use mirai_annotations::verify_unreachable;
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

    pub fn public_key(&self) -> Option<T::PublicKeyMaterial> {
        match self {
            PrivateKeyContainer::Present(private_key) => Some(private_key.public_key()),
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
            PrivateKeyContainer::Present(key) => key.serialize(serializer),
            _ => serializer.serialize_str(""),
        }
    }
}

impl<'de, T> Deserialize<'de> for PrivateKeyContainer<T>
where
    T: ValidKeyStringExt,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<PrivateKeyContainer<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Note: Any error in parsing is assumed to be due to the private key being absent.
        T::deserialize(deserializer)
            .map(PrivateKeyContainer::Present)
            .or_else(|_| Ok(PrivateKeyContainer::Absent))
    }
}

#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct KeyPair<T>
where
    T: PrivateKey + Serialize + ValidKeyStringExt,
{
    #[serde(bound(deserialize = "PrivateKeyContainer<T>: Deserialize<'de>"))]
    private_key: PrivateKeyContainer<T>,
    public_key: T::PublicKeyMaterial,
}

impl<T> Default for KeyPair<T>
where
    T: PrivateKey + Serialize + Uniform + ValidKeyStringExt,
    T::PublicKeyMaterial: DeserializeOwned + 'static + Serialize + ValidKeyStringExt,
{
    fn default() -> Self {
        let private_key = T::generate_for_testing();
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
