use std::{string::String, sync::Arc};

use futures::Future;
use grpcio::{self, Environment, RpcContext, ServerBuilder, UnarySink};
use proto::miner::{
    create_miner_proxy, MineCtxRequest, MineCtxResponse, MinedBlockRequest, MinedBlockResponse,
    MinerProxy,
};

use crate::types::MineState;

#[derive(Clone)]
pub struct MinerProxyServer<S>
where
    S: MineState + Clone + Send + Clone + 'static,
{
    miner_proxy_inner: Arc<MinerProxyServerInner<S>>,
}

struct MinerProxyServerInner<S>
where
    S: MineState + Clone + Send + Clone + 'static,
{
    state: S,
}

impl<S: MineState + Clone + Send + Clone + 'static> MinerProxy for MinerProxyServer<S> {
    fn get_mine_ctx(
        &mut self,
        ctx: RpcContext,
        req: MineCtxRequest,
        sink: UnarySink<MineCtxResponse>,
    ) {
        let mine_ctx_rpc = if let Some(mine_ctx) = self
            .miner_proxy_inner
            .state
            .get_current_mine_ctx(req.algo.into())
        {
            Some(mine_ctx.into())
        } else {
            None
        };
        let resp = MineCtxResponse {
            mine_ctx: mine_ctx_rpc,
        };
        let fut = sink
            .success(resp)
            .map_err(|e| eprintln!("Failed to response to get_mine_ctx {}", e));
        ctx.spawn(fut);
    }

    fn mined(
        &mut self,
        ctx: RpcContext,
        req: MinedBlockRequest,
        sink: UnarySink<MinedBlockResponse>,
    ) {
        let mut accept = false;
        if let Some(ctx) = req.mine_ctx {
            accept = self.miner_proxy_inner.state.mine_accept(
                &ctx.into(),
                req.solution.into(),
                req.nonce,
            );
        }

        let resp = MinedBlockResponse { accept };
        let fut = sink
            .success(resp)
            .map_err(|e| eprintln!("Failed to response to mined {}", e));
        ctx.spawn(fut);
    }
}

pub fn setup_minerproxy_service<S>(mine_state: S, addr: String) -> grpcio::Server
where
    S: MineState + Clone + Send + Sync + 'static,
{
    let env = Arc::new(Environment::new(1));
    let miner_proxy_srv = MinerProxyServer {
        miner_proxy_inner: Arc::new(MinerProxyServerInner { state: mine_state }),
    };
    let service = create_miner_proxy(miner_proxy_srv);
    let addr: Vec<_> = addr.split(":").collect();
    assert_eq!(addr.len(), 2);

    let server = ServerBuilder::new(env)
        .register_service(service)
        .bind(addr[0], addr[1].parse().unwrap())
        .build()
        .unwrap();
    server
}
