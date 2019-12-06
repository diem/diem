pub mod miner;
pub mod server;
pub mod types;
pub use miner::MineClient;
#[cfg(test)]
mod tests {
    use crate::server;
    #[test]
    fn test_miner_rpc_server() {
        server::run_service();
    }
}
