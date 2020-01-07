use anyhow::{format_err, Result};
use consensus_types::block::Block;
use consensus_types::common::Payload;
use lcs;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ChainStateRequest {
    nonce: u64,
}

impl ChainStateRequest {
    pub fn new(nonce: u64) -> Self {
        Self { nonce }
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

impl TryFrom<network::proto::ChainStateRequest> for ChainStateRequest {
    type Error = anyhow::Error;

    fn try_from(proto: network::proto::ChainStateRequest) -> anyhow::Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl TryFrom<ChainStateRequest> for network::proto::ChainStateRequest {
    type Error = anyhow::Error;

    fn try_from(req: ChainStateRequest) -> anyhow::Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&req)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ChainStateResponse<T> {
    nonce: u64,
    height: u64,
    #[serde(bound(deserialize = "Block<T>: Deserialize<'de>"))]
    latest_block: Block<T>,
}

impl<T> ChainStateResponse<T> {
    pub fn new(nonce: u64, block: Block<T>) -> Self {
        Self {
            nonce,
            height: block.round(),
            latest_block: block,
        }
    }
}

impl<T: Payload> TryFrom<network::proto::ChainStateResponse> for ChainStateResponse<T> {
    type Error = anyhow::Error;

    fn try_from(proto: network::proto::ChainStateResponse) -> anyhow::Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl<T: Payload> TryFrom<ChainStateResponse<T>> for network::proto::ChainStateResponse {
    type Error = anyhow::Error;

    fn try_from(req: ChainStateResponse<T>) -> anyhow::Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&req)?,
        })
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Eq, Debug, PartialEq)]
pub enum ChainStateMsg<T> {
    CsReq(ChainStateRequest),
    CsResp(ChainStateResponse<T>),
}

impl<T: Payload> TryFrom<network::proto::ChainStateMsg> for ChainStateMsg<T> {
    type Error = anyhow::Error;

    fn try_from(proto: network::proto::ChainStateMsg) -> Result<Self> {
        let item = proto
            .message
            .ok_or_else(|| format_err!("Missing message"))?;

        let response = match item {
            ::network::proto::ChainStateMsg_oneof::CsReq(r) => {
                ChainStateMsg::CsReq(ChainStateRequest::try_from(r)?)
            }
            ::network::proto::ChainStateMsg_oneof::CsResp(r) => {
                ChainStateMsg::CsResp(ChainStateResponse::try_from(r)?)
            }
        };

        Ok(response)
    }
}

impl<T: Payload> TryFrom<ChainStateMsg<T>> for network::proto::ChainStateMsg {
    type Error = anyhow::Error;

    fn try_from(response: ChainStateMsg<T>) -> Result<Self> {
        let resp = match response {
            ChainStateMsg::CsReq(r) => network::proto::ChainStateMsg_oneof::CsReq(r.try_into()?),
            ChainStateMsg::CsResp(r) => network::proto::ChainStateMsg_oneof::CsResp(r.try_into()?),
        };

        Ok(Self {
            message: Some(resp),
        })
    }
}
