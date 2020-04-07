// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Dictates a set of permissions
#[derive(Debug, Deserialize, Serialize)]
pub struct Policy {
    pub permissions: Vec<Permission>,
}

impl Policy {
    pub fn new(permissions: Vec<Permission>) -> Self {
        Self { permissions }
    }

    pub fn public() -> Self {
        Self::new(vec![Permission::new(
            Identity::Anyone,
            vec![Capability::Read, Capability::Write],
        )])
    }
}

/// Maps an identity to a set of capabilities
#[derive(Debug, Deserialize, Serialize)]
pub struct Permission {
    pub id: Identity,
    pub capabilities: Vec<Capability>,
}

impl Permission {
    pub fn new(id: Identity, capabilities: Vec<Capability>) -> Self {
        Self { id, capabilities }
    }
}

/// Id represents a Libra internal identifier for a given process. For example, safety_rules or
/// key_manager. It is up to the Storage and its deployment to translate these identifiers into
/// verifiable material. For example, the process running safety_rules may have a token that is
/// intended for only safety_rules to own. The specifics are left to the implementation of the
/// storage backend interface layer.
#[derive(Debug, Deserialize, Serialize)]
pub enum Identity {
    User(String),
    Anyone,
    NoOne,
}

/// Represents actions
#[derive(Debug, Deserialize, Serialize)]
pub enum Capability {
    Export,
    Read,
    Rotate,
    Sign,
    Write,
}
