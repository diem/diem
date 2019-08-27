// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A "user string" is a string literal in Move code. User strings can be present either within
//! Move modules or scripts, or as arguments to transactions.
//!
//! The name is inspired by the "user string" table described in ECMA-335 [1].
//!
//! Within this code base, user strings are represented as a different type to prevent them from
//! mixing with other sorts of strings. For example, it is not possible to use one as an
//! identifier for name resolution.
//!
//! [1] https://www.ecma-international.org/publications/standards/Ecma-335.htm

use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
};
use failure::prelude::*;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, fmt, ops::Deref, result, string::FromUtf8Error};

/// An owned string literal in a Move transaction.
///
/// For more details, see the module level documentation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "testing"), proptest(no_params))]
pub struct UserString(Box<str>);
// A UserString cannot be mutated so use Box<str> instead of String -- it is 1 word smaller.

impl UserString {
    /// Creates a new `UserString` instance.
    pub fn new(s: impl Into<Box<str>>) -> Self {
        Self(s.into())
    }

    /// Converts a vector of bytes to a `UserString`.
    pub fn from_utf8(vec: Vec<u8>) -> result::Result<Self, FromUtf8Error> {
        let s = String::from_utf8(vec)?;
        Ok(Self::new(s))
    }

    /// Creates a borrowed version of `self`.
    pub fn as_user_str(&self) -> &UserStr {
        self
    }

    /// Converts this `UserString` into a `String`.
    ///
    /// This is not implemented as a `From` trait to discourage automatic conversions -- these
    /// conversions should not typically happen.
    pub fn into_string(self) -> String {
        self.0.into()
    }

    /// Converts this `UserString` into a UTF-8-encoded byte sequence.
    pub fn into_bytes(self) -> Vec<u8> {
        self.into_string().into_bytes()
    }
}

impl From<String> for UserString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<Box<str>> for UserString {
    fn from(s: Box<str>) -> Self {
        Self::new(s)
    }
}

impl<'a> From<&'a str> for UserString {
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl<'a> From<&'a UserStr> for UserString {
    fn from(s: &'a UserStr) -> Self {
        Self::new(&s.0)
    }
}

impl AsRef<UserStr> for UserString {
    fn as_ref(&self) -> &UserStr {
        self
    }
}

impl Deref for UserString {
    type Target = UserStr;

    fn deref(&self) -> &UserStr {
        UserStr::new(&self.0)
    }
}

impl fmt::Display for UserString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A borrowed string literal in Move code.
///
/// For more details, see the module level documentation.
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct UserStr(str);

impl UserStr {
    pub fn new<'a>(s: impl AsRef<str> + 'a) -> &'a UserStr {
        let s = s.as_ref();
        // UserStr and str have the same layout, so this is safe to do.
        // This follows the pattern in Rust core https://doc.rust-lang.org/src/std/path.rs.html.
        unsafe { &*(s as *const str as *const UserStr) }
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

impl<'a> From<&'a str> for &'a UserStr {
    fn from(s: &'a str) -> Self {
        UserStr::new(s)
    }
}

impl Borrow<UserStr> for UserString {
    fn borrow(&self) -> &UserStr {
        UserStr::new(&self.0)
    }
}

impl ToOwned for UserStr {
    type Owned = UserString;

    fn to_owned(&self) -> UserString {
        UserString::new(&self.0)
    }
}

impl fmt::Display for UserStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// LCS does not define any sort of extra annotation for user strings -- they're serialized exactly
/// the same way regular strings are, and are represented only within the type system for now.
impl CanonicalSerialize for UserString {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_string(&self.0)?;
        Ok(())
    }
}

/// LCS does not define any sort of extra annotation for user strings -- they're serialized exactly
/// the same way regular strings are, and are represented only within the type system for now.
impl CanonicalSerialize for UserStr {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_string(&self.0)?;
        Ok(())
    }
}

impl CanonicalDeserialize for UserString {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        Ok(UserString::new(deserializer.decode_string()?))
    }
}
