// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vote_data::VoteData;
use anyhow::{ensure, Context};
use libra_crypto::{hash::CryptoHash, HashValue};
use libra_types::{
    block_info::BlockInfo,
    crypto_proxies::ValidatorVerifier,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct QuorumCert {
    /// The vote information certified by the quorum.
    vote_data: VoteData,
    /// The signed LedgerInfo of a committed block that carries the data about the certified block.
    signed_ledger_info: LedgerInfoWithSignatures,
}

impl Display for QuorumCert {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "QuorumCert: [{}, {}]",
            self.vote_data, self.signed_ledger_info
        )
    }
}

impl QuorumCert {
    pub fn new(vote_data: VoteData, signed_ledger_info: LedgerInfoWithSignatures) -> Self {
        QuorumCert {
            vote_data,
            signed_ledger_info,
        }
    }

    pub fn vote_data(&self) -> &VoteData {
        &self.vote_data
    }

    pub fn certified_block(&self) -> &BlockInfo {
        self.vote_data().proposed()
    }

    pub fn parent_block(&self) -> &BlockInfo {
        self.vote_data().parent()
    }

    pub fn ledger_info(&self) -> &LedgerInfoWithSignatures {
        &self.signed_ledger_info
    }

    pub fn commit_info(&self) -> &BlockInfo {
        self.ledger_info().ledger_info().commit_info()
    }

    /// If the QC commits reconfiguration and starts a new epoch
    pub fn ends_epoch(&self) -> bool {
        self.signed_ledger_info
            .ledger_info()
            .next_validator_set()
            .is_some()
    }

    /// QuorumCert for the genesis block deterministically generated from end-epoch LedgerInfo:
    /// - the ID of the block is determined by the generated genesis block.
    /// - the accumulator root hash of the LedgerInfo is set to the last executed state of previous
    ///   epoch.
    /// - the map of signatures is empty because genesis block is implicitly agreed.
    pub fn certificate_for_genesis_from_ledger_info(
        ledger_info: &LedgerInfo,
        genesis_id: HashValue,
    ) -> QuorumCert {
        let ancestor = BlockInfo::new(
            ledger_info.epoch() + 1,
            0,
            genesis_id,
            ledger_info.transaction_accumulator_hash(),
            ledger_info.version(),
            ledger_info.timestamp_usecs(),
            None,
        );
        let vote_data = VoteData::new(ancestor.clone(), ancestor.clone());
        let li = LedgerInfo::new(ancestor, vote_data.hash());

        QuorumCert::new(
            vote_data,
            LedgerInfoWithSignatures::new(li, BTreeMap::new()),
        )
    }

    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        let vote_hash = self.vote_data.hash();
        ensure!(
            self.ledger_info().ledger_info().consensus_data_hash() == vote_hash,
            "Quorum Cert's hash mismatch LedgerInfo"
        );
        // Genesis's QC is implicitly agreed upon, it doesn't have real signatures.
        // If someone sends us a QC on a fake genesis, it'll fail to insert into BlockStore
        // because of the round constraint.
        if self.certified_block().round() == 0 {
            ensure!(
                self.parent_block() == self.certified_block(),
                "Genesis QC has inconsistent parent block with certified block"
            );
            ensure!(
                self.certified_block() == self.ledger_info().ledger_info().commit_info(),
                "Genesis QC has inconsistent commit block with certified block"
            );
            ensure!(
                self.ledger_info().signatures().is_empty(),
                "Genesis QC should not carry signatures"
            );
            return Ok(());
        }
        self.ledger_info()
            .verify_signatures(validator)
            .context("Fail to verify QuorumCert")?;
        Ok(())
    }
}
