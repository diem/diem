// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the structs transported during the network handshake protocol v1.
//! These should serialize as per [link](TODO: Add ref).
//!
//! During the v1 Handshake protocol, both end-points of a connection send a serialized and
//! length-prefixed `HandshakeMsg` to each other. The handshake message contains a map from
//! supported messaging protocol versions to a bit vector representing application protocols
//! supported over that messaging protocol. On receipt, both ends will determine the highest
//! intersecting messaging protocol version and use that for the remainder of the session.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(test)]
mod test;

/// The HandshakeMsg contains a mapping from MessagingProtocolVersion suppported by the node to a
/// bit-vector specifying application-level protocols supported over that version.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HandshakeMsg {
    pub supported_protocols: BTreeMap<MessagingProtocolVersion, bitvec::BitVec>,
}

/// Enum representing different versions of the Libra network protocol. These should be listed from
/// old to new, old having the smallest value.
/// We derive `PartialOrd` since nodes need to find highest intersecting protocol version.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Debug, Hash, Deserialize, Serialize)]
pub enum MessagingProtocolVersion {
    V1 = 0,
}
