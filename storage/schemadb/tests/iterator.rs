// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use schemadb::{
    define_schema,
    schema::{KeyCodec, Schema, SeekKeyCodec, ValueCodec},
    ColumnFamilyOptions, ColumnFamilyOptionsMap, SchemaIterator, DB, DEFAULT_CF_NAME,
};

define_schema!(TestSchema, TestKey, TestValue, "TestCF");

#[derive(Debug, Eq, PartialEq)]
struct TestKey(u32, u32, u32);

#[derive(Debug, Eq, PartialEq)]
struct TestValue(u32);

impl KeyCodec<TestSchema> for TestKey {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let mut bytes = vec![];
        bytes.write_u32::<BigEndian>(self.0)?;
        bytes.write_u32::<BigEndian>(self.1)?;
        bytes.write_u32::<BigEndian>(self.2)?;
        Ok(bytes)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        let mut reader = std::io::Cursor::new(data);
        Ok(TestKey(
            reader.read_u32::<BigEndian>()?,
            reader.read_u32::<BigEndian>()?,
            reader.read_u32::<BigEndian>()?,
        ))
    }
}

impl ValueCodec<TestSchema> for TestValue {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.0.to_be_bytes().to_vec())
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        let mut reader = std::io::Cursor::new(data);
        Ok(TestValue(reader.read_u32::<BigEndian>()?))
    }
}

pub struct KeyPrefix1(u32);

impl SeekKeyCodec<TestSchema> for KeyPrefix1 {
    fn encode_seek_key(&self) -> Result<Vec<u8>> {
        Ok(self.0.to_be_bytes().to_vec())
    }
}

pub struct KeyPrefix2(u32, u32);

impl SeekKeyCodec<TestSchema> for KeyPrefix2 {
    fn encode_seek_key(&self) -> Result<Vec<u8>> {
        let mut bytes = vec![];
        bytes.write_u32::<BigEndian>(self.0)?;
        bytes.write_u32::<BigEndian>(self.1)?;
        Ok(bytes)
    }
}

fn collect_values(iter: SchemaIterator<TestSchema>) -> Vec<u32> {
    iter.map(|row| (row.unwrap().1).0).collect()
}

struct TestDB {
    _tmpdir: libra_temppath::TempPath,
    db: DB,
}

impl TestDB {
    fn new() -> Self {
        let tmpdir = libra_temppath::TempPath::new();
        let cf_opts_map: ColumnFamilyOptionsMap = [
            (DEFAULT_CF_NAME, ColumnFamilyOptions::default()),
            (
                TestSchema::COLUMN_FAMILY_NAME,
                ColumnFamilyOptions::default(),
            ),
        ]
        .iter()
        .cloned()
        .collect();
        let db = DB::open(&tmpdir.path(), cf_opts_map).unwrap();

        db.put::<TestSchema>(&TestKey(1, 0, 0), &TestValue(100))
            .unwrap();
        db.put::<TestSchema>(&TestKey(1, 0, 2), &TestValue(102))
            .unwrap();
        db.put::<TestSchema>(&TestKey(1, 0, 4), &TestValue(104))
            .unwrap();
        db.put::<TestSchema>(&TestKey(1, 1, 0), &TestValue(110))
            .unwrap();
        db.put::<TestSchema>(&TestKey(1, 1, 2), &TestValue(112))
            .unwrap();
        db.put::<TestSchema>(&TestKey(1, 1, 4), &TestValue(114))
            .unwrap();
        db.put::<TestSchema>(&TestKey(2, 0, 0), &TestValue(200))
            .unwrap();
        db.put::<TestSchema>(&TestKey(2, 0, 2), &TestValue(202))
            .unwrap();

        TestDB {
            _tmpdir: tmpdir,
            db,
        }
    }
}

impl TestDB {
    fn iter(&self) -> SchemaIterator<TestSchema> {
        self.db
            .iter(Default::default())
            .expect("Failed to create iterator.")
    }
}

impl std::ops::Deref for TestDB {
    type Target = DB;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

#[test]
fn test_seek_to_first() {
    let db = TestDB::new();
    let mut iter = db.iter();
    iter.seek_to_first().unwrap();
    assert_eq!(
        collect_values(iter),
        [100, 102, 104, 110, 112, 114, 200, 202]
    );
}

#[test]
fn test_seek_to_last() {
    let db = TestDB::new();
    let mut iter = db.iter();
    iter.seek_to_last().unwrap();
    assert_eq!(collect_values(iter), [202]);
}

#[test]
fn test_seek_by_existing_key() {
    let db = TestDB::new();
    let mut iter = db.iter();
    iter.seek(&TestKey(1, 1, 0)).unwrap();
    assert_eq!(collect_values(iter), [110, 112, 114, 200, 202]);
}

#[test]
fn test_seek_by_nonexistent_key() {
    let db = TestDB::new();
    let mut iter = db.iter();
    iter.seek(&TestKey(1, 1, 1)).unwrap();
    assert_eq!(collect_values(iter), [112, 114, 200, 202]);
}

#[test]
fn test_seek_for_prev_by_existing_key() {
    let db = TestDB::new();
    let mut iter = db.iter();
    iter.seek_for_prev(&TestKey(1, 1, 0)).unwrap();
    assert_eq!(collect_values(iter), [110, 112, 114, 200, 202]);
}

#[test]
fn test_seek_for_prev_by_nonexistent_key() {
    let db = TestDB::new();
    let mut iter = db.iter();
    iter.seek_for_prev(&TestKey(1, 1, 1)).unwrap();
    assert_eq!(collect_values(iter), [110, 112, 114, 200, 202]);
}

#[test]
fn test_seek_by_1prefix() {
    let db = TestDB::new();
    let mut iter = db.iter();
    iter.seek(&KeyPrefix1(2)).unwrap();
    assert_eq!(collect_values(iter), [200, 202]);
}

#[test]
fn test_seek_for_prev_by_1prefix() {
    let db = TestDB::new();
    let mut iter = db.iter();
    iter.seek_for_prev(&KeyPrefix1(2)).unwrap();
    assert_eq!(collect_values(iter), [114, 200, 202]);
}

#[test]
fn test_seek_by_2prefix() {
    let db = TestDB::new();
    let mut iter = db.iter();
    iter.seek(&KeyPrefix2(2, 0)).unwrap();
    assert_eq!(collect_values(iter), [200, 202]);
}

#[test]
fn test_seek_for_prev_by_2prefix() {
    let db = TestDB::new();
    let mut iter = db.iter();
    iter.seek_for_prev(&KeyPrefix2(2, 0)).unwrap();
    assert_eq!(collect_values(iter), [114, 200, 202]);
}
