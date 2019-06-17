// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    bounded_fetch,
    loaded_data::{function::FunctionReference, loaded_module::LoadedModule},
    value::Local,
};
use std::{fmt, marker::PhantomData, mem::replace};
use vm::{
    errors::{Location, VMInvariantViolation, VMResult},
    file_format::{Bytecode, CodeOffset, LocalIndex},
    IndexKind,
};

pub struct Frame<'txn, F: 'txn> {
    pc: u16,
    locals: Vec<Local>,
    function: F,
    phantom: PhantomData<&'txn F>,
}

impl<'txn, F> Frame<'txn, F>
where
    F: FunctionReference<'txn>,
{
    pub fn new(function: F, mut args: Vec<Local>) -> Self {
        args.resize(function.local_count(), Local::Invalid);
        Frame {
            pc: 0,
            locals: args,
            function,
            phantom: PhantomData,
        }
    }

    pub fn code_definition(&self) -> &'txn [Bytecode] {
        self.function.code_definition()
    }

    pub fn jump(&mut self, offset: CodeOffset) {
        self.pc = offset;
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn get_local(&self, idx: LocalIndex) -> Result<&Local, VMInvariantViolation> {
        bounded_fetch(&self.locals, idx as usize, IndexKind::LocalPool)
    }

    pub fn invalidate_local(&mut self, idx: LocalIndex) -> Result<Local, VMInvariantViolation> {
        if let Some(local_ref) = self.locals.get_mut(idx as usize) {
            let old_local = replace(local_ref, Local::Invalid);
            Ok(old_local)
        } else {
            Err(VMInvariantViolation::IndexOutOfBounds(
                IndexKind::LocalPool,
                idx as usize,
                self.locals.len(),
            ))
        }
    }

    pub fn store_local(&mut self, idx: LocalIndex, local: Local) -> VMResult<()> {
        // We don't need to check if the local matches the local signature
        // definition as VM is oblivous to value types.
        if let Some(local_ref) = self.locals.get_mut(idx as usize) {
            // What should we do if local already has some other values?
            *local_ref = local;
            Ok(Ok(()))
        } else {
            Err(VMInvariantViolation::IndexOutOfBounds(
                IndexKind::LocalPool,
                idx as usize,
                self.locals.len(),
            ))
        }
    }

    pub fn module(&self) -> &'txn LoadedModule {
        self.function.module()
    }
}

impl<'txn, F> Into<Location> for &Frame<'txn, F> {
    fn into(self) -> Location {
        Location::new()
    }
}

impl<'txn, F> fmt::Debug for Frame<'txn, F>
where
    F: FunctionReference<'txn>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n\tFunction: {}", self.function.name())?;
        write!(f, "\n\tLocals: [")?;
        for l in self.locals.iter() {
            write!(f, "\n\t\t{:?},", l)?;
        }
        write!(f, "\n\t]")
    }
}

#[cfg(any(test, feature = "instruction_synthesis"))]
impl<'txn, F> Frame<'txn, F>
where
    F: FunctionReference<'txn>,
{
    pub fn set_with_states(&mut self, pc: u16, locals: Vec<Local>) {
        self.pc = pc;
        self.locals = locals;
    }

    pub fn get_locals(&self) -> &Vec<Local> {
        &self.locals
    }
}
