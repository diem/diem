// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{
    access_path::AccessPath,
    contract_event::ContractEvent,
    transaction::{ChangeSet, Transaction, TransactionPayload},
    write_set::{WriteOp, WriteSet},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::prelude::*,
    path::PathBuf,
};
use structopt::StructOpt;
use vm::CompiledModule;

#[derive(Debug, StructOpt)]
#[structopt(name = "Genesis Viewer")]
/// Tool to display the content of a genesis blob (genesis transaction).
///
/// Takes a genesis blob transaction that can be produced by the genesis generator tool.
/// It prints the genesis data in different forms that can be used to review a genesis blob
/// or to compare different genesis transactions.
///
/// The default or `--all` flag produces a somewhat pretty printing in a sorted order which
/// can be used to review or compare with a common `diff` command different genesis transactions.
/// However, just performing a diff with keys (sorted and not) can give a good sense of what
/// are the changes in different genesis transactions.
///
/// Printing can be done by the different types of data in a genesis blob. As far as this
/// tool is concerned the different data types are: Modules, Resources and Events.
pub struct Args {
    #[structopt(short = "f", long, parse(from_os_str))]
    genesis_file: PathBuf,
    #[structopt(
        short = "a",
        long,
        help = "[default] Print everything in the most verbose form and sorted, ignore all other options"
    )]
    all: bool,
    #[structopt(long, help = "Print raw events")]
    event_raw: bool,
    #[structopt(long, help = "Print events key and events type")]
    event_keys: bool,
    #[structopt(short = "e", long, help = "Print events only (pretty format)")]
    type_events: bool,
    #[structopt(short = "m", long, help = "Print modules only (pretty format)")]
    type_modules: bool,
    #[structopt(short = "r", long, help = "Print resources only (pretty format)")]
    type_resources: bool,
    #[structopt(short = "w", long, help = "Print raw WriteSet only (no events)")]
    write_set: bool,
    #[structopt(
        short = "t",
        long,
        help = "Print WriteSet by type (Module/Resource, pretty format)"
    )]
    ws_by_type: bool,
    #[structopt(
        short = "k",
        long,
        help = "Print WriteSet keys in the order defined in the binary blob"
    )]
    ws_keys: bool,
    #[structopt(short = "s", long, help = "Print WriteSet keys sorted")]
    ws_sorted_keys: bool,
}

pub fn main() {
    // this is a hacky way to find out if no optional args were provided.
    // We get the arg count and when processing the options if the value is 3
    // (program_name, `-f`, file_name) then no args were provided and
    // we default to `all`
    let arg_count = std::env::args().len();
    let args = Args::from_args();
    let mut f = File::open(args.genesis_file.clone())
        .unwrap_or_else(|_| panic!("Cannot open file {:?}", args.genesis_file));
    let mut bytes = vec![];
    f.read_to_end(&mut bytes)
        .expect("genesis file cannot be read");
    let txn =
        lcs::from_bytes(&bytes).expect("genesis blob did not deserialize correctly (lcs error)");
    if let Transaction::UserTransaction(txn) = txn {
        if let TransactionPayload::WriteSet(ws) = txn.payload() {
            if args.all || arg_count == 3 {
                print_all(ws);
            } else {
                if args.event_raw {
                    print_events_raw(ws.events());
                }
                if args.event_keys {
                    print_events_key(ws.events());
                }
                if args.type_events {
                    print_events(ws.events());
                }
                if args.type_modules {
                    print_modules(ws.write_set());
                }
                if args.type_resources {
                    print_resources(ws.write_set());
                }
                if args.write_set {
                    print_write_set_raw(ws.write_set());
                }
                if args.ws_by_type {
                    print_write_set_by_type(ws.write_set());
                }
                if args.ws_keys {
                    print_keys(ws.write_set());
                }
                if args.ws_sorted_keys {
                    print_keys_sorted(ws.write_set());
                }
            }
            return;
        }
    }
    panic!("error parsing genesis transaction, WriteSet not found");
}

fn print_all(cs: &ChangeSet) {
    print_write_set_by_type(cs.write_set());
    println!("* Events:");
    print_events(cs.events());
}

fn print_events_raw(events: &[ContractEvent]) {
    println!("{:#?}", events);
}

fn print_write_set_raw(ws: &WriteSet) {
    println!("{:#?}", ws);
}

fn print_keys(ws: &WriteSet) {
    for (key, _) in ws {
        println!("{:?}", key);
    }
}

fn print_keys_sorted(ws: &WriteSet) {
    let mut sorted_ws: BTreeSet<AccessPath> = BTreeSet::new();
    for (key, _) in ws {
        sorted_ws.insert(key.clone());
    }
    for key in sorted_ws {
        println!("{:?}", key);
    }
}

fn print_events_key(events: &[ContractEvent]) {
    for event in events {
        println!("+ {:?} ->\n\tType: {:?}", event.key(), event.type_tag());
    }
}

fn print_write_set_by_type(ws: &WriteSet) {
    println!("* Modules:");
    print_modules(ws);
    println!("* Resources:");
    print_resources(ws);
}

fn print_events(events: &[ContractEvent]) {
    for event in events {
        println!("+ {:?}", event.key());
        println!("\tType: {:?}", event.type_tag());
        println!("\tSeq. Num.: {}", event.sequence_number());
        println!("\tData: {:?}", event.event_data());
    }
}

fn print_modules(ws: &WriteSet) {
    let mut modules: BTreeMap<AccessPath, CompiledModule> = BTreeMap::new();
    for (k, v) in ws {
        match v {
            WriteOp::Deletion => panic!("found WriteOp::Deletion in WriteSet"),
            WriteOp::Value(blob) => {
                let tag = k.path.get(0).expect("empty blob in WriteSet");
                if *tag == 0 {
                    modules.insert(
                        k.clone(),
                        CompiledModule::deserialize(blob).expect("CompiledModule must deserialize"),
                    );
                }
            }
        }
    }
    for (k, v) in &modules {
        println!("+ Type: {:?}", k);
        println!("CompiledModule: {:#?}", v);
    }
}

fn print_resources(ws: &WriteSet) {
    let mut resources: BTreeMap<AccessPath, Vec<u8>> = BTreeMap::new();
    for (k, v) in ws {
        match v {
            WriteOp::Deletion => panic!("found WriteOp::Deletion in WriteSet"),
            WriteOp::Value(blob) => {
                let tag = k.path.get(0).expect("empty blob in WriteSet");
                if *tag == 1 {
                    resources.insert(k.clone(), blob.clone());
                }
            }
        }
    }
    for (k, v) in &resources {
        println!("+ Key: {:?}", k);
        println!("Data: {:?}", v);
    }
}
