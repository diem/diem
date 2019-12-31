use crate::miner;
use crate::types::{Algo, MineCtx, U256};
use async_std::{
    prelude::*,
    stream::Stream,
    task,
    task::{Context, Poll},
};
use grpcio::{self, ChannelBuilder, EnvBuilder};
use proto::miner::{MineCtxRequest, MinedBlockRequest, MinerProxyClient};
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
                task::sleep(Duration::from_secs(1)).await;
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
        let algo = Algo::SCRYPT;
        match self
            .client
            .get_mine_ctx(&MineCtxRequest { algo: algo.into() })
        {
            Ok(resp) => {
                if let Some(mine_ctx_rpc) = resp.mine_ctx {
                    Poll::Ready(Some(mine_ctx_rpc.into()))
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
            let target: U256 = ctx.target.clone().unwrap().into();
            let (nonce, solution) = miner::solve(&ctx.header, &ctx.algo.clone().unwrap(), &target);
            let req = MinedBlockRequest {
                mine_ctx: Some(ctx.into()),
                nonce,
                solution: solution.into(),
            };
            let _resp = self.rpc_client.mined(&req);
        }
    }
}
