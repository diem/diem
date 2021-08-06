// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::verifying_client::state_store::{StateStore, WriteThroughCache};
use diem_crypto::hash::{CryptoHasher, HashValue};
use diem_types::{
    transaction::Version,
    trusted_state::{TrustedState, TrustedStateHasher},
};
use std::{
    cmp::max_by_key,
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom, Write},
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
    pub fn new(dir: &Path) -> io::Result<Self> {
        let store = ACIDStateFiles::new(dir)?;
        let store_cache = WriteThroughCache::new(store)?;
        Ok(Self(Arc::new(store_cache)))
    }
}

impl StateStore for FileStateStore {
    type Error = io::Error;

    fn latest_state(&self) -> io::Result<Option<TrustedState>> {
        self.0.latest_state()
    }
    fn latest_state_version(&self) -> io::Result<Option<u64>> {
        self.0.latest_state_version()
    }
    fn store(&self, new_state: &TrustedState) -> io::Result<()> {
        self.0.store(new_state)
    }
}

////////////////////
// ACIDStateFiles //
////////////////////

impl ACIDStateFiles {
    fn new(dir: &Path) -> io::Result<Self> {
        let state_file0 = StateFile::open(&dir.join("trusted_state.0"))?;
        let state_file1 = StateFile::open(&dir.join("trusted_state.1"))?;
        let state_files = [state_file0, state_file1];

        // make sure any newly created files are synced
        fsync_dir(dir)?;

        Ok(Self(Mutex::new(state_files)))
    }
}

impl StateStore for ACIDStateFiles {
    type Error = io::Error;

    fn latest_state(&self) -> io::Result<Option<TrustedState>> {
        let mut state_files = self.0.lock().unwrap();

        let state0 = state_files[0].read()?;
        let state1 = state_files[1].read()?;
        Ok(max_by_key(state0, state1, |opt| {
            opt.as_ref().map(|s| s.version())
        }))
    }

    fn store(&self, new_state: &TrustedState) -> io::Result<()> {
        let mut state_files = self.0.lock().unwrap();

        let newest_version = state_files
            .iter_mut()
            .max_by_key(|f| f.version)
            .unwrap()
            .version;
        let oldest_state_file = state_files.iter_mut().min_by_key(|f| f.version).unwrap();

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

// A dumb format for dumping a TrustedState to a File.
//
// serialized format: <bcs-serialized state blob> || <sha3-checksum(bcs-serialized state blob)>
//
// basic requirements:
// 1. fs writes are unreliable and not atomic:
//    ==> we validate the serialized bytes are uncorrupted with a sha3 checksum.
// 2. upgrade format:
//    ==> add a new TrustedState enum variant to modify.

fn decode_and_validate_checksum(mut buf: Vec<u8>) -> io::Result<(Vec<u8>, HashValue)> {
    let offset = buf
        .len()
        .checked_sub(HashValue::LENGTH)
        .ok_or_else(|| invalid_data("state file: empty or too small"))?;
    let file_hash = HashValue::from_slice(&buf[offset..]).expect("cannot fail");

    buf.truncate(offset);
    let computed_hash = TrustedStateHasher::hash_all(&buf);

    if file_hash != computed_hash {
        Err(invalid_data(format!(
            "state file: corrupt: file checksum ({:x}) != computed checksum ({:x})",
            file_hash, computed_hash
        )))
    } else {
        Ok((buf, file_hash))
    }
}

fn decode_state(buf: Vec<u8>) -> io::Result<TrustedState> {
    let (buf, _) = decode_and_validate_checksum(buf)?;
    let state = bcs::from_bytes(&buf).map_err(invalid_data)?;
    Ok(state)
}

fn encode_state(state: &TrustedState) -> io::Result<Vec<u8>> {
    let mut buf = bcs::to_bytes(state).map_err(invalid_input)?;
    let hash = TrustedStateHasher::hash_all(&buf);
    buf.extend_from_slice(hash.as_ref());
    Ok(buf)
}

fn read_file(file: &mut File) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    file.seek(SeekFrom::Start(0))?;
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

// Note: this method can take anywhere from 20ms to 50ms...
fn write_file(file: &mut File, buf: &[u8]) -> io::Result<()> {
    file.seek(SeekFrom::Start(0))?;
    file.write_all(buf)?;
    file.set_len(buf.len() as u64)?;
    // call fsync here to flush kernel cache to disk. this way we can (more) safely
    // assume that the write is actually durable once the write completes and clients
    // can't observe non-durable state.
    file.sync_all()?;
    Ok(())
}

impl StateFile {
    fn open(path: &Path) -> io::Result<Self> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        Ok(Self::new(file))
    }

    fn new(file: File) -> Self {
        Self {
            version: None,
            file,
        }
    }

    // only returns an error on io::Error, otherwise assumes the file is just corrupt.
    fn read(&mut self) -> io::Result<Option<TrustedState>> {
        let buf = read_file(&mut self.file)?;
        let maybe_state = decode_state(buf)
            .map_err(|_err| () /* TODO: how to log in client sdk? */)
            .ok();
        self.version = maybe_state.as_ref().map(|s| s.version());
        Ok(maybe_state)
    }

    fn write(&mut self, new_state: &TrustedState) -> io::Result<()> {
        let buf = encode_state(new_state)?;
        write_file(&mut self.file, &buf)?;
        self.version = Some(new_state.version());
        Ok(())
    }
}

///////////
// Utils //
///////////

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

fn invalid_input(err: impl Into<BoxError>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, err)
}

fn invalid_data(err: impl Into<BoxError>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err)
}

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

///////////
// Tests //
///////////

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{collection::vec, prelude::*, sample::Index};
    use tempfile::{tempdir, tempfile};

    fn max_state(idx: usize, states: &[TrustedState]) -> &TrustedState {
        states[..=idx].iter().max_by_key(|s| s.version()).unwrap()
    }

    // simulate a crash during write by truncating the file
    fn corrupt_file(corrupt_idx: &Index, file: &mut File) {
        let len = file.metadata().unwrap().len();
        if len > 0 {
            // note: Index::index(N) returns x in [0, N)
            let new_len = corrupt_idx.index(len as usize);
            file.set_len(new_len as u64).unwrap();
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn state_file_read_corrupt(
            state in any::<TrustedState>(),
            corrupt_idx in any::<Index>(),
        ) {
            let file = tempfile().unwrap();
            let mut state_file = StateFile::new(file);

            // write a state
            state_file.write(&state).unwrap();
            assert_eq!(state_file.version, Some(state.version()));

            // read that state back
            let maybe_state = state_file.read().unwrap();
            assert_eq!(maybe_state, Some(state));

            // simulate a crash during write by truncating the file
            corrupt_file(&corrupt_idx, &mut state_file.file);

            // read should return None b/c file is corrupt.
            let maybe_state = state_file.read().unwrap();
            assert_eq!(maybe_state, None);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(25))]

        #[test]
        fn file_state_store(
            states in vec(any::<TrustedState>(), 1..10),
            corrupt_idx in any::<Index>(),
        ) {
            let dir = tempdir().unwrap();
            let state_store = FileStateStore::new(dir.path()).unwrap();
            assert_eq!(None, state_store.latest_state().unwrap());

            // store some states (without any particular order)
            for (idx, state) in states.iter().enumerate() {
                state_store.store(state).unwrap();

                // latest_state should be monotonically increasing by version
                let store_max = state_store.latest_state().unwrap().unwrap();
                let expected_max = max_state(idx, &states);
                assert_eq!(expected_max, &store_max);
            }

            // final highest state
            let store_max1 = state_store.latest_state().unwrap().unwrap();
            drop(state_store);

            // restarting should recover the same final state
            let state_store = FileStateStore::new(dir.path()).unwrap();
            let store_max2 = state_store.latest_state().unwrap().unwrap();
            assert_eq!(store_max1, store_max2);

            // "corrupt" the older state file (if it exists) to simulate crashing
            // while writing
            {
                let mut state_files = state_store.0.as_inner().0.lock().unwrap();
                if let Some(oldest_state_file) = state_files.iter_mut().min_by_key(|f| f.version).as_mut() {
                    corrupt_file(&corrupt_idx, &mut oldest_state_file.file);
                }
            }
            drop(state_store);

            // restarting after partial write should still recover state
            let state_store = FileStateStore::new(dir.path()).unwrap();
            let store_max3 = state_store.latest_state().unwrap().unwrap();
            assert_eq!(store_max1, store_max3);
        }
    }
}
