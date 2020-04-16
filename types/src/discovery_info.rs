// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use libra_crypto::x25519;
use parity_multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};

/// A validator's discovery information, which describes how to dial the
/// validator's node and full nodes.
///
/// Other validators will use the `validator_network_address` to dial the this
/// validator and only accept inbound connections from this validator if it's
/// authenticated to `validator_network_identity_pubkey`.
///
/// In contrast, other full nodes and clients will use the
/// `fullnodes_network_identity_pubkey` and `fullnodes_network_address` fields
/// respectively to contact this validator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryInfo {
    // The validator's account address.
    pub account_address: AccountAddress,
    // This static pubkey is used in the connection handshake to authenticate
    // this particular validator.
    pub validator_network_identity_pubkey: x25519::PublicKey,
    // Other validators can dial this validator at this multiaddress.
    pub validator_network_address: Multiaddr,
    // This static pubkey is used in the connection handshake to authenticate
    // this validator's full nodes.
    pub fullnodes_network_identity_pubkey: x25519::PublicKey,
    // Other full nodes and clients can dial this validator's full nodes at this
    // multiaddress.
    pub fullnodes_network_address: Multiaddr,
}
