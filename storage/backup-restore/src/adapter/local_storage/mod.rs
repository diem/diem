// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod local_storage_test;

use super::Adapter;
use crate::FileHandle;
use anyhow::Result;
use async_trait::async_trait;
use futures::{stream::BoxStream, StreamExt};
use rand::RngCore;
use std::{
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

const FILENAME_LEN: usize = 16;

/// A storage backend that stores everything in a local directory.
pub struct LocalStorage {
    /// The path where everything is stored.
    dir: PathBuf,
}

impl LocalStorage {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}

#[async_trait]
impl Adapter for LocalStorage {
    async fn write_new_file(
        &self,
        mut content: impl StreamExt<Item = Vec<u8>> + Send + Unpin + 'async_trait,
    ) -> Result<FileHandle> {
        let (mut file, handle) = create_random_file(&self.dir)?;
        while let Some(bytes) = content.next().await {
            file.write_all(&bytes)?;
        }
        file.sync_data()?;
        Ok(handle)
    }

    fn read_file_content(&self, file_handle: &FileHandle) -> BoxStream<Result<Vec<u8>>> {
        let file = match std::fs::File::open(&file_handle) {
            Ok(f) => f,
            Err(e) => return futures::stream::once(async { Err(e.into()) }).boxed(),
        };

        futures::stream::iter(FileIterator::new(file)).boxed()
    }
}

/// Creates a file with random name in the given directory. Returns the reference to the open file
/// as well as the handle (name) of the file.
fn create_random_file(dir: &Path) -> Result<(std::fs::File, FileHandle)> {
    loop {
        let filename = create_random_filename(dir)
            .into_os_string()
            .into_string()
            .expect("Filename should contain valid unicode.");

        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&filename)
        {
            Ok(file) => return Ok((file, filename)),
            Err(err) => {
                if let std::io::ErrorKind::AlreadyExists = err.kind() {
                    // This is very unlikely.
                    continue;
                } else {
                    return Err(err.into());
                }
            }
        }
    }
}

/// Generates a random filename under given directory.
fn create_random_filename(dir: &Path) -> PathBuf {
    let mut filename = [0; FILENAME_LEN];
    rand::thread_rng().fill_bytes(&mut filename);
    dir.join(hex::encode(filename))
}

/// An iterator that reads one chunk from a file at a time and yields the bytes.
struct FileIterator {
    /// The reference to the open file.
    file: BufReader<std::fs::File>,
}

impl FileIterator {
    fn new(file: std::fs::File) -> Self {
        Self {
            file: BufReader::new(file),
        }
    }
}

impl Iterator for FileIterator {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.file.fill_buf() {
            Ok(buf) => {
                let len = buf.len();
                if len == 0 {
                    None
                } else {
                    let ret = Some(Ok(buf.to_vec()));
                    self.file.consume(len);
                    ret
                }
            }
            Err(e) => Some(Err(e.into())),
        }
    }
}
