// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{Error, Result},
    verifying_client::state_store::{StateStore, WriteThroughCache},
};
use diem_types::{transaction::Version, trusted_state::TrustedState};
use std::{
    cmp::max_by_key,
    fs::{self, File},
    io,
};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

/// A `StateStore` that stores `TrustedState`s in files inside a given directory
/// on the local filesystem.
///
/// After initialization, the given directory will contain two files
/// `/<dir>/trusted_state.0` and `/<dir>/trusted_state.1`.
///
/// Note: we assumes that calls to `fsync` after every a write will make the
/// store durable, though this is not actually true for all file systems or storage
/// devices. Unfortunately, the it's also the best we can do at this layer.
///
/// Note: we don't (yet) use FS advisory locks, so this store will fail in fun
/// and exciting ways if multiple processes try to use the same directory.
#[derive(Debug, Clone)]
pub struct FileStateStore(Arc<WriteThroughCache<ACIDStateFiles>>);

/// A pair of files that can durably store [`TrustedState`]s.
///
/// The main idea here is that the highest version state file is durable and not
/// being written to at all times. It only supports either one writer or one
/// reader at a time.
///
/// When a writer wants to store a new state, it gets an exclusive lock and writes
/// to the on-disk state file with the lower version (if the new state is actually newer).
/// That way, if the machine crashes or restarts during a store, we'll only corrupt
/// the older state file and the newer state file will remain untouched.
#[derive(Debug)]
struct ACIDStateFiles(Mutex<[StateFile; 2]>);

/// Metadata for an on-disk [`TrustedState`] file.
#[derive(Debug)]
struct StateFile {
    /// The version of the on-disk trusted state, or `None` if the on-disk state
    /// has not been written yet.
    version: Option<Version>,
    /// The opened file handle for the on-disk trusted state.
    file: File,
}

////////////////////
// FileStateStore //
////////////////////

impl FileStateStore {
    pub fn new(dir: &Path) -> Result<Self> {
        let store = ACIDStateFiles::new(dir)?;
        let store_cache = WriteThroughCache::new(store)?;
        Ok(Self(Arc::new(store_cache)))
    }
}

impl StateStore for FileStateStore {
    fn latest_state(&self) -> Result<Option<TrustedState>> {
        self.0.latest_state()
    }
    fn latest_state_version(&self) -> Result<Option<u64>> {
        self.0.latest_state_version()
    }
    fn store(&self, new_state: &TrustedState) -> Result<()> {
        self.0.store(new_state)
    }
}

////////////////////
// ACIDStateFiles //
////////////////////

impl ACIDStateFiles {
    fn new(dir: &Path) -> Result<Self> {
        let state_file0 = StateFile::new(&dir.join("trusted_state.0"))?;
        let state_file1 = StateFile::new(&dir.join("trusted_state.1"))?;
        let state_files = [state_file0, state_file1];

        // make sure any newly created files are synced
        fsync_dir(dir).map_err(Error::unknown)?;

        Ok(Self(Mutex::new(state_files)))
    }
}

impl StateStore for ACIDStateFiles {
    fn latest_state(&self) -> Result<Option<TrustedState>> {
        let mut state_files = self.0.lock().unwrap();

        let state0 = state_files[0].read()?;
        let state1 = state_files[1].read()?;
        Ok(max_by_key(state0, state1, |opt| {
            opt.as_ref().map(|s| s.version())
        }))
    }

    fn store(&self, new_state: &TrustedState) -> Result<()> {
        let mut state_files = self.0.lock().unwrap();

        let newest_version = state_files
            .iter_mut()
            .min_by_key(|f| f.version)
            .unwrap()
            .version;
        let oldest_state_file = state_files.iter_mut().max_by_key(|f| f.version).unwrap();

        // the new state is actually newer; write it to the oldest state file.
        if Some(new_state.version()) > newest_version {
            oldest_state_file.write(new_state)?;
        }

        Ok(())
    }
}

///////////////
// StateFile //
///////////////

fn read_state(_file: &mut File) -> io::Result<Option<TrustedState>> {
    todo!()
}

fn write_state(_file: &mut File, _new_state: &TrustedState) -> io::Result<()> {
    todo!()
}

impl StateFile {
    fn new(path: &Path) -> Result<Self> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(Error::unknown)?;
        Ok(Self {
            version: None,
            file,
        })
    }

    fn read(&mut self) -> Result<Option<TrustedState>> {
        let maybe_state = read_state(&mut self.file).map_err(Error::unknown)?;
        self.version = maybe_state.as_ref().map(|s| s.version());
        Ok(maybe_state)
    }

    fn write(&mut self, new_state: &TrustedState) -> Result<()> {
        write_state(&mut self.file, new_state).map_err(Error::unknown)?;
        self.version = Some(new_state.version());
        Ok(())
    }
}

///////////
// Utils //
///////////

/// fsync a directory. In certain file systems, this is required to persistently create a file.
// Note: shamelessly copied from tantivy-search:
// https://github.com/tantivy-search/tantivy/blob/b8a10c84067fa1e7841a0e6c962b023ed806d307/src/directory/mmap_directory.rs#L225
fn fsync_dir<P: AsRef<Path>>(dir: P) -> io::Result<()> {
    let mut open_opts = fs::OpenOptions::new();

    // Linux needs read to be set, otherwise returns EINVAL
    // write must not be set, or it fails with EISDIR
    open_opts.read(true);

    // On Windows, opening a directory requires FILE_FLAG_BACKUP_SEMANTICS
    // and calling sync_all() only works if write access is requested.
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use winapi::winbase;

        open_opts
            .write(true)
            .custom_flags(winbase::FILE_FLAG_BACKUP_SEMANTICS);
    }

    let fd = open_opts.open(dir)?;
    fd.sync_all()?;
    Ok(())
}
