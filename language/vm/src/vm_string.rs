// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A "VM string" is a string in Move code. VM strings can be present either within Move modules
//! or scripts, or as arguments to transactions.
//!
//! Within this code base, VM strings are represented as a different type to prevent them from
//! mixing with other sorts of strings. For example, it is not possible to use one as an
//! identifier for name resolution.

#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use ref_cast::RefCast;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, fmt, ops::Deref, result, string::FromUtf8Error};

/// An owned string in a Move transaction.
///
/// For more details, see the module level documentation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct VMString(Box<str>);
// A VMString cannot be mutated so use Box<str> instead of String -- it is 1 word smaller.

impl VMString {
    /// Creates a new `VMString` instance.
    pub fn new(s: impl Into<Box<str>>) -> Self {
        Self(s.into())
    }

    /// Converts a vector of bytes to a `VMString`.
    pub fn from_utf8(vec: Vec<u8>) -> result::Result<Self, FromUtf8Error> {
        let s = String::from_utf8(vec)?;
        Ok(Self::new(s))
    }

    /// Creates a borrowed version of `self`.
    pub fn as_vm_str(&self) -> &VMStr {
        self
    }

    /// Converts this `VMString` into a `String`.
    ///
    /// This is not implemented as a `From` trait to discourage automatic conversions -- these
    /// conversions should not typically happen.
    pub fn into_string(self) -> String {
        self.0.into()
    }

    /// Converts this `VMString` into a UTF-8-encoded byte sequence.
    pub fn into_bytes(self) -> Vec<u8> {
        self.into_string().into_bytes()
    }
}

impl From<String> for VMString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<Box<str>> for VMString {
    fn from(s: Box<str>) -> Self {
        Self::new(s)
    }
}

impl<'a> From<&'a str> for VMString {
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl<'a> From<&'a VMStr> for VMString {
    fn from(s: &'a VMStr) -> Self {
        Self::new(&s.0)
    }
}

impl AsRef<VMStr> for VMString {
    fn as_ref(&self) -> &VMStr {
        self
    }
}

impl Deref for VMString {
    type Target = VMStr;

    fn deref(&self) -> &VMStr {
        VMStr::new(&self.0)
    }
}

impl fmt::Display for VMString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A borrowed string in Move code.
///
/// For more details, see the module level documentation.
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd, RefCast)]
#[repr(transparent)]
pub struct VMStr(str);

impl VMStr {
    pub fn new(s: &str) -> &VMStr {
        VMStr::ref_cast(s)
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

impl<'a> From<&'a str> for &'a VMStr {
    fn from(s: &'a str) -> Self {
        VMStr::new(s)
    }
}

impl Borrow<VMStr> for VMString {
    fn borrow(&self) -> &VMStr {
        VMStr::new(&self.0)
    }
}

impl ToOwned for VMStr {
    type Owned = VMString;

    fn to_owned(&self) -> VMString {
        VMString::new(&self.0)
    }
}

impl fmt::Display for VMStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}
