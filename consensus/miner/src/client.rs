use crate::config::MinerConfig;
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
    algo: Algo,
}

impl MineCtxStream {
    fn new(client: MinerProxyClient, algo: Algo) -> Self {
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
        MineCtxStream {
            client,
            waker,
            algo,
        }
    }
}

impl Stream for MineCtxStream {
    type Item = MineCtx;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut waker = self.waker.lock().unwrap();
        let algo = self.algo.clone();
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
    cfg: MinerConfig,
}

impl Default for MineClient {
    fn default() -> Self {
        MineClient::new(MinerConfig::default())
    }
}

impl MineClient {
    pub fn new(cfg: MinerConfig) -> Self {
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(&cfg.miner_server_addr);
        let rpc_client = MinerProxyClient::new(ch);
        MineClient { rpc_client, cfg }
    }

    pub async fn start(&self) {
        let mut ctx_stream =
            MineCtxStream::new(self.rpc_client.clone(), self.cfg.algorithm.clone());
        while let Some(ctx) = ctx_stream.next().await {
            let target: U256 = ctx.target.clone().unwrap().into();
            let (nonce, solution) = miner::solve(&ctx.header, &target, &self.cfg);
            let req = MinedBlockRequest {
                mine_ctx: Some(ctx.into()),
                nonce,
                solution: solution.into(),
            };
            let _resp = self.rpc_client.mined(&req);
        }
    }
}
