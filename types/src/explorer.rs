use anyhow::Result;
use super::account_address::AccountAddress;
use std::convert::TryFrom;
use libra_crypto::HashValue;

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

pub mod prelude {
    pub use super::*;
}
