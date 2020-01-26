// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This provides a execution separation between SafetyRules and Consensus without requiring the
//! use of processes. Rust does not support fork and so the mechanics to actually construct a
//! SafetyRules that would run together and be started by Consensus requires a separate binary and
//! making a call to start that via a command. This is a lightweight means of accomplishing a goal
//! in testing correctness of the communication layer between Consensus and SafetyRules.

use crate::{
    persistent_storage::PersistentStorage,
    remote_service::{self, RemoteService},
};
use consensus_types::common::Payload;
use libra_config::utils;
use libra_types::crypto_proxies::ValidatorSigner;
use std::{
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread::{self, JoinHandle},
};

/// ThreadClient is the actual owner of the thread but in the context of Consenus and SafetyRules
/// is on the client side of the operations as it makes queries / requests to SafetyRules.
pub struct ThreadService<T> {
    _child: JoinHandle<()>,
    server_addr: SocketAddr,
    marker: PhantomData<T>,
}

impl<T: Payload> ThreadService<T> {
    pub fn new(storage: PersistentStorage, validator_signer: ValidatorSigner) -> Self {
        let listen_port = utils::get_available_port();
        let listen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), listen_port);
        let server_addr = listen_addr;

        let child = thread::spawn(move || {
            remote_service::execute::<T>(storage, validator_signer, listen_addr)
        });

        Self {
            _child: child,
            server_addr,
            marker: PhantomData,
        }
    }
}

impl<T: Payload> RemoteService<T> for ThreadService<T> {
    fn server_address(&self) -> SocketAddr {
        self.server_addr
    }
}
