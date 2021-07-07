// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress, diem_id_identifier::DiemIdVaspDomainIdentifier,
    event::EventHandle,
};
use anyhow::Result;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

/// The Identifier for the VASPDomain module.
pub const VASP_DOMAIN_MODULE_IDENTIFIER: &IdentStr = ident_str!("VASPDomain");

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VASPDomains {
    pub domains: Vec<VASPDomain>,
}

impl VASPDomains {
    pub fn domains(&self) -> &[VASPDomain] {
        &self.domains
    }

    pub fn get_domains_list(&self) -> Vec<DiemIdVaspDomainIdentifier> {
        self.domains
            .iter()
            .map(|vasp_domain| vasp_domain.domain().clone())
            .collect()
    }
}

impl MoveStructType for VASPDomains {
    const MODULE_NAME: &'static IdentStr = VASP_DOMAIN_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("VASPDomains");
}

impl MoveResource for VASPDomains {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VASPDomain {
    pub domain: DiemIdVaspDomainIdentifier,
}

impl VASPDomain {
    pub fn domain(&self) -> &DiemIdVaspDomainIdentifier {
        &self.domain
    }
}

impl MoveStructType for VASPDomain {
    const MODULE_NAME: &'static IdentStr = VASP_DOMAIN_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("VASPDomain");
}

impl MoveResource for VASPDomain {}

#[derive(Debug, Serialize, Deserialize)]
pub struct VASPDomainEvent {
    // Whether a domain was added or removed
    removed: bool,
    // VASP Domain string of the account
    domain: VASPDomain,
    // On-chain account address
    address: AccountAddress,
}

impl VASPDomainEvent {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }

    pub fn removed(&self) -> bool {
        self.removed
    }

    pub fn domain(&self) -> &VASPDomain {
        &self.domain
    }

    pub fn address(&self) -> AccountAddress {
        self.address
    }
}

impl MoveStructType for VASPDomainEvent {
    const MODULE_NAME: &'static IdentStr = VASP_DOMAIN_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("VASPDomainEvent");
}

impl MoveResource for VASPDomainEvent {}

#[derive(Debug, Serialize, Deserialize)]
pub struct VASPDomainManager {
    vasp_domain_events: EventHandle,
}

impl VASPDomainManager {
    pub fn vasp_domain_events(&self) -> &EventHandle {
        &self.vasp_domain_events
    }
}

impl MoveStructType for VASPDomainManager {
    const MODULE_NAME: &'static IdentStr = VASP_DOMAIN_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("VASPDomainManager");
}

impl MoveResource for VASPDomainManager {}
