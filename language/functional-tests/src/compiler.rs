// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bytecode_verifier::VerifiedModule;
use libra_types::account_address::AccountAddress;
use vm::file_format::{CompiledModule, CompiledScript};

pub trait Compiler {
    /// Compile a transaction script or module.
    fn compile<Logger: FnMut(String) -> ()>(
        &mut self,
        log: Logger,
        address: AccountAddress,
        input: &str,
    ) -> Result<ScriptOrModule>;

    /// Return the (ordered) list of modules to be used for genesis. If None is returned the staged
    /// version of the stdlib is used.
    fn stdlib() -> Option<Vec<VerifiedModule>>;
}

pub enum ScriptOrModule {
    Script(CompiledScript),
    Module(CompiledModule),
}
