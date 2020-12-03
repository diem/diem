// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ConfigurationChangeListener;
use channel::diem_channel;
use diem_config::network_id::NetworkContext;
use diem_network_address_encryption::Encryptor;
use diem_types::on_chain_config::OnChainConfigPayload;
use network::connectivity_manager::ConnectivityRequest;
use std::sync::Arc;
use tokio::runtime::Handle;

struct ConfigurationChangeListenerConfig {
    network_context: Arc<NetworkContext>,
    encryptor: Encryptor,
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    reconfig_events: diem_channel::Receiver<(), OnChainConfigPayload>,
}

impl ConfigurationChangeListenerConfig {
    fn new(
        network_context: Arc<NetworkContext>,
        encryptor: Encryptor,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
        reconfig_events: diem_channel::Receiver<(), OnChainConfigPayload>,
    ) -> Self {
        Self {
            network_context,
            encryptor,
            conn_mgr_reqs_tx,
            reconfig_events,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
enum State {
    CREATED,
    BUILT,
    STARTED,
}

pub struct ConfigurationChangeListenerBuilder {
    config: Option<ConfigurationChangeListenerConfig>,
    listener: Option<ConfigurationChangeListener>,
    state: State,
}

impl ConfigurationChangeListenerBuilder {
    pub fn create(
        network_context: Arc<NetworkContext>,
        encryptor: Encryptor,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
        reconfig_events: diem_channel::Receiver<(), OnChainConfigPayload>,
    ) -> ConfigurationChangeListenerBuilder {
        Self {
            config: Some(ConfigurationChangeListenerConfig::new(
                network_context,
                encryptor,
                conn_mgr_reqs_tx,
                reconfig_events,
            )),
            listener: None,
            state: State::CREATED,
        }
    }

    pub fn build(&mut self) -> &mut Self {
        assert_eq!(self.state, State::CREATED);
        self.state = State::BUILT;
        let config = self.config.take().expect("Listener must be configured");
        self.listener = Some(ConfigurationChangeListener::new(
            config.network_context,
            config.encryptor,
            config.conn_mgr_reqs_tx,
            config.reconfig_events,
        ));
        self
    }

    pub fn start(&mut self, executor: &Handle) -> &mut Self {
        assert_eq!(self.state, State::BUILT);
        self.state = State::STARTED;
        let listener = self.listener.take().expect("Listener must be built");
        executor.spawn(listener.start());
        self
    }
}
