// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

// Use to expose non-numeric metrics
pub fn get_json_metrics() -> HashMap<String, String> {
    let mut json_metrics: HashMap<String, String> = HashMap::new();
    json_metrics = add_revision_hash(json_metrics);
    json_metrics
}

fn add_revision_hash(mut json_metrics: HashMap<String, String>) -> HashMap<String, String> {
    json_metrics.insert("revision".to_string(), env!("GIT_REV").to_string());
    json_metrics
}
