use crate::types::{MineCtx, CYCLE_LENGTH, DUMMY_INPUT, MAX_EDGE};
use async_std::{
    prelude::*,
    stream::Stream,
    task,
    task::{Context, Poll},
};
use byteorder::{ByteOrder, LittleEndian};
use cuckoo::util::blake2b_256;
use grpcio;
use grpcio::{ChannelBuilder, EnvBuilder};
use proto::miner::{MineCtx as MineCtxRpc, MineCtxRequest, MinedBlockRequest, MinerProxyClient};
use std::{pin::Pin, sync::Arc};
use std::{sync::Mutex, task::Waker, time::Duration};

struct MineCtxStream {
    client: MinerProxyClient,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl MineCtxStream {
    fn new(client: MinerProxyClient) -> Self {
        let waker: Arc<Mutex<Option<Waker>>> = Arc::new(Mutex::new(None));
        let task_waker = waker.clone();

        task::spawn(async move {
            loop {
                task::sleep(Duration::from_secs(10)).await;
                let mut inner_waker = task_waker.lock().unwrap();
                if let Some(waker) = inner_waker.take() {
                    waker.wake();
                }
            }
        });
        MineCtxStream { client, waker }
    }
}

impl Stream for MineCtxStream {
    type Item = MineCtx;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut waker = self.waker.lock().unwrap();
        match self.client.get_mine_ctx(&MineCtxRequest {}) {
            Ok(resp) => {
                if let Some(mine_ctx) = resp.mine_ctx {
                    let ctx = MineCtx {
                        header: mine_ctx.header,
                        nonce: mine_ctx.nonce,
                    };
                    Poll::Ready(Some(ctx))
                } else {
                    *waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
            Err(_e) => {
                *waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

pub struct MineClient {
    rpc_client: MinerProxyClient,
}

impl Default for MineClient {
    fn default() -> Self {
        MineClient::new("127.0.0.1:4251".to_string())
    }
}

impl MineClient {
    pub fn new(miner_server: String) -> Self {
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(&miner_server);
        let rpc_client = MinerProxyClient::new(ch);
        MineClient { rpc_client }
    }

    pub async fn start(&self) {
        let mut ctx_stream = MineCtxStream::new(self.rpc_client.clone());
        while let Some(ctx) = ctx_stream.next().await {
            let proof = mine(&ctx.header, ctx.nonce, MAX_EDGE, CYCLE_LENGTH);
            if let Some(proof) = proof {
                let req = MinedBlockRequest {
                    mine_ctx: Some(MineCtxRpc {
                        header: ctx.header,
                        nonce: ctx.nonce,
                    }),
                    proof,
                };
                let resp = self.rpc_client.mined(&req);
                println!("mined{:?}", resp);
            } else {
                task::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

extern "C" {
    pub fn c_solve(output: *mut u32, input: *const u8, max_edge: u64, cycle_length: u32) -> u32;
}

fn pow_input(header_hash: &[u8], nonce: u64) -> [u8; 40] {
    let mut input = [0; 40];
    assert!(header_hash.len() == 32);
    input[8..40].copy_from_slice(&header_hash[..32]);
    LittleEndian::write_u64(&mut input, nonce);
    input
}

pub fn mine(
    header_hash: &[u8],
    nonce: u64,
    max_edge_bits: u8,
    cycle_length: usize,
) -> Option<Vec<u8>> {
    unsafe {
        let _pow_input = pow_input(header_hash, nonce);
        //let input = blake2b_256(&pow_input.as_ref());
        let input = DUMMY_INPUT;
        let input = blake2b_256(&input.as_ref());
        let mut output = vec![0u32; cycle_length];
        let max_edge = 1 << max_edge_bits;
        if c_solve(
            output.as_mut_ptr(),
            input.as_ptr(),
            max_edge,
            cycle_length as u32,
        ) > 0
        {
            let mut output_u8 = vec![0u8; CYCLE_LENGTH << 2];
            LittleEndian::write_u32_into(&output, &mut output_u8);
            return Some(output_u8);
        }
        return None;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cuckoo::Cuckoo;

    #[test]
    fn test_mine() {
        unsafe {
            let mut output = vec![0u32; CYCLE_LENGTH];
            let input = DUMMY_INPUT;
            let input_hash = blake2b_256(input.as_ref());
            if c_solve(
                output.as_mut_ptr(),
                input_hash.as_ptr(),
                1 << MAX_EDGE,
                CYCLE_LENGTH as u32,
            ) > 0
            {
                let mut output_u8 = vec![0u8; CYCLE_LENGTH << 2];
                LittleEndian::write_u32_into(&output, &mut output_u8);
            }

            let cuckoo = Cuckoo::new(MAX_EDGE, CYCLE_LENGTH);
            assert!(cuckoo.verify(&input, &output));
        }
    }
}
