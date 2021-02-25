// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_config::{
    config::{NodeConfig, PeerRole, RoleType},
    network_id::NetworkId,
};
use diem_types::PeerId;
use enum_dispatch::enum_dispatch;
use rand::rngs::StdRng;

/// A union type for all types of simulated nodes
#[enum_dispatch(NodeTrait)]
#[derive(Clone, Debug)]
pub enum Node {
    Validator(ValidatorNode),
    ValidatorFull(ValidatorFullNode),
    Full(FullNode),
}

/// Accessors to the union type of all simulated nodes
#[enum_dispatch]
pub trait NodeTrait {
    fn primary_peer_id(&self) -> PeerId;
    fn primary_network(&self) -> NetworkId;
    fn role(&self) -> RoleType;
    fn peer_role(&self) -> PeerRole;
    fn secondary_peer_id(&self) -> Option<PeerId>;
    fn secondary_network(&self) -> Option<NetworkId>;

    fn peer_id(&self, is_primary: bool) -> PeerId {
        if is_primary {
            self.primary_peer_id()
        } else {
            self.secondary_peer_id().unwrap()
        }
    }

    fn network(&self, is_primary: bool) -> NetworkId {
        if is_primary {
            self.primary_network()
        } else {
            self.secondary_network().unwrap()
        }
    }
}

#[derive(Clone, Debug)]
pub struct ValidatorNode {
    primary_peer_id: PeerId,
}

impl ValidatorNode {
    fn new(peer_id: PeerId) -> Self {
        ValidatorNode {
            primary_peer_id: peer_id,
        }
    }
}

impl NodeTrait for ValidatorNode {
    fn primary_peer_id(&self) -> PeerId {
        self.primary_peer_id
    }

    fn primary_network(&self) -> NetworkId {
        NetworkId::Validator
    }

    fn role(&self) -> RoleType {
        RoleType::Validator
    }

    fn peer_role(&self) -> PeerRole {
        PeerRole::Validator
    }

    fn secondary_peer_id(&self) -> Option<PeerId> {
        None
    }

    fn secondary_network(&self) -> Option<NetworkId> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct ValidatorFullNode {
    primary_peer_id: PeerId,
    secondary_peer_id: PeerId,
}

impl ValidatorFullNode {
    fn new(primary_peer_id: PeerId, secondary_peer_id: PeerId) -> Self {
        ValidatorFullNode {
            primary_peer_id,
            secondary_peer_id,
        }
    }
}

impl NodeTrait for ValidatorFullNode {
    fn primary_peer_id(&self) -> PeerId {
        self.primary_peer_id
    }

    fn primary_network(&self) -> NetworkId {
        NetworkId::vfn_network()
    }

    fn role(&self) -> RoleType {
        RoleType::FullNode
    }

    fn peer_role(&self) -> PeerRole {
        PeerRole::ValidatorFullNode
    }

    fn secondary_peer_id(&self) -> Option<PeerId> {
        Some(self.secondary_peer_id)
    }

    fn secondary_network(&self) -> Option<NetworkId> {
        Some(NetworkId::Public)
    }
}

#[derive(Clone, Debug)]
pub struct FullNode {
    peer_id: PeerId,
    peer_role: PeerRole,
}

impl FullNode {
    fn new(peer_id: PeerId, peer_role: PeerRole) -> Self {
        FullNode { peer_id, peer_role }
    }
}

impl NodeTrait for FullNode {
    fn primary_peer_id(&self) -> PeerId {
        self.peer_id
    }

    fn primary_network(&self) -> NetworkId {
        NetworkId::Public
    }

    fn role(&self) -> RoleType {
        RoleType::FullNode
    }

    fn peer_role(&self) -> PeerRole {
        self.peer_role
    }

    fn secondary_peer_id(&self) -> Option<PeerId> {
        None
    }

    fn secondary_network(&self) -> Option<NetworkId> {
        None
    }
}

pub fn validator(rng: &mut StdRng, account_idx: u32) -> (ValidatorNode, NodeConfig) {
    let config =
        NodeConfig::random_with_template(account_idx, &NodeConfig::default_for_validator(), rng);

    let peer_id = config
        .validator_network
        .as_ref()
        .expect("Validator must have a validator network")
        .peer_id();
    (ValidatorNode::new(peer_id), config)
}

pub fn vfn(rng: &mut StdRng, account_idx: u32) -> (ValidatorFullNode, NodeConfig) {
    let vfn_config = NodeConfig::random_with_template(
        account_idx,
        &NodeConfig::default_for_validator_full_node(),
        rng,
    );

    let primary_peer_id = vfn_config
        .full_node_networks
        .iter()
        .find(|network| network.network_id.is_vfn_network())
        .expect("VFN must have a VFN network")
        .peer_id();

    let secondary_peer_id = vfn_config
        .full_node_networks
        .iter()
        .filter(|network| network.network_id == NetworkId::Public)
        .last()
        .unwrap()
        .peer_id();
    (
        ValidatorFullNode::new(primary_peer_id, secondary_peer_id),
        vfn_config,
    )
}

pub fn full_node(
    rng: &mut StdRng,
    account_idx: u32,
    peer_role: PeerRole,
) -> (FullNode, NodeConfig) {
    let fn_config = NodeConfig::random_with_template(
        account_idx,
        &NodeConfig::default_for_public_full_node(),
        rng,
    );

    let peer_id = fn_config
        .full_node_networks
        .iter()
        .find(|network| network.network_id == NetworkId::Public)
        .expect("Full Node must have a public network")
        .peer_id();
    (FullNode::new(peer_id, peer_role), fn_config)
}
