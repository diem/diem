// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use generate_format::Corpus;
use serde_reflection::Registry;
use std::collections::{btree_map::Entry, BTreeMap};

#[test]
fn analyze_serde_formats() {
    let mut all_corpuses = BTreeMap::new();

    for corpus in Corpus::values() {
        // Compute the Serde formats of this corpus by analyzing the codebase.
        let registry = corpus.get_registry();

        // If the corpus was recorded on disk, test that the formats have not changed since then.
        if let Some(path) = corpus.output_file() {
            let content = std::fs::read_to_string(path).unwrap();
            let expected = serde_yaml::from_str::<Registry>(content.as_str()).unwrap();
            assert_registry_has_not_changed(&corpus.to_string(), path, registry.clone(), expected);
        }

        // Test that the definitions in all corpus are unique.
        for (key, value) in registry {
            match all_corpuses.entry(key.clone()) {
                Entry::Vacant(e) => {
                    e.insert(value);
                }
                Entry::Occupied(e) => assert_eq!(
                    e.get(),
                    &value,
                    "Type {} in corpus {} differs with previous definition in another corpus: {:?} vs {:?}",
                    key,
                    corpus.to_string(),
                    e.get(),
                    &value,
                ),
            }
        }
    }
}

fn message(name: &str) -> String {
    format!(
        r#"
You may run `cargo run -p generate-format -- --corpus {} --record` to refresh the records.
Please verify the changes to the recorded file(s) and consider tagging your pull-request as `breaking`."#,
        name
    )
}

fn assert_registry_has_not_changed(name: &str, path: &str, registry: Registry, expected: Registry) {
    for (key, value) in expected.iter() {
        assert_eq!(
            Some(value),
            registry.get(key),
            r#"
----
The recorded format for type `{}` was removed or does not match the recorded value in {}.{}
----
"#,
            key,
            path,
            message(name),
        );
    }

    for key in registry.keys() {
        assert!(
            expected.contains_key(key),
            r#"
----
Type `{}` was added and has no recorded format in {} yet.{}
----
"#,
            key,
            path,
            message(name),
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

    let value1 = serde_yaml::from_str::<Registry>(yaml1).unwrap();
    let value2 = serde_yaml::from_str::<Registry>(yaml2).unwrap();
    assert_ne!(value1, value2);
    assert_ne!(value1.get("Person").unwrap(), value2.get("Person").unwrap());
}
