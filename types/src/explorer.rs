use super::account_address::AccountAddress;
use crate::transaction::Transaction;
use anyhow::{format_err, Result};
use libra_crypto::HashValue;
use std::convert::TryFrom;

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
            Some(id) => Some(BlockId::try_from(id).expect("BlockId err.")),
            None => None,
        };
        Ok(Self { block_id })
    }
}

impl From<GetBlockSummaryListRequest> for crate::proto::types::GetBlockSummaryListRequest {
    fn from(req: GetBlockSummaryListRequest) -> Self {
        let block_id = match req.block_id {
            Some(b) => Some(b.into()),
            None => None,
        };
        Self { block_id }
    }
}

/// Helper to construct and parse [`proto::types::BlockSummary`]
#[derive(PartialEq, Debug, Eq, Clone)]
pub struct BlockSummary {
    pub block_id: HashValue,
    pub height: u64,
    pub parent_id: HashValue,
    pub accumulator_root_hash: HashValue,
    pub state_root_hash: HashValue,
    pub miner: AccountAddress,
    pub nonce: u32,
    pub target: HashValue,
    pub algo: u32,
}

impl TryFrom<crate::proto::types::BlockSummary> for BlockSummary {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::BlockSummary) -> Result<Self> {
        let block_id = HashValue::from_slice(&proto.block_id[..]).expect("block_id err.");
        let height = proto.height;
        let parent_id = HashValue::from_slice(&proto.parent_id[..]).expect("parent_id err.");
        let accumulator_root_hash = HashValue::from_slice(&proto.accumulator_root_hash[..])
            .expect("accumulator_root_hash err.");
        let state_root_hash =
            HashValue::from_slice(&proto.state_root_hash[..]).expect("state_root_hash err.");
        let miner = AccountAddress::try_from(&proto.miner[..]).expect("miner err.");
        let nonce = proto.nonce;
        let target = HashValue::from_slice(&proto.target[..]).expect("target err.");
        let algo = proto.algo;

        Ok(Self {
            block_id,
            height,
            parent_id,
            accumulator_root_hash,
            state_root_hash,
            miner,
            nonce,
            target,
            algo,
        })
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
#[derive(PartialEq, Debug, Eq, Clone)]
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

        Ok(Self { blocks })
    }
}

impl From<GetBlockSummaryListResponse> for crate::proto::types::GetBlockSummaryListResponse {
    fn from(req: GetBlockSummaryListResponse) -> Self {
        let mut blocks = Vec::new();
        for b in req.blocks {
            let tmp = b.into();
            blocks.push(tmp);
        }
        Self { blocks }
    }
}

////////////////
/// Helper to construct and parse [`proto::types::Version`]
#[derive(PartialEq, Debug, Eq, Clone)]
pub struct Version {
    pub ver: u64,
}

impl TryFrom<crate::proto::types::Version> for Version {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::Version) -> Result<Self> {
        Ok(Self { ver: proto.ver })
    }
}

impl From<Version> for crate::proto::types::Version {
    fn from(req: Version) -> Self {
        Self { ver: req.ver }
    }
}

/// Helper to construct and parse [`proto::types::LatestVersionResponse`]
#[derive(PartialEq, Eq, Clone)]
pub struct LatestVersionResponse {
    pub version: Option<Version>,
}

impl TryFrom<crate::proto::types::LatestVersionResponse> for LatestVersionResponse {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::LatestVersionResponse) -> Result<Self> {
        let version = match proto.version {
            Some(v) => Some(Version::try_from(v).expect("to Version err.")),
            None => None,
        };
        Ok(Self { version })
    }
}

impl From<LatestVersionResponse> for crate::proto::types::LatestVersionResponse {
    fn from(req: LatestVersionResponse) -> Self {
        let version = match req.version {
            Some(v) => Some(v.into()),
            None => None,
        };
        Self { version }
    }
}

/// Helper to construct and parse [`proto::types::GetTransactionListRequest`]
#[derive(PartialEq, Debug, Eq, Clone)]
pub struct GetTransactionListRequest {
    pub version: Option<Version>,
}

impl TryFrom<crate::proto::types::GetTransactionListRequest> for GetTransactionListRequest {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::GetTransactionListRequest) -> Result<Self> {
        let version = match proto.version {
            Some(v) => Some(Version::try_from(v).expect("to Version err.")),
            None => None,
        };
        Ok(Self { version })
    }
}

impl From<GetTransactionListRequest> for crate::proto::types::GetTransactionListRequest {
    fn from(req: GetTransactionListRequest) -> Self {
        let version = match req.version {
            Some(v) => Some(v.into()),
            None => None,
        };
        Self { version }
    }
}

/// Helper to construct and parse [`proto::types::GetTransactionListResponse`]
#[derive(PartialEq, Debug, Eq, Clone)]
pub struct GetTransactionListResponse {
    pub transactions: Vec<Transaction>,
}

impl TryFrom<crate::proto::types::GetTransactionListResponse> for GetTransactionListResponse {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::GetTransactionListResponse) -> Result<Self> {
        let mut transactions = Vec::new();
        for txn in proto.transactions {
            let tmp = Transaction::try_from(txn).expect("to BlockSummary err.");
            transactions.push(tmp);
        }

        Ok(Self { transactions })
    }
}

impl From<GetTransactionListResponse> for crate::proto::types::GetTransactionListResponse {
    fn from(req: GetTransactionListResponse) -> Self {
        let mut transactions = Vec::new();
        for txn in req.transactions {
            let tmp = txn.into();
            transactions.push(tmp);
        }
        Self { transactions }
    }
}

/// Helper to construct and parse [`proto::types::GetTransactionByVersionResponse`]
#[derive(PartialEq, Eq, Clone)]
pub struct GetTransactionByVersionResponse {
    pub txn: Option<Transaction>,
}

impl TryFrom<crate::proto::types::GetTransactionByVersionResponse>
    for GetTransactionByVersionResponse
{
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::GetTransactionByVersionResponse) -> Result<Self> {
        let txn = match proto.txn {
            Some(t) => Some(Transaction::try_from(t).expect("to BlockSummary err.")),
            None => None,
        };
        Ok(Self { txn })
    }
}

impl From<GetTransactionByVersionResponse>
    for crate::proto::types::GetTransactionByVersionResponse
{
    fn from(req: GetTransactionByVersionResponse) -> Self {
        let txn = match req.txn {
            Some(t) => Some(t.into()),
            None => None,
        };
        Self { txn }
    }
}

/// Helper to construct and parse [`proto::types::DifficultHashRate`]
#[derive(PartialEq, Debug, Eq, Clone)]
pub struct DifficultHashRate {
    pub difficulty: u64,
}

impl TryFrom<crate::proto::types::DifficultHashRate> for DifficultHashRate {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::DifficultHashRate) -> Result<Self> {
        Ok(Self {
            difficulty: proto.difficulty,
        })
    }
}

impl From<DifficultHashRate> for crate::proto::types::DifficultHashRate {
    fn from(req: DifficultHashRate) -> Self {
        Self {
            difficulty: req.difficulty,
        }
    }
}

/// Helper to construct and parse [`proto::types::BlockDetail`]
#[derive(PartialEq, Debug, Eq, Clone)]
pub struct BlockDetail {
    pub bytes: Vec<u8>,
}

impl TryFrom<crate::proto::types::BlockDetail> for BlockDetail {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::BlockDetail) -> Result<Self> {
        Ok(Self { bytes: proto.bytes })
    }
}

impl From<BlockDetail> for crate::proto::types::BlockDetail {
    fn from(req: BlockDetail) -> Self {
        Self { bytes: req.bytes }
    }
}

/// Helper to construct and parse [`proto::types::GetBlockByBlockIdResponse`]
#[derive(PartialEq, Debug, Eq, Clone)]
pub struct GetBlockByBlockIdResponse {
    pub block_detail: Option<BlockDetail>,
}

impl TryFrom<crate::proto::types::GetBlockByBlockIdResponse> for GetBlockByBlockIdResponse {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::GetBlockByBlockIdResponse) -> Result<Self> {
        let block_detail = match proto.block_detail {
            Some(block) => Some(BlockDetail::try_from(block).expect("BlockDetail err.")),
            None => None,
        };
        Ok(Self { block_detail })
    }
}

impl From<GetBlockByBlockIdResponse> for crate::proto::types::GetBlockByBlockIdResponse {
    fn from(req: GetBlockByBlockIdResponse) -> Self {
        let block_detail = match req.block_detail {
            Some(b) => Some(b.into()),
            None => None,
        };
        Self { block_detail }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Eq, PartialEq)]
pub enum BlockRequestItem {
    BlockIdItem { block_id: BlockId },
    GetBlockSummaryListRequestItem { request: GetBlockSummaryListRequest },
    LatestBlockHeightRequestItem,
    DifficultHashRateRequestItem,
}

impl TryFrom<crate::proto::types::BlockRequestItem> for BlockRequestItem {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::BlockRequestItem) -> Result<Self> {
        use crate::proto::types::block_request_item::BlockRequestedItems::*;

        let item = proto
            .block_requested_items
            .ok_or_else(|| format_err!("Missing block_requested_items"))?;

        let request = match item {
            BlockIdItem(request) => {
                let block_id = BlockId::try_from(request)?;
                BlockRequestItem::BlockIdItem { block_id }
            }
            GetBlockSummaryListRequestItem(r) => {
                let request = GetBlockSummaryListRequest::try_from(r)?;
                BlockRequestItem::GetBlockSummaryListRequestItem { request }
            }
            LatestBlockHeightRequestItem(_) => BlockRequestItem::LatestBlockHeightRequestItem,
            DifficultHashRateRequestItem(_) => BlockRequestItem::DifficultHashRateRequestItem,
        };

        Ok(request)
    }
}

impl From<BlockRequestItem> for crate::proto::types::BlockRequestItem {
    fn from(request: BlockRequestItem) -> Self {
        use crate::proto::types::block_request_item::BlockRequestedItems;

        let req = match request {
            BlockRequestItem::BlockIdItem { block_id } => {
                let block_id = block_id.into();
                BlockRequestedItems::BlockIdItem(block_id)
            }
            BlockRequestItem::GetBlockSummaryListRequestItem { request } => {
                let get_block_summary_list_request = request.into();
                BlockRequestedItems::GetBlockSummaryListRequestItem(get_block_summary_list_request)
            }
            BlockRequestItem::LatestBlockHeightRequestItem => {
                BlockRequestedItems::LatestBlockHeightRequestItem({ () })
            }
            BlockRequestItem::DifficultHashRateRequestItem => {
                BlockRequestedItems::DifficultHashRateRequestItem({ () })
            }
        };

        Self {
            block_requested_items: Some(req),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlockResponseItem {
    GetBlockByBlockIdResponseItem(GetBlockByBlockIdResponse),
    GetBlockSummaryListResponseItem { resp: GetBlockSummaryListResponse },
    LatestBlockHeightResponseItem { height: u64 },
    DifficultHashRateResponseItem(DifficultHashRate),
}

impl TryFrom<crate::proto::types::BlockResponseItem> for BlockResponseItem {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::BlockResponseItem) -> Result<Self> {
        use crate::proto::types::block_response_item::BlockResponseItems::*;

        let item = proto
            .block_response_items
            .ok_or_else(|| format_err!("Missing block_response_items"))?;

        let response = match item {
            GetBlockByBlockIdResponseItem(r) => BlockResponseItem::GetBlockByBlockIdResponseItem(
                GetBlockByBlockIdResponse::try_from(r)?,
            ),
            GetBlockSummaryListResponseItem(r) => {
                let resp = GetBlockSummaryListResponse::try_from(r)?;
                BlockResponseItem::GetBlockSummaryListResponseItem { resp }
            }
            LatestBlockHeightResponseItem(r) => {
                BlockResponseItem::LatestBlockHeightResponseItem { height: r.height }
            }
            DifficultHashRateResponseItem(r) => {
                BlockResponseItem::DifficultHashRateResponseItem(DifficultHashRate::try_from(r)?)
            }
        };

        Ok(response)
    }
}

impl From<BlockResponseItem> for crate::proto::types::BlockResponseItem {
    fn from(response: BlockResponseItem) -> Self {
        use crate::proto::types::block_response_item::BlockResponseItems;

        let resp = match response {
            BlockResponseItem::GetBlockByBlockIdResponseItem(resp) => {
                BlockResponseItems::GetBlockByBlockIdResponseItem(resp.into())
            }
            BlockResponseItem::GetBlockSummaryListResponseItem { resp } => {
                let r = resp.into();
                BlockResponseItems::GetBlockSummaryListResponseItem(r)
            }
            BlockResponseItem::LatestBlockHeightResponseItem { height } => {
                let mut r = crate::proto::types::LatestBlockHeightResponse::default();
                r.height = height;
                BlockResponseItems::LatestBlockHeightResponseItem(r)
            }
            BlockResponseItem::DifficultHashRateResponseItem(difficulty) => {
                BlockResponseItems::DifficultHashRateResponseItem(difficulty.into())
            }
        };

        Self {
            block_response_items: Some(resp),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Eq, PartialEq)]
pub enum TxnRequestItem {
    LatestVersionRequestItem,
    GetTransactionListRequestItem { request: GetTransactionListRequest },
    GetTransactionByVersionRequestItem { version: u64 },
}

impl TryFrom<crate::proto::types::TxnRequestItem> for TxnRequestItem {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::TxnRequestItem) -> Result<Self> {
        use crate::proto::types::txn_request_item::TxnRequestedItems::*;

        let item = proto
            .txn_requested_items
            .ok_or_else(|| format_err!("Missing block_requested_items"))?;

        let request = match item {
            LatestVersionRequestItem(_) => TxnRequestItem::LatestVersionRequestItem,
            GetTransactionListRequestItem(request) => {
                let r = GetTransactionListRequest::try_from(request)?;
                TxnRequestItem::GetTransactionListRequestItem { request: r }
            }
            GetTransactionByVersionRequestItem(r) => {
                let ver = Version::try_from(r)?;
                TxnRequestItem::GetTransactionByVersionRequestItem { version: ver.ver }
            }
        };

        Ok(request)
    }
}

impl From<TxnRequestItem> for crate::proto::types::TxnRequestItem {
    fn from(request: TxnRequestItem) -> Self {
        use crate::proto::types::txn_request_item::TxnRequestedItems;

        let req = match request {
            TxnRequestItem::LatestVersionRequestItem => {
                TxnRequestedItems::LatestVersionRequestItem({ () })
            }
            TxnRequestItem::GetTransactionListRequestItem { request } => {
                TxnRequestedItems::GetTransactionListRequestItem(request.into())
            }
            TxnRequestItem::GetTransactionByVersionRequestItem { version } => {
                let ver = Version { ver: version };
                TxnRequestedItems::GetTransactionByVersionRequestItem(ver.into())
            }
        };

        Self {
            txn_requested_items: Some(req),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Eq, PartialEq)]
pub enum TxnResponseItem {
    LatestVersionResponseItem(LatestVersionResponse),
    GetTransactionListResponseItem(GetTransactionListResponse),
    GetTransactionByVersionResponseItem(GetTransactionByVersionResponse),
}

impl TryFrom<crate::proto::types::TxnResponseItem> for TxnResponseItem {
    type Error = anyhow::Error;

    fn try_from(proto: crate::proto::types::TxnResponseItem) -> Result<Self> {
        use crate::proto::types::txn_response_item::TxnResponseItems::*;

        let item = proto
            .txn_response_items
            .ok_or_else(|| format_err!("Missing txn_response_items"))?;

        let response = match item {
            LatestVersionResponseItem(resp) => {
                TxnResponseItem::LatestVersionResponseItem(LatestVersionResponse::try_from(resp)?)
            }
            GetTransactionListResponseItem(resp) => {
                TxnResponseItem::GetTransactionListResponseItem(
                    GetTransactionListResponse::try_from(resp)?,
                )
            }
            GetTransactionByVersionResponseItem(resp) => {
                TxnResponseItem::GetTransactionByVersionResponseItem(
                    GetTransactionByVersionResponse::try_from(resp)?,
                )
            }
        };

        Ok(response)
    }
}

impl From<TxnResponseItem> for crate::proto::types::TxnResponseItem {
    fn from(response: TxnResponseItem) -> Self {
        use crate::proto::types::txn_response_item::TxnResponseItems;

        let resp = match response {
            TxnResponseItem::LatestVersionResponseItem(r) => {
                TxnResponseItems::LatestVersionResponseItem(r.into())
            }
            TxnResponseItem::GetTransactionListResponseItem(r) => {
                TxnResponseItems::GetTransactionListResponseItem(r.into())
            }
            TxnResponseItem::GetTransactionByVersionResponseItem(r) => {
                TxnResponseItems::GetTransactionByVersionResponseItem(r.into())
            }
        };

        Self {
            txn_response_items: Some(resp),
        }
    }
}

pub mod prelude {
    pub use super::*;
}
