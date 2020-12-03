// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This provides a runtime separation between ExecutionCorrectness and the rest without requiring the
//! use of processes. Rust does not support fork and so the mechanics to actually construct a
//! ExecutionCorrectness that would run together and be started by SafetyRules requires a separate binary and
//! making a call to start that via a command. This is a lightweight means of accomplishing a goal
//! in testing correctness of the communication layer between ExecutionCorrectness and SafetyRules.

use crate::remote_service::{self, RemoteService};
use diem_config::utils;
use diem_crypto::ed25519::Ed25519PrivateKey;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread::{self, JoinHandle},
};

/// ThreadClient is the actual owner of the thread but in the context of SafetyRules and ExecutionCorrectness
/// is on the client side of the operations as it makes queries / requests to ExecutionCorrectness.
pub struct ThreadService {
    _child: JoinHandle<()>,
    server_addr: SocketAddr,
    network_timeout: u64,
}

impl ThreadService {
    pub fn new(
        storage_addr: SocketAddr,
        prikey: Option<Ed25519PrivateKey>,
        network_timeout: u64,
    ) -> Self {
        let listen_port = utils::get_available_port();
        let listen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), listen_port);
        let server_addr = listen_addr;

        let child = thread::spawn(move || {
            remote_service::execute(storage_addr, listen_addr, prikey, network_timeout)
        });

        Self {
            _child: child,
            server_addr,
            network_timeout,
        }
    }
}

impl RemoteService for ThreadService {
    fn server_address(&self) -> SocketAddr {
        self.server_addr
    }
    fn network_timeout(&self) -> u64 {
        self.network_timeout
    }
}
