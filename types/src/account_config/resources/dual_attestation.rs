// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{event::EventHandle, on_chain_config::OnChainConfig};
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Credential {
    human_name: String,
    base_url: String,
    compliance_public_key: Vec<u8>,
    expiration_date: u64,
    compliance_key_rotation_events: EventHandle,
    base_url_rotation_events: EventHandle,
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

    pub fn compliance_key_rotation_events(&self) -> &EventHandle {
        &self.compliance_key_rotation_events
    }

    pub fn base_url_rotation_events(&self) -> &EventHandle {
        &self.base_url_rotation_events
    }
}

impl MoveStructType for Credential {
    const MODULE_NAME: &'static IdentStr = ident_str!("DualAttestation");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Credential");
}

impl MoveResource for Credential {}

/// Defines the dual attest limit in microDiem XDX
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Limit {
    pub micro_xdx_limit: u64,
}

impl OnChainConfig for Limit {
    const IDENTIFIER: &'static str = "Limit";
}

impl MoveStructType for Limit {
    const MODULE_NAME: &'static IdentStr = ident_str!("DualAttestation");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Limit");
}

impl MoveResource for Limit {}
