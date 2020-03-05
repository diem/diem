// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::config::{LoggerConfig, NodeConfig};
use libra_logger::{info, Logger};
use libra_types::PeerId;
use std::path::Path;

pub fn load_config_from_path(config_path: Option<&Path>) -> NodeConfig {
    // Load the config
    let node_config = match config_path {
        Some(path) => {
            info!("Loading node config from: {}", path.display());
            NodeConfig::load(path).expect("Failed to load node config.")
        }
        None => {
            info!("Loading test configs");
            NodeConfig::random()
        }
    };

    // Node configuration contains important ephemeral port information and should
    // not be subject to being disabled as with other logs
    println!("Using node config {:?}", &node_config);

    node_config
}

pub fn setup_metrics(peer_id: PeerId, node_config: &NodeConfig) {
    if node_config.metrics.enabled {
        libra_metrics::dump_all_metrics_to_file_periodically(
            &node_config.metrics.dir(),
            &format!("{}.metrics", peer_id),
            node_config.metrics.collection_interval_ms,
        );
    }
}

pub fn setup_executable(config_path: Option<&Path>, no_logging: bool) -> NodeConfig {
    let config = load_config_from_path(config_path);

    crash_handler::setup_panic_handler();
    set_default_global_logger(no_logging, &config.logger);
    for network in &config.full_node_networks {
        setup_metrics(network.peer_id, &config);
    }
    if let Some(network) = &config.validator_network {
        setup_metrics(network.peer_id, &config);
    }

    config
}

fn set_default_global_logger(is_logging_disabled: bool, logger_config: &LoggerConfig) {
    if is_logging_disabled {
        return;
    }

    Logger::new()
        .channel_size(logger_config.chan_size)
        .is_async(logger_config.is_async)
        .level(logger_config.level)
        .init();
}
