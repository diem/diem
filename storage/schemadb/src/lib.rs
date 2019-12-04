// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This library implements a schematized DB on top of [RocksDB](https://rocksdb.org/). It makes
//! sure all data passed in and out are structured according to predefined schemas and prevents
//! access to raw keys and values. This library also enforces a set of Libra specific DB options,
//! like custom comparators and schema-to-column-family mapping.
//!
//! It requires that different kinds of key-value pairs be stored in separate column
//! families.  To use this library to store a kind of key-value pairs, the user needs to use the
//! [`define_schema!`] macro to define the schema name, the types of key and value, and name of the
//! column family.

#[macro_use]
pub mod schema;

use crate::schema::{KeyCodec, Schema, SeekKeyCodec, ValueCodec};
use anyhow::{format_err, Result};
use lazy_static::lazy_static;
use libra_metrics::OpMetrics;
use rocksdb::{
    rocksdb_options::ColumnFamilyDescriptor, CFHandle, DBOptions, Writable, WriteOptions,
};
use std::{
    collections::{BTreeMap, HashMap},
    iter::Iterator,
    marker::PhantomData,
    path::Path,
};

lazy_static! {
    static ref OP_COUNTER: OpMetrics = OpMetrics::new_and_registered("schemadb");
}

/// Type alias to `rocksdb::ColumnFamilyOptions`. See [`rocksdb doc`](https://github.com/pingcap/rust-rocksdb/blob/master/src/rocksdb_options.rs)
pub type ColumnFamilyOptions = rocksdb::ColumnFamilyOptions;
/// Type alias to `rocksdb::ReadOptions`. See [`rocksdb doc`](https://github.com/pingcap/rust-rocksdb/blob/master/src/rocksdb_options.rs)
pub type ReadOptions = rocksdb::ReadOptions;

/// Type alias to improve readability.
pub type ColumnFamilyName = &'static str;
/// Type alias to improve readability.
pub type ColumnFamilyOptionsMap = HashMap<ColumnFamilyName, ColumnFamilyOptions>;

/// Name for the `default` column family that's always open by RocksDB. We use it to store
/// [`LedgerInfo`](../types/ledger_info/struct.LedgerInfo.html).
pub const DEFAULT_CF_NAME: ColumnFamilyName = "default";

#[derive(Debug)]
enum WriteOp {
    Value(Vec<u8>),
    Deletion,
}

/// `SchemaBatch` holds a collection of updates that can be applied to a DB atomically. The updates
/// will be applied in the order in which they are added to the `SchemaBatch`.
#[derive(Debug, Default)]
pub struct SchemaBatch {
    rows: HashMap<ColumnFamilyName, BTreeMap<Vec<u8>, WriteOp>>,
}

impl SchemaBatch {
    /// Creates an empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an insert/update operation to the batch.
    pub fn put<S: Schema>(&mut self, key: &S::Key, value: &S::Value) -> Result<()> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let value = <S::Value as ValueCodec<S>>::encode_value(value)?;
        self.rows
            .entry(S::COLUMN_FAMILY_NAME)
            .or_insert_with(BTreeMap::new)
            .insert(key, WriteOp::Value(value));

        Ok(())
    }

    /// Adds a delete operation to the batch.
    pub fn delete<S: Schema>(&mut self, key: &S::Key) -> Result<()> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        self.rows
            .entry(S::COLUMN_FAMILY_NAME)
            .or_insert_with(BTreeMap::new)
            .insert(key, WriteOp::Deletion);

        Ok(())
    }
}

/// DB Iterator parameterized on [`Schema`] that seeks with [`Schema::Key`] and yields
/// [`Schema::Key`] and [`Schema::Value`]
pub struct SchemaIterator<'a, S> {
    db_iter: rocksdb::DBIterator<&'a rocksdb::DB>,
    phantom: PhantomData<S>,
}

impl<'a, S> SchemaIterator<'a, S>
where
    S: Schema,
{
    fn new(db_iter: rocksdb::DBIterator<&'a rocksdb::DB>) -> Self {
        SchemaIterator {
            db_iter,
            phantom: PhantomData,
        }
    }

    /// Seeks to the first key.
    pub fn seek_to_first(&mut self) -> bool {
        self.db_iter.seek(rocksdb::SeekKey::Start)
    }

    /// Seeks to the last key.
    pub fn seek_to_last(&mut self) -> bool {
        self.db_iter.seek(rocksdb::SeekKey::End)
    }

    /// Seeks to the first key whose binary representation is equal to or greater than that of the
    /// `seek_key`.
    pub fn seek<SK>(&mut self, seek_key: &SK) -> Result<bool>
    where
        SK: SeekKeyCodec<S>,
    {
        let key = <SK as SeekKeyCodec<S>>::encode_seek_key(seek_key)?;
        Ok(self.db_iter.seek(rocksdb::SeekKey::Key(&key)))
    }

    /// Seeks to the last key whose binary representation is less than or equal to that of the
    /// `seek_key`.
    ///
    /// See example in [`RocksDB doc`](https://github.com/facebook/rocksdb/wiki/SeekForPrev).
    pub fn seek_for_prev<SK>(&mut self, seek_key: &SK) -> Result<bool>
    where
        SK: SeekKeyCodec<S>,
    {
        let key = <SK as SeekKeyCodec<S>>::encode_seek_key(seek_key)?;
        Ok(self.db_iter.seek_for_prev(rocksdb::SeekKey::Key(&key)))
    }
}

impl<'a, S> Iterator for SchemaIterator<'a, S>
where
    S: Schema,
{
    type Item = Result<(S::Key, S::Value)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.db_iter.kv().map(|(raw_key, raw_value)| {
            self.db_iter.next();
            Ok((
                <S::Key as KeyCodec<S>>::decode_key(&raw_key)?,
                <S::Value as ValueCodec<S>>::decode_value(&raw_value)?,
            ))
        })
    }
}

/// Checks underlying Rocksdb instance existence by checking `CURRENT` file existence, the same way
/// Rocksdb adopts to detect db existence.
fn db_exists(path: &Path) -> bool {
    let rocksdb_current_file = path.join("CURRENT");
    rocksdb_current_file.is_file()
}

/// All the RocksDB methods return `std::result::Result<T, String>`. Since our methods return
/// `anyhow::Result<T>`, manual conversion is needed.
fn convert_rocksdb_err(msg: String) -> anyhow::Error {
    format_err!("RocksDB internal error: {}.", msg)
}

/// This DB is a schematized RocksDB wrapper where all data passed in and out are typed according to
/// [`Schema`]s.
#[derive(Debug)]
pub struct DB {
    inner: rocksdb::DB,
}

impl DB {
    /// Create db with all the column families provided if it doesn't exist at `path`; Otherwise,
    /// try to open it with all the column families.
    pub fn open<P: AsRef<Path>>(path: P, mut cf_opts_map: ColumnFamilyOptionsMap) -> Result<Self> {
        let mut db_opts = DBOptions::new();

        // For now we set the max total WAL size to be 1G. This config can be useful when column
        // families are updated at non-uniform frequencies.
        db_opts.set_max_total_wal_size(1 << 30);

        // If db exists, just open it with all cfs.
        if db_exists(path.as_ref()) {
            return DB::open_cf(db_opts, &path, cf_opts_map.into_iter().collect());
        }

        // If db doesn't exist, create a db first with all column families.
        db_opts.create_if_missing(true);

        let mut db = DB::open_cf(
            db_opts,
            path,
            vec![cf_opts_map
                .remove_entry(&DEFAULT_CF_NAME)
                .ok_or_else(|| format_err!("No \"default\" column family name found"))?],
        )?;
        cf_opts_map
            .into_iter()
            .map(|(cf_name, cf_opts)| db.create_cf((cf_name, cf_opts)))
            .collect::<Result<Vec<_>>>()?;
        Ok(db)
    }

    fn open_cf<'a, P, T>(opts: DBOptions, path: P, cfds: Vec<T>) -> Result<DB>
    where
        P: AsRef<Path>,
        T: Into<ColumnFamilyDescriptor<'a>>,
    {
        let inner = rocksdb::DB::open_cf(
            opts,
            path.as_ref().to_str().ok_or_else(|| {
                format_err!("Path {:?} can not be converted to string.", path.as_ref())
            })?,
            cfds,
        )
        .map_err(convert_rocksdb_err)?;

        Ok(DB { inner })
    }

    fn create_cf<'a, T>(&mut self, cfd: T) -> Result<()>
    where
        T: Into<ColumnFamilyDescriptor<'a>>,
    {
        let _cf_handle = self.inner.create_cf(cfd).map_err(convert_rocksdb_err)?;
        Ok(())
    }

    /// Reads single record by key.
    pub fn get<S: Schema>(&self, schema_key: &S::Key) -> Result<Option<S::Value>> {
        let k = <S::Key as KeyCodec<S>>::encode_key(&schema_key)?;
        let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY_NAME)?;
        let time = std::time::Instant::now();

        let result = self
            .inner
            .get_cf(cf_handle, &k)
            .map_err(convert_rocksdb_err)?;
        OP_COUNTER.observe_duration(&format!("db_get_{}", S::COLUMN_FAMILY_NAME), time.elapsed());
        result
            .map(|raw_value| <S::Value as ValueCodec<S>>::decode_value(&raw_value))
            .transpose()
    }

    /// Writes single record.
    pub fn put<S: Schema>(&self, key: &S::Key, value: &S::Value) -> Result<()> {
        let k = <S::Key as KeyCodec<S>>::encode_key(&key)?;
        let v = <S::Value as ValueCodec<S>>::encode_value(&value)?;
        let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY_NAME)?;

        self.inner
            .put_cf_opt(cf_handle, &k, &v, &default_write_options())
            .map_err(convert_rocksdb_err)
    }

    /// Delete all keys in range [begin, end).
    ///
    /// `SK` has to be an explicit type parameter since
    /// https://github.com/rust-lang/rust/issues/44721
    pub fn range_delete<S, SK>(&self, begin: &SK, end: &SK) -> Result<()>
    where
        S: Schema,
        SK: SeekKeyCodec<S>,
    {
        let raw_begin = begin.encode_seek_key()?;
        let raw_end = end.encode_seek_key()?;
        let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY_NAME)?;

        self.inner
            .delete_range_cf(&cf_handle, &raw_begin, &raw_end)
            .map_err(convert_rocksdb_err)
    }

    /// Returns a [`SchemaIterator`] on a certain schema.
    pub fn iter<S: Schema>(&self, opts: ReadOptions) -> Result<SchemaIterator<S>> {
        let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY_NAME)?;
        Ok(SchemaIterator::new(self.inner.iter_cf_opt(cf_handle, opts)))
    }

    /// Writes a group of records wrapped in a [`SchemaBatch`].
    pub fn write_schemas(&self, batch: SchemaBatch) -> Result<()> {
        let db_batch = rocksdb::WriteBatch::new();
        for (cf_name, rows) in &batch.rows {
            let cf_handle = self.get_cf_handle(cf_name)?;
            for (key, write_op) in rows {
                match write_op {
                    WriteOp::Value(value) => db_batch.put_cf(cf_handle, key, value),
                    WriteOp::Deletion => db_batch.delete_cf(cf_handle, key),
                }
                .map_err(convert_rocksdb_err)?;
            }
        }

        self.inner
            .write_opt(&db_batch, &default_write_options())
            .map_err(convert_rocksdb_err)?;

        // Bump counters only after DB write succeeds.
        for (cf_name, rows) in &batch.rows {
            for (key, write_op) in rows {
                match write_op {
                    WriteOp::Value(value) => OP_COUNTER.observe(
                        &format!("db_put_bytes_{}", cf_name),
                        (key.len() + value.len()) as f64,
                    ),
                    WriteOp::Deletion => OP_COUNTER.inc(&format!("db_delete_{}", cf_name)),
                }
            }
        }

        Ok(())
    }

    fn get_cf_handle(&self, cf_name: &str) -> Result<&CFHandle> {
        self.inner.cf_handle(cf_name).ok_or_else(|| {
            format_err!(
                "DB::cf_handle not found for column family name: {}",
                cf_name
            )
        })
    }

    /// Returns the approximate size of each non-empty column family in bytes.
    pub fn get_approximate_sizes_cf(&self) -> Result<BTreeMap<String, u64>> {
        let mut cf_sizes = BTreeMap::new();

        for cf_name in self.inner.cf_names().into_iter().map(ToString::to_string) {
            let cf_handle = self.get_cf_handle(&cf_name)?;
            let size = self
                .inner
                .get_property_int_cf(cf_handle, "rocksdb.estimate-live-data-size")
                .ok_or_else(|| {
                    format_err!(
                        "Unable to get approximate size of {} column family.",
                        cf_name,
                    )
                })?;
            cf_sizes.insert(cf_name, size);
        }

        Ok(cf_sizes)
    }

    /// Flushes all memtable data. If `sync` is true, the flush will wait until it's done. This is
    /// only used for testing `get_approximate_sizes_cf` in unit tests.
    pub fn flush_all(&self, sync: bool) -> Result<()> {
        for cf_name in self.inner.cf_names() {
            let cf_handle = self.get_cf_handle(cf_name)?;
            self.inner
                .flush_cf(cf_handle, sync)
                .map_err(convert_rocksdb_err)?;
        }
        Ok(())
    }
}

/// For now we always use synchronous writes. This makes sure that once the operation returns
/// `Ok(())` the data is persisted even if the machine crashes. In the future we might consider
/// selectively turning this off for some non-critical writes to improve performance.
fn default_write_options() -> WriteOptions {
    let mut opts = WriteOptions::new();
    opts.set_sync(true);
    opts
}
