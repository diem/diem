use anyhow::{Error, Result};
use libra_crypto::HashValue;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockIndex {
    id: HashValue,
    parent_block_id: HashValue,
}

impl BlockIndex {
    pub fn new(id: &HashValue, parent_id: &HashValue) -> Self {
        BlockIndex {
            id: id.clone(),
            parent_block_id: parent_id.clone(),
        }
    }

    pub fn id(&self) -> HashValue {
        self.id
    }

    pub fn parent_id(&self) -> HashValue {
        self.parent_block_id
    }

    pub fn parent_id_ref(&self) -> &HashValue {
        &self.parent_block_id
    }
}

impl TryFrom<crate::proto::types::BlockIndex> for BlockIndex {
    type Error = Error;

    fn try_from(proto: crate::proto::types::BlockIndex) -> Result<Self> {
        Ok(BlockIndex::new(
            &HashValue::from_slice(proto.block_id.as_ref())?,
            &HashValue::from_slice(proto.parent_block_id.as_ref())?,
        ))
    }
}

impl From<BlockIndex> for crate::proto::types::BlockIndex {
    fn from(block_index: BlockIndex) -> Self {
        Self {
            block_id: block_index.id.to_vec(),
            parent_block_id: block_index.parent_block_id.to_vec(),
        }
    }
}
