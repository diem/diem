use crate::chained_bft::consensusdb::ConsensusDB;
use crate::pow::payload_ext::BlockPayloadExt;
use crate::pow::target::{difficult_1_target, get_next_work_required, BlockInfo, TBlockIndex};
use async_std::{
    sync::{channel, Receiver, Sender},
    task,
};
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use miner::miner::verify;
use miner::types::{from_slice, Algo, MineCtx, MineState, Proof, Solution, H256, U256};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct MineStateManager<B>
where
    B: TBlockIndex,
{
    inner: Arc<Mutex<StateInner>>,
    block_index: B,
}

struct StateInner {
    mine_ctx: Option<MineCtx>,
    tx: Option<Sender<Proof>>,
}

impl<B> MineStateManager<B>
where
    B: TBlockIndex,
{
    pub fn new(block_index: B) -> Self {
        MineStateManager {
            inner: Arc::new(Mutex::new(StateInner {
                mine_ctx: None,
                tx: None,
            })),
            block_index,
        }
    }

    pub fn set_latest_block(&mut self, block: HashValue) {
        self.block_index.set_latest(block)
    }
}

impl<B> MineState for MineStateManager<B>
where
    B: TBlockIndex,
{
    fn get_current_mine_ctx(&self, algo: Algo) -> Option<MineCtx> {
        let mut inner = self.inner.lock().unwrap();
        let ctx = inner.mine_ctx.as_mut()?;
        let target = Some(get_next_work_required(
            self.block_index.clone(),
            algo.clone(),
        ));
        ctx.algo = Some(algo.clone());
        ctx.target = target.clone();
        debug!(
            "Prepare to mine block algo:{:?}, header:{:?}, target:{:?}",
            algo.clone(),
            ctx.header.clone(),
            target
        );
        Some(MineCtx {
            header: ctx.header.clone(),
            target,
            algo: Some(algo),
        })
    }

    fn mine_accept(&self, mine_ctx_req: &MineCtx, solution: Solution, nonce: u32) -> bool {
        let mut x = self.inner.lock().unwrap();
        if let Some(mine_ctx) = &x.mine_ctx {
            if mine_ctx.target.clone().is_none() {
                return false;
            }
            let target: U256 = mine_ctx.target.clone().unwrap().into();
            if mine_ctx == mine_ctx_req {
                if verify(
                    &mine_ctx.header,
                    nonce,
                    solution.clone(),
                    &mine_ctx.algo.clone().unwrap(),
                    &target,
                ) != true
                {
                    return false;
                }
                let proof = Proof {
                    solution,
                    nonce,
                    algo: mine_ctx.algo.clone().unwrap(),
                    target: mine_ctx.target.clone().unwrap(),
                };
                if let Some(tx) = x.tx.take() {
                    task::block_on(async move {
                        debug!("Received Mined block");
                        tx.send(proof).await;
                    });
                    *x = StateInner {
                        mine_ctx: None,
                        tx: None,
                    };
                    return true;
                }
            }
        }
        return false;
    }

    fn mine_block(&mut self, header: Vec<u8>) -> (Receiver<Proof>, Sender<Proof>) {
        let mut x = self.inner.lock().unwrap();
        let (tx, rx) = channel(1);
        let mine_ctx = MineCtx {
            header,
            target: None,
            algo: None,
        };
        *x = StateInner {
            mine_ctx: Some(mine_ctx),
            tx: Some(tx.clone()),
        };
        (rx, tx)
    }
}

#[derive(Clone)]
pub struct BlockIndex {
    block_store: Arc<ConsensusDB>,
    latest_block: Arc<RwLock<Option<HashValue>>>,
}

impl Iterator for BlockIndex {
    type Item = BlockInfo;
    fn next(&mut self) -> Option<Self::Item> {
        let ret: Option<Self::Item>;
        let latest_hash = self.latest_block.read().unwrap().unwrap();
        if latest_hash == HashValue::zero() {
            return None;
        }
        let mut next_hash = latest_hash.clone();
        if let Some(block) = self
            .block_store
            .get_block_by_hash::<BlockPayloadExt>(&next_hash)
        {
            let _ = match block.payload() {
                Some(payload) => {
                    let target: H256 = from_slice(&payload.target).into();
                    let algo: Algo = payload.algo.clone().into();
                    ret = Some(BlockInfo {
                        timestamp: block.timestamp_usecs(),
                        target,
                        algo,
                    });
                    next_hash = block.parent_id();
                }
                None => {
                    ret = Some(BlockInfo {
                        timestamp: block.timestamp_usecs(),
                        target: difficult_1_target(),
                        algo: Algo::SCRYPT,
                    });
                    next_hash = HashValue::zero();
                }
            };
        } else {
            ret = None;
            next_hash = HashValue::zero();
        }
        self.set_latest(next_hash);
        return ret;
    }
}

impl TBlockIndex for BlockIndex {
    fn set_latest(&mut self, block: HashValue) {
        *self.latest_block.write().unwrap() = Some(block);
    }
}

impl BlockIndex {
    pub fn new(block_store: Arc<ConsensusDB>) -> Self {
        Self {
            block_store,
            latest_block: Arc::new(RwLock::new(None)),
        }
    }
}

#[derive(Clone)]
pub struct DummyBlockIndex {
    pub inner: Arc<Mutex<DummyBlockInner>>,
}

pub struct DummyBlockInner {
    pub blocks: Vec<BlockInfo>,
    pub count: usize,
}

impl DummyBlockIndex {
    pub fn new() -> Self {
        let difficult_1_target = H256::max_value();
        let genesis_block = BlockInfo {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            target: difficult_1_target,
            algo: Algo::SCRYPT,
        };
        let mut blocks = Vec::<BlockInfo>::new();
        blocks.push(genesis_block);
        Self {
            inner: Arc::new(Mutex::new(DummyBlockInner { blocks, count: 1 })),
        }
    }
    pub fn add_block(&mut self, target: H256, algo: Algo) {
        self.inner.lock().unwrap().blocks.push(BlockInfo {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            target,
            algo,
        });
        self.inner.lock().unwrap().count += 1;
    }
    pub fn reset_iter(&mut self) {
        let len = self.inner.lock().unwrap().blocks.len();
        self.inner.lock().unwrap().count = len;
    }
}

impl TBlockIndex for DummyBlockIndex {
    fn set_latest(&mut self, _block: HashValue) {}
}

impl Iterator for DummyBlockIndex {
    type Item = BlockInfo;

    fn next(&mut self) -> Option<Self::Item> {
        let ret: Option<BlockInfo>;
        let c = self.inner.lock().unwrap().count;
        if c >= 1 {
            ret = Some(self.inner.lock().unwrap().blocks[c - 1].clone());
            self.inner.lock().unwrap().count -= 1;
            return ret;
        } else {
            return None;
        }
    }
}
