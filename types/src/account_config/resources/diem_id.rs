// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress, diem_id_identifier::DiemIdVaspDomainIdentifier,
    event::EventHandle,
};
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

/// The Identifier for the DiemID module.
pub const DIEM_ID_MODULE_IDENTIFIER: &IdentStr = ident_str!("DiemId");

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiemIdDomains {
    domains: Vec<DiemIdDomain>,
}

impl DiemIdDomains {
    pub fn domains(&self) -> &[DiemIdDomain] {
        &self.domains
    }
}

impl MoveStructType for DiemIdDomains {
    const MODULE_NAME: &'static IdentStr = DIEM_ID_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("DiemIdDomains");
}

impl MoveResource for DiemIdDomains {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiemIdDomain {
    domain: DiemIdVaspDomainIdentifier,
}

impl DiemIdDomain {
    pub fn domain(&self) -> &DiemIdVaspDomainIdentifier {
        &self.domain
    }
}

impl MoveStructType for DiemIdDomain {
    const MODULE_NAME: &'static IdentStr = DIEM_ID_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("DiemIdDomain");
}

impl MoveResource for DiemIdDomain {}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiemIdDomainEvent {
    // Whether a domain was added or removed
    removed: bool,
    // Diem ID Domain string of the account
    domain: DiemIdDomain,
    // On-chain account address
    address: AccountAddress,
}

impl DiemIdDomainEvent {
    pub fn removed(&self) -> bool {
        self.removed
    }

    pub fn domain(&self) -> &DiemIdDomain {
        &self.domain
    }

    pub fn address(&self) -> &AccountAddress {
        &self.address
    }
}

impl MoveStructType for DiemIdDomainEvent {
    const MODULE_NAME: &'static IdentStr = DIEM_ID_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("DiemIdDomainEvent");
}

impl MoveResource for DiemIdDomainEvent {}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiemIdDomainManager {
    diem_id_domain_events: EventHandle,
}

impl DiemIdDomainManager {
    pub fn diem_id_domain_events(&self) -> &EventHandle {
        &self.diem_id_domain_events
    }
}

impl MoveStructType for DiemIdDomainManager {
    const MODULE_NAME: &'static IdentStr = DIEM_ID_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("DiemIdDomainManager");
}

impl MoveResource for DiemIdDomainManager {}
