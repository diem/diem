// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

/// Libra Wallet Error is a convenience enum for generating arbitrary WalletErrors. Currently, only
/// the LibraWalletGeneric error is being used, but there are plans to add more specific errors as
/// the Libra Wallet matures
#[derive(Debug, Error)]
pub enum WalletError {
    /// generic error message
    #[error("{0}")]
    LibraWalletGeneric(String),
}
