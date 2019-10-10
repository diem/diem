// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod consensusdb_test;
mod schema;

use crate::chained_bft::consensusdb::schema::{
    block::{BlockSchema, SchemaBlock},
    quorum_certificate::QCSchema,
    single_entry::{SingleEntryKey, SingleEntrySchema},
};
use consensus_types::{block::Block, common::Payload, quorum_cert::QuorumCert};
use crypto::HashValue;
use failure::prelude::*;
use logger::prelude::*;
use schema::{BLOCK_CF_NAME, QC_CF_NAME, SINGLE_ENTRY_CF_NAME};
use schemadb::{
    ColumnFamilyOptions, ColumnFamilyOptionsMap, ReadOptions, SchemaBatch, DB, DEFAULT_CF_NAME,
};
use std::{collections::HashMap, iter::Iterator, path::Path, time::Instant};

type HighestTimeoutCertificates = Vec<u8>;
type ConsensusStateData = Vec<u8>;
type VoteMsgData = Vec<u8>;

pub struct ConsensusDB {
    db: DB,
}

impl ConsensusDB {
    pub fn new<P: AsRef<Path> + Clone>(db_root_path: P) -> Self {
        let cf_opts_map: ColumnFamilyOptionsMap = [
            (
                /* UNUSED CF = */ DEFAULT_CF_NAME,
                ColumnFamilyOptions::default(),
            ),
            (BLOCK_CF_NAME, ColumnFamilyOptions::default()),
            (QC_CF_NAME, ColumnFamilyOptions::default()),
            (SINGLE_ENTRY_CF_NAME, ColumnFamilyOptions::default()),
        ]
        .iter()
        .cloned()
        .collect();

        let path = db_root_path.as_ref().join("consensusdb");
        let instant = Instant::now();
        let db = DB::open(path.clone(), cf_opts_map).unwrap_or_else(|e| {
            panic!("ConsensusDB open failed due to {:?}, unable to continue", e)
        });

        info!(
            "Opened ConsensusDB at {:?} in {} ms",
            path,
            instant.elapsed().as_millis()
        );

        Self { db }
    }

    pub fn get_data<T: Payload>(
        &self,
    ) -> Result<(
        Option<ConsensusStateData>,
        Option<VoteMsgData>,
        Option<HighestTimeoutCertificates>,
        Vec<Block<T>>,
        Vec<QuorumCert>,
    )> {
        let consensus_state = self.get_state()?;
        let last_vote_msg_data = self.get_last_vote_msg_data()?;
        let highest_timeout_certificates = self.get_highest_timeout_certificates()?;
        let consensus_blocks = self
            .get_blocks()?
            .into_iter()
            .map(|(_block_hash, block_content)| block_content)
            .collect::<Vec<_>>();
        let consensus_qcs = self
            .get_quorum_certificates()?
            .into_iter()
            .map(|(_block_hash, qc)| qc)
            .collect::<Vec<_>>();
        Ok((
            consensus_state,
            last_vote_msg_data,
            highest_timeout_certificates,
            consensus_blocks,
            consensus_qcs,
        ))
    }

    pub fn save_highest_timeout_certificates(
        &self,
        highest_timeout_certificates: HighestTimeoutCertificates,
    ) -> Result<()> {
        let mut batch = SchemaBatch::new();
        batch.put::<SingleEntrySchema>(
            &SingleEntryKey::HighestTimeoutCertificates,
            &highest_timeout_certificates,
        )?;
        self.commit(batch)
    }

    pub fn save_state(&self, state: ConsensusStateData, last_vote: VoteMsgData) -> Result<()> {
        let mut batch = SchemaBatch::new();
        batch.put::<SingleEntrySchema>(&SingleEntryKey::ConsensusState, &state)?;
        batch.put::<SingleEntrySchema>(&SingleEntryKey::LastVoteMsg, &last_vote)?;
        self.commit(batch)
    }

    pub fn save_blocks_and_quorum_certificates<T: Payload>(
        &self,
        block_data: Vec<Block<T>>,
        qc_data: Vec<QuorumCert>,
    ) -> Result<()> {
        ensure!(
            !block_data.is_empty() || !qc_data.is_empty(),
            "Consensus block and qc data is empty!"
        );
        let mut batch = SchemaBatch::new();
        block_data
            .iter()
            .map(|block| {
                batch.put::<BlockSchema<T>>(
                    &block.id(),
                    &SchemaBlock::<T>::from_block(block.clone()),
                )
            })
            .collect::<Result<()>>()?;
        qc_data
            .iter()
            .map(|qc| batch.put::<QCSchema>(&qc.certified_block_id(), qc))
            .collect::<Result<()>>()?;
        self.commit(batch)
    }

    pub fn delete_blocks_and_quorum_certificates<T: Payload>(
        &self,
        block_ids: Vec<HashValue>,
    ) -> Result<()> {
        ensure!(!block_ids.is_empty(), "Consensus block ids is empty!");
        let mut batch = SchemaBatch::new();
        block_ids
            .iter()
            .map(|hash| {
                batch.delete::<BlockSchema<T>>(hash)?;
                batch.delete::<QCSchema>(hash)
            })
            .collect::<Result<_>>()?;
        self.commit(batch)
    }

    /// Write the whole schema batch including all data necessary to mutate the ledger
    /// state of some transaction by leveraging rocksdb atomicity support.
    fn commit(&self, batch: SchemaBatch) -> Result<()> {
        self.db.write_schemas(batch)
    }

    /// Get latest timeout certificates (we only store the latest highest timeout certificates).
    fn get_highest_timeout_certificates(&self) -> Result<Option<Vec<u8>>> {
        self.db
            .get::<SingleEntrySchema>(&SingleEntryKey::HighestTimeoutCertificates)
    }

    /// Get latest consensus state (we only store the latest state).
    fn get_state(&self) -> Result<Option<Vec<u8>>> {
        self.db
            .get::<SingleEntrySchema>(&SingleEntryKey::ConsensusState)
    }

    /// Get latest vote message data (if available)
    fn get_last_vote_msg_data(&self) -> Result<Option<Vec<u8>>> {
        self.db
            .get::<SingleEntrySchema>(&SingleEntryKey::LastVoteMsg)
    }

    /// Get all consensus blocks.
    fn get_blocks<T: Payload>(&self) -> Result<HashMap<HashValue, Block<T>>> {
        let mut iter = self.db.iter::<BlockSchema<T>>(ReadOptions::default())?;
        iter.seek_to_first();
        iter.map(|value| value.and_then(|(k, v)| Ok((k, v.borrow_into_block().clone()))))
            .collect::<Result<HashMap<HashValue, Block<T>>>>()
    }

    /// Get all consensus QCs.
    fn get_quorum_certificates(&self) -> Result<HashMap<HashValue, QuorumCert>> {
        let mut iter = self.db.iter::<QCSchema>(ReadOptions::default())?;
        iter.seek_to_first();
        iter.collect::<Result<HashMap<HashValue, QuorumCert>>>()
    }
}
