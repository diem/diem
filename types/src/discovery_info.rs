// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use libra_crypto::x25519::X25519StaticPublicKey;
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
    account_address: AccountAddress,
    // This static pubkey is used in the connection handshake to authenticate
    // this particular validator.
    validator_network_identity_pubkey: X25519StaticPublicKey,
    // Other validators can dial this validator at this multiaddress.
    validator_network_address: Multiaddr,
    // This static pubkey is used in the connection handshake to authenticate
    // this validator's full nodes.
    fullnodes_network_identity_pubkey: X25519StaticPublicKey,
    // Other full nodes and clients can dial this validator's full nodes at this
    // multiaddress.
    fullnodes_network_address: Multiaddr,
}

impl DiscoveryInfo {
    pub fn new(
        account_address: AccountAddress,
        validator_network_identity_pubkey: X25519StaticPublicKey,
        validator_network_address: Multiaddr,
        fullnodes_network_identity_pubkey: X25519StaticPublicKey,
        fullnodes_network_address: Multiaddr,
    ) -> Self {
        DiscoveryInfo {
            account_address,
            validator_network_identity_pubkey,
            validator_network_address,
            fullnodes_network_identity_pubkey,
            fullnodes_network_address,
        }
    }

    pub fn account_address(&self) -> &AccountAddress {
        &self.account_address
    }

    pub fn validator_network_identity_pubkey(&self) -> &X25519StaticPublicKey {
        &self.validator_network_identity_pubkey
    }

    pub fn validator_network_address(&self) -> &Multiaddr {
        &self.validator_network_address
    }

    pub fn fullnodes_network_identity_pubkey(&self) -> &X25519StaticPublicKey {
        &self.fullnodes_network_identity_pubkey
    }

    pub fn fullnodes_network_address(&self) -> &Multiaddr {
        &self.fullnodes_network_address
    }
}
