// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::DiscoveryChangeListener;
use channel::diem_channel;
use diem_config::network_id::NetworkContext;
use diem_crypto::x25519::PublicKey;
use diem_network_address_encryption::Encryptor;
use diem_secure_storage::Storage;
use diem_types::on_chain_config::OnChainConfigPayload;
use network::connectivity_manager::ConnectivityRequest;
use std::sync::Arc;
use tokio::runtime::Handle;

pub struct ValidatorSetChangeListenerBuilder {
    listener: Option<DiscoveryChangeListener>,
}

impl ValidatorSetChangeListenerBuilder {
    pub fn create(
        network_context: Arc<NetworkContext>,
        expected_pubkey: PublicKey,
        encryptor: Encryptor<Storage>,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
        reconfig_events: diem_channel::Receiver<(), OnChainConfigPayload>,
    ) -> ValidatorSetChangeListenerBuilder {
        Self {
            listener: Some(DiscoveryChangeListener::validator_set(
                network_context,
                conn_mgr_reqs_tx,
                expected_pubkey,
                encryptor,
                reconfig_events,
            )),
        }
    }

    pub fn start(&mut self, executor: &Handle) -> &mut Self {
        let listener = self.listener.take().expect("Listener must be built");
        listener.start(executor);
        self
    }
}
