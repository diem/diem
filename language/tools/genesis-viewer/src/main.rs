// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use compiled_stdlib::StdLibOptions;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    contract_event::ContractEvent,
    transaction::ChangeSet,
    write_set::{WriteOp, WriteSet},
};
use resource_viewer::{MoveValueAnnotator, NullStateView};
use std::collections::{BTreeMap, BTreeSet};
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
    #[structopt(short = "c", long, help = "Print the resources under each account")]
    print_account_states: bool,
}

pub fn main() {
    // this is a hacky way to find out if no optional args were provided.
    // We get the arg count and when processing the options if the value is 3
    // (program_name, `-f`, file_name) then no args were provided and
    // we default to `all`
    let arg_count = std::env::args().len();
    let args = Args::from_args();
    let ws = vm_genesis::generate_genesis_change_set_for_testing(StdLibOptions::Compiled);
    if args.all || arg_count == 3 {
        print_all(&ws);
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
        if args.print_account_states {
            print_account_states(ws.write_set());
        }
    }
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
    let view = NullStateView::default();
    let annotator = MoveValueAnnotator::new(&view);

    for event in events {
        println!("+ {:?}", event.key());
        println!("Type: {:?}", event.type_tag());
        println!("Seq. Num.: {}", event.sequence_number());
        match annotator.view_contract_event(event) {
            Ok(v) => println!("Data: {}", v),
            Err(e) => println!("Unable to parse event: {:?}", e),
        }
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
    let view = NullStateView::default();
    let annotator = MoveValueAnnotator::new(&view);
    for (k, v) in &resources {
        println!("AccessPath: {:?}", k);
        match annotator.view_access_path(k.clone(), v.as_ref()) {
            Ok(v) => println!("Data: {}", v),
            Err(e) => println!("Unable To parse blobs: {:?}", e),
        };
        println!("RawData: {:?}", v);
    }
}

fn print_account_states(ws: &WriteSet) {
    let mut accounts: BTreeMap<AccountAddress, Vec<(AccessPath, Vec<u8>)>> = BTreeMap::new();
    for (k, v) in ws {
        match v {
            WriteOp::Deletion => panic!("found WriteOp::Deletion in WriteSet"),
            WriteOp::Value(blob) => {
                let tag = k.path.get(0).expect("empty blob in WriteSet");
                if *tag == 1 {
                    accounts
                        .entry(k.address)
                        .or_insert_with(Vec::new)
                        .push((k.clone(), blob.clone()));
                }
            }
        }
    }
    let view = NullStateView::default();
    let annotator = MoveValueAnnotator::new(&view);
    for (k, v) in &accounts {
        println!("Address: {}", k);
        for (ap, resource) in v {
            println!(
                "\tData: {}",
                annotator
                    .view_access_path(ap.clone(), resource.as_ref())
                    .unwrap()
            )
        }
    }
}
