// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    interpreter::Interpreter,
    loader::{Function, Loader},
};
use move_binary_format::file_format::Bytecode;
use move_vm_types::values::{self, Locals};
use std::{
    collections::BTreeSet,
    io::{self, Write},
    str::FromStr,
};

#[derive(Debug)]
enum DebugCommand {
    PrintStack,
    Step,
    Continue,
    Breakpoint(String),
    DeleteBreakpoint(String),
    PrintBreakpoints,
}

impl DebugCommand {
    pub fn debug_string(&self) -> &str {
        match self {
            Self::PrintStack => "stack",
            Self::Step => "step",
            Self::Continue => "continue",
            Self::Breakpoint(_) => "breakpoint ",
            Self::DeleteBreakpoint(_) => "delete ",
            Self::PrintBreakpoints => "breakpoints",
        }
    }

    pub fn commands() -> Vec<DebugCommand> {
        vec![
            Self::PrintStack,
            Self::Step,
            Self::Continue,
            Self::Breakpoint("".to_string()),
            Self::DeleteBreakpoint("".to_string()),
            Self::PrintBreakpoints,
        ]
    }
}

impl FromStr for DebugCommand {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use DebugCommand::*;
        let s = s.trim();
        if s.starts_with(PrintStack.debug_string()) {
            return Ok(PrintStack);
        }
        if s.starts_with(Step.debug_string()) {
            return Ok(Step);
        }
        if s.starts_with(Continue.debug_string()) {
            return Ok(Continue);
        }
        if let Some(breakpoint) = s.strip_prefix(Breakpoint("".to_owned()).debug_string()) {
            return Ok(Breakpoint(breakpoint.to_owned()));
        }
        if let Some(breakpoint) = s.strip_prefix(DeleteBreakpoint("".to_owned()).debug_string()) {
            return Ok(DeleteBreakpoint(breakpoint.to_owned()));
        }
        if s.starts_with(PrintBreakpoints.debug_string()) {
            return Ok(PrintBreakpoints);
        }
        Err(format!(
            "Unrecognized command: {}\nAvailable commands: {}",
            s,
            Self::commands()
                .iter()
                .map(|command| command.debug_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

#[derive(Debug)]
pub(crate) struct DebugContext {
    breakpoints: BTreeSet<String>,
    should_take_input: bool,
}

impl DebugContext {
    pub(crate) fn new() -> Self {
        Self {
            breakpoints: BTreeSet::new(),
            should_take_input: true,
        }
    }

    pub(crate) fn debug_loop(
        &mut self,
        function_desc: &Function,
        locals: &Locals,
        pc: u16,
        instr: &Bytecode,
        resolver: &Loader,
        interp: &Interpreter,
    ) {
        let instr_string = format!("{:?}", instr);
        let function_string = function_desc.pretty_string();
        let breakpoint_hit = self.breakpoints.contains(&function_string)
            || self
                .breakpoints
                .iter()
                .any(|bp| instr_string[..].starts_with(bp.as_str()));

        if self.should_take_input || breakpoint_hit {
            self.should_take_input = true;
            if breakpoint_hit {
                let bp_match = self
                    .breakpoints
                    .iter()
                    .find(|bp| instr_string.starts_with(bp.as_str()))
                    .unwrap()
                    .clone();
                println!(
                    "Breakpoint {} hit with instruction {}",
                    bp_match, instr_string
                );
            }
            println!(
                "function >> {}\ninstruction >> {:?}\nprogram counter >> {}",
                function_string, instr, pc
            );
            loop {
                print!("> ");
                std::io::stdout().flush().unwrap();
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => match input.parse::<DebugCommand>() {
                        Err(err) => println!("{}", err),
                        Ok(command) => match command {
                            DebugCommand::Step => {
                                self.should_take_input = true;
                                break;
                            }
                            DebugCommand::Continue => {
                                self.should_take_input = false;
                                break;
                            }
                            DebugCommand::Breakpoint(breakpoint) => {
                                self.breakpoints.insert(breakpoint.to_string());
                            }
                            DebugCommand::DeleteBreakpoint(breakpoint) => {
                                self.breakpoints.remove(&breakpoint);
                            }
                            DebugCommand::PrintBreakpoints => self
                                .breakpoints
                                .iter()
                                .enumerate()
                                .for_each(|(i, bp)| println!("[{}] {}", i, bp)),
                            DebugCommand::PrintStack => {
                                let mut s = String::new();
                                interp.debug_print_stack_trace(&mut s, resolver).unwrap();
                                println!("{}", s);
                                println!("Current frame: {}\n", function_string);
                                let code = function_desc.code();
                                println!("        Code:");
                                for (i, instr) in code.iter().enumerate() {
                                    if i as u16 == pc {
                                        println!("          > [{}] {:?}", pc, instr);
                                    } else {
                                        println!("            [{}] {:?}", i, instr);
                                    }
                                }
                                println!("        Locals:");
                                if function_desc.local_count() > 0 {
                                    let mut s = String::new();
                                    values::debug::print_locals(&mut s, locals).unwrap();
                                    println!("{}", s);
                                } else {
                                    println!("            (none)");
                                }
                            }
                        },
                    },
                    Err(err) => {
                        println!("Error reading input: {}", err);
                        break;
                    }
                }
            }
        }
    }
}
