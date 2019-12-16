pub mod proto;

use anyhow::Result;
use libra_types::account_address::AccountAddress;
use std::convert::TryFrom;

/// Helper to construct and parse [`proto::block_storage::EmptyResponse`]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmptyResponse {}

pub mod prelude {
    pub use super::*;
}
