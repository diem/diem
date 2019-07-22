use proto_conv::IntoProtoBytes;
use serde_json;
use std::{
    fs::{self, File},
    io::prelude::*,
    time::Duration,
};
use structopt::StructOpt;
use types::{
    account_address::AccountAddress,
    transaction::{parse_as_transaction_argument, Program, RawTransaction, TransactionArgument},
};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Libra Transaction Builder",
    author = "The Libra Association",
    about = "CLI to build compiled result into transactions that can be emitted to the blockchain"
)]
struct Args {
    pub sender: AccountAddress,
    pub sequence_number: u64,
    #[structopt(
        help = "Path to the program file emitted by the compiler\nExample: Run ./compiler -o out peer_to_peer_transfer.mvir first and pass in the path to `out`."
    )]
    pub program: String,
    #[structopt(help = "Path to the output RawTransaction file")]
    pub output: String,

    #[structopt(long, default_value = "1000000")]
    pub max_gas_amount: u64,
    #[structopt(long, default_value = "0")]
    pub gas_unit_price: u64,
    #[structopt(long, parse(try_from_str = "parse_as_transaction_argument"))]
    pub args: Vec<TransactionArgument>,
}

fn main() {
    let args = Args::from_args();

    let program_bytes = fs::read(args.program).expect("Unable to read file");
    let program: Program = serde_json::from_slice(&program_bytes)
        .expect("Unable to deserialize program, is it the output of the compiler?");
    let (script, _, modules) = program.into_inner();
    let program_with_args = Program::new(script, modules, args.args);
    let transaction = RawTransaction::new(
        args.sender,
        args.sequence_number,
        program_with_args,
        args.max_gas_amount,
        args.gas_unit_price,
        Duration::new(u64::max_value(), 0),
    )
    .into_proto_bytes()
    .expect("Can't serialize transaction into raw bytes");
    let mut file = File::create(&args.output).expect("Can't create file");
    file.write_all(&transaction)
        .expect("Can't write transaction to file");
}
