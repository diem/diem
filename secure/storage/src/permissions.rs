// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Permissions dictate which Ids may perform different operations.
#[derive(Debug, Deserialize, Serialize)]
pub struct Permissions {
    pub readers: Permission,
    pub writers: Permission,
}

/// Different possibilities for permissions, a set of Ids that are eligible, Any Id, or No Ids.
#[derive(Debug, Deserialize, Serialize)]
pub enum Permission {
    Users(Vec<Id>),
    Anyone,
    NoOne,
}

/// Id represents a Libra internal identifier for a given process. For example, safety_rules or
/// key_manager. It is up to the Storage and its deployment to translate these identifiers into
/// verifiable material. For example, the process running safety_rules may have a token that is
/// intended for only safety_rules to own. The specifics are left to the implementation of the
/// storage backend interface layer.
pub type Id = String;
