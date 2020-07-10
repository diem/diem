// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ConfigurationChangeListener;
use channel::libra_channel;
use libra_config::config::RoleType;
use libra_network_address::encrypted::{Key, KeyVersion};
use libra_types::on_chain_config::OnChainConfigPayload;
use network::connectivity_manager::ConnectivityRequest;
use std::collections::HashMap;
use tokio::runtime::Handle;

struct ConfigurationChangeListenerConfig {
    role: RoleType,
    shared_val_netaddr_key_map: HashMap<KeyVersion, Key>,
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
}

impl ConfigurationChangeListenerConfig {
    fn new(
        role: RoleType,
        shared_val_netaddr_key_map: HashMap<KeyVersion, Key>,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
        reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
    ) -> Self {
        Self {
            role,
            shared_val_netaddr_key_map,
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
        role: RoleType,
        shared_val_netaddr_key_map: HashMap<KeyVersion, Key>,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
        reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
    ) -> ConfigurationChangeListenerBuilder {
        Self {
            config: Some(ConfigurationChangeListenerConfig::new(
                role,
                shared_val_netaddr_key_map,
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
            config.role,
            config.shared_val_netaddr_key_map,
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
