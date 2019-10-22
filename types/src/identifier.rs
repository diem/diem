// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! An identifier is the name of an entity (module, resource, function, etc) in Move.
//!
//! Among other things, identifiers are used to:
//! * specify keys for lookups in storage
//! * do cross-module lookups while executing transactions

// TODO: restrict identifiers to a subset of ASCII

use failure::prelude::*;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, fmt, ops::Deref};

/// An owned identifier.
///
/// For more details, see the module level documentation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Identifier(Box<str>);
// An identifier cannot be mutated so use Box<str> instead of String -- it is 1 word smaller.

impl Identifier {
    /// Creates a new `Identifier` instance.
    pub fn new(s: impl Into<Box<str>>) -> Result<Self> {
        // TODO: restrict identifiers to a subset of ASCII
        Ok(Self(s.into()))
    }

    /// Converts a vector of bytes to an `Identifier`.
    pub fn from_utf8(vec: Vec<u8>) -> Result<Self> {
        let s = String::from_utf8(vec)?;
        Self::new(s)
    }

    /// Creates a borrowed version of `self`.
    pub fn as_ident_str(&self) -> &IdentStr {
        self
    }

    /// Converts this `Identifier` into a `String`.
    ///
    /// This is not implemented as a `From` trait to discourage automatic conversions -- these
    /// conversions should not typically happen.
    pub fn into_string(self) -> String {
        self.0.into()
    }

    /// Converts this `Identifier` into a UTF-8-encoded byte sequence.
    pub fn into_bytes(self) -> Vec<u8> {
        self.into_string().into_bytes()
    }
}

impl<'a> From<&'a IdentStr> for Identifier {
    fn from(ident_str: &'a IdentStr) -> Self {
        ident_str.to_owned()
    }
}

impl AsRef<IdentStr> for Identifier {
    fn as_ref(&self) -> &IdentStr {
        self
    }
}

impl Deref for Identifier {
    type Target = IdentStr;

    fn deref(&self) -> &IdentStr {
        // Identifier and IdentStr maintain the same invariants, so it is safe to use
        // str_to_ident_str.
        str_to_ident_str(&self.0)
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// A borrowed identifier.
///
/// For more details, see the module level documentation.
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct IdentStr(str);

impl IdentStr {
    pub fn new(s: &str) -> Result<&IdentStr> {
        // TODO: restrict identifiers to a subset of ASCII.
        Ok(str_to_ident_str(s))
    }

    /// Returns the length of `self` in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if `self` has a length of zero bytes.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts `self` to a `&str`.
    ///
    /// This is not implemented as a `From` trait to discourage automatic conversions -- these
    /// conversions should not typically happen.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts `self` to a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Borrow<IdentStr> for Identifier {
    fn borrow(&self) -> &IdentStr {
        self
    }
}

impl ToOwned for IdentStr {
    type Owned = Identifier;

    fn to_owned(&self) -> Identifier {
        Identifier(self.0.into())
    }
}

impl fmt::Display for IdentStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

// This function is private -- it is used by code within this module once it has verified
// identifier invariants.
fn str_to_ident_str(s: &str) -> &IdentStr {
    // IdentStr and str have the same layout, so this is safe to do.
    // This follows the pattern in Rust core https://doc.rust-lang.org/src/std/path.rs.html.
    unsafe { &*(s as *const str as *const IdentStr) }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for Identifier {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): ()) -> Self::Strategy {
        // TODO: restrict identifiers to a subset of ASCII
        ".*".prop_map(|s| Identifier::new(s).unwrap()).boxed()
    }
}
