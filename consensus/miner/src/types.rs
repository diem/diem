use async_std::sync::{Receiver, Sender};

pub use blake2_rfc::blake2b::blake2b;
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use cuckoo::PROOF_SIZE;
pub use ethereum_types::{H256, U256};
use proto::miner::MineCtx as MineCtxRpc;
use std::convert::{From, Into};
use std::fmt::{Debug, Error, Formatter};

pub trait MineState: Send + Sync {
    fn get_current_mine_ctx(&self, algo: Algo) -> Option<MineCtx>;
    fn mine_accept(&self, mine_ctx: &MineCtx, solution: Solution, nonce: u32) -> bool;
    fn mine_block(&mut self, header: Vec<u8>) -> (Receiver<Option<Proof>>, Sender<Option<Proof>>);
}

const CYCLE_LENGTH_U8: usize = PROOF_SIZE << 3;

#[derive(Clone)]
pub struct Solution([u8; CYCLE_LENGTH_U8]);

impl Default for Solution {
    fn default() -> Self {
        Solution([0u8; CYCLE_LENGTH_U8])
    }
}

impl Eq for Solution {}

impl Debug for Solution {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.to_vec().fmt(f)
    }
}

impl PartialEq for Solution {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_vec() == other.0.to_vec()
    }
}

impl Solution {
    pub fn hash(&self) -> H256 {
        let b = blake2b(32, &[], &self.0).as_bytes().to_owned();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&b);
        hash.into()
    }
}

impl Into<Vec<u8>> for Solution {
    fn into(self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl From<Vec<u8>> for Solution {
    fn from(s: Vec<u8>) -> Self {
        let mut sol = [0u8; CYCLE_LENGTH_U8];
        sol.copy_from_slice(&s);
        Solution(sol)
    }
}

impl From<Vec<u64>> for Solution {
    fn from(s: Vec<u64>) -> Self {
        let mut sol: [u8; CYCLE_LENGTH_U8] = [0u8; CYCLE_LENGTH_U8];
        LittleEndian::write_u64_into(&s, &mut sol);
        Solution(sol)
    }
}

impl Into<Vec<u64>> for Solution {
    fn into(self) -> Vec<u64> {
        let sol: [u64; PROOF_SIZE] = self.into();
        sol.to_vec()
    }
}

impl From<[u64; PROOF_SIZE]> for Solution {
    fn from(solution: [u64; PROOF_SIZE]) -> Self {
        let mut sol = [0u8; CYCLE_LENGTH_U8];
        LittleEndian::write_u64_into(&solution, &mut sol);
        Solution(sol)
    }
}

impl Into<[u64; PROOF_SIZE]> for Solution {
    fn into(self) -> [u64; PROOF_SIZE] {
        let mut dst = [0u64; PROOF_SIZE];
        LittleEndian::read_u64_into(&self.0, &mut dst);
        dst
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Proof {
    pub solution: Solution,
    pub nonce: u32,
    pub algo: Algo,
    pub target: U256,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MineCtx {
    pub header: Vec<u8>,
    pub target: Option<U256>,
    pub algo: Option<Algo>,
}

impl From<MineCtxRpc> for MineCtx {
    fn from(ctx: MineCtxRpc) -> Self {
        MineCtx {
            header: ctx.header,
            target: Some(U256::from_little_endian(&ctx.target)),
            algo: Some(ctx.algo.into()),
        }
    }
}

impl Into<MineCtxRpc> for MineCtx {
    fn into(self) -> MineCtxRpc {
        MineCtxRpc {
            header: self.header.to_vec(),
            target: u256_to_vec(self.target.unwrap()),
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

pub fn set_header_nonce(header: &[u8], nonce: u32) -> Vec<u8> {
    let len = header.len();
    let mut header = header.to_owned();
    header.truncate(len - 4);
    let _ = header.write_u32::<LittleEndian>(nonce);
    header
}

pub fn u256_to_vec(u: U256) -> Vec<u8> {
    let mut t = vec![0u8; 32];
    u.to_little_endian(&mut t);
    t
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_u256() {
        let u: U256 = 1.into();
        let b = u256_to_vec(u);
        let i = U256::from_little_endian(&b);
        assert!(u == i);
    }
}
