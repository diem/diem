// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::loaded_data::{function::FunctionReference, loaded_module::LoadedModule};
use libra_vm::{
    errors::{Location, VMResult},
    file_format::{Bytecode, CodeOffset, LocalIndex},
};
use libra_vm_runtime_types::value::{Locals, Value};
use std::{fmt, marker::PhantomData};

pub struct Frame<'txn, F: 'txn> {
    pc: u16,
    locals: Locals,
    function: F,
    phantom: PhantomData<&'txn F>,
}

impl<'txn, F> Frame<'txn, F>
where
    F: FunctionReference<'txn>,
{
    pub fn new(function: F, locals: Locals) -> Self {
        Frame {
            pc: 0,
            locals,
            function,
            phantom: PhantomData,
        }
    }

    pub fn code_definition(&self) -> &'txn [Bytecode] {
        self.function.code_definition()
    }

    pub fn save_pc(&mut self, offset: CodeOffset) {
        self.pc = offset;
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn module(&self) -> &'txn LoadedModule {
        self.function.module()
    }

    pub fn copy_loc(&self, idx: LocalIndex) -> VMResult<Value> {
        self.locals.copy_loc(idx as usize)
    }

    pub fn move_loc(&mut self, idx: LocalIndex) -> VMResult<Value> {
        self.locals.move_loc(idx as usize)
    }

    pub fn store_loc(&mut self, idx: LocalIndex, value: Value) -> VMResult<()> {
        self.locals.store_loc(idx as usize, value)
    }

    pub fn borrow_loc(&mut self, idx: LocalIndex) -> VMResult<Value> {
        self.locals.borrow_loc(idx as usize)
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
        write!(f, "\n\tLocals: {:?}", self.locals)?;
        write!(f, "\n\t]")
    }
}

#[cfg(any(test, feature = "instruction_synthesis"))]
impl<'txn, F> Frame<'txn, F>
where
    F: FunctionReference<'txn>,
{
    pub fn set_with_states(&mut self, pc: u16, locals: Locals) {
        self.pc = pc;
        self.locals = locals;
    }

    pub fn get_locals(&self) -> &Locals {
        &self.locals
    }
}
