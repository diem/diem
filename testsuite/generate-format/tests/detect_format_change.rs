// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use generate_format::Corpus;
use serde_reflection::RegistryOwned;
use serde_yaml;

#[test]
fn test_that_recorded_formats_did_not_change() {
    for corpus in Corpus::values() {
        let registry: RegistryOwned = corpus
            .get_registry(/* skip_deserialization */ false)
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        // Some corpus may not be recorded on disk.
        if let Some(path) = corpus.output_file() {
            let content = std::fs::read_to_string(path).unwrap();
            let expected = serde_yaml::from_str::<RegistryOwned>(content.as_str()).unwrap();
            assert_registry_has_not_changed(&corpus.to_string(), path, registry, expected);
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

fn assert_registry_has_not_changed(
    name: &str,
    path: &str,
    registry: RegistryOwned,
    expected: RegistryOwned,
) {
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

    let value1 = serde_yaml::from_str::<RegistryOwned>(yaml1).unwrap();
    let value2 = serde_yaml::from_str::<RegistryOwned>(yaml2).unwrap();
    assert_ne!(value1, value2);
    assert_ne!(value1.get("Person").unwrap(), value2.get("Person").unwrap());
}
