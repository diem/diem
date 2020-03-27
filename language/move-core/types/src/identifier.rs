// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! An identifier is the name of an entity (module, resource, function, etc) in Move.
//!
//! A valid identifier consists of an ASCII string which satisfies any of the conditions:
//!
//! * The first character is a letter and the remaining characters are letters, digits or
//!   underscores.
//! * The first character is an underscore, and there is at least one further letter, digit or
//!   underscore.
//!
//! The spec for allowed identifiers is similar to Rust's spec
//! ([as of version 1.38](https://doc.rust-lang.org/1.38.0/reference/identifiers.html)).
//!
//! Allowed identifiers are currently restricted to ASCII due to unresolved issues with Unicode
//! normalization. See [Rust issue #55467](https://github.com/rust-lang/rust/issues/55467) and the
//! associated RFC for some discussion. Unicode identifiers may eventually be supported once these
//! issues are worked out.
//!
//! This module only determines allowed identifiers at the bytecode level. Move source code will
//! likely be more restrictive than even this, with a "raw identifier" escape hatch similar to
//! Rust's `r#` identifiers.
//!
//! Among other things, identifiers are used to:
//! * specify keys for lookups in storage
//! * do cross-module lookups while executing transactions

use anyhow::{bail, Result};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
use ref_cast::RefCast;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, fmt, ops::Deref};

/// Describes what identifiers are allowed.
///
/// For now this is deliberately restrictive -- we would like to evolve this in the future.
// TODO: "<SELF>" is coded as an exception. It should be removed once CompiledScript goes away.
fn is_valid(s: &str) -> bool {
    fn is_underscore_alpha_or_digit(c: char) -> bool {
        matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9')
    }

    if s == "<SELF>" {
        return true;
    }
    let len = s.len();
    let mut chars = s.chars();
    match chars.next() {
        Some('a'..='z') | Some('A'..='Z') => chars.all(is_underscore_alpha_or_digit),
        Some('_') if len > 1 => chars.all(is_underscore_alpha_or_digit),
        _ => false,
    }
}

/// A regex describing what identifiers are allowed. Used for proptests.
// TODO: "<SELF>" is coded as an exception. It should be removed once CompiledScript goes away.
#[cfg(any(test, feature = "fuzzing"))]
pub(crate) static ALLOWED_IDENTIFIERS: &str =
    r"(?:[a-zA-Z][a-zA-Z0-9_]*)|(?:_[a-zA-Z0-9_]+)|(?:<SELF>)";

/// An owned identifier.
///
/// For more details, see the module level documentation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Identifier(Box<str>);
// An identifier cannot be mutated so use Box<str> instead of String -- it is 1 word smaller.

impl Identifier {
    /// Creates a new `Identifier` instance.
    pub fn new(s: impl Into<Box<str>>) -> Result<Self> {
        let s = s.into();
        if Self::is_valid(&s) {
            Ok(Self(s))
        } else {
            bail!("Invalid identifier '{}'", s);
        }
    }

    /// Returns true if this string is a valid identifier.
    pub fn is_valid(s: impl AsRef<str>) -> bool {
        is_valid(s.as_ref())
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
        // Identifier and IdentStr maintain the same invariants, so it is safe to
        // convert.
        IdentStr::ref_cast(&self.0)
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
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd, RefCast)]
#[repr(transparent)]
pub struct IdentStr(str);

impl IdentStr {
    pub fn new(s: &str) -> Result<&IdentStr> {
        if Self::is_valid(s) {
            Ok(IdentStr::ref_cast(s))
        } else {
            bail!("Invalid identifier '{}'", s);
        }
    }

    /// Returns true if this string is a valid identifier.
    pub fn is_valid(s: impl AsRef<str>) -> bool {
        is_valid(s.as_ref())
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

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for Identifier {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): ()) -> Self::Strategy {
        ALLOWED_IDENTIFIERS
            .prop_map(|s| {
                // Identifier::new will verify that generated identifiers are correct.
                Identifier::new(s).unwrap()
            })
            .boxed()
    }
}
