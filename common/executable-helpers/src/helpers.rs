// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::config::{LoggerConfig, NodeConfig};
use libra_logger::prelude::*;
use libra_types::PeerId;
use slog_scope::GlobalLoggerGuard;
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

pub fn setup_executable(
    config_path: Option<&Path>,
    no_logging: bool,
) -> (NodeConfig, Option<GlobalLoggerGuard>) {
    crash_handler::setup_panic_handler();
    let mut _logger = set_default_global_logger(no_logging, &LoggerConfig::default());

    let config = load_config_from_path(config_path);

    // Reset the global logger using config (for chan_size currently).
    // We need to drop the global logger guard first before resetting it.
    _logger = None;
    let logger = set_default_global_logger(no_logging, &config.logger);
    for network in &config.full_node_networks {
        setup_metrics(network.peer_id, &config);
    }
    if let Some(network) = &config.validator_network {
        setup_metrics(network.peer_id, &config);
    }

    (config, logger)
}

fn set_default_global_logger(
    is_logging_disabled: bool,
    logger_config: &LoggerConfig,
) -> Option<GlobalLoggerGuard> {
    if is_logging_disabled {
        return None;
    }

    Some(libra_logger::set_default_global_logger(
        logger_config.is_async,
        Some(logger_config.chan_size),
    ))
}
