// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::hkdf::HkdfError;
use std::{convert, error::Error, fmt, io};

pub type Result<T, E = WalletError> = ::std::result::Result<T, E>;

/// Libra Wallet Error is a convenience enum for generating arbitrary WalletErrors. Currently, only
/// the LibraWalletGeneric error is being used, but there are plans to add more specific errors as
/// the Libra Wallet matures
pub enum WalletError {
    /// generic error message
    LibraWalletGeneric(String),
}

impl Error for WalletError {
    fn description(&self) -> &str {
        match *self {
            WalletError::LibraWalletGeneric(ref s) => s,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            WalletError::LibraWalletGeneric(_) => None,
        }
    }
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WalletError::LibraWalletGeneric(ref s) => write!(f, "LibraWalletGeneric: {}", s),
        }
    }
}

impl fmt::Debug for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self as &dyn fmt::Display).fmt(f)
    }
}

impl convert::From<WalletError> for io::Error {
    fn from(_err: WalletError) -> io::Error {
        match _err {
            WalletError::LibraWalletGeneric(s) => io::Error::new(io::ErrorKind::Other, s),
        }
    }
}

impl convert::From<io::Error> for WalletError {
    fn from(err: io::Error) -> WalletError {
        WalletError::LibraWalletGeneric(err.description().to_string())
    }
}

impl convert::From<anyhow::Error> for WalletError {
    fn from(err: anyhow::Error) -> WalletError {
        WalletError::LibraWalletGeneric(format!("{}", err))
    }
}

impl convert::From<ed25519_dalek::SignatureError> for WalletError {
    fn from(err: ed25519_dalek::SignatureError) -> WalletError {
        WalletError::LibraWalletGeneric(format!("{}", err))
    }
}

impl convert::From<HkdfError> for WalletError {
    fn from(err: HkdfError) -> WalletError {
        WalletError::LibraWalletGeneric(format!("{}", err))
    }
}
