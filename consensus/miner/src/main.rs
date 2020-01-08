use async_std::task;
use clap::{App, Arg};
use miner::client::MineClient;
use miner::config::MinerConfig;
use miner::types::Algo;

fn main() {
    let matches = App::new("sg-miner")
        .version("v0.1")
        .author("dev@starcoin.org")
        .about("StarGate chain miner client")
        .arg(
            Arg::with_name("address")
                .help("chain server miner rpc address")
                .short("a")
                .default_value("127.0.0.1:4251"),
        )
        .arg(
            Arg::with_name("scrypt")
                .help("mine block by algorithm scrypt")
                .long("scrypt")
                .conflicts_with_all(&vec!["cuckoo_gpu", "cuckoo"])
                .short("s"),
        )
        .arg(
            Arg::with_name("cuckoo")
                .help("mine block by cuckoo with cpu")
                .long("cuckoo")
                .conflicts_with_all(&vec!["cuckoo_gpu"])
                .short("c"),
        )
        .arg(
            Arg::with_name("cuckoo_gpu")
                .help("mine block by cuckoo with gpu")
                .long("cuckoo_gpu")
                .short("g"),
        )
        .arg(
            Arg::with_name("thread")
                .help("thread number")
                .short("n")
                .default_value("1"),
        )
        .arg(
            Arg::with_name("device")
                .help("gpu device number")
                .short("d")
                .default_value("0")
                .alias("cuckoo_gpu"),
        )
        .get_matches();
    let address = matches.value_of("address").unwrap();
    let (algo, gpu_anable) = {
        if matches.is_present("scrypt") {
            (Algo::SCRYPT, false)
        } else if matches.is_present("cuckoo") {
            (Algo::CUCKOO, false)
        } else if matches.is_present("cuckoo_gpu") {
            (Algo::CUCKOO, true)
        } else {
            (Algo::SCRYPT, false)
        }
    };

    let mut cfg = MinerConfig::default();
    cfg.miner_server_addr = address.to_string();
    cfg.algorithm = algo;
    cfg.gpu = gpu_anable;
    cfg.device = matches.value_of("device").unwrap().parse().unwrap();
    cfg.nthread = matches.value_of("thread").unwrap().parse().unwrap();
    println!("mine cfg:{:?}", &cfg);
    let miner = MineClient::new(cfg);
    task::block_on(miner.start());
}
