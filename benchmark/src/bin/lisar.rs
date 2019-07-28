use benchmark::{
    bin_utils::{
        create_benchmarker_from_opt, linear_search_max_throughput, try_start_metrics_server,
    },
    ruben_opt::{RuBenOpt, TransactionPattern},
    txn_generator::{LoadGenerator, PairwiseTransferTxnGenerator, RingTransferTxnGenerator},
};
use logger::{self, prelude::*};
use std::ops::DerefMut;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "LiSar",
    author = "Libra",
    about = "LiSar (Li)near (S)e(ar)ch Maximum Throughput."
)]
pub struct LiSarOpt {
    #[structopt(flatten)]
    pub ruben_opt: RuBenOpt,
    /// Upper bound value of submission rate for each client.
    #[structopt(short = "u", long = "upper_bound", default_value = "100")]
    pub upper_bound: u64,
    /// Upper bound value of submission rate for each client.
    #[structopt(short = "l", long = "lower_bound", default_value = "10")]
    pub lower_bound: u64,
    /// Increase step of submission rate for each client.
    #[structopt(short = "i", long = "inc_step", default_value = "10")]
    pub inc_step: u64,
    /// How many times to repeat the same linear search. Each time with new accounts/TXNs.
    #[structopt(short = "b", long = "num_searches", default_value = "10")]
    pub num_searches: u64,
}

impl LiSarOpt {
    pub fn new_from_args() -> Self {
        let mut args = LiSarOpt::from_args();
        args.ruben_opt.try_parse_validator_addresses();
        if args.ruben_opt.num_clients == 0 {
            args.ruben_opt.num_clients = args.ruben_opt.validator_addresses.len();
        }
        assert!(args.lower_bound > 0);
        assert!(args.inc_step > 0);
        assert!(args.lower_bound < args.upper_bound);
        args
    }
}

fn main() {
    let _g = logger::set_default_global_logger(false, Some(256));
    info!("LiSar: the utility to (Li)near (S)e(ar)ch maximum throughput");
    let args = LiSarOpt::new_from_args();
    info!("Parsed arguments: {:#?}", args);

    try_start_metrics_server(&args.ruben_opt);
    let mut bm = create_benchmarker_from_opt(&args.ruben_opt);
    let mut faucet_account = bm.load_faucet_account(&args.ruben_opt.faucet_key_file_path);
    let mut generator: Box<dyn LoadGenerator> = match args.ruben_opt.txn_pattern {
        TransactionPattern::Ring => Box::new(RingTransferTxnGenerator::new()),
        TransactionPattern::Pairwise => Box::new(PairwiseTransferTxnGenerator::new()),
    };
    for _ in 0..args.num_searches {
        linear_search_max_throughput(
            &mut bm,
            generator.deref_mut(),
            &mut faucet_account,
            args.lower_bound,
            args.upper_bound,
            args.inc_step,
            args.ruben_opt.num_accounts,
            args.ruben_opt.num_rounds,
            args.ruben_opt.num_epochs,
        );
    }
}
