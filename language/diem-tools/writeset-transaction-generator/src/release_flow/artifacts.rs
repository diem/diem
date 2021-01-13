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
    sync::Mutex,
};
use vm::CompiledModule;

/// Checkin the release artifact to a local file to make sure the release process is reproducible
#[derive(Clone, Serialize, Deserialize)]
pub struct ReleaseArtifact {
    pub chain_id: ChainId,
    pub version: Version,
    pub stdlib_hash: HashValue,
}

const ARTIFACT_PATH: &str = "release";
const ARTIFACT_EXTENSION: &str = "json";

// Store the artifact in memory if ChainId is TESTING.
static TESTING_ARTIFACT: Lazy<Mutex<Option<ReleaseArtifact>>> = Lazy::new(|| Mutex::new(None));

fn artifact_path(chain_id: &ChainId) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(ARTIFACT_PATH.to_string());
    path.push(chain_id.to_string());
    path.with_extension(ARTIFACT_EXTENSION)
}

pub fn save_release_artifact(artifact: ReleaseArtifact) -> Result<()> {
    if artifact.chain_id == ChainId::test() {
        *TESTING_ARTIFACT.lock().unwrap().deref_mut() = Some(artifact);
        Ok(())
    } else {
        std::fs::write(
            artifact_path(&artifact.chain_id).as_path(),
            serde_json::to_vec(&artifact)?.as_slice(),
        )
        .map_err(|_| format_err!("Unable to write to path"))
    }
}

pub fn load_artifact(chain_id: &ChainId) -> Result<ReleaseArtifact> {
    if chain_id == &ChainId::test() {
        TESTING_ARTIFACT
            .lock()
            .unwrap()
            .deref()
            .clone()
            .ok_or_else(|| format_err!("Unable to read from artifact"))
    } else {
        serde_json::from_slice(std::fs::read(artifact_path(chain_id).as_path())?.as_slice())
            .map_err(|_| format_err!("Unable to read from artifact"))
    }
}

/// Generate a unique hash for a list of modules
pub fn hash_for_modules(modules: &[CompiledModule]) -> Result<HashValue> {
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
    for module in modules {
        let id = module.self_id();
        let mut bytes = vec![];
        module.serialize(&mut bytes)?;
        stdlib.modules.insert(id, bytes);
    }
    Ok(stdlib.hash())
}
