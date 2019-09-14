// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executable_helpers::helpers::{setup_executable, ARG_CONFIG_PATH, ARG_DISABLE_LOGGING};
use secret_service::secret_service_node;

/// Run a SecretService in its own process.
fn main() {
    let (config, _logger, _args) = setup_executable(
        "Libra Secret Service".to_string(),
        vec![ARG_CONFIG_PATH, ARG_DISABLE_LOGGING],
    );

    let secret_service_node = secret_service_node::SecretServiceNode::new(config);

    secret_service_node
        .run()
        .expect("Unable to run SecretService");
}
