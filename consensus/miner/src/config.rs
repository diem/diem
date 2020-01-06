use crate::types::Algo;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MinerConfig {
    pub miner_server_addr: String,
    // miner server rpc listen address
    pub algorithm: Algo,
    pub gpu: bool,
    pub nthread: u32,
    pub device: u32,
}

impl Default for MinerConfig {
    fn default() -> Self {
        MinerConfig {
            miner_server_addr: "127.0.0.1:4251".to_string(),
            algorithm: Algo::SCRYPT,
            gpu: false,
            nthread: 1,
            device: 0,
        }
    }
}
