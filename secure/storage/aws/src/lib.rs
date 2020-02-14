// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use rusoto_secretsmanager::SecretsManagerClient;

pub struct SecretsManagerStorage {
    client: SecretsManagerClient,
}

impl SecretsManagerStorage {
    pub fn new(region: &str) -> Self {
        Self {
            client: SecretsManagerClient::new(region.parse().unwrap())
        }
    }
}
