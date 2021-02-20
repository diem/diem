// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use std::{
    borrow::Cow,
    convert::AsRef,
    ffi::{OsStr, OsString},
    fmt,
    path::{Path, PathBuf},
    process::Command,
};
use tar::Archive;
use tempfile::{tempdir, TempDir};

/// A Git hash.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct GitHash([u8; 20]);

impl GitHash {
    /// Creates a new Git hash from a hex-encoded string.
    pub fn from_hex(hex: impl AsRef<[u8]>) -> Result<Self> {
        let hex = hex.as_ref();
        Ok(GitHash(hex::FromHex::from_hex(hex).map_err(|err| {
            anyhow!("parsing a Git hash: {:?} with error {:?}", hex, err)
        })?))
    }
}

impl<'a, 'b> From<&'a GitHash> for Cow<'b, OsStr> {
    fn from(git_hash: &'a GitHash) -> Cow<'b, OsStr> {
        OsString::from(format!("{:x}", git_hash)).into()
    }
}

impl fmt::LowerHex for GitHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

pub(crate) fn get_archived_files<P: AsRef<Path>>(
    path_to_repo_root: P,
    hash: GitHash,
) -> Result<TempDir> {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tmp_dir = tempdir()?;
    let git_output = Command::new("git")
        .args(&["archive", "--format=tar"])
        .args(&[
            &format!("{:x}", hash),
            path_to_repo_root
                .as_ref()
                .to_str()
                .ok_or_else(|| anyhow!("Unexpected path for archived files"))?,
        ])
        .current_dir(&crate_dir)
        .output()?;

    let mut archive = Archive::new(&*git_output.stdout);
    archive.unpack(tmp_dir.path())?;
    Ok(tmp_dir)
}
