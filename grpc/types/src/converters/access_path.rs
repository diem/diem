// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use libra_types::access_path::AccessPath;
use std::convert::{TryFrom, TryInto};

impl TryFrom<crate::proto::types::AccessPath> for AccessPath {
    type Error = Error;

    fn try_from(proto: crate::proto::types::AccessPath) -> Result<Self> {
        Ok(AccessPath::new(proto.address.try_into()?, proto.path))
    }
}

impl From<AccessPath> for crate::proto::types::AccessPath {
    fn from(path: AccessPath) -> Self {
        Self {
            address: path.address.to_vec(),
            path: path.path,
        }
    }
}
