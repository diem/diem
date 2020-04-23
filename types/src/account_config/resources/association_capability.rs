// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AssociationCapabilityResource {
    is_certified: bool,
}

impl AssociationCapabilityResource {
    pub fn is_certified(&self) -> bool {
        self.is_certified
    }
}
