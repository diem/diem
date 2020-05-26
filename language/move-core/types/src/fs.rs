// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{de, export::Formatter, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    ffi::OsString,
    fmt, fs,
    fs::{File, OpenOptions},
    io::{Error, ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

thread_local! {
     static DESCRIPTORS: RefCell<Option<Descriptors>> = RefCell::new(None);
}

#[derive(Debug)]
struct Descriptors {
    fs_counter: usize,
    paths: Vec<String>,
}

impl Descriptors {
    pub fn in_context<A, R>(action: A) -> R
    where
        A: Fn(&[String]) -> R,
    {
        DESCRIPTORS.with(|descriptors| {
            descriptors
                .borrow()
                .as_ref()
                .map(|descriptors| action(&descriptors.paths))
                .expect("Expected AFS in context.")
        })
    }

    pub fn in_context_mut<A, R>(action: A) -> R
    where
        A: Fn(&mut Vec<String>) -> R,
    {
        DESCRIPTORS.with(|descriptors| {
            descriptors
                .borrow_mut()
                .as_mut()
                .map(|descriptors| action(&mut descriptors.paths))
                .expect("Expected AFS in context.")
        })
    }

    pub fn on_crate_fs() {
        DESCRIPTORS.with(|descriptors| {
            let mut descriptors = descriptors.borrow_mut();
            if let Some(descriptors) = descriptors.as_mut() {
                descriptors.fs_counter += 1;
            } else {
                *descriptors = Some(Descriptors {
                    fs_counter: 1,
                    paths: vec![],
                });
            }
        });
    }

    pub fn on_drop_fs() {
        DESCRIPTORS.with(|descriptors| {
            let mut descriptors = descriptors.borrow_mut();
            let drop = if let Some(descriptors) = descriptors.as_mut() {
                descriptors.fs_counter -= 1;
                descriptors.fs_counter == 0
            } else {
                false
            };

            if drop {
                *descriptors = None;
            }
        });
    }

    pub fn has_context() -> bool {
        DESCRIPTORS.with(|descriptors| descriptors.borrow().is_some())
    }
}

/// File identifier.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct FileName(usize);

impl FileName {
    pub fn new(name: &str) -> Self {
        Descriptors::in_context_mut(|desc| {
            match desc.iter().enumerate().find(|(_, v)| *v == name) {
                Some((id, _)) => FileName(id),
                None => {
                    desc.push(name.to_owned());
                    FileName(desc.len() - 1)
                }
            }
        })
    }

    pub fn name(self) -> String {
        Descriptors::in_context(|desc| desc[self.0].to_owned())
    }

    pub fn has_suffix(self, suffix: &str) -> bool {
        Descriptors::in_context(|desc| desc[self.0].ends_with(suffix))
    }

    pub fn has_context() -> bool {
        Descriptors::has_context()
    }

    pub fn into_path_buf(self) -> PathBuf {
        PathBuf::from(self.name())
    }
}

impl From<PathBuf> for FileName {
    fn from(path: PathBuf) -> Self {
        FileName::new(path.to_string_lossy().as_ref())
    }
}

impl From<&PathBuf> for FileName {
    fn from(path: &PathBuf) -> Self {
        FileName::new(path.to_string_lossy().as_ref())
    }
}

impl From<&Path> for FileName {
    fn from(path: &Path) -> Self {
        FileName::new(path.to_string_lossy().as_ref())
    }
}

impl Ord for FileName {
    fn cmp(&self, other: &Self) -> Ordering {
        Descriptors::in_context(|desc| desc[self.0].cmp(&desc[other.0]))
    }
}

impl PartialOrd for FileName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Descriptors::in_context(|desc| desc[self.0].partial_cmp(&desc[other.0]))
    }
}

impl Serialize for FileName {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.name())
    }
}

impl Into<OsString> for FileName {
    fn into(self) -> OsString {
        OsString::from(self.name())
    }
}

impl fmt::Display for FileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for FileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.name())
    }
}

struct FIdVisitor;

impl<'de> de::Visitor<'de> for FIdVisitor {
    type Value = FileName;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FileName::new(v))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FileName::new(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FileName::new(&v))
    }
}

impl<'de> Deserialize<'de> for FileName {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FIdVisitor)
    }
}

pub trait FS {
    fn has_entry(&self, name: FileName) -> bool;
    fn find_with_prefix(&self, name: FileName) -> Result<Vec<FileName>, Error>;
    fn load(&self, name: FileName) -> Result<String, Error>;
    fn load_bytes(&self, name: FileName) -> Result<Vec<u8>, Error>;
    fn store(&self, name: FileName, data: String) -> Result<(), Error>;
    fn store_bytes(&self, name: FileName, data: Vec<u8>) -> Result<(), Error>;
    fn delete(&self, name: FileName) -> Result<(), Error>;
}

#[derive(Debug)]
pub enum AFS {
    FS {
        base_path: Option<PathBuf>,
    },
    InMemory {
        store: RefCell<HashMap<FileName, Vec<u8>>>,
    },
}

impl AFS {
    pub fn new() -> AFS {
        Descriptors::on_crate_fs();
        AFS::FS { base_path: None }
    }

    pub fn with_path(base_path: PathBuf) -> AFS {
        Descriptors::on_crate_fs();
        AFS::FS {
            base_path: Some(base_path),
        }
    }

    pub fn in_memory() -> AFS {
        Descriptors::on_crate_fs();
        AFS::InMemory {
            store: RefCell::new(Default::default()),
        }
    }

    fn make_path(&self, path: String) -> PathBuf {
        if let AFS::FS { base_path } = self {
            if let Some(base_path) = base_path {
                base_path.join(path)
            } else {
                PathBuf::from(path)
            }
        } else {
            PathBuf::from(path)
        }
    }
}

impl Drop for AFS {
    fn drop(&mut self) {
        Descriptors::on_drop_fs();
    }
}

impl FS for AFS {
    fn has_entry(&self, id: FileName) -> bool {
        match self {
            AFS::FS { base_path: _ } => {
                let path = self.make_path(id.name());
                path.is_file() && path.exists()
            }
            AFS::InMemory { store } => store.borrow().contains_key(&id),
        }
    }

    fn find_with_prefix(&self, prefix_id: FileName) -> Result<Vec<FileName>, Error> {
        match self {
            AFS::FS { base_path } => {
                let path = self.make_path(prefix_id.name());

                walkdir::WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .map(|e| {
                        let path = e.into_path();
                        let path = if let Some(base_path) = base_path {
                            path.strip_prefix(base_path)
                                .expect("Unreachable.")
                                .to_path_buf()
                        } else {
                            path
                        };

                        if let Some(name) = path.to_str() {
                            Ok(FileName::new(name))
                        } else {
                            Err(std::io::Error::new(
                                ErrorKind::Other,
                                "non-Unicode file name",
                            ))
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
            }
            AFS::InMemory { store } => {
                let prefix = prefix_id.name();
                let store = store.borrow();
                Ok(Descriptors::in_context(|d| {
                    d.iter()
                        .enumerate()
                        .map(|(i, name)| (FileName(i), name))
                        .filter_map(|(i, name)| {
                            if name.starts_with(&prefix) && store.contains_key(&i) {
                                Some(i)
                            } else {
                                None
                            }
                        })
                        .collect()
                }))
            }
        }
    }

    fn load(&self, id: FileName) -> Result<String, Error> {
        self.load_bytes(id).and_then(|buff| {
            String::from_utf8(buff).map_err(|err| {
                std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Expected valid UTF-8 '{}' {:?}", id.name(), err),
                )
            })
        })
    }

    fn load_bytes(&self, id: FileName) -> Result<Vec<u8>, Error> {
        match self {
            AFS::FS { base_path: _ } => {
                let path = self.make_path(id.name());
                let mut file = File::open(path).map_err(|err| {
                    std::io::Error::new(err.kind(), format!("{}: {}", err, id.name()))
                })?;

                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;

                Ok(buffer)
            }
            AFS::InMemory { store } => store
                .borrow()
                .get(&id)
                .map(|buf| buf.to_owned())
                .ok_or_else(|| {
                    std::io::Error::new(
                        ErrorKind::NotFound,
                        format!("No such file or directory '{}'", id.name()),
                    )
                }),
        }
    }

    fn store(&self, id: FileName, data: String) -> Result<(), Error> {
        self.store_bytes(id, data.into_bytes())
    }

    fn store_bytes(&self, id: FileName, data: Vec<u8>) -> Result<(), Error> {
        match self {
            AFS::FS { base_path: _ } => {
                let path = self.make_path(id.name());

                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(&parent)?;
                    }
                }

                let mut file = OpenOptions::new().write(true).create(true).open(path)?;
                file.set_len(0)?;
                file.write_all(&data)?;
                Ok(())
            }
            AFS::InMemory { store } => {
                let mut store = store.borrow_mut();
                store.insert(id, data);
                Ok(())
            }
        }
    }

    fn delete(&self, id: FileName) -> Result<(), Error> {
        match self {
            AFS::FS { base_path: _ } => {
                let path = self.make_path(id.name());
                if path.exists() {
                    fs::remove_file(path)?;
                }
                Ok(())
            }
            AFS::InMemory { store } => {
                let mut store = store.borrow_mut();
                store.remove(&id);
                Ok(())
            }
        }
    }
}

impl Default for AFS {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::fs::{FileName, AFS, FS};
    use libra_temppath::TempPath;

    macro_rules! name {
        ($name:expr) => {
            FileName::new($name)
        };
    }

    #[test]
    #[should_panic]
    fn test_create_file_name_without_context() {
        name!("test");
    }

    #[test]
    fn test_create_file_name_in_context() {
        let _fs = AFS::new();
        assert_eq!("test_name.move", name!("test_name.move").to_string())
    }

    #[test]
    fn test_identical_names_identical_id() {
        let _fs = AFS::new();
        assert_eq!(name!("test_name.move").0, name!("test_name.move").0);
        assert_eq!(name!("test_name_1.move").0, name!("test_name_1.move").0);
        assert_ne!(name!("test_name.move").0, name!("test_name_1.move").0);
    }

    #[test]
    fn test_drop_descriptors_context() {
        {
            let _fs = AFS::new();
            name!("test");
            assert!(FileName::has_context());
            {
                let _fs = AFS::in_memory();
                name!("test_1");
                assert!(FileName::has_context());
            }
            assert!(FileName::has_context());
        }
        assert!(!FileName::has_context());
    }

    #[test]
    #[should_panic]
    fn test_file_name_without_context() {
        let f_name = {
            let _fs = AFS::in_memory();
            name!("test_1")
        };
        f_name.name();
    }

    #[test]
    fn test_suffix() {
        let _fs = AFS::in_memory();
        assert!(name!("test_1.move").has_suffix("move"));
    }

    #[test]
    fn test_in_memory_fs() {
        let fs = AFS::in_memory();
        test_fs(&fs);
    }

    #[test]
    fn test_in_fs() {
        let temp_dir = TempPath::new();
        let fs = AFS::with_path(temp_dir.path().to_path_buf());
        test_fs(&fs);
    }

    fn test_fs(fs: &AFS) {
        fs.store(name!("a/1"), "Test data 1".to_owned()).unwrap();
        fs.store(name!("a/2"), "Test data 2".to_owned()).unwrap();
        fs.store_bytes(name!("a/3"), b"Test data 3".to_vec())
            .unwrap();
        fs.store_bytes(name!("b/1"), b"Test data b1".to_vec())
            .unwrap();

        assert!(fs.has_entry(name!("a/2")));
        assert!(!fs.has_entry(name!("c/2")));

        let mut files_with_prefix = fs.find_with_prefix(name!("a")).unwrap();
        files_with_prefix.sort_unstable();
        let mut expected_files = vec![name!("a/1"), name!("a/2"), name!("a/3")];
        expected_files.sort_unstable();

        assert_eq!(files_with_prefix, expected_files);
        assert_eq!(fs.find_with_prefix(name!("c")).unwrap(), vec![]);

        assert_eq!(&fs.load(name!("b/1")).unwrap(), "Test data b1");
        assert_eq!(&fs.load(name!("a/1")).unwrap(), "Test data 1");
        assert_eq!(
            fs.load_bytes(name!("a/1")).unwrap(),
            b"Test data 1".to_vec()
        );

        assert!(fs.load_bytes(name!("a/10")).is_err());

        fs.delete(name!("a/1")).unwrap();
        assert!(fs.load_bytes(name!("a/1")).is_err());
    }
}
