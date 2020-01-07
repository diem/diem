use super::block::Block;
use anyhow::{Error, Result};
use libra_crypto::hash::{BlockPayloadExtHasher, CryptoHash, CryptoHasher};
use libra_crypto::HashValue;
use libra_types::transaction::SignedTransaction;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;

#[derive(Clone, Eq, PartialEq, Default, Hash, Serialize, Deserialize)]
pub struct BlockPayloadExt {
    pub txns: Vec<SignedTransaction>,
    pub nonce: u32,
    pub solve: Vec<u8>,
    pub target: Vec<u8>,
    pub algo: u32,
}

impl BlockPayloadExt {
    pub fn get_txns(&self) -> Vec<SignedTransaction> {
        self.txns.clone()
    }
}

impl fmt::Debug for BlockPayloadExt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BlockPayloadExt {{ \n \
             {{ txns: {:#?}, \n \
             nonce: {:#?}, \n \
             solve: {:#?}, \n \
             target:{:#?},\n \
             algo: {:#?}, \n \
             }} \n \
             }}",
            self.txns, self.nonce, self.solve, self.target, self.algo,
        )
    }
}

impl TryFrom<network::proto::BlockPayloadExt> for BlockPayloadExt {
    type Error = Error;

    fn try_from(proto: network::proto::BlockPayloadExt) -> Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl TryFrom<BlockPayloadExt> for network::proto::BlockPayloadExt {
    type Error = Error;

    fn try_from(payload: BlockPayloadExt) -> Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&payload)?,
        })
    }
}

impl CryptoHash for BlockPayloadExt {
    type Hasher = BlockPayloadExtHasher;

    fn hash(&self) -> HashValue {
        let bytes = lcs::to_bytes(self).expect("BlockData serialization failed");
        let mut state = Self::Hasher::default();
        state.write(bytes.as_ref());
        state.finish()
    }
}

pub fn genesis_id() -> HashValue {
    let genesis_block: Block<BlockPayloadExt> = Block::make_genesis_block();
    genesis_block.id()
}
