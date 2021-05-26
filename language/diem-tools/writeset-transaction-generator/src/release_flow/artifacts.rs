// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::{format_err, Result};
use diem_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};
use diem_crypto_derive::CryptoHasher;
use diem_types::{chain_id::ChainId, transaction::Version};
use move_core_types::language_storage::ModuleId;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    path::PathBuf,
    process::Command,
    sync::Mutex,
};

/// Checkin the release artifact to a local file to make sure the release process is reproducible
#[derive(Clone, Serialize, Deserialize)]
pub struct ReleaseArtifact {
    pub chain_id: ChainId,
    pub version: Version,
    pub commit_hash: String,
    pub stdlib_hash: HashValue,
    pub diem_version: Option<u64>,
    pub release_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ReleaseArtifacts(Vec<ReleaseArtifact>);

const ARTIFACT_PATH: &str = "release";
const ARTIFACT_NAME: &str = "artifacts";
const ARTIFACT_EXTENSION: &str = "json";

// Store the artifact in memory if ChainId is TESTING.
static TESTING_ARTIFACT: Lazy<Mutex<Option<ReleaseArtifact>>> = Lazy::new(|| Mutex::new(None));

impl ReleaseArtifacts {
    pub fn load_artifacts() -> Result<ReleaseArtifacts> {
        serde_json::from_slice(
            std::fs::read(artifact_path().as_path())
                .map_err(|_| {
                    format_err!("Artifact file not found. Is it the first time doing release?")
                })?
                .as_slice(),
        )
        .map_err(|_| format_err!("Unable to read from artifact"))
    }

    pub fn save_artifact(artifact: ReleaseArtifact) -> Result<()> {
        let mut existing_artifacts =
            Self::load_artifacts().unwrap_or_else(|_| ReleaseArtifacts(vec![]));
        existing_artifacts.0.push(artifact);
        std::fs::write(
            artifact_path().as_path(),
            format!("{}\n", serde_json::to_string_pretty(&existing_artifacts)?),
        )
        .map_err(|err| format_err!("Unable to write to path: {:?}", err))
    }

    pub fn load_latest_artifact() -> Result<ReleaseArtifact> {
        Self::load_artifacts()?
            .0
            .pop()
            .ok_or_else(|| format_err!("No previous artifact found"))
    }
}

fn artifact_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(ARTIFACT_PATH.to_string());
    path.push(ARTIFACT_NAME.to_string());
    path.with_extension(ARTIFACT_EXTENSION)
}

pub fn save_release_artifact(artifact: ReleaseArtifact) -> Result<()> {
    if artifact.chain_id == ChainId::test() {
        *TESTING_ARTIFACT.lock().unwrap().deref_mut() = Some(artifact);
        Ok(())
    } else {
        ReleaseArtifacts::save_artifact(artifact)
    }
}

pub fn load_latest_artifact(chain_id: &ChainId) -> Result<ReleaseArtifact> {
    if chain_id == &ChainId::test() {
        TESTING_ARTIFACT
            .lock()
            .unwrap()
            .deref()
            .clone()
            .ok_or_else(|| format_err!("Unable to read from artifact"))
    } else {
        ReleaseArtifacts::load_latest_artifact()
    }
}

pub fn get_commit_hash() -> Result<String> {
    Ok(String::from_utf8(
        Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()?
            .stdout,
    )?
    .trim_end()
    .to_string())
}

/// Generate a unique hash for a list of modules
pub fn hash_for_modules<'a>(
    modules: impl IntoIterator<Item = (ModuleId, &'a Vec<u8>)>,
) -> Result<HashValue> {
    #[derive(Serialize, Deserialize, CryptoHasher)]
    struct Stdlib {
        modules: BTreeMap<ModuleId, Vec<u8>>,
    }

    impl CryptoHash for Stdlib {
        type Hasher = StdlibHasher;

        fn hash(&self) -> HashValue {
            let mut hasher = Self::Hasher::default();
            let stdlib_blobs = bcs::to_bytes(self).expect("Can't serialize stdlib modules");
            hasher.update(&stdlib_blobs);
            hasher.finish()
        }
    }

    let mut stdlib = Stdlib {
        modules: BTreeMap::new(),
    };
    for (id, module_bytes) in modules {
        stdlib.modules.insert(id, module_bytes.to_vec());
    }
    Ok(stdlib.hash())
}
