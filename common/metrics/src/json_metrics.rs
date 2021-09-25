// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

// Use to expose non-numeric metrics
pub fn get_json_metrics() -> BTreeMap<String, String> {
    let mut json_metrics: BTreeMap<String, String> = BTreeMap::new();
    json_metrics = add_revision_hash(json_metrics);
    json_metrics
}

fn add_revision_hash(mut json_metrics: BTreeMap<String, String>) -> BTreeMap<String, String> {
    json_metrics.insert("revision".to_string(), env!("GIT_REV").to_string());
    json_metrics
}
