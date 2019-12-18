use anyhow::Result;
use super::account_address::AccountAddress;
use std::convert::TryFrom;
use libra_crypto::HashValue;
use crate::transaction::Transaction;

/// Helper to construct and parse [`proto::types::BlockId`]
#[derive(PartialEq, Eq, Clone)]
pub struct BlockId {
    pub id: HashValue,
}

impl TryFrom<crate::proto::types::BlockId> for BlockId {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::BlockId) -> Result<Self> {
        let id = HashValue::from_slice(&proto.id[..]).expect("BlockId err.");

        Ok(Self { id })
    }
}

impl From<BlockId> for crate::proto::types::BlockId {
    fn from(req: BlockId) -> Self {
        Self {
            id: req.id.to_vec(),
        }
    }
}

/// Helper to construct and parse [`proto::types::GetBlockSummaryListRequest`]
#[derive(PartialEq, Eq, Clone)]
pub struct GetBlockSummaryListRequest {
    pub block_id: Option<BlockId>,
}

impl TryFrom<crate::proto::types::GetBlockSummaryListRequest> for GetBlockSummaryListRequest {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::GetBlockSummaryListRequest) -> Result<Self> {
        let block_id = match proto.block_id {
            Some(id) => {
                Some(BlockId::try_from(id).expect("BlockId err."))
            },
            None => None,
        };
        Ok(Self { block_id })
    }
}

/// Helper to construct and parse [`proto::types::BlockSummary`]
#[derive(PartialEq, Eq, Clone)]
pub struct BlockSummary {
    pub block_id: HashValue,
    pub height: u64,
    pub parent_id: HashValue,
    pub accumulator_root_hash: HashValue,
    pub state_root_hash: HashValue,
    pub miner: AccountAddress,
    pub nonce: u64,
    pub target: HashValue,
    pub algo: u32,
}

impl TryFrom<crate::proto::types::BlockSummary> for BlockSummary {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::BlockSummary) -> Result<Self> {
        let block_id = HashValue::from_slice(&proto.block_id[..]).expect("block_id err.");
        let height = proto.height;
        let parent_id = HashValue::from_slice(&proto.parent_id[..]).expect("parent_id err.");
        let accumulator_root_hash = HashValue::from_slice(&proto.accumulator_root_hash[..]).expect("accumulator_root_hash err.");
        let state_root_hash = HashValue::from_slice(&proto.state_root_hash[..]).expect("state_root_hash err.");
        let miner = AccountAddress::try_from(&proto.miner[..]).expect("miner err.");
        let nonce = proto.nonce;
        let target = HashValue::from_slice(&proto.target[..]).expect("target err.");
        let algo = proto.algo;

        Ok(Self { block_id, height, parent_id, accumulator_root_hash, state_root_hash, miner, nonce, target, algo})
    }
}

impl From<BlockSummary> for crate::proto::types::BlockSummary {
    fn from(req: BlockSummary) -> Self {
        Self {
            block_id: req.block_id.to_vec(),
            height: req.height,
            parent_id: req.parent_id.to_vec(),
            accumulator_root_hash: req.accumulator_root_hash.to_vec(),
            state_root_hash: req.state_root_hash.to_vec(),
            miner: req.miner.to_vec(),
            nonce: req.nonce,
            target: req.target.to_vec(),
            algo: req.algo,
        }
    }
}

/// Helper to construct and parse [`proto::types::GetBlockSummaryListResponse`]
#[derive(PartialEq, Eq, Clone)]
pub struct GetBlockSummaryListResponse {
    pub blocks: Vec<BlockSummary>,
}

impl TryFrom<crate::proto::types::GetBlockSummaryListResponse> for GetBlockSummaryListResponse {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::GetBlockSummaryListResponse) -> Result<Self> {
        let mut blocks = Vec::new();
        for b in proto.blocks {
            let tmp = BlockSummary::try_from(b).expect("to BlockSummary err.");
            blocks.push(tmp);
        }

        Ok(Self { blocks})
    }
}

impl From<GetBlockSummaryListResponse> for crate::proto::types::GetBlockSummaryListResponse {
    fn from(req: GetBlockSummaryListResponse) -> Self {
        let mut blocks= Vec::new();
        for b in req.blocks {
            let tmp = b.into();
            blocks.push(tmp);
        }
        Self {
            blocks
        }
    }
}

////////////////
/// Helper to construct and parse [`proto::types::Version`]
#[derive(PartialEq, Eq, Clone)]
pub struct Version {
    pub ver:u64,
}

impl TryFrom<crate::proto::types::Version> for Version {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::Version) -> Result<Self> {
        Ok(Self { ver:proto.ver})
    }
}

impl From<Version> for crate::proto::types::Version {
    fn from(req: Version) -> Self {
        Self {
            ver:req.ver
        }
    }
}

/// Helper to construct and parse [`proto::types::LatestVersionResponse`]
#[derive(PartialEq, Eq, Clone)]
pub struct LatestVersionResponse {
    pub version:Option<Version>,
}

impl TryFrom<crate::proto::types::LatestVersionResponse> for LatestVersionResponse {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::LatestVersionResponse) -> Result<Self> {
        let version = match proto.version {
            Some(v) => Some(Version::try_from(v).expect("to Version err.")),
            None => None,
        };
        Ok(Self { version})
    }
}

impl From<LatestVersionResponse> for crate::proto::types::LatestVersionResponse {
    fn from(req: LatestVersionResponse) -> Self {
        let version = match req.version {
            Some(v) => {
                Some(v.into())
            },
            None => None,
        };
        Self {
            version
        }
    }
}

/// Helper to construct and parse [`proto::types::GetTransactionListRequest`]
#[derive(PartialEq, Eq, Clone)]
pub struct GetTransactionListRequest {
    pub version:Option<Version>,
}

impl TryFrom<crate::proto::types::GetTransactionListRequest> for GetTransactionListRequest {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::GetTransactionListRequest) -> Result<Self> {
        let version = match proto.version {
            Some(v) => Some(Version::try_from(v).expect("to Version err.")),
            None => None,
        };
        Ok(Self { version})
    }
}

impl From<GetTransactionListRequest> for crate::proto::types::GetTransactionListRequest {
    fn from(req: GetTransactionListRequest) -> Self {
        let version = match req.version {
            Some(v) => {
                Some(v.into())
            },
            None => None,
        };
        Self {
            version
        }
    }
}

/// Helper to construct and parse [`proto::types::GetTransactionListResponse`]
#[derive(PartialEq, Eq, Clone)]
pub struct GetTransactionListResponse {
    pub transactions:Vec<Transaction>,
}

impl TryFrom<crate::proto::types::GetTransactionListResponse> for GetTransactionListResponse {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::GetTransactionListResponse) -> Result<Self> {
        let mut transactions = Vec::new();
        for txn in proto.transactions {
            let tmp = Transaction::try_from(txn).expect("to BlockSummary err.");
            transactions.push(tmp);
        }

        Ok(Self { transactions})
    }
}

impl From<GetTransactionListResponse> for crate::proto::types::GetTransactionListResponse {
    fn from(req: GetTransactionListResponse) -> Self {
        let mut transactions= Vec::new();
        for txn in req.transactions {
            let tmp = txn.into();
            transactions.push(tmp);
        }
        Self {
            transactions
        }
    }
}

/// Helper to construct and parse [`proto::types::GetTransactionByVersionResponse`]
#[derive(PartialEq, Eq, Clone)]
pub struct GetTransactionByVersionResponse {
    pub txn: Option<Transaction>,
}

impl TryFrom<crate::proto::types::GetTransactionByVersionResponse> for GetTransactionByVersionResponse {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::GetTransactionByVersionResponse) -> Result<Self> {
        let txn = match proto.txn {
            Some(t) => {Some(Transaction::try_from(t).expect("to BlockSummary err."))},
            None => None,
        };
        Ok(Self { txn})
    }
}

impl From<GetTransactionByVersionResponse> for crate::proto::types::GetTransactionByVersionResponse {
    fn from(req: GetTransactionByVersionResponse) -> Self {
        let txn = match req.txn {
            Some(t) => {
                Some(t.into())
            },
            None => None,
        };
        Self {
            txn
        }
    }
}

pub mod prelude {
    pub use super::*;
}
