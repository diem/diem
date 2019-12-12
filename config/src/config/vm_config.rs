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
            "0c346f47196df53f9a329a3a5bca0026c0659a9eaad4ba9125f79dc7ed41a09b",
            "6e85e269b8a5a65804d7d19067d3b4b7499a8122013c83a807392dab1d5bd771",
            "e7b9a243d0e3f8dcbac016b2362175cc83a04f45a77b8c91b872306093e93f5e",
            "df82843f6e4dd3ead0dcc687a9f85fe410c7c6b57d1afcd29896eda4696999c5",
            "d378251b1684a219ed94ca20333fd5a8aaefd59625ba40c748d48e9adaec51a0",
            "8633641da4e332731a3f0893a4ba3f227fad2ae20ac9e860034a98a9103db499",
            "53664cbd12559b873ad036955c51e6b764fa833549844dde4719c765a333a58a",
            "ee6e395e2e0009f07c42f384765f83e99c2e02a07090a021dbf41cd4df9b5c85",
            "642b046f2bd83e4e63f290f89d5fbf5895df8ea5f801908baa0834519507bb83",
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
