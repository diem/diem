// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::new_local_swarm,
    test_utils::{
        assert_balance, create_and_fund_account, transfer_coins, transfer_coins_non_blocking,
    },
};
use diem_config::{
    config::{DiscoveryMethod, NodeConfig, Peer, PeerRole, HANDSHAKE_VERSION},
    network_id::NetworkId,
};
use diem_types::network_address::{NetworkAddress, Protocol};
use forge::{NodeExt, Swarm, SwarmExt};
use std::{
    collections::BTreeSet,
    net::Ipv4Addr,
    time::{Duration, Instant},
};

#[test]
fn test_full_node_basic_flow() {
    let mut swarm = new_local_swarm(1);

    let transaction_factory = swarm.chain_info().transaction_factory();
    let version = swarm.versions().max().unwrap();
    let validator_peer_id = swarm.validators().next().unwrap().peer_id();
    let vfn_peer_id = swarm
        .add_validator_fullnode(
            &version,
            NodeConfig::default_for_validator_full_node(),
            validator_peer_id,
        )
        .unwrap();
    let pfn_peer_id = swarm
        .add_full_node(&version, NodeConfig::default_for_public_full_node())
        .unwrap();
    swarm
        .validator_mut(validator_peer_id)
        .unwrap()
        .wait_until_healthy(Instant::now() + Duration::from_secs(10))
        .unwrap();
    for fullnode in swarm.full_nodes_mut() {
        fullnode
            .wait_until_healthy(Instant::now() + Duration::from_secs(10))
            .unwrap();
    }

    // create clients for all nodes
    let validator_client = swarm
        .validator(validator_peer_id)
        .unwrap()
        .json_rpc_client();
    let vfn_client = swarm.full_node(vfn_peer_id).unwrap().json_rpc_client();
    let pfn_client = swarm.full_node(pfn_peer_id).unwrap().json_rpc_client();

    let mut account_0 = create_and_fund_account(&mut swarm, 10);
    let account_1 = create_and_fund_account(&mut swarm, 10);

    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(10))
        .unwrap();

    // Send txn to PFN
    let _txn = transfer_coins(
        &pfn_client,
        &transaction_factory,
        &mut account_0,
        &account_1,
        1,
    );

    assert_balance(&validator_client, &account_0, 9);
    assert_balance(&validator_client, &account_1, 11);
    assert_balance(&vfn_client, &account_0, 9);
    assert_balance(&vfn_client, &account_1, 11);
    assert_balance(&pfn_client, &account_0, 9);
    assert_balance(&pfn_client, &account_1, 11);

    // Send txn to VFN
    let txn = transfer_coins(
        &vfn_client,
        &transaction_factory,
        &mut account_0,
        &account_1,
        1,
    );

    assert_balance(&validator_client, &account_0, 8);
    assert_balance(&validator_client, &account_1, 12);
    assert_balance(&vfn_client, &account_0, 8);
    assert_balance(&vfn_client, &account_1, 12);

    pfn_client
        .wait_for_signed_transaction(&txn, None, None)
        .unwrap();
    assert_balance(&pfn_client, &account_0, 8);
    assert_balance(&pfn_client, &account_1, 12);

    // Send txn to Validator
    let txn = transfer_coins(
        &vfn_client,
        &transaction_factory,
        &mut account_0,
        &account_1,
        1,
    );

    assert_balance(&validator_client, &account_0, 7);
    assert_balance(&validator_client, &account_1, 13);

    vfn_client
        .wait_for_signed_transaction(&txn, None, None)
        .unwrap();
    assert_balance(&vfn_client, &account_0, 7);
    assert_balance(&vfn_client, &account_1, 13);

    pfn_client
        .wait_for_signed_transaction(&txn, None, None)
        .unwrap();
    assert_balance(&pfn_client, &account_0, 7);
    assert_balance(&pfn_client, &account_1, 13);
}

#[test]
fn test_vfn_failover() {
    let mut swarm = new_local_swarm(4);
    let transaction_factory = swarm.chain_info().transaction_factory();
    let version = swarm.versions().max().unwrap();
    let validator_peer_ids = swarm.validators().map(|v| v.peer_id()).collect::<Vec<_>>();

    let validator = validator_peer_ids[1];
    let vfn = swarm
        .add_validator_fullnode(
            &version,
            NodeConfig::default_for_validator_full_node(),
            validator,
        )
        .unwrap();

    for validator in swarm.validators_mut() {
        validator
            .wait_until_healthy(Instant::now() + Duration::from_secs(10))
            .unwrap();
    }
    for fullnode in swarm.full_nodes_mut() {
        fullnode
            .wait_until_healthy(Instant::now() + Duration::from_secs(10))
            .unwrap();
        fullnode
            .wait_for_connectivity(Instant::now() + Duration::from_secs(60))
            .unwrap();
    }

    // Setup accounts
    let mut account_0 = create_and_fund_account(&mut swarm, 100);
    let account_1 = create_and_fund_account(&mut swarm, 100);

    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(10))
        .unwrap();

    // set up client
    let vfn_client = swarm.full_node(vfn).unwrap().json_rpc_client();

    // submit client requests directly to VFN of dead V
    swarm.validator_mut(validator).unwrap().stop();
    transfer_coins(
        &vfn_client,
        &transaction_factory,
        &mut account_0,
        &account_1,
        1,
    );

    for _ in 0..8 {
        transfer_coins_non_blocking(
            &vfn_client,
            &transaction_factory,
            &mut account_0,
            &account_1,
            1,
        );
    }

    transfer_coins(
        &vfn_client,
        &transaction_factory,
        &mut account_0,
        &account_1,
        1,
    );
}

#[test]
fn test_private_full_node() {
    let mut swarm = new_local_swarm(4);
    let transaction_factory = swarm.chain_info().transaction_factory();
    let version = swarm.versions().max().unwrap();

    // Here we want to add two swarms, a private full node, followed by a user full node connected to it
    let mut private_config = NodeConfig::default_for_public_full_node();
    let private_network = private_config.full_node_networks.first_mut().unwrap();
    // Disallow public connections
    private_network.max_inbound_connections = 0;
    // Also, we only want it to purposely connect to 1 VFN
    private_network.max_outbound_connections = 1;

    let mut user_config = NodeConfig::default_for_public_full_node();
    let user_network = user_config.full_node_networks.first_mut().unwrap();
    // Disallow fallbacks to VFNs
    user_network.max_outbound_connections = 1;
    user_network.discovery_method = DiscoveryMethod::None;

    // The secret sauce, add the user as a downstream to the seeds
    add_node_to_seeds(
        &mut private_config,
        &user_config,
        NetworkId::Public,
        PeerRole::Downstream,
    );

    // Now we need to connect the VFNs to the private swarm
    add_node_to_seeds(
        &mut private_config,
        swarm.validators().next().unwrap().config(),
        NetworkId::Public,
        PeerRole::PreferredUpstream,
    );
    let private = swarm.add_full_node(&version, private_config).unwrap();

    // And connect the user to the private swarm
    add_node_to_seeds(
        &mut user_config,
        swarm.full_node(private).unwrap().config(),
        NetworkId::Public,
        PeerRole::PreferredUpstream,
    );
    let user = swarm.add_full_node(&version, user_config).unwrap();

    swarm
        .wait_for_connectivity(Instant::now() + Duration::from_secs(60))
        .unwrap();

    // Ensure that User node is connected to private node and only the private node
    {
        let user_node = swarm.full_node(user).unwrap();
        assert_eq!(
            1,
            user_node
                .get_connected_peers(NetworkId::Public, None)
                .unwrap()
                .unwrap_or(0),
            "User node is connected to more than one peer"
        );
    }

    // read state from full node client
    let validator_client = swarm.validators().next().unwrap().json_rpc_client();
    let user_client = swarm.full_node(user).unwrap().json_rpc_client();

    let mut account_0 = create_and_fund_account(&mut swarm, 100);
    let account_1 = create_and_fund_account(&mut swarm, 10);

    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(60))
        .unwrap();

    // send txn from user node and check both validator and user node have correct balance
    transfer_coins(
        &user_client,
        &transaction_factory,
        &mut account_0,
        &account_1,
        10,
    );
    assert_balance(&user_client, &account_0, 90);
    assert_balance(&user_client, &account_1, 20);
    assert_balance(&validator_client, &account_0, 90);
    assert_balance(&validator_client, &account_1, 20);
}

fn add_node_to_seeds(
    dest_config: &mut NodeConfig,
    seed_config: &NodeConfig,
    network_id: NetworkId,
    peer_role: PeerRole,
) {
    let dest_network_config = dest_config
        .full_node_networks
        .iter_mut()
        .find(|network| network.network_id == network_id)
        .unwrap();
    let seed_network_config = seed_config
        .full_node_networks
        .iter()
        .find(|network| network.network_id == network_id)
        .unwrap();

    let seed_peer_id = seed_network_config.peer_id();
    let seed_key = seed_network_config.identity_key().public_key();

    let seed_peer = if peer_role != PeerRole::Downstream {
        // For upstreams, we know the address, but so don't duplicate the keys in the config (lazy way)
        // TODO: This is ridiculous, we need a better way to manipulate these `NetworkAddress`s
        let address = seed_network_config.listen_address.clone();
        let port_protocol = address
            .as_slice()
            .iter()
            .find(|protocol| matches!(protocol, Protocol::Tcp(_)))
            .unwrap();
        let address = NetworkAddress::from(Protocol::Ip4(Ipv4Addr::new(127, 0, 0, 1)))
            .push(port_protocol.clone())
            .push(Protocol::NoiseIK(seed_key))
            .push(Protocol::Handshake(HANDSHAKE_VERSION));

        Peer::new(vec![address], BTreeSet::new(), peer_role)
    } else {
        // For downstreams, we don't know the address, but we know the keys
        let mut seed_keys = BTreeSet::new();
        seed_keys.insert(seed_key);
        Peer::new(vec![], seed_keys, peer_role)
    };

    dest_network_config.seeds.insert(seed_peer_id, seed_peer);
}
