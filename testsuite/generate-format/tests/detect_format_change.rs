// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use generate_format::{add_proptest_serialization_tracing, FILE_PATH};
use serde_reflection::{RegistryOwned, Tracer};
use serde_yaml;
use std::collections::BTreeMap;

static MESSAGE: &str = "Run `cargo run -p generate-format -- --record` to refresh the records and `git diff` to study the changes before amending your commit.";

#[test]
fn test_format_change() {
    let mut tracer = Tracer::new(lcs::is_human_readable());
    tracer = add_proptest_serialization_tracing(tracer);
    let registry: BTreeMap<_, _> = tracer
        .registry()
        .unwrap()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

    let content = std::fs::read_to_string(FILE_PATH).unwrap();
    let expected = serde_yaml::from_str::<RegistryOwned>(content.as_str()).unwrap();

    for (key, value) in expected.iter() {
        assert_eq!(
            Some(value),
            registry.get(key),
            r#"
----
The extracted Serde format for type {} is missing or does not match the recorded value on disk. {}
----
"#,
            key,
            MESSAGE
        );
    }

    for key in registry.keys() {
        assert!(
            expected.contains_key(key),
            r#"
----
Type {} was added and has no recorded Serde format on disk yet. {}
----
"#,
            key,
            MESSAGE
        );
    }
}
