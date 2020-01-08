use libra_crypto::HashValue;
use libra_logger::prelude::*;
use miner::types::{Algo, U256};

pub const BLOCK_WINDOW: u32 = 12;
pub const BLOCK_TIME_SEC: u32 = 5;

pub fn difficult_1_target() -> U256 {
    U256::max_value() / DIFF_1_HASH_TIMES.into()
}

pub const DIFF_1_HASH_TIMES: u32 = 10000;

pub fn current_hash_rate(target: &[u8]) -> u64 {
    // current_hash_rate = (difficult_1_target/target_current) * difficult_1_hash/block_per_esc
    let target_u256: U256 = target.into();
    ((difficult_1_target() / target_u256) * DIFF_1_HASH_TIMES).low_u64() / (BLOCK_TIME_SEC as u64)
}

pub fn get_next_work_required<B>(block_index: B, algo: Algo) -> U256
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
        debug!(
            "Block length less than 1, set target to 1 difficult:{:?}",
            difficult_1_target()
        );
        return difficult_1_target();
    }
    let target = blocks[0].target;
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
    let time_plan = BLOCK_TIME_SEC * (blocks.len() - 1) as u32;
    // new_target = old_target * time_used/time_plan
    let new_target = if let Some(target) = (target / time_plan.into()).checked_mul(time_used.into())
    {
        target
    } else {
        debug!("target large than max value, set to 1_difficult");
        difficult_1_target()
    };

    info!(
        "time_used:{:?}s, time_plan:{:?}s, each block used::{:?}s",
        time_used,
        time_plan,
        time_used / (blocks.len() - 1) as u64
    );
    new_target
    //difficult_1_target()
}

#[derive(Clone)]
pub struct BlockInfo {
    pub timestamp: u64,
    pub target: U256,
    pub algo: Algo,
}

pub trait TBlockIndex: Iterator<Item = BlockInfo> + Send + Sync + Clone {
    fn set_latest(&mut self, block: HashValue);
}
