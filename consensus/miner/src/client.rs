use grpcio;

use std::env;
use std::sync::Arc;

use grpcio::{ChannelBuilder, EnvBuilder};
use proto::miner::{MineCtxRequest, MinerProxyClient};

fn main() {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect("127.0.0.1:4251");
    let client = MinerProxyClient::new(ch);

    let req = MineCtxRequest {};
    let mine_ctx = client.get_mine_ctx(&req);
    //client.Mined(MinedBlockRequest)
    println!("proof {:?}", mine_ctx);
}
