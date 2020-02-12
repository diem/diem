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
            "72879f8b1f13bde89da0d8f98feb96858379460b47c08a9c675ed205f4588d4b",
            "d12fa14a2433cb7d41c22d51fa13b71284c071209e6dd6cde5d70de7b4ccac1b",
            "91db31c1c0b0aa76aa187142cca435b67d6560bed7bdbe9de57e556844fc118a",
            "d20b64c9b93ec570a01bee1a8ca9019faac567fa97ffd0813ee1bae0a1437f8e",
            "22fb39863f38fac2e39d2f13b9b6b8eea20c3748d929c97a3c7a7786c19dae0e",
            "076a91ebb665e7d792cdec9afbcf94c691f1022aacaadd5a8d9f0c3cd66c65ba",
            "0a1a43ffdc03183add932adfb040854a60310c111f62119aae05481831cfa19d",
            "23309204120936fa3ee9ebbb3caef5b667e0787ce07b371a521a8ca394f69450",
            "f88614e0cd1b17a7a4fa0993c1159bd30d7df7258a6388225081c640026b826f",
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
