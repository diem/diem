use libra_crypto::HashValue;
use miner::types::{from_slice, Algo, H256, U256};

pub const BLOCK_WINDOW: u32 = 12;
pub const BLOCK_TIME_SEC: u64 = 15;

pub fn difficult_1_target() -> H256 {
    (U256::max_value() / DIFF_1_HASH_TIMES).into()
}

pub const DIFF_1_HASH_TIMES: u128 = 1000;

pub fn current_hash_rate(target: &[u8]) -> u64 {
    // current_hash_rate = (difficult_1_target/target_current) * difficult_1_hash/block_per_esc
    let target_h256: H256 = from_slice(target).into();
    let target_u256: U256 = target_h256.into();
    let target_1_u256: U256 = difficult_1_target().into();
    ((target_1_u256 / target_u256) * DIFF_1_HASH_TIMES / BLOCK_TIME_SEC).0[0]
}

pub fn get_next_work_required<B>(block_index: B, algo: Algo) -> H256
where
    B: TBlockIndex,
{
    let blocks = {
        let mut blocks: Vec<BlockInfo> = vec![];
        let mut count = 0;
        for b in block_index {
            if b.algo != algo {
                continue;
            }
            blocks.push(b);
            count += 1;
            if count == BLOCK_WINDOW {
                break;
            }
        }
        blocks
    };
    if blocks.len() <= 1 {
        return difficult_1_target();
    }
    let target = blocks[0].target.clone();
    let time_used = {
        let mut time_used: u64 = 0;
        let mut latest_block_index = 0;
        while latest_block_index < blocks.len() - 1 {
            let diff =
                blocks[latest_block_index].timestamp - blocks[latest_block_index + 1].timestamp;
            time_used += diff;
            latest_block_index += 1;
        }
        if time_used == 0 {
            1
        } else {
            time_used
        }
    };
    //new_target = old_target * (old_time/plan_time);
    let diff_1_target_u256: U256 = difficult_1_target().into();
    let target_u256: U256 = target.into();
    if target_u256.clone() / (BLOCK_TIME_SEC * blocks.len() as u64)
        >= diff_1_target_u256 / time_used
    {
        return difficult_1_target();
    }
    let new_target_u256 = target_u256 / (BLOCK_TIME_SEC * blocks.len() as u64) * time_used;
    let new_target: H256 = new_target_u256.into();
    new_target
}

#[derive(Clone)]
pub struct BlockInfo {
    pub timestamp: u64,
    pub target: H256,
    pub algo: Algo,
}

pub trait TBlockIndex: Iterator<Item = BlockInfo> + Send + Sync + Clone {
    fn set_latest(&mut self, block: HashValue);
}
