// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use generate_format::{add_deserialization_tracing, add_proptest_serialization_tracing, FILE_PATH};
use serde_reflection::{RegistryOwned, Tracer};
use serde_yaml;
use std::collections::BTreeMap;

static MESSAGE: &str = r#"
You may run `cargo run -p generate-format -- --record` to refresh the records.
Please verify the changes to the recorded file(s) and tag your pull-request as `breaking`."#;

#[test]
fn test_recorded_formats_did_not_change() {
    let mut tracer = Tracer::new(lcs::is_human_readable());
    tracer = add_proptest_serialization_tracing(tracer);
    tracer = add_deserialization_tracing(tracer);
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
The recorded format for type `{}` is missing or does not match the recorded value on disk.{}
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
Type `{}` was added and has no recorded format on disk yet.{}
----
"#,
            key,
            MESSAGE
        );
    }
}

#[test]
fn test_we_can_detect_changes_in_yaml() {
    let yaml1 = r#"---
Person:
  ENUM:
    0:
      NickName:
        NEWTYPE:
          STR
"#;

    let yaml2 = r#"---
Person:
  ENUM:
    0:
      NickName:
        NEWTYPE:
          STR
    1:
      FullName: UNIT
"#;

    let value1 = serde_yaml::from_str::<RegistryOwned>(yaml1).unwrap();
    let value2 = serde_yaml::from_str::<RegistryOwned>(yaml2).unwrap();
    assert_ne!(value1, value2);
    assert_ne!(value1.get("Person").unwrap(), value2.get("Person").unwrap());
}
