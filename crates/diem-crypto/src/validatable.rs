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
