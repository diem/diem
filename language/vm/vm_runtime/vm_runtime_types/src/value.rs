// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::loaded_data::{struct_def::StructDef, types::Type};
use std::{
    cell::{Ref, RefCell},
    ops::Add,
    rc::Rc,
};
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
    byte_array::ByteArray,
    user_string::UserString,
};
use vm::{
    errors::*,
    gas_schedule::{
        words_in, AbstractMemorySize, GasAlgebra, GasCarrier, CONST_SIZE, REFERENCE_SIZE,
        STRUCT_SIZE,
    },
};

#[cfg(test)]
#[path = "unit_tests/value_prop_tests.rs"]
mod value_prop_tests;
#[cfg(test)]
#[path = "unit_tests/value_tests.rs"]
mod value_tests;
#[cfg(test)]
#[path = "unit_tests/vm_types.rs"]
mod vm_types;

#[derive(Debug, Clone)]
pub enum Value {
    Address(AccountAddress),
    U64(u64),
    Bool(bool),
    UserString(UserString),
    Struct(Vec<MutVal>),
    ByteArray(ByteArray),
}

impl Value {
    fn size(&self) -> AbstractMemorySize<GasCarrier> {
        match self {
            Value::U64(_) | Value::Bool(_) => *CONST_SIZE,
            Value::Address(_) => AbstractMemorySize::new(ADDRESS_LENGTH as u64),
            // Possible debate topic: Should we charge based upon the size of the string.
            // At this moment, we take the view that you should be charged as though you are
            // copying the string onto the stack here. This doesn't replicate
            // the semantics that we utilize currently, but this string may
            // need to be copied at some later time, so we need to charge based
            // upon the size of the memory that will possibly need to be accessed.
            Value::UserString(s) => words_in(AbstractMemorySize::new(s.len() as u64)),
            Value::Struct(vals) => vals
                .iter()
                .fold(*STRUCT_SIZE, |acc, vl| acc.map2(vl.size(), Add::add)),
            Value::ByteArray(key) => AbstractMemorySize::new(key.len() as u64),
        }
    }

    /// Normal code should always know what type this value has. This is made available only for
    /// tests.
    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub fn to_struct_def_FOR_TESTING(&self) -> StructDef {
        let values = match self {
            Value::Struct(values) => values,
            _ => panic!("Value must be a struct {:?}", self),
        };

        let fields = values
            .iter()
            .map(|mut_val| {
                let val = &*mut_val.peek();
                match val {
                    Value::Bool(_) => Type::Bool,
                    Value::Address(_) => Type::Address,
                    Value::U64(_) => Type::U64,
                    Value::UserString(_) => Type::UserString,
                    Value::ByteArray(_) => Type::ByteArray,
                    Value::Struct(_) => Type::Struct(val.to_struct_def_FOR_TESTING()),
                }
            })
            .collect();
        StructDef::new(fields)
    }

    // Structural equality for Move values
    // Cannot use Rust's equality due to:
    // - Collections possibly having different representations but still being "equal" semantically
    pub fn equals(&self, v2: &Value) -> Result<bool, VMInvariantViolation> {
        Ok(match (self, v2) {
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            (Value::Address(a1), Value::Address(a2)) => a1 == a2,
            (Value::U64(u1), Value::U64(u2)) => u1 == u2,
            (Value::UserString(s1), Value::UserString(s2)) => s1 == s2,
            (Value::Struct(s1), Value::Struct(s2)) => {
                if s1.len() != s2.len() {
                    return Err(VMInvariantViolation::InternalTypeError);
                }
                for (mv1, mv2) in s1.iter().zip(s2) {
                    if !MutVal::equals(mv1, mv2)? {
                        return Ok(false);
                    }
                }
                true
            }
            (Value::ByteArray(ba1), Value::ByteArray(ba2)) => ba1 == ba2,
            _ => return Err(VMInvariantViolation::InternalTypeError),
        })
    }

    // Structural non-equality for Move values
    // Implemented by hand instead of `!equals` to allow for short circuiting
    pub fn not_equals(&self, v2: &Value) -> Result<bool, VMInvariantViolation> {
        Ok(match (self, v2) {
            (Value::Bool(b1), Value::Bool(b2)) => b1 != b2,
            (Value::Address(a1), Value::Address(a2)) => a1 != a2,
            (Value::U64(u1), Value::U64(u2)) => u1 != u2,
            (Value::UserString(s1), Value::UserString(s2)) => s1 != s2,
            (Value::Struct(s1), Value::Struct(s2)) => {
                if s1.len() != s2.len() {
                    return Err(VMInvariantViolation::InternalTypeError);
                }
                for (mv1, mv2) in s1.iter().zip(s2) {
                    if MutVal::not_equals(mv1, mv2)? {
                        return Ok(true);
                    }
                }
                false
            }
            (Value::ByteArray(ba1), Value::ByteArray(ba2)) => ba1 != ba2,
            _ => return Err(VMInvariantViolation::InternalTypeError),
        })
    }
}

pub trait Reference
where
    Self: std::marker::Sized + Clone,
{
    fn borrow_field(&self, idx: u32) -> Option<Self>;
    fn read_reference(self) -> MutVal;
    fn mutate_reference(self, v: MutVal);

    fn size(&self) -> AbstractMemorySize<GasCarrier>;
}

#[derive(Debug)]
pub struct MutVal(pub Rc<RefCell<Value>>);

#[derive(Debug)]
pub enum Local {
    Ref(MutVal),
    GlobalRef(GlobalRef),
    Value(MutVal),
    Invalid,
}

/// Status for on chain data (published resources):
/// CLEAN - the data was only read
/// DIRTY - the data was changed anywhere in the data tree of the given resource
/// DELETED - MoveFrom was called on the given AccessPath for the given resource
#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Debug, Clone)]
enum GlobalDataStatus {
    CLEAN   = 0,
    DIRTY   = 1,
    DELETED = 2,
}

/// A root into an instance on chain.
/// Holds flags about the status of the instance and a reference count to balance
/// Borrow* and ReleaseRef
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RootAccessPath {
    status: GlobalDataStatus,
    ap: AccessPath,
}

/// A GlobalRef holds the reference to the data and a shared reference to the root so
/// status flags and reference count can be properly managed
#[derive(Debug, Clone)]
pub struct GlobalRef {
    root: Rc<RefCell<RootAccessPath>>,
    reference: MutVal,
}

impl Clone for MutVal {
    fn clone(&self) -> Self {
        MutVal(Rc::new(RefCell::new(self.peek().clone())))
    }
}

impl Clone for Local {
    fn clone(&self) -> Self {
        match self {
            Local::Ref(v) => Local::Ref(v.shallow_clone()),
            Local::GlobalRef(v) => Local::GlobalRef(v.shallow_clone()),
            Local::Value(v) => Local::Value(v.clone()),
            Local::Invalid => Local::Invalid,
        }
    }
}

impl MutVal {
    pub fn try_own(mv: Self) -> Result<Value, VMInvariantViolation> {
        match Rc::try_unwrap(mv.0) {
            Ok(cell) => Ok(cell.into_inner()),
            Err(_) => Err(VMInvariantViolation::LocalReferenceError),
        }
    }

    pub fn peek(&self) -> Ref<Value> {
        self.0.borrow()
    }

    pub fn new(v: Value) -> Self {
        MutVal(Rc::new(RefCell::new(v)))
    }

    fn shallow_clone(&self) -> Self {
        MutVal(Rc::clone(&self.0))
    }

    fn address(addr: AccountAddress) -> Self {
        MutVal::new(Value::Address(addr))
    }

    fn u64(i: u64) -> Self {
        MutVal::new(Value::U64(i))
    }

    fn bool(b: bool) -> Self {
        MutVal::new(Value::Bool(b))
    }

    fn user_string(s: UserString) -> Self {
        MutVal::new(Value::UserString(s))
    }

    fn struct_(v: Vec<MutVal>) -> Self {
        MutVal::new(Value::Struct(v))
    }

    fn bytearray(v: ByteArray) -> Self {
        MutVal::new(Value::ByteArray(v))
    }

    fn size(&self) -> AbstractMemorySize<GasCarrier> {
        self.peek().size()
    }

    // Structural equality for Move values
    // Cannot use Rust's equality due to:
    // - Collections possibly having different representations but still being "equal" semantically
    pub fn equals(&self, mv2: &MutVal) -> Result<bool, VMInvariantViolation> {
        self.peek().equals(&mv2.peek())
    }

    // Structural non-equality for Move values
    // Implemented by hand instead of `!equals` to allow for short circuiting
    pub fn not_equals(&self, mv2: &MutVal) -> Result<bool, VMInvariantViolation> {
        self.peek().not_equals(&mv2.peek())
    }
}

impl Reference for MutVal {
    fn borrow_field(&self, idx: u32) -> Option<Self> {
        match &*self.peek() {
            Value::Struct(ref vec) => vec.get(idx as usize).map(MutVal::shallow_clone),
            _ => None,
        }
    }

    fn read_reference(self) -> MutVal {
        self.clone()
    }

    fn mutate_reference(self, v: MutVal) {
        self.0.replace(v.peek().clone());
    }

    fn size(&self) -> AbstractMemorySize<GasCarrier> {
        words_in(*REFERENCE_SIZE)
    }
}

impl Local {
    pub fn address(addr: AccountAddress) -> Self {
        Local::Value(MutVal::address(addr))
    }

    pub fn u64(i: u64) -> Self {
        Local::Value(MutVal::u64(i))
    }

    pub fn bool(b: bool) -> Self {
        Local::Value(MutVal::bool(b))
    }

    pub fn user_string(s: UserString) -> Self {
        Local::Value(MutVal::user_string(s))
    }

    pub fn struct_(v: Vec<MutVal>) -> Self {
        Local::Value(MutVal::struct_(v))
    }

    pub fn bytearray(v: ByteArray) -> Self {
        Local::Value(MutVal::bytearray(v))
    }

    pub fn borrow_local(&self) -> Option<Self> {
        match self {
            Local::Value(v) => Some(Local::Ref(v.shallow_clone())),
            _ => None,
        }
    }

    pub fn borrow_field(&self, idx: u32) -> Option<Self> {
        match self {
            Local::Ref(v) => v.borrow_field(idx).map(Local::Ref),
            Local::GlobalRef(v) => v.borrow_field(idx).map(Local::GlobalRef),
            _ => None,
        }
    }

    pub fn read_reference(self) -> Option<Self> {
        match self {
            Local::Ref(r) => Some(Local::Value(r.read_reference())),
            Local::GlobalRef(gr) => Some(Local::Value(gr.read_reference())),
            _ => None,
        }
    }

    pub fn mutate_reference(self, v: MutVal) {
        match self {
            Local::Ref(r) => r.mutate_reference(v),
            Local::GlobalRef(r) => r.mutate_reference(v),
            _ => (),
        }
    }

    pub fn value(self) -> Option<MutVal> {
        match self {
            Local::Value(v) => Some(v),
            _ => None,
        }
    }

    pub fn value_as<T>(self) -> Option<T>
    where
        Option<T>: From<MutVal>,
    {
        self.value().and_then(std::convert::Into::into)
    }

    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        match self {
            Local::Ref(v) => v.size(),
            Local::GlobalRef(v) => v.size(),
            Local::Value(v) => v.size(),
            Local::Invalid => *CONST_SIZE,
        }
    }

    // Structural equality for Move values
    // Cannot use Rust's equality due to:
    // - Internal representation of references
    // - Collections possibly having different representations but still being "equal" semantically
    pub fn equals(self, l2: Local) -> Result<bool, VMInvariantViolation> {
        match (self, l2) {
            (Local::Ref(mv1), Local::Ref(mv2)) | (Local::Value(mv1), Local::Value(mv2)) => {
                mv1.equals(&mv2)
            }
            (Local::GlobalRef(gr1), Local::GlobalRef(gr2)) => {
                gr1.read_reference().equals(&gr2.read_reference())
            }
            (Local::GlobalRef(gr), Local::Ref(mv)) => gr.read_reference().equals(&mv),
            (Local::Ref(mv), Local::GlobalRef(gr)) => mv.equals(&gr.read_reference()),
            (Local::Invalid, Local::Invalid) => Ok(true),
            _ => Err(VMInvariantViolation::InternalTypeError),
        }
    }

    // Structural non-equality for Move values
    // Implemented by hand instead of `!equals` to allow for short circuiting
    pub fn not_equals(self, l2: Local) -> Result<bool, VMInvariantViolation> {
        match (self, l2) {
            (Local::Ref(mv1), Local::Ref(mv2)) | (Local::Value(mv1), Local::Value(mv2)) => {
                mv1.not_equals(&mv2)
            }
            (Local::GlobalRef(gr1), Local::GlobalRef(gr2)) => {
                gr1.read_reference().not_equals(&gr2.read_reference())
            }
            (Local::GlobalRef(gr), Local::Ref(mv)) => gr.read_reference().not_equals(&mv),
            (Local::Ref(mv), Local::GlobalRef(gr)) => mv.not_equals(&gr.read_reference()),
            (Local::Invalid, Local::Invalid) => Ok(false),
            _ => Err(VMInvariantViolation::InternalTypeError),
        }
    }
}

impl RootAccessPath {
    pub fn new(ap: AccessPath) -> Self {
        RootAccessPath {
            status: GlobalDataStatus::CLEAN,
            ap,
        }
    }

    fn mark_dirty(&mut self) {
        self.status = GlobalDataStatus::DIRTY;
    }

    fn mark_deleted(&mut self) {
        self.status = GlobalDataStatus::DELETED;
    }
}

impl GlobalRef {
    pub fn make_root(ap: AccessPath, reference: MutVal) -> Self {
        GlobalRef {
            root: Rc::new(RefCell::new(RootAccessPath::new(ap))),
            reference,
        }
    }

    pub fn move_to(ap: AccessPath, reference: MutVal) -> Self {
        let mut root = RootAccessPath::new(ap);
        root.mark_dirty();
        GlobalRef {
            root: Rc::new(RefCell::new(root)),
            reference,
        }
    }

    fn new_ref(root: &GlobalRef, reference: MutVal) -> Self {
        GlobalRef {
            root: Rc::clone(&root.root),
            reference,
        }
    }

    // Return the resource behind the reference.
    // If the reference is not exclusively held by the cache (ref count 0) returns None
    pub fn get_data(self) -> Option<Value> {
        match Rc::try_unwrap(self.root) {
            Ok(_) => match Rc::try_unwrap(self.reference.0) {
                Ok(res) => Some(res.into_inner()),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    pub fn is_loadable(&self) -> bool {
        !self.is_deleted()
    }

    pub fn is_dirty(&self) -> bool {
        self.root.borrow().status == GlobalDataStatus::DIRTY
    }

    pub fn is_deleted(&self) -> bool {
        self.root.borrow().status == GlobalDataStatus::DELETED
    }

    pub fn is_clean(&self) -> bool {
        self.root.borrow().status == GlobalDataStatus::CLEAN
    }

    pub fn move_from(&mut self) -> MutVal {
        self.root.borrow_mut().mark_deleted();
        self.reference.shallow_clone()
    }

    pub fn shallow_clone(&self) -> Self {
        GlobalRef {
            root: Rc::clone(&self.root),
            reference: self.reference.shallow_clone(),
        }
    }

    fn size(&self) -> AbstractMemorySize<GasCarrier> {
        *REFERENCE_SIZE
    }
}

impl Reference for GlobalRef {
    fn borrow_field(&self, idx: u32) -> Option<Self> {
        match &*self.reference.peek() {
            Value::Struct(ref vec) => match vec.get(idx as usize) {
                Some(field_ref) => Some(GlobalRef::new_ref(self, field_ref.shallow_clone())),
                None => None,
            },
            _ => None,
        }
    }

    fn read_reference(self) -> MutVal {
        self.reference.clone()
    }

    fn mutate_reference(self, v: MutVal) {
        self.root.borrow_mut().mark_dirty();
        self.reference.mutate_reference(v);
    }

    fn size(&self) -> AbstractMemorySize<GasCarrier> {
        words_in(*REFERENCE_SIZE)
    }
}

//
// Conversion routines for the interpreter
//

impl From<MutVal> for Option<u64> {
    fn from(value: MutVal) -> Option<u64> {
        match &*value.peek() {
            Value::U64(i) => Some(*i),
            _ => None,
        }
    }
}

impl From<MutVal> for Option<bool> {
    fn from(value: MutVal) -> Option<bool> {
        match &*value.peek() {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl From<MutVal> for Option<AccountAddress> {
    fn from(value: MutVal) -> Option<AccountAddress> {
        match *value.peek() {
            Value::Address(addr) => Some(addr),
            _ => None,
        }
    }
}

impl From<MutVal> for Option<ByteArray> {
    fn from(value: MutVal) -> Option<ByteArray> {
        match &*value.peek() {
            Value::ByteArray(blob) => Some(blob.clone()),
            _ => None,
        }
    }
}

impl From<GlobalRef> for Option<AccountAddress> {
    fn from(value: GlobalRef) -> Option<AccountAddress> {
        match *value.reference.peek() {
            Value::Address(addr) => Some(addr),
            _ => None,
        }
    }
}
