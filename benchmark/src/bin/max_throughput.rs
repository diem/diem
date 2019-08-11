use benchmark::{
    bin_utils::{
        create_benchmarker_from_opt, linear_search_max_throughput, try_start_metrics_server,
    },
    cli_opt::SearchOpt,
    load_generator::PairwiseTransferTxnGenerator,
};
use logger::{self, prelude::*};

/// During linear search, both submission rate and number of TXNs are increased.
/// With 32 new accounts and 2 rounds, we add extra 2K TXNs as rate increases.
const NUM_NEW_ACCOUNTS: u64 = 32;
const NUM_REPEATED_ROUNDS: u64 = 2;

fn main() {
    let _g = logger::set_default_global_logger(false, Some(256));
    let args = SearchOpt::new_from_args();
    info!("Parsed and adjusted arguments: {:#?}", args);
    try_start_metrics_server(&args.bench_opt);
    let mut bm = create_benchmarker_from_opt(&args.bench_opt);
    let mut faucet_account = bm.load_faucet_account(&args.bench_opt.faucet_key_file_path);
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
            args.num_epochs,
        );
    }
}
