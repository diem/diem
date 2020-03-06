// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::transaction::SCRIPT_HASH_LENGTH;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashSet, hash::BuildHasher};

/// Holds the VM configuration, currently this is only the publishing options for scripts and
/// modules, but in the future this may need to be expanded to hold more information.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct VMConfig {
    pub publishing_options: VMPublishingOption,
}

impl Default for VMConfig {
    fn default() -> VMConfig {
        let whitelist = vec![
            "8e2c0ba80c130faac6cb3903f935b007f288b9159a34c52ec342046e3d914b91",
            "37ac01c31fb95c0d3f421e0f0f3bc9627978e2f2151a8caaab0459b6a249f4c2",
            "cb61dc36b6878381b3e50dace6fca7b9c2237c1290e9ac8da696888402595448",
            "4e20e9e1aa3c88060252498e8cb887d8e2ac311521518a22b3c5084cd71b193b",
            "785530b91de4335ced642551f3b82329c3d5d7ab499cf8b9b1849a48a4103bd2",
            "a25ca5ae3f600fa45c3a1442785035823f6181dc422948d1c9f8c9a2cb579532",
            "d91b8ce171bb89a86201b7d7dda16970e937fac17666a2cc8e2826987ac58dd0",
            "9dba5d25a76306d3457904421320873c0c79f264d12a1104d384778263c3b4f1",
            "d1fda1dfdd07a513e996ee342bbd6d4fead20c6291c53b22801f6c6e336a384e",
        ]
        .iter()
        .map(|s| string_to_script_hash(s))
        .collect();

        VMConfig {
            publishing_options: VMPublishingOption::Locked(whitelist),
        }
    }
}

impl VMConfig {
    /// Creates a new `VMConfig` where the whitelist is empty. This should only be used for testing.
    #[allow(non_snake_case)]
    #[doc(hidden)]
    #[cfg(any(test, feature = "fuzzing"))]
    pub fn empty_whitelist_FOR_TESTING() -> Self {
        VMConfig {
            publishing_options: VMPublishingOption::Locked(HashSet::new()),
        }
    }
}

/// Defines and holds the publishing policies for the VM. There are three possible configurations:
/// 1. No module publishing, only whitelisted scripts are allowed.
/// 2. No module publishing, custom scripts are allowed.
/// 3. Both module publishing and custom scripts are allowed.
/// We represent these as an enum instead of a struct since whitelisting and module/script
/// publishing are mutually exclusive options.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "whitelist")]
pub enum VMPublishingOption {
    /// Only allow scripts on a whitelist to be run
    #[serde(deserialize_with = "deserialize_whitelist")]
    #[serde(serialize_with = "serialize_whitelist")]
    Locked(HashSet<[u8; SCRIPT_HASH_LENGTH]>),
    /// Allow custom scripts, but _not_ custom module publishing
    CustomScripts,
    /// Allow both custom scripts and custom module publishing
    Open,
}

impl VMPublishingOption {
    pub fn is_open(&self) -> bool {
        match self {
            VMPublishingOption::Open => true,
            _ => false,
        }
    }

    pub fn get_whitelist_set(&self) -> Option<&HashSet<[u8; SCRIPT_HASH_LENGTH]>> {
        match self {
            VMPublishingOption::Locked(whitelist) => Some(&whitelist),
            _ => None,
        }
    }
}

fn string_to_script_hash(input: &str) -> [u8; SCRIPT_HASH_LENGTH] {
    let mut hash = [0u8; SCRIPT_HASH_LENGTH];
    let decoded_hash =
        hex::decode(input).expect("Unable to decode script hash from configuration file.");
    assert_eq!(decoded_hash.len(), SCRIPT_HASH_LENGTH);
    hash.copy_from_slice(decoded_hash.as_slice());
    hash
}

fn deserialize_whitelist<'de, D>(
    deserializer: D,
) -> Result<HashSet<[u8; SCRIPT_HASH_LENGTH]>, D::Error>
where
    D: Deserializer<'de>,
{
    let whitelisted_scripts: Vec<String> = Deserialize::deserialize(deserializer)?;
    let whitelist = whitelisted_scripts
        .iter()
        .map(|s| string_to_script_hash(s))
        .collect();
    Ok(whitelist)
}

fn serialize_whitelist<S, H>(
    whitelist: &HashSet<[u8; SCRIPT_HASH_LENGTH], H>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    H: BuildHasher,
{
    let encoded_whitelist: Vec<String> = whitelist.iter().map(hex::encode).collect();
    encoded_whitelist.serialize(serializer)
}
