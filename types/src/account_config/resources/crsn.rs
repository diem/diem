// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};

#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AccountSequenceNumber {
    SequenceNumber(u64),
    CRSN { min_nonce: u64, size: u64 },
}

impl AccountSequenceNumber {
    pub fn min_seq(&self) -> u64 {
        match self {
            Self::SequenceNumber(seqno) => *seqno,
            Self::CRSN { min_nonce, .. } => *min_nonce,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct CRSNResource {
    min_nonce: u64,
    size: u64,
    // NG: The length of these slots are not necessarily the size of the CRSN.
    slots: Vec<u8>,
}

impl CRSNResource {
    pub fn min_nonce(&self) -> u64 {
        self.min_nonce
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

impl MoveStructType for CRSNResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("CRSN");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CRSN");
}

impl MoveResource for CRSNResource {}
