// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Credential {
    human_name: String,
    base_url: String,
    compliance_public_key: Vec<u8>,
    expiration_date: u64,
}

impl Credential {
    pub fn human_name(&self) -> &str {
        &self.human_name
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn expiration_date(&self) -> u64 {
        self.expiration_date
    }

    pub fn compliance_public_key(&self) -> &[u8] {
        &self.compliance_public_key
    }
}

impl MoveResource for Credential {
    const MODULE_NAME: &'static str = "DualAttestation";
    const STRUCT_NAME: &'static str = "Credential";
}
