// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_transaction_replay::DiemDebugger;
use diem_types::{account_address::AccountAddress, transaction::Version};
use difference::Changeset;
use move_vm_test_utils::ChangeSet;
use std::path::PathBuf;
use stdlib::build_stdlib;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Path to the local DiemDB file
    #[structopt(long, parse(from_os_str))]
    db: Option<PathBuf>,
    /// Full URL address to connect to - should include port number, if applicable
    #[structopt(short = "u", long)]
    url: Option<String>,
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "replay-transactions")]
    ReplayTransactions { start: Version, limit: u64 },
    #[structopt(name = "replay-recent-transactions")]
    ReplayRecentTransactions { txns: u64 },
    #[structopt(name = "replay-transaction-by-sequence-number")]
    ReplayTransactionBySequence {
        #[structopt(parse(try_from_str))]
        account: AccountAddress,
        seq: u64,
    },
    #[structopt(name = "annotate-account")]
    AnnotateAccount {
        #[structopt(parse(try_from_str))]
        account: AccountAddress,
        version: Version,
    },
    #[structopt(name = "diff-account")]
    DiffAccount {
        #[structopt(parse(try_from_str))]
        account: AccountAddress,
        base_version: Version,
        revision: Version,
    },
    #[structopt(name = "bisect-transaction")]
    BisectTransaction {
        #[structopt(parse(from_os_str))]
        script_path: PathBuf,
        #[structopt(parse(try_from_str))]
        sender: AccountAddress,
        begin: Version,
        end: Version,
        #[structopt(long)]
        rebuild_stdlib: bool,
    },
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let debugger = if let Some(p) = opt.db {
        DiemDebugger::db(p)?
    } else if let Some(url) = opt.url {
        DiemDebugger::json_rpc(url.as_str())?
    } else {
        panic!("No debugger attached")
    };

    println!("Connection Succeeded");

    match opt.cmd {
        Command::ReplayTransactions { start, limit } => {
            println!("{:#?}", debugger.execute_past_transactions(start, limit));
        }
        Command::ReplayRecentTransactions { txns } => {
            let latest_version = debugger
                .get_latest_version()
                .expect("Failed to get latest version");
            assert!(latest_version >= txns);
            println!(
                "{:#?}",
                debugger.execute_past_transactions(latest_version - txns, txns)
            );
        }
        Command::ReplayTransactionBySequence { account, seq } => {
            let version = debugger
                .get_version_by_account_sequence(account, seq)?
                .expect("Version not found");
            println!(
                "Executing transaction at version: {:?}\n{:#?}",
                version,
                debugger.execute_past_transactions(version, 1)
            );
        }
        Command::AnnotateAccount { account, version } => println!(
            "{}",
            debugger
                .annotate_account_state_at_version(account, version)?
                .expect("Account not found")
        ),
        Command::DiffAccount {
            account,
            base_version,
            revision,
        } => {
            let base_annotation = format!(
                "{}",
                debugger
                    .annotate_account_state_at_version(account, base_version)?
                    .expect("Account not found")
            );
            let revision_annotation = format!(
                "{}",
                debugger
                    .annotate_account_state_at_version(account, revision)?
                    .expect("Account not found")
            );
            println!(
                "{}",
                Changeset::new(&base_annotation, &revision_annotation, "\n")
            );
        }
        Command::BisectTransaction {
            sender,
            script_path,
            begin,
            end,
            rebuild_stdlib: reload_stdlib,
        } => println!(
            "{:?}",
            debugger.bisect_transactions_by_script(
                script_path.to_str().expect("Expect an str"),
                sender,
                begin,
                end,
                if reload_stdlib {
                    let mut change_set = ChangeSet::new();
                    for (_, module) in build_stdlib().into_iter() {
                        let mut bytes = vec![];
                        module.serialize(&mut bytes)?;
                        change_set.publish_module(module.self_id(), bytes)?;
                    }
                    Some(change_set)
                } else {
                    None
                },
            )
        ),
    }
    Ok(())
}
