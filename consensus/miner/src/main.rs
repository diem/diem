use async_std::task;
use clap::{App, Arg};
use miner::client::MineClient;

fn main() {
    let matches = App::new("sg-miner")
        .version("v0.1")
        .author("dev@starcoin.org")
        .about("StarGate chain miner client")
        .arg(
            Arg::with_name("address")
                .help("chain server miner rpc address")
                .short("d")
                .default_value("127.0.0.1:4251"),
        )
        .get_matches();
    let address = matches.value_of("address").unwrap();
    println!("Connect to address {}", address);
    let miner = MineClient::new(address.to_string());
    task::block_on(miner.start());
}
