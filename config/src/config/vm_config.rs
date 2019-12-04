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
            "1cf66b5f5c911e80dad222b8ee8dfe3ad4830f75bb412ba12ea8e429203d9c83",
            "4160344b9d9cf5c5da277014a24bb187d40a8d64a44291aa8d3eefa51b0b9488",
            "5ee07d4ac1ecf88f1b41c2c458f15699fe9d811c61563338253b3807b75c04c1",
            "6aabc87f543f85e10216432d02b0251297d4c7723e906de481dfa04b057c2371",
            "a2180395d1632a0793f34e8a8a6be20b3b03bdceee35affe8c751fc8467b73a4",
            "d4ed6341aada016d9d675f48445f720c61d1d66b808ec5a95bdab04db9b7856e",
            "e4de36a91d0c0cd495d340337d3023102161425cab9aafa80aca59a763365671",
            "f37517131a78bab737c090037671975557b9d810f45bfbba24526c0cdfbadb09",
            "ff47e2dcb1884af7d608eb3063dcb78f33b1af864d0e160cb3b76ba6b368b928",
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
