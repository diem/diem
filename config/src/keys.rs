// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file implements a KeyPair data structure.
//!
//! The point of a KeyPair is to deserialize a private key into a structure
//! that will only allow the private key to be moved out once
//! (hence providing good key hygiene)
//! while allowing access to the public key part forever.
//!
//! The public key part is dynamically derived during deserialization,
//! while ignored during serialization.
//!

use libra_crypto::{PrivateKey, ValidKeyStringExt};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(test)]
use libra_crypto::Uniform;

/// A KeyPair has a private key that can only be taken once,
/// and a public key that can be cloned ad infinitum.
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
#[derive(Debug, PartialEq)]
pub struct KeyPair<T>
where
    T: PrivateKey + Serialize + ValidKeyStringExt,
{
    private_key: Option<T>,
    public_key: T::PublicKeyMaterial,
}

#[cfg(test)]
impl<T> Default for KeyPair<T>
where
    T: PrivateKey + Serialize + Uniform + ValidKeyStringExt,
{
    fn default() -> Self {
        let private_key = T::generate_for_testing();
        let public_key = private_key.public_key();
        Self {
            private_key: Some(private_key),
            public_key,
        }
    }
}

impl<T> KeyPair<T>
where
    T: PrivateKey + Serialize + ValidKeyStringExt,
{
    /// This transforms a private key into a keypair data structure.
    pub fn load(private_key: T) -> Self {
        let public_key = private_key.public_key();
        Self {
            private_key: Some(private_key),
            public_key,
        }
    }

    /// Takes the key from the data structure, calling this function a second time will return None.
    pub fn take_private(&mut self) -> Option<T> {
        self.private_key.take()
    }

    /// Returns the public key part. This always work, even after the private key was taken.
    pub fn public_key(&self) -> T::PublicKeyMaterial {
        self.public_key.clone()
    }
}

//
// Serialization and Deserialization functions
//

/// Serialization for a KeyPair only serializes the private key part (if present).
impl<T> Serialize for KeyPair<T>
where
    T: PrivateKey + Serialize + ValidKeyStringExt,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize + ValidKeyStringExt,
    {
        match &self.private_key {
            Some(key) => key.serialize(serializer),
            None => serializer.serialize_str(""),
        }
    }
}

/// Deserializing a keypair only deserializes the private key, and dynamically derives the public key.
impl<'de, T> Deserialize<'de> for KeyPair<T>
where
    T: PrivateKey + ValidKeyStringExt,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<KeyPair<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match T::deserialize(deserializer) {
            Ok(private_key) => {
                let public_key = private_key.public_key();
                Ok(KeyPair {
                    private_key: Some(private_key),
                    public_key,
                })
            }
            Err(err) => Err(err),
        }
    }
}
