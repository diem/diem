// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use crate::{experimental, sandbox::utils::mode::Mode, Move};
use anyhow::Result;
use move_core_types::{
    language_storage::TypeTag, parser, transaction_argument::TransactionArgument,
};

use structopt::{clap::arg_enum, StructOpt};

#[derive(StructOpt)]
pub enum ExperimentalCommand {
    /// Perform a read/write set analysis and print the results for
    /// `module_file`::`script_name`.
    #[structopt(name = "read-write-set")]
    ReadWriteSet {
        /// Path to .mv file containing module bytecode.
        #[structopt(name = "module", parse(from_os_str))]
        module_file: PathBuf,
        /// A function inside `module_file`.
        #[structopt(name = "function")]
        fun_name: String,
        #[structopt(long = "signers")]
        signers: Vec<String>,
        #[structopt(long = "args", parse(try_from_str = parser::parse_transaction_argument))]
        args: Vec<TransactionArgument>,
        #[structopt(long = "type-args", parse(try_from_str = parser::parse_type_tag))]
        type_args: Vec<TypeTag>,
        #[structopt(long = "concretize", possible_values = &ConcretizeMode::variants(), case_insensitive = true, default_value = "dont")]
        concretize: ConcretizeMode,
    },
}

// Specify if/how the analysis should concretize and filter the static analysis summary
arg_enum! {
    // Specify if/how the analysis should concretize and filter the static analysis summary
    #[derive(Debug, Clone, Copy)]
    pub enum ConcretizeMode {
        // Show the full concretized access paths read or written (e.g. 0xA/0x1::M::S/f/g)
        Paths,
        // Show only the concrete resource keys that are read (e.g. 0xA/0x1::M::S)
        Reads,
        // Show only the concrete resource keys that are written (e.g. 0xA/0x1::M::S)
        Writes,
        // Do not concretize; show the results from the static analysis
        Dont,
    }
}

impl ExperimentalCommand {
    pub fn handle_command(&self, move_args: &Move, mode: &Mode) -> Result<()> {
        match self {
            ExperimentalCommand::ReadWriteSet {
                module_file,
                fun_name,
                signers,
                args,
                type_args,
                concretize,
            } => {
                let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
                experimental::commands::analyze_read_write_set(
                    &state,
                    module_file,
                    fun_name,
                    signers,
                    args,
                    type_args,
                    *concretize,
                    move_args.verbose,
                )
            }
        }
    }
}
