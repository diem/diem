use async_std::task;
use futures::Future;
use futures03::{channel::oneshot, compat::Future01CompatExt};
use grpcio::{self, Environment, RpcContext, Server, ServerBuilder, UnarySink};
use proto::miner::{
    create_miner_proxy, MineCtxRequest, MineCtxResponse, MinedBlockRequest, MinedBlockResponse,
    MinerProxy,
};
use std::{
    io::{self, Read},
    sync::Arc,
};

#[derive(Clone)]
pub struct MinerProxyServer<S>
where
    S: MineState + Clone + Send + Clone + 'static,
{
    miner_proxy_inner: Arc<MinerProxyServerInner<S>>,
}

pub struct MinerProxyServerInner<S>
where
    S: MineState + Clone + Send + Clone + 'static,
{
    state: S,
}

pub trait MineState: Send + Sync {
    fn get_current_mine_ctx(&self) -> MineCtx;
}

#[derive(Clone)]
struct DummyMineState;

impl MineState for DummyMineState {
    fn get_current_mine_ctx(&self) -> MineCtx {
        return MineCtx {
            nonce: 0,
            header: vec![0],
        };
    }
}

pub struct MineCtx {
    nonce: u64,
    header: Vec<u8>,
}

impl<S: MineState + Clone + Send + Clone + 'static> MinerProxy for MinerProxyServer<S> {
    fn get_mine_ctx(
        &mut self,
        ctx: RpcContext,
        req: MineCtxRequest,
        sink: UnarySink<MineCtxResponse>,
    ) {
        let mine_ctx = self.miner_proxy_inner.state.get_current_mine_ctx();
        let resp = MineCtxResponse {
            nonce: mine_ctx.nonce,
            header: mine_ctx.header,
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
        /*
        let mined_block = MinedBlockInfo = MinedBlockRequest.into();
        let accept = self.miner_proxy_inner.state.verify(mined_block);
        let resp = MinedBlockResponse { accept };
        */
        unimplemented!()
    }
}

pub fn setup_minerproxy_service<S>(mine_state: S) -> grpcio::Server
where
    S: MineState + Clone + Send + Sync + 'static,
{
    let env = Arc::new(Environment::new(1));
    let miner_proxy_srv = MinerProxyServer {
        miner_proxy_inner: Arc::new(MinerProxyServerInner { state: mine_state }),
    };
    let service = create_miner_proxy(miner_proxy_srv);
    let server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 4251)
        .build()
        .unwrap();
    server
}

pub fn run_service() {
    let mut grpc_srv = setup_minerproxy_service(DummyMineState);
    grpc_srv.start();
    for &(ref host, port) in grpc_srv.bind_addrs() {
        println!("listening on {}:{}", host, port);
    }
    let (tx, rx) = oneshot::channel();

    task::spawn(async {
        println!("Press enter to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        let _ = tx.send(());
    });

    task::block_on(async move {
        rx.await.unwrap();
        grpc_srv.shutdown().compat().await.unwrap();
    });
}

fn main() {
    run_service();
}
