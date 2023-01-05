// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod consensusdb_test;
mod schema;

use crate::{
    consensusdb::schema::{
        block::BlockSchema,
        quorum_certificate::QCSchema,
        single_entry::{SingleEntryKey, SingleEntrySchema},
    },
    error::DbError,
};
use anyhow::Result;
use consensus_types::{block::Block, quorum_cert::QuorumCert};
use diem_crypto::HashValue;
use diem_logger::prelude::*;
use schema::{BLOCK_CF_NAME, QC_CF_NAME, SINGLE_ENTRY_CF_NAME};
use schemadb::{Options, ReadOptions, SchemaBatch, DB, DEFAULT_CF_NAME};
use std::{collections::HashMap, iter::Iterator, path::Path, time::Instant};

pub struct ConsensusDB {
    db: DB,
}

impl ConsensusDB {
    pub fn new<P: AsRef<Path> + Clone>(db_root_path: P) -> Self {
        let column_families = vec![
            /* UNUSED CF = */ DEFAULT_CF_NAME,
            BLOCK_CF_NAME,
            QC_CF_NAME,
            SINGLE_ENTRY_CF_NAME,
        ];

        let path = db_root_path.as_ref().join("consensusdb");
        let instant = Instant::now();
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let db = DB::open(path.clone(), "consensus", column_families, &opts)
            .expect("ConsensusDB open failed; unable to continue");

        info!(
            "Opened ConsensusDB at {:?} in {} ms",
            path,
            instant.elapsed().as_millis()
        );

        Self { db }
    }

    pub fn get_data(
        &self,
    ) -> Result<(
        Option<Vec<u8>>,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
        Vec<Block>,
        Vec<QuorumCert>,
    )> {
        let last_vote = self.get_last_vote()?;
        let highest_timeout_certificate = self.get_highest_timeout_certificate()?;
        let highest_2chain_timeout_certificate = self.get_highest_2chain_timeout_certificate()?;
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
            last_vote,
            highest_timeout_certificate,
            highest_2chain_timeout_certificate,
            consensus_blocks,
            consensus_qcs,
        ))
    }

    pub fn save_highest_timeout_certificate(
        &self,
        highest_timeout_certificate: Vec<u8>,
    ) -> Result<(), DbError> {
        let mut batch = SchemaBatch::new();
        batch.put::<SingleEntrySchema>(
            &SingleEntryKey::HighestTimeoutCertificate,
            &highest_timeout_certificate,
        )?;
        self.commit(batch)?;
        Ok(())
    }

    pub fn save_highest_2chain_timeout_certificate(&self, tc: Vec<u8>) -> Result<(), DbError> {
        let mut batch = SchemaBatch::new();
        batch.put::<SingleEntrySchema>(&SingleEntryKey::Highest2ChainTimeoutCert, &tc)?;
        self.commit(batch)?;
        Ok(())
    }

    pub fn save_vote(&self, last_vote: Vec<u8>) -> Result<(), DbError> {
        let mut batch = SchemaBatch::new();
        batch.put::<SingleEntrySchema>(&SingleEntryKey::LastVoteMsg, &last_vote)?;
        self.commit(batch)
    }

    pub fn save_blocks_and_quorum_certificates(
        &self,
        block_data: Vec<Block>,
        qc_data: Vec<QuorumCert>,
    ) -> Result<(), DbError> {
        if block_data.is_empty() && qc_data.is_empty() {
            return Err(anyhow::anyhow!("Consensus block and qc data is empty!").into());
        }
        let mut batch = SchemaBatch::new();
        block_data
            .iter()
            .try_for_each(|block| batch.put::<BlockSchema>(&block.id(), block))?;
        qc_data
            .iter()
            .try_for_each(|qc| batch.put::<QCSchema>(&qc.certified_block().id(), qc))?;
        self.commit(batch)
    }

    pub fn delete_blocks_and_quorum_certificates(
        &self,
        block_ids: Vec<HashValue>,
    ) -> Result<(), DbError> {
        if block_ids.is_empty() {
            return Err(anyhow::anyhow!("Consensus block ids is empty!").into());
        }
        let mut batch = SchemaBatch::new();
        block_ids.iter().try_for_each(|hash| {
            batch.delete::<BlockSchema>(hash)?;
            batch.delete::<QCSchema>(hash)
        })?;
        self.commit(batch)
    }

    /// Write the whole schema batch including all data necessary to mutate the ledger
    /// state of some transaction by leveraging rocksdb atomicity support.
    fn commit(&self, batch: SchemaBatch) -> Result<(), DbError> {
        self.db.write_schemas(batch)?;
        Ok(())
    }

    /// Get latest timeout certificates (we only store the latest highest timeout certificates).
    fn get_highest_timeout_certificate(&self) -> Result<Option<Vec<u8>>, DbError> {
        Ok(self
            .db
            .get::<SingleEntrySchema>(&SingleEntryKey::HighestTimeoutCertificate)?)
    }

    /// Get latest timeout certificates (we only store the latest highest timeout certificates).
    fn get_highest_2chain_timeout_certificate(&self) -> Result<Option<Vec<u8>>, DbError> {
        Ok(self
            .db
            .get::<SingleEntrySchema>(&SingleEntryKey::Highest2ChainTimeoutCert)?)
    }

    /// Delete the timeout certificates
    pub fn delete_highest_timeout_certificate(&self) -> Result<(), DbError> {
        let mut batch = SchemaBatch::new();
        batch.delete::<SingleEntrySchema>(&SingleEntryKey::HighestTimeoutCertificate)?;
        self.commit(batch)
    }

    pub fn delete_highest_2chain_timeout_certificate(&self) -> Result<(), DbError> {
        let mut batch = SchemaBatch::new();
        batch.delete::<SingleEntrySchema>(&SingleEntryKey::Highest2ChainTimeoutCert)?;
        self.commit(batch)
    }
    /// Get serialized latest vote (if available)
    fn get_last_vote(&self) -> Result<Option<Vec<u8>>, DbError> {
        Ok(self
            .db
            .get::<SingleEntrySchema>(&SingleEntryKey::LastVoteMsg)?)
    }

    pub fn delete_last_vote_msg(&self) -> Result<(), DbError> {
        let mut batch = SchemaBatch::new();
        batch.delete::<SingleEntrySchema>(&SingleEntryKey::LastVoteMsg)?;
        self.commit(batch)?;
        Ok(())
    }

    /// Get all consensus blocks.
    fn get_blocks(&self) -> Result<HashMap<HashValue, Block>, DbError> {
        let mut iter = self.db.iter::<BlockSchema>(ReadOptions::default())?;
        iter.seek_to_first();
        Ok(iter.collect::<Result<HashMap<HashValue, Block>>>()?)
    }

    /// Get all consensus QCs.
    fn get_quorum_certificates(&self) -> Result<HashMap<HashValue, QuorumCert>, DbError> {
        let mut iter = self.db.iter::<QCSchema>(ReadOptions::default())?;
        iter.seek_to_first();
        Ok(iter.collect::<Result<HashMap<HashValue, QuorumCert>>>()?)
    }
}
