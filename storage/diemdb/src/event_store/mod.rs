// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file defines event store APIs that are related to the event accumulator and events
//! themselves.
#![allow(unused)]

use super::DiemDB;
use crate::{
    change_set::ChangeSet,
    errors::DiemDbError,
    ledger_counters::{LedgerCounter, LedgerCounterBumps},
    schema::{
        event::EventSchema, event_accumulator::EventAccumulatorSchema,
        event_by_key::EventByKeySchema, event_by_version::EventByVersionSchema,
    },
};
use accumulator::{HashReader, MerkleAccumulator};
use anyhow::{ensure, format_err, Result};
use diem_crypto::{
    hash::{CryptoHash, EventAccumulatorHasher},
    HashValue,
};
use diem_types::{
    account_address::AccountAddress,
    account_config::NewBlockEvent,
    block_metadata::new_block_event_key,
    contract_event::ContractEvent,
    event::EventKey,
    proof::{position::Position, EventAccumulatorProof, EventProof},
    transaction::Version,
};
use schemadb::{schema::ValueCodec, ReadOptions, SchemaIterator, DB};
use std::{
    convert::{TryFrom, TryInto},
    iter::Peekable,
    sync::Arc,
};

#[derive(Debug)]
pub(crate) struct EventStore {
    db: Arc<DB>,
}

impl EventStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    /// Get all of the events given a transaction version.
    /// We don't need a proof for this because it's only used to get all events
    /// for a version which can be proved from the root hash of the event tree.
    pub fn get_events_by_version(&self, version: Version) -> Result<Vec<ContractEvent>> {
        let mut events = vec![];

        let mut iter = self.db.iter::<EventSchema>(ReadOptions::default())?;
        // Grab the first event and then iterate until we get all events for this version.
        iter.seek(&version)?;
        while let Some(((ver, index), event)) = iter.next().transpose()? {
            if ver != version {
                break;
            }
            events.push(event);
        }

        Ok(events)
    }

    pub fn get_events_by_version_iter(
        &self,
        start_version: Version,
        num_versions: usize,
    ) -> Result<EventsByVersionIter> {
        let mut iter = self.db.iter::<EventSchema>(Default::default())?;
        iter.seek(&start_version)?;

        Ok(EventsByVersionIter {
            inner: iter.peekable(),
            expected_next_version: start_version,
            end_version: start_version
                .checked_add(num_versions as u64)
                .ok_or_else(|| format_err!("Too many versions requested."))?,
        })
    }

    fn get_event_by_version_and_index(
        &self,
        version: Version,
        index: u64,
    ) -> Result<ContractEvent> {
        self.db
            .get::<EventSchema>(&(version, index))?
            .ok_or_else(|| {
                DiemDbError::NotFound(format!("Event {} of Txn {}", index, version)).into()
            })
    }

    /// Get the event raw data given transaction version and the index of the event queried.
    pub fn get_event_with_proof_by_version_and_index(
        &self,
        version: Version,
        index: u64,
    ) -> Result<(ContractEvent, EventAccumulatorProof)> {
        // Get event content.
        let event = self.get_event_by_version_and_index(version, index)?;

        // Get the number of events in total for the transaction at `version`.
        let mut iter = self.db.iter::<EventSchema>(ReadOptions::default())?;
        iter.seek_for_prev(&(version + 1))?;
        let num_events = match iter.next().transpose()? {
            Some(((ver, index), _)) if ver == version => (index + 1),
            _ => unreachable!(), // since we've already got at least one event above
        };

        // Get proof.
        let proof =
            Accumulator::get_proof(&EventHashReader::new(self, version), num_events, index)?;

        Ok((event, proof))
    }

    fn get_txn_ver_by_seq_num(&self, event_key: &EventKey, seq_num: u64) -> Result<u64> {
        let (ver, _) = self
            .db
            .get::<EventByKeySchema>(&(*event_key, seq_num))?
            .ok_or_else(|| format_err!("Index entry should exist for seq_num {}", seq_num))?;
        Ok(ver)
    }

    fn get_event_by_key(
        &self,
        event_key: &EventKey,
        seq_num: u64,
        ledger_version: Version,
    ) -> Result<ContractEvent> {
        let (version, index) = self.lookup_event_by_key(event_key, seq_num, ledger_version)?;
        self.get_event_by_version_and_index(version, index)
    }

    /// Get the latest sequence number on `event_key` considering all transactions with versions
    /// no greater than `ledger_version`.
    pub fn get_latest_sequence_number(
        &self,
        ledger_version: Version,
        event_key: &EventKey,
    ) -> Result<Option<u64>> {
        let mut iter = self
            .db
            .iter::<EventByVersionSchema>(ReadOptions::default())?;
        iter.seek_for_prev(&(*event_key, ledger_version, u64::max_value()));

        Ok(iter.next().transpose()?.and_then(
            |((key, _version, seq), _idx)| if &key == event_key { Some(seq) } else { None },
        ))
    }

    /// Get the next sequence number for specified event key.
    /// Returns 0 if there's no events already in the event stream.
    pub fn get_next_sequence_number(
        &self,
        ledger_version: Version,
        event_key: &EventKey,
    ) -> Result<u64> {
        self.get_latest_sequence_number(ledger_version, event_key)?
            .map_or(Ok(0), |seq| {
                seq.checked_add(1)
                    .ok_or_else(|| format_err!("Seq num overflowed."))
            })
    }

    /// Given `event_key` and `start_seq_num`, returns events identified by transaction version and
    /// index among all events emitted by the same transaction. Result won't contain records with a
    /// transaction version > `ledger_version` and is in ascending order.
    pub fn lookup_events_by_key(
        &self,
        event_key: &EventKey,
        start_seq_num: u64,
        limit: u64,
        ledger_version: u64,
    ) -> Result<
        Vec<(
            u64,     // sequence number
            Version, // transaction version it belongs to
            u64,     // index among events for the same transaction
        )>,
    > {
        let mut iter = self.db.iter::<EventByKeySchema>(ReadOptions::default())?;
        iter.seek(&(*event_key, start_seq_num))?;

        let mut result = Vec::new();
        let mut cur_seq = start_seq_num;
        for res in iter.take(limit as usize) {
            let ((path, seq), (ver, idx)) = res?;
            if path != *event_key || ver > ledger_version {
                break;
            }
            ensure!(
                seq == cur_seq,
                "DB corrupt: Sequence number not continuous, expected: {}, actual: {}.",
                cur_seq,
                seq
            );
            result.push((seq, ver, idx));
            cur_seq += 1;
        }

        Ok(result)
    }

    fn lookup_event_by_key(
        &self,
        event_key: &EventKey,
        seq_num: u64,
        ledger_version: Version,
    ) -> Result<(Version, u64)> {
        let indices = self.lookup_events_by_key(event_key, seq_num, 1, ledger_version)?;
        if indices.is_empty() {
            return Err(DiemDbError::NotFound(format!(
                "Event {} of seq num {}.",
                event_key, seq_num
            ))
            .into());
        }
        let (_seq, version, index) = indices[0];

        Ok((version, index))
    }

    /// Save contract events yielded by the transaction at `version` and return root hash of the
    /// event accumulator formed by these events.
    pub fn put_events(
        &self,
        version: u64,
        events: &[ContractEvent],
        cs: &mut ChangeSet,
    ) -> Result<HashValue> {
        cs.counter_bumps(version)
            .bump(LedgerCounter::EventsCreated, events.len());

        // Event table and indices updates
        events
            .iter()
            .enumerate()
            .try_for_each::<_, Result<_>>(|(idx, event)| {
                cs.batch.put::<EventSchema>(&(version, idx as u64), event)?;
                cs.batch.put::<EventByKeySchema>(
                    &(*event.key(), event.sequence_number()),
                    &(version, idx as u64),
                )?;
                cs.batch.put::<EventByVersionSchema>(
                    &(*event.key(), version, event.sequence_number()),
                    &(idx as u64),
                )?;
                Ok(())
            })?;

        // EventAccumulatorSchema updates
        let event_hashes: Vec<HashValue> = events.iter().map(ContractEvent::hash).collect();
        let (root_hash, writes) = EmptyAccumulator::append(&EmptyReader, 0, &event_hashes)?;
        writes.into_iter().try_for_each(|(pos, hash)| {
            cs.batch
                .put::<EventAccumulatorSchema>(&(version, pos), &hash)
        })?;

        Ok(root_hash)
    }

    pub(crate) fn put_events_multiple_versions(
        &self,
        first_version: u64,
        event_vecs: &[Vec<ContractEvent>],
        cs: &mut ChangeSet,
    ) -> Result<Vec<HashValue>> {
        event_vecs
            .iter()
            .enumerate()
            .map(|(idx, events)| {
                let version = first_version
                    .checked_add(idx as Version)
                    .ok_or_else(|| format_err!("version overflow"))?;
                self.put_events(version, events, cs)
            })
            .collect::<Result<Vec<_>>>()
    }

    /// Finds the first event sequence number in a specified stream on which `comp` returns false.
    /// (assuming the whole stream is partitioned by `comp`)
    fn search_for_event_lower_bound<C>(
        &self,
        event_key: &EventKey,
        mut comp: C,
        ledger_version: Version,
    ) -> Result<Option<u64>>
    where
        C: FnMut(&ContractEvent) -> Result<bool>,
    {
        let mut begin = 0u64;
        let mut end = match self.get_latest_sequence_number(ledger_version, event_key)? {
            Some(s) => s
                .checked_add(1)
                .ok_or_else(|| format_err!("event sequence number overflew."))?,
            None => return Ok(None),
        };

        // overflow not possible
        #[allow(clippy::integer_arithmetic)]
        {
            let mut count = end - begin;
            while count > 0 {
                let step = count / 2;
                let mid = begin + step;
                let event = self.get_event_by_key(event_key, mid, ledger_version)?;
                if comp(&event)? {
                    begin = mid + 1;
                    count -= step + 1;
                } else {
                    count = step;
                }
            }
        }

        if begin == end {
            Ok(None)
        } else {
            Ok(Some(begin))
        }
    }

    /// Gets the version of the last transaction committed before timestamp,
    /// a commited block at or after the required timestamp must exist (otherwise it's possible
    /// the next block committed as a timestamp smaller than the one in the request).
    pub(crate) fn get_last_version_before_timestamp(
        &self,
        timestamp: u64,
        ledger_version: Version,
    ) -> Result<Version> {
        let event_key = new_block_event_key();
        let seq_at_or_after_ts = self.search_for_event_lower_bound(
            &event_key,
            |event| {
                let new_block_event: NewBlockEvent = event.try_into()?;
                Ok(new_block_event.proposed_time() < timestamp)
            },
            ledger_version,
        )?.ok_or_else(|| format_err!(
            "No new block found beyond timestmap {}, so can't determine the last version before it.",
            timestamp,
        ))?;

        ensure!(
            seq_at_or_after_ts > 0,
            "First block started at or after timestamp {}.",
            timestamp,
        );

        let (version, _idx) =
            self.lookup_event_by_key(&event_key, seq_at_or_after_ts, ledger_version)?;

        version
            .checked_sub(1)
            .ok_or_else(|| format_err!("A block with non-zero seq num started at version 0."))
    }
}

type Accumulator<'a> = MerkleAccumulator<EventHashReader<'a>, EventAccumulatorHasher>;

struct EventHashReader<'a> {
    store: &'a EventStore,
    version: Version,
}

impl<'a> EventHashReader<'a> {
    fn new(store: &'a EventStore, version: Version) -> Self {
        Self { store, version }
    }
}

impl<'a> HashReader for EventHashReader<'a> {
    fn get(&self, position: Position) -> Result<HashValue> {
        self.store
            .db
            .get::<EventAccumulatorSchema>(&(self.version, position))?
            .ok_or_else(|| format_err!("Hash at position {:?} not found.", position))
    }
}

type EmptyAccumulator = MerkleAccumulator<EmptyReader, EventAccumulatorHasher>;

struct EmptyReader;

// Asserts `get()` is never called.
impl HashReader for EmptyReader {
    fn get(&self, _position: Position) -> Result<HashValue> {
        unreachable!()
    }
}

pub struct EventsByVersionIter<'a> {
    inner: Peekable<SchemaIterator<'a, EventSchema>>,
    expected_next_version: Version,
    end_version: Version,
}

impl<'a> EventsByVersionIter<'a> {
    fn next_impl(&mut self) -> Result<Option<Vec<ContractEvent>>> {
        if self.expected_next_version >= self.end_version {
            return Ok(None);
        }

        let mut ret = Vec::new();
        while let Some(res) = self.inner.peek() {
            let ((version, _index), _event) = res
                .as_ref()
                .map_err(|e| format_err!("Hit error iterating events: {}", e))?;
            if *version != self.expected_next_version {
                break;
            }
            let ((_version, _index), event) =
                self.inner.next().transpose()?.expect("Known to exist.");
            ret.push(event);
        }
        self.expected_next_version = self
            .expected_next_version
            .checked_add(1)
            .ok_or_else(|| format_err!("expected version overflowed."))?;
        Ok(Some(ret))
    }
}

impl<'a> Iterator for EventsByVersionIter<'a> {
    type Item = Result<Vec<ContractEvent>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl().transpose()
    }
}

#[cfg(test)]
mod test;
