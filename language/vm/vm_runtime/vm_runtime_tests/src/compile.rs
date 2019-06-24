// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for compiling scripts and modules in tests.

use compiler::{
    compiler::{compile_module, compile_program as compiler_compile_program},
    parser::parse_program,
};
use std::mem;
use stdlib::stdlib::*;
use types::{
    account_address::AccountAddress,
    transaction::{Program, TransactionArgument},
};
use vm::file_format::{CompiledModule, CompiledProgram};

/// An API for the compiler. Supports setting custom options.
#[derive(Clone, Debug, Default)]
pub struct Compiler<'a> {
    /// The address used as the sender for the compiler.
    pub address: AccountAddress,
    /// The Move IR code to compile.
    pub code: &'a str,
    /// Skip stdlib dependencies if true.
    pub skip_stdlib_deps: bool,
    /// The address to use for stdlib.
    pub stdlib_address: AccountAddress,
    /// Extra dependencies to compile with.
    pub extra_deps: Vec<CompiledModule>,

    // The typical way this should be used is with functional record update syntax:
    //
    // let compiler = Compiler { address, code, ..Compiler::new() };
    //
    // Until the #[non_exhaustive] attribute is available (see
    // https://github.com/rust-lang/rust/issues/44109), this workaround is required to make the
    // syntax be mandatory.
    #[allow(missing_docs)]
    #[doc(hidden)]
    pub _non_exhaustive: (),
}

impl<'a> Compiler<'a> {
    /// Compiles into a `CompiledProgram` where the bytecode hasn't been serialized.
    pub fn into_compiled_program(mut self) -> CompiledProgram {
        self.compile_impl()
    }

    /// Compiles the script into a serialized form.
    pub fn into_script_blob(mut self) -> Vec<u8> {
        let compiled_program = self.compile_impl();

        let mut serialized_script = Vec::<u8>::new();
        compiled_program
            .script
            .serialize(&mut serialized_script)
            .expect("script must serialize");
        serialized_script
    }

    /// Compiles the code and arguments into a `Program` -- the bytecode is serialized.
    pub fn into_program(mut self, args: Vec<TransactionArgument>) -> Program {
        let compiled_program = self.compile_impl();

        let mut serialized_script = Vec::<u8>::new();
        compiled_program
            .script
            .serialize(&mut serialized_script)
            .expect("script must serialize");
        let mut serialized_modules = vec![];
        for m in compiled_program.modules {
            let mut module = vec![];
            m.serialize(&mut module).expect("module must serialize");
            serialized_modules.push(module);
        }
        Program::new(serialized_script, serialized_modules, args)
    }

    fn compile_impl(&mut self) -> CompiledProgram {
        let parsed_program = parse_program(self.code).expect("program must parse");
        let deps = self.deps();
        compiler_compile_program(&self.address, &parsed_program, &deps)
            .expect("program must compile")
    }

    fn deps(&mut self) -> Vec<CompiledModule> {
        let extra_deps = mem::replace(&mut self.extra_deps, vec![]);
        if self.skip_stdlib_deps {
            extra_deps
        } else {
            let mut deps = stdlib_deps(&self.stdlib_address);
            deps.extend(extra_deps);
            deps
        }
    }
}

// The functions below are legacy functions -- feel free to use them if convenient, but advanced
// functionality should go through Compiler above.

/// Compile the provided Move code into a blob which can be used as the code for a [`Program`].
///
/// The script is compiled with the default account address (`0x0`).
pub fn compile_script(code: &str) -> Vec<u8> {
    let compiler = Compiler {
        code,
        ..Compiler::default()
    };
    compiler.into_script_blob()
}

/// Compile the provided Move code and arguments into a `Program` using `address` as the
/// self address for any modules in `code`.
pub fn compile_program_with_address(
    address: &AccountAddress,
    code: &str,
    args: Vec<TransactionArgument>,
) -> Program {
    let compiler = Compiler {
        address: *address,
        code,
        ..Compiler::default()
    };
    compiler.into_program(args)
}

/// Compile the provided Move code and arguments into a `Program`.
///
/// This supports both scripts and modules defined in the same Move code. The code is compiled with
/// the default account address (`0x0`).
pub fn compile_program(code: &str, args: Vec<TransactionArgument>) -> Program {
    let compiler = Compiler {
        code,
        ..Compiler::default()
    };
    compiler.into_program(args)
}

fn stdlib_deps(address: &AccountAddress) -> Vec<CompiledModule> {
    let coin_module = coin_module();
    let compiled_coin_module =
        compile_module(&address, &coin_module, &[]).expect("coin must compile");

    let hash_module = native_hash_module();
    let compiled_hash_module =
        compile_module(&address, &hash_module, &[]).expect("hash must compile");

    let account_module = account_module();

    let mut deps = vec![compiled_coin_module, compiled_hash_module];
    let compiled_account_module =
        compile_module(&address, &account_module, &deps).expect("account must compile");

    deps.push(compiled_account_module);
    deps
}
