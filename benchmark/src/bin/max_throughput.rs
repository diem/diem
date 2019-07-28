use benchmark::{
    bin_utils::{
        create_benchmarker_from_opt, linear_search_max_throughput, try_start_metrics_server,
    },
    load_generator::PairwiseTransferTxnGenerator,
    ruben_opt::RubenOpt,
};
use logger::{self, prelude::*};
use structopt::StructOpt;

/// During linear search, both submission rate and number of TXNs are increased.
/// With 32 new accounts and 2 rounds, we add extra 2K TXNs as rate increases.
const NUM_NEW_ACCOUNTS: u64 = 32;
const NUM_REPEATED_ROUNDS: u64 = 2;

/// CLI options for linear search max throughput.
#[derive(Debug, StructOpt)]
#[structopt(
    name = "max_throughput",
    author = "Libra",
    about = "Linear Search Maximum Throughput."
)]
pub struct SearchOpt {
    // Some arguments from RubenOpt will be set to linear search's default values, including
    // num_accounts, num_rounds, num_clients, txn_pattern.
    #[structopt(flatten)]
    pub ruben_opt: RubenOpt,
    /// Upper bound value of submission rate for each client.
    #[structopt(short = "u", long = "upper_bound", default_value = "8000")]
    pub upper_bound: u64,
    /// Lower bound value of submission rate for each client.
    #[structopt(short = "l", long = "lower_bound", default_value = "10")]
    pub lower_bound: u64,
    /// Increase step of submission rate for each client.
    #[structopt(short = "i", long = "inc_step", default_value = "10")]
    pub inc_step: u64,
    /// How many times to repeat the same linear search. Each time with new accounts/TXNs.
    #[structopt(short = "b", long = "num_searches", default_value = "10")]
    pub num_searches: u64,
}

impl SearchOpt {
    pub fn new_from_args() -> Self {
        let mut args = SearchOpt::from_args();
        args.ruben_opt.try_parse_validator_addresses();
        args.ruben_opt.num_clients = args.ruben_opt.validator_addresses.len() * num_cpus::get();
        info!(
            "Search max throughput with {} clients",
            args.ruben_opt.num_clients
        );
        assert!(args.lower_bound > 0);
        assert!(args.inc_step > 0);
        assert!(args.lower_bound < args.upper_bound);
        args
    }
}

fn main() {
    let _g = logger::set_default_global_logger(false, Some(256));
    let args = SearchOpt::new_from_args();
    info!("Parsed and adjusted arguments: {:#?}", args);
    try_start_metrics_server(&args.ruben_opt);
    let mut bm = create_benchmarker_from_opt(&args.ruben_opt);
    let mut faucet_account = bm.load_faucet_account(&args.ruben_opt.faucet_key_file_path);
    let mut generator = PairwiseTransferTxnGenerator::new();
    for _ in 0..args.num_searches {
        linear_search_max_throughput(
            &mut bm,
            &mut generator,
            &mut faucet_account,
            args.lower_bound,
            args.upper_bound,
            args.inc_step,
            NUM_NEW_ACCOUNTS,
            NUM_REPEATED_ROUNDS,
            args.ruben_opt.num_epochs,
        );
    }
}
