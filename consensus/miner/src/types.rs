use async_std::sync::{Receiver, Sender};
pub use numext_fixed_hash::H256;
pub use numext_fixed_uint::U256;
use proto::miner::MineCtx as MineCtxRpc;
use std::convert::{From, Into};

pub trait MineState: Send + Sync {
    fn get_current_mine_ctx(&self, algo: Algo) -> Option<MineCtx>;
    fn mine_accept(&self, mine_ctx: &MineCtx, solution: Vec<u8>, nonce: u64) -> bool;
    fn mine_block(&mut self, header: Vec<u8>) -> (Receiver<Option<Proof>>, Sender<Option<Proof>>);
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Proof {
    pub solution: Vec<u8>,
    pub nonce: u64,
    pub algo: Algo,
    pub target: H256,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MineCtx {
    pub header: [u8; 32],
    pub target: Option<H256>,
    pub algo: Option<Algo>,
}

pub fn from_slice(bytes: &[u8]) -> [u8; 32] {
    let mut array = [0; 32];
    assert_eq!(32, bytes.len());
    let bytes = &bytes[..array.len()];
    array.copy_from_slice(bytes);
    array
}

impl From<MineCtxRpc> for MineCtx {
    fn from(ctx: MineCtxRpc) -> Self {
        MineCtx {
            header: from_slice(&ctx.header),
            target: Some(from_slice(&ctx.target).into()),
            algo: Some(ctx.algo.into()),
        }
    }
}

impl Into<MineCtxRpc> for MineCtx {
    fn into(self) -> MineCtxRpc {
        MineCtxRpc {
            header: self.header.to_vec(),
            target: self.target.unwrap().to_vec(),
            algo: self.algo.unwrap().into(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Algo {
    CUCKOO,
    SCRYPT,
}

impl From<u32> for Algo {
    fn from(algo: u32) -> Self {
        match algo {
            0 => Algo::CUCKOO,
            1 => Algo::SCRYPT,
            _ => Algo::CUCKOO,
        }
    }
}

impl Into<u32> for Algo {
    fn into(self) -> u32 {
        match self {
            Algo::CUCKOO => 0,
            Algo::SCRYPT => 1,
        }
    }
}
