// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use hex::FromHex;
use std::{convert::TryFrom, fmt};

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct Subaddress([u8; Subaddress::LENGTH]);

// Implementation is inspired by AccountAddress.
impl Subaddress {
    pub const fn new(address: [u8; Self::LENGTH]) -> Self {
        Self(address)
    }

    /// The number of bytes in an address.
    pub const LENGTH: usize = 8;

    /// Hex address: 0x0
    pub const ZERO: Self = Self([0u8; Self::LENGTH]);

    pub fn generate<R>(rng: &mut R) -> Self
    where
        R: ::rand_core::RngCore + ::rand_core::CryptoRng,
    {
        let mut buf = [0u8; Self::LENGTH];
        rng.fill_bytes(&mut buf);
        Self(buf)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, SubaddressParseError> {
        <[u8; Self::LENGTH]>::from_hex(hex)
            .map_err(|_| SubaddressParseError)
            .map(Self)
    }

    pub fn to_hex(&self) -> String {
        format!("{:x}", self)
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, SubaddressParseError> {
        <[u8; Self::LENGTH]>::try_from(bytes.as_ref())
            .map_err(|_| SubaddressParseError)
            .map(Self)
    }
}

impl fmt::Display for Subaddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl fmt::Debug for Subaddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl fmt::LowerHex for Subaddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl fmt::UpperHex for Subaddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SubaddressParseError;

impl fmt::Display for SubaddressParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to parse Subaddress")
    }
}

impl std::error::Error for SubaddressParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subaddress_random() {
        let mut rng = ::rand::thread_rng();
        let subaddr = Subaddress::generate(&mut rng);
        let zeroaddr = Subaddress::ZERO;
        assert_ne!(subaddr, zeroaddr);
    }
}
