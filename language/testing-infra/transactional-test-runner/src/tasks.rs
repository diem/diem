// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::*;
use move_command_line_common::files::{MOVE_EXTENSION, MOVE_IR_EXTENSION};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    parser,
    transaction_argument::TransactionArgument,
};
use move_lang::shared::AddressBytes;
use std::{fmt::Debug, path::Path, str::FromStr};
use structopt::*;
use tempfile::NamedTempFile;

#[derive(Debug)]
pub struct TaskInput<Command> {
    pub command: Command,
    pub name: String,
    pub number: usize,
    pub start_line: usize,
    pub command_lines_stop: usize,
    pub stop_line: usize,
    pub data: Option<NamedTempFile>,
}

pub fn taskify<Command: Debug + StructOpt>(filename: &Path) -> Result<Vec<TaskInput<Command>>> {
    use regex::Regex;
    use std::{
        fs::File,
        io::{self, BufRead, Write},
    };
    #[allow(non_snake_case)]
    let WHITESPACE = Regex::new(r"^\s*$").unwrap();
    #[allow(non_snake_case)]
    let COMMAND_TEXT = Regex::new(r"^\s*//#\s*(.*)\s*$").unwrap();

    let file = File::open(filename).unwrap();
    let lines: Vec<String> = io::BufReader::new(file)
        .lines()
        .map(|ln| ln.expect("Could not parse line"))
        .collect();

    let lines_iter = lines.into_iter().enumerate().map(|(idx, l)| (idx + 1, l));
    let skipped_whitespace =
        lines_iter.skip_while(|(_line_number, line)| WHITESPACE.is_match(line));
    let mut bucketed_lines = vec![];
    let mut cur_commands = vec![];
    let mut cur_text = vec![];
    let mut in_command = true;
    for (line_number, line) in skipped_whitespace {
        if let Some(captures) = COMMAND_TEXT.captures(&line) {
            if !in_command {
                bucketed_lines.push((cur_commands, cur_text));
                cur_commands = vec![];
                cur_text = vec![];
                in_command = true;
            }
            let command_text = match captures.len() {
                1 => continue,
                2 => captures.get(1).unwrap().as_str().to_string(),
                n => panic!("COMMAND_TEXT captured {}. expected 1 or 2", n),
            };
            if command_text.is_empty() {
                continue;
            }
            cur_commands.push((line_number, command_text))
        } else if WHITESPACE.is_match(&line) {
            in_command = false;
            continue;
        } else {
            in_command = false;
            cur_text.push((line_number, line))
        }
    }
    bucketed_lines.push((cur_commands, cur_text));

    if bucketed_lines.is_empty() {
        return Ok(vec![]);
    }

    let mut tasks = vec![];
    for (number, (commands, text)) in bucketed_lines.into_iter().enumerate() {
        if commands.is_empty() {
            assert!(number == 0);
            bail!("No initial command")
        }

        let start_line = commands.first().unwrap().0;
        let command_lines_stop = commands.last().unwrap().0;
        let mut command_text = "task ".to_string();
        for (line_number, text) in commands {
            assert!(!text.is_empty(), "{}: {}", line_number, text);
            command_text = format!("{} {}", command_text, text);
        }
        let command_split = command_text.split_ascii_whitespace().collect::<Vec<_>>();
        let name_opt = command_split.get(1).map(|s| (*s).to_owned());
        let command = match Command::from_iter_safe(command_split) {
            Ok(command) => command,
            Err(_e) => {
                let mut spit_iter = command_text.split_ascii_whitespace();
                // skip 'task'
                spit_iter.next();
                let help_command = match spit_iter.next() {
                    None => vec!["task", "--help"],
                    Some(c) => vec!["task", c, "--help"],
                };
                let help = match Command::from_iter_safe(help_command) {
                    Ok(_) => panic!(),
                    Err(e) => e,
                };
                bail!(
                    "Invalid command. Lines {} - {}.\n{}",
                    start_line,
                    command_lines_stop,
                    help
                )
            }
        };
        let name = name_opt.unwrap();

        let stop_line = if text.is_empty() {
            command_lines_stop
        } else {
            text[text.len() - 1].0
        };

        // Keep fucking this up somehow
        // let last_non_whitespace = text
        //     .iter()
        //     .rposition(|(_, l)| !WHITESPACE.is_match(l))
        //     .unwrap_or(0);
        // let initial_text = text
        //     .into_iter()
        //     .take_while(|(i, _)| *i < last_non_whitespace)
        //     .map(|(_, l)| l);
        let file_text_vec = (0..command_lines_stop)
            .map(|_| String::new())
            .chain(text.into_iter().map(|(_ln, l)| l))
            .collect::<Vec<String>>();
        let data = if file_text_vec.iter().all(|s| WHITESPACE.is_match(s)) {
            None
        } else {
            let data = NamedTempFile::new()?;
            data.reopen()?
                .write_all(file_text_vec.join("\n").as_bytes())?;
            Some(data)
        };

        tasks.push(TaskInput {
            command,
            name,
            number,
            start_line,
            command_lines_stop,
            stop_line,
            data,
        })
    }
    Ok(tasks)
}

impl<T> TaskInput<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> TaskInput<U> {
        let Self {
            command,
            name,
            number,
            start_line,
            command_lines_stop,
            stop_line,
            data,
        } = self;
        TaskInput {
            command: f(command),
            name,
            number,
            start_line,
            command_lines_stop,
            stop_line,
            data,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxChoice {
    Source,
    IR,
}

#[derive(Debug, StructOpt)]
pub struct InitCommand {
    #[structopt(
        long = "addresses",
        parse(try_from_str = move_lang::shared::parse_named_address)
    )]
    pub named_addresses: Vec<(String, AddressBytes)>,
}

#[derive(Debug, StructOpt)]
pub struct PublishCommand {
    #[structopt(long = "gas-budget")]
    pub gas_budget: Option<u64>,
    #[structopt(long = "syntax")]
    pub syntax: Option<SyntaxChoice>,
    #[structopt(long = "address", parse(try_from_str = parse_account_address))]
    pub address: Option<AccountAddress>,
}

#[derive(Debug, StructOpt)]
pub struct RunCommand {
    #[structopt(long = "signers", parse(try_from_str = parse_account_address))]
    pub signers: Vec<AccountAddress>,
    #[structopt(long = "args", parse(try_from_str = parser::parse_transaction_argument))]
    pub args: Vec<TransactionArgument>,
    #[structopt(long = "type-args", parse(try_from_str = parser::parse_type_tag))]
    pub type_args: Vec<TypeTag>,
    #[structopt(long = "gas-budget")]
    pub gas_budget: Option<u64>,
    #[structopt(long = "syntax")]
    pub syntax: Option<SyntaxChoice>,
    #[structopt(name = "NAME", parse(try_from_str = parse_qualified_module_access))]
    pub name: Option<(ModuleId, Identifier)>,
}

#[derive(Debug, StructOpt)]
pub struct ViewCommand {
    #[structopt(long = "address", parse(try_from_str = parse_account_address))]
    pub address: AccountAddress,
    #[structopt(long = "resource", parse(try_from_str = parse_qualified_module_access_with_type_args))]
    pub resource: (ModuleId, Identifier, Vec<TypeTag>),
}

#[macro_export]
macro_rules! define_commands {
    ($command:ident
        $(, init: $exinit:ident)?
        $(, publish: $expub:ident)?
        $(, run: $exrun:ident)?
        $(, subcommands: $subcommand:ident)?
    ) => {
        #[allow(unused_imports)]
        use structopt::*;
        #[derive(Debug, StructOpt)]
        #[structopt(
            name = "task",
            about = "Transactional task commands",
            rename_all = "kebab-case"
        )]
        pub enum $command {
            #[structopt(name = "init")]
            Init {
                #[structopt(flatten)]
                command: $crate::tasks::InitCommand
                $(, #[structopt(flatten)] extra_args: $exinit)?
            },

            #[structopt(name = "publish")]
            Publish {
                #[structopt(flatten)]
                command: $crate::tasks::PublishCommand
                $(, #[structopt(flatten)] extra_args: $expub)?
            },

            #[structopt(name = "run")]
            Run {
                #[structopt(flatten)]
                command: $crate::tasks::RunCommand
                $(, #[structopt(flatten)] extra_args: $exrun)?
            },

            #[structopt(name = "view")]
            View {
                #[structopt(flatten)]
                command: $crate::tasks::ViewCommand,
            },

            $(
                #[structopt(flatten)]
                Subcommand($subcommand),
            )?
        }
    }
}

pub enum KnownCommandFormat<ExtraInitArgs, ExtraPublishArgs, ExtraRunArgs, SubCommands> {
    Init(InitCommand, ExtraInitArgs),
    Publish(PublishCommand, ExtraPublishArgs),
    Run(RunCommand, ExtraRunArgs),
    View(ViewCommand),
    Subcommand(SubCommands),
}

#[derive(Debug, StructOpt)]
pub enum EmptyCommand {}

fn parse_account_address(s: &str) -> Result<AccountAddress> {
    let n = move_lang::shared::parse_u128(s)
        .map_err(|e| anyhow!("Failed to parse address. Got error: {}", e))?;
    Ok(AccountAddress::new(n.to_be_bytes()))
}

fn parse_qualified_module_access(s: &str) -> Result<(ModuleId, Identifier)> {
    match move_core_types::parser::parse_type_tag(s)? {
        TypeTag::Struct(s) => {
            let id = ModuleId::new(s.address, s.module);
            if !s.type_params.is_empty() {
                bail!("Invalid module access. Did not expect type arguments")
            }
            Ok((id, s.name))
        }
        _ => bail!("Invalid module access"),
    }
}

fn parse_qualified_module_access_with_type_args(
    s: &str,
) -> Result<(ModuleId, Identifier, Vec<TypeTag>)> {
    match move_core_types::parser::parse_type_tag(s)? {
        TypeTag::Struct(s) => {
            let id = ModuleId::new(s.address, s.module);
            Ok((id, s.name, s.type_params))
        }
        _ => bail!("Invalid module access"),
    }
}

impl FromStr for SyntaxChoice {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            MOVE_EXTENSION => Ok(SyntaxChoice::Source),
            MOVE_IR_EXTENSION => Ok(SyntaxChoice::IR),
            _ => Err(anyhow!(
                "Invalid syntax choice. Expected '{}' or '{}'",
                MOVE_EXTENSION,
                MOVE_IR_EXTENSION
            )),
        }
    }
}
