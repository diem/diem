// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::network::{NetworkClient, NetworkServer};
use libra_config::utils;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[test]
fn test_ping() {
    let server_port = utils::get_available_port();
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);
    let mut server = NetworkServer::new(server_addr);
    let mut client = NetworkClient::connect(server_addr).unwrap();

    let data = vec![0, 1, 2, 3];
    client.write(&data).unwrap();
    let result = server.read().unwrap();
    assert_eq!(data, result);

    let data = vec![4, 5, 6, 7];
    server.write(&data).unwrap();
    let result = client.read().unwrap();
    assert_eq!(data, result);
}
