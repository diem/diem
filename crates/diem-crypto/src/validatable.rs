// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides the `Validate` trait and `Validatable` type in order to aid in deferred
//! validation.

use anyhow::Result;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, hash::Hash};

/// The `Validate` trait is used in tandem with the `Validatable` type in order to provide deferred
/// validation for types.
///
/// ## Trait Contract
///
/// Any type `V` which implement this trait must adhere to the following contract:
///
/// * `V` and `V::Unvalidated` are byte-for-byte equivalent.
/// * `V` and `V::Unvalidated` have equivalent `Hash` implementations.
/// * `V` and `V::Unvalidated` must have equivalent `Serialize` and `Deserialize` implementation.
///   This means that `V` and `V:Unvalidated` have equivalent serialized formats and that you can
///   deserialize a `V::Unvalidated` from a `V` that was previously serialized.
pub trait Validate: Sized {
    /// The unvalidated form of some type `V`
    type Unvalidated;

    /// Attempt to validate a `V::Unvalidated` and returning a validated `V` on success
    fn validate(unvalidated: &Self::Unvalidated) -> Result<Self>;

    /// Return the unvalidated form of type `V`
    fn to_unvalidated(&self) -> Self::Unvalidated;
}

/// Used in connection with the `Validate` trait to be able to represent types which can benefit
/// from deferred validation as a performance optimization.
#[derive(Clone, Debug)]
pub struct Validatable<V: Validate> {
    unvalidated: V::Unvalidated,
    maybe_valid: OnceCell<V>,
}

impl<V: Validate> Validatable<V> {
    /// Create a new `Validatable` from a valid type
    pub fn new_valid(valid: V) -> Self {
        let unvalidated = valid.to_unvalidated();

        let maybe_valid = OnceCell::new();
        maybe_valid.set(valid).unwrap_or_else(|_| unreachable!());

        Self {
            unvalidated,
            maybe_valid,
        }
    }

    /// Create a new `Validatable` from an unvalidated type
    pub fn new_unvalidated(unvalidated: V::Unvalidated) -> Self {
        Self {
            unvalidated,
            maybe_valid: OnceCell::new(),
        }
    }

    /// Return a reference to the unvalidated form `V::Unvalidated`
    pub fn unvalidated(&self) -> &V::Unvalidated {
        &self.unvalidated
    }

    /// Try to validate the unvalidated form returning `Some(&V)` on success and `None` on failure.
    pub fn valid(&self) -> Option<&V> {
        self.validate().ok()
    }

    // TODO maybe optimize to only try once and keep track when we fail
    /// Attempt to validate `V::Unvalidated` and return a reference to a valid `V`
    pub fn validate(&self) -> Result<&V> {
        self.maybe_valid
            .get_or_try_init(|| V::validate(&self.unvalidated))
    }
}

impl<V> Serialize for Validatable<V>
where
    V: Validate + Serialize,
    V::Unvalidated: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.unvalidated.serialize(serializer)
    }
}

impl<'de, V> Deserialize<'de> for Validatable<V>
where
    V: Validate,
    V::Unvalidated: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let unvalidated = <V::Unvalidated>::deserialize(deserializer)?;
        Ok(Self::new_unvalidated(unvalidated))
    }
}

impl<V> PartialEq for Validatable<V>
where
    V: Validate,
    V::Unvalidated: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.unvalidated == other.unvalidated
    }
}

impl<V> Eq for Validatable<V>
where
    V: Validate,
    V::Unvalidated: Eq,
{
}

impl<V> Hash for Validatable<V>
where
    V: Validate,
    V::Unvalidated: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.unvalidated.hash(state);
    }
}

//
// Implement for Ed25519
//

use crate::ed25519::{Ed25519PublicKey, ED25519_PUBLIC_KEY_LENGTH};

/// An unvalidated `Ed25519PublicKey`
#[derive(Debug, Clone, Eq)]
pub struct UnvalidatedEd25519PublicKey([u8; ED25519_PUBLIC_KEY_LENGTH]);

impl UnvalidatedEd25519PublicKey {
    /// Return key as bytes
    pub fn to_bytes(&self) -> [u8; ED25519_PUBLIC_KEY_LENGTH] {
        self.0
    }
}

impl Serialize for UnvalidatedEd25519PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let encoded = ::hex::encode(&self.0);
            serializer.serialize_str(&encoded)
        } else {
            // See comment in deserialize_key.
            serializer.serialize_newtype_struct(
                "Ed25519PublicKey",
                serde_bytes::Bytes::new(self.0.as_ref()),
            )
        }
    }
}

impl<'de> Deserialize<'de> for UnvalidatedEd25519PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        if deserializer.is_human_readable() {
            let encoded_key = <String>::deserialize(deserializer)?;
            let bytes_out = ::hex::decode(encoded_key).map_err(D::Error::custom)?;
            <[u8; ED25519_PUBLIC_KEY_LENGTH]>::try_from(bytes_out.as_ref())
                .map(UnvalidatedEd25519PublicKey)
                .map_err(D::Error::custom)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(Deserialize)]
            #[serde(rename = "Ed25519PublicKey")]
            struct Value<'a>(&'a [u8]);

            let value = Value::deserialize(deserializer)?;
            <[u8; ED25519_PUBLIC_KEY_LENGTH]>::try_from(value.0)
                .map(UnvalidatedEd25519PublicKey)
                .map_err(D::Error::custom)
        }
    }
}

impl Hash for UnvalidatedEd25519PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.0)
    }
}

impl PartialEq for UnvalidatedEd25519PublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Validate for Ed25519PublicKey {
    type Unvalidated = UnvalidatedEd25519PublicKey;

    fn validate(unvalidated: &Self::Unvalidated) -> Result<Self> {
        Self::try_from(unvalidated.0.as_ref()).map_err(Into::into)
    }

    fn to_unvalidated(&self) -> Self::Unvalidated {
        UnvalidatedEd25519PublicKey(self.to_bytes())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
        test_utils::uniform_keypair_strategy,
        validatable::{UnvalidatedEd25519PublicKey, Validate},
    };
    use proptest::prelude::*;
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    proptest! {
        #[test]
        fn unvalidated_ed25519_public_key_equivalence(
            keypair in uniform_keypair_strategy::<Ed25519PrivateKey, Ed25519PublicKey>()
        ) {
            let valid = keypair.public_key;
            let unvalidated = valid.to_unvalidated();

            prop_assert_eq!(&unvalidated, &UnvalidatedEd25519PublicKey(valid.to_bytes()));
            prop_assert_eq!(&valid, &Ed25519PublicKey::validate(&unvalidated).unwrap());

            // Ensure Serialize and Deserialize are implemented the same

            // BCS - A non-human-readable format
            {
                let serialized_valid = bcs::to_bytes(&valid).unwrap();
                let serialized_unvalidated = bcs::to_bytes(&unvalidated).unwrap();
                prop_assert_eq!(&serialized_valid, &serialized_unvalidated);

                let deserialized_valid_from_unvalidated: Ed25519PublicKey = bcs::from_bytes(&serialized_unvalidated).unwrap();
                let deserialized_unvalidated_from_valid: UnvalidatedEd25519PublicKey = bcs::from_bytes(&serialized_valid).unwrap();

                prop_assert_eq!(&valid, &deserialized_valid_from_unvalidated);
                prop_assert_eq!(&unvalidated, &deserialized_unvalidated_from_valid);
            }

            // JSON A human-readable format
            {
                let serialized_valid = serde_json::to_string(&valid).unwrap();
                let serialized_unvalidated = serde_json::to_string(&unvalidated).unwrap();
                prop_assert_eq!(&serialized_valid, &serialized_unvalidated);

                let deserialized_valid_from_unvalidated: Ed25519PublicKey = serde_json::from_str(&serialized_unvalidated).unwrap();
                let deserialized_unvalidated_from_valid: UnvalidatedEd25519PublicKey = serde_json::from_str(&serialized_valid).unwrap();

                prop_assert_eq!(&valid, &deserialized_valid_from_unvalidated);
                prop_assert_eq!(&unvalidated, &deserialized_unvalidated_from_valid);
            }


            // Ensure Hash is implemented the same
            let valid_hash = {
                let mut hasher = DefaultHasher::new();
                valid.hash(&mut hasher);
                hasher.finish()
            };

            let unvalidated_hash = {
                let mut hasher = DefaultHasher::new();
                unvalidated.hash(&mut hasher);
                hasher.finish()
            };

            prop_assert_eq!(valid_hash, unvalidated_hash);
        }
    }
}
