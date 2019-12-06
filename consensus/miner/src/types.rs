use async_std::{
    sync::{channel, Receiver, Sender},
    task,
};

use byteorder::{ByteOrder, LittleEndian};
use cuckoo::util::pow_input;
use cuckoo::Cuckoo;
use std::sync::{Arc, Mutex};

pub const MAX_EDGE: u8 = 6;
pub const CYCLE_LENGTH: usize = 8;
pub const DUMMY_INPUT: [u8; 80] = [
    238, 237, 143, 251, 211, 26, 16, 237, 158, 89, 77, 62, 49, 241, 85, 233, 49, 77, 230, 148, 177,
    49, 129, 38, 152, 148, 40, 170, 1, 115, 145, 191, 44, 10, 206, 23, 226, 132, 186, 196, 204,
    205, 133, 173, 209, 20, 116, 16, 159, 161, 117, 167, 151, 171, 246, 181, 209, 140, 189, 163,
    206, 155, 209, 157, 110, 2, 79, 249, 34, 228, 252, 245, 141, 27, 9, 156, 85, 58, 121, 46,
];

#[derive(PartialEq, Eq, Debug)]
pub struct MineCtx {
    pub nonce: u64,
    pub header: Vec<u8>,
}

#[derive(Clone)]
pub struct MineStateManager {
    inner: Arc<Mutex<StateInner>>,
    cuckoo: Cuckoo,
}

struct StateInner {
    mine_ctx: Option<MineCtx>,
    tx: Option<Sender<Vec<u8>>>,
}

impl MineStateManager {
    pub fn new() -> Self {
        MineStateManager {
            inner: Arc::new(Mutex::new(StateInner {
                mine_ctx: None,
                tx: None,
            })),
            cuckoo: Cuckoo::new(MAX_EDGE, CYCLE_LENGTH),
        }
    }
}

pub trait MineState: Send + Sync {
    fn get_current_mine_ctx(&self) -> Option<MineCtx>;
    fn mine_accept(&self, mine_ctx: &MineCtx, proof: Vec<u8>) -> bool;
    fn mine_block(&mut self, mine_ctx: MineCtx) -> (Receiver<Vec<u8>>, Sender<Vec<u8>>);
}

impl MineState for MineStateManager {
    fn get_current_mine_ctx(&self) -> Option<MineCtx> {
        let inner = self.inner.lock().unwrap();
        let ctx = inner.mine_ctx.as_ref()?;
        Some(MineCtx {
            header: ctx.header.clone(),
            nonce: ctx.nonce,
        })
    }
    fn mine_accept(&self, mine_ctx_req: &MineCtx, proof: Vec<u8>) -> bool {
        let mut x = self.inner.lock().unwrap();
        if let Some(mine_ctx) = &x.mine_ctx {
            if true || mine_ctx == mine_ctx_req {
                let pow_input = pow_input(&mine_ctx.header, mine_ctx.nonce);
                assert!(pow_input.len() != 0);
                let input = DUMMY_INPUT;
                let mut proof_u32 = vec![0u32; CYCLE_LENGTH];
                LittleEndian::read_u32_into(&proof, &mut proof_u32);

                if self.cuckoo.verify(&input, &proof_u32) != true {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            return false;
        }

        if let Some(tx) = x.tx.take() {
            task::block_on(async move {
                tx.send(proof).await;
            });

            *x = StateInner {
                mine_ctx: None,
                tx: None,
            };
        } else {
            return false;
        }
        return true;
    }

    fn mine_block(&mut self, mine_ctx: MineCtx) -> (Receiver<Vec<u8>>, Sender<Vec<u8>>) {
        let mut x = self.inner.lock().unwrap();
        let (tx, rx) = channel(1);
        *x = StateInner {
            mine_ctx: Some(mine_ctx),
            tx: Some(tx.clone()),
        };
        (rx, tx)
    }
}
