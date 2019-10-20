// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    code_cache::module_cache::ModuleCache,
    frame::Frame,
    loaded_data::function::{FunctionRef, FunctionReference},
    IndexKind,
};
use libra_types::vm_error::{StatusCode, VMStatus};
use std::{fmt, marker::PhantomData};
use vm::errors::*;
use vm_runtime_types::value::{Locals, Value};

// TODO Determine stack size limits based on gas limit
const EXECUTION_STACK_SIZE_LIMIT: u64 = 1024;
const FUNCTION_STACK_SIZE_LIMIT: u64 = 1024;

pub struct ExecutionStack<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    stack: Vec<Value>,
    function_stack: Vec<Frame<'txn, FunctionRef<'txn>>>,
    pub module_cache: P,

    // A execution stack will holds an instance of code cache for the lifetime of alloc.
    phantom: PhantomData<&'alloc ()>,
}

impl<'alloc, 'txn, P> ExecutionStack<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    pub fn new(module_cache: P) -> Self {
        ExecutionStack {
            function_stack: vec![],
            stack: vec![],
            module_cache,
            phantom: PhantomData,
        }
    }

    pub fn push_call(&mut self, function: FunctionRef<'txn>) -> VMResult<()> {
        let mut locals = Locals::new(function.local_count());
        let arg_count = function.arg_count();
        for i in 0..arg_count {
            locals.store_loc(arg_count - i - 1, self.pop()?)?;
        }
        if self.function_stack.len() < (FUNCTION_STACK_SIZE_LIMIT as usize) {
            self.function_stack.push(Frame::new(function, locals));
            Ok(())
        } else {
            Err(vm_error(self.location()?, StatusCode::CALL_STACK_OVERFLOW))
        }
    }

    pub fn pop_call(&mut self) -> VMResult<()> {
        self.function_stack
            .pop()
            .ok_or_else(|| vm_error(Location::default(), StatusCode::EMPTY_CALL_STACK))?;
        Ok(())
    }

    pub fn top_frame(&self) -> VMResult<&Frame<'txn, FunctionRef<'txn>>> {
        Ok(self
            .function_stack
            .last()
            .ok_or_else(|| vm_error(Location::default(), StatusCode::EMPTY_CALL_STACK))?)
    }

    pub fn top_frame_mut(&mut self) -> VMResult<&mut Frame<'txn, FunctionRef<'txn>>> {
        Ok(self
            .function_stack
            .last_mut()
            .ok_or_else(|| vm_error(Location::default(), StatusCode::EMPTY_CALL_STACK))?)
    }

    pub fn is_call_stack_empty(&self) -> bool {
        self.function_stack.is_empty()
    }

    pub fn location(&self) -> VMResult<Location> {
        Ok(self.top_frame()?.into())
    }

    pub fn push(&mut self, value: Value) -> VMResult<()> {
        if self.stack.len() < (EXECUTION_STACK_SIZE_LIMIT as usize) {
            self.stack.push(value);
            Ok(())
        } else {
            Err(vm_error(
                self.location()?,
                StatusCode::EXECUTION_STACK_OVERFLOW,
            ))
        }
    }

    pub fn peek(&self) -> VMResult<&Value> {
        Ok(self.stack.last().ok_or_else(|| {
            vm_error(
                self.location().unwrap_or_default(),
                StatusCode::EMPTY_VALUE_STACK,
            )
        })?)
    }

    pub fn peek_at(&self, index: usize) -> VMResult<&Value> {
        let size = self.stack.len();
        if let Some(valid_index) = size
            .checked_sub(index)
            .and_then(|index| index.checked_sub(1))
        {
            Ok(self
                .stack
                .get(valid_index)
                .ok_or_else(|| VMStatus::new(StatusCode::EMPTY_VALUE_STACK))?)
        } else {
            let msg = format!(
                "Index {} out of bounds for {} while indexing {}",
                index,
                size,
                IndexKind::LocalPool
            );
            Err(VMStatus::new(StatusCode::INDEX_OUT_OF_BOUNDS).with_message(msg))
        }
    }

    pub fn pop(&mut self) -> VMResult<Value> {
        Ok(self.stack.pop().ok_or_else(|| {
            vm_error(
                self.location().unwrap_or_default(),
                StatusCode::EMPTY_VALUE_STACK,
            )
        })?)
    }

    pub fn pop_as<T>(&mut self) -> VMResult<T>
    where
        Option<T>: From<Value>,
    {
        let top = self.pop()?.value_as();
        top.ok_or_else(|| vm_error(self.location().unwrap_or_default(), StatusCode::TYPE_ERROR))
    }

    pub fn popn(&mut self, n: u16) -> VMResult<Vec<Value>> {
        let remaining_stack_size = self
            .stack
            .len()
            .checked_sub(n as usize)
            .ok_or_else(|| VMStatus::new(StatusCode::EMPTY_VALUE_STACK))?;
        let args = self.stack.split_off(remaining_stack_size);
        Ok(args)
    }

    pub fn call_stack_height(&self) -> usize {
        self.function_stack.len()
    }

    pub fn set_stack(&mut self, stack: Vec<Value>) {
        self.stack = stack;
    }

    pub fn get_value_stack(&self) -> &Vec<Value> {
        &self.stack
    }

    pub fn push_frame(&mut self, func: FunctionRef<'txn>) -> VMResult<()> {
        if self.function_stack.len() < (FUNCTION_STACK_SIZE_LIMIT as usize) {
            let count = func.local_count();
            self.function_stack
                .push(Frame::new(func, Locals::new(count)));
            Ok(())
        } else {
            Err(vm_error(self.location()?, StatusCode::CALL_STACK_OVERFLOW))
        }
    }
}

impl<'alloc, 'txn, P> fmt::Debug for ExecutionStack<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Stack: {:?}", self.stack)?;
        writeln!(f, "Current Frames: {:?}", self.function_stack)
    }
}
