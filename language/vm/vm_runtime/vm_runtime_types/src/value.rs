// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::{struct_def::StructDef, types::Type},
    native_structs::{serializer::deserialize_native, NativeStructValue},
};
use canonical_serialization::*;
use failure::prelude::*;
use std::{
    cell::{Ref, RefCell},
    convert::TryFrom,
    mem::replace,
    ops::Add,
    rc::Rc,
};
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
    byte_array::ByteArray,
    vm_error::{StatusCode, VMStatus},
};
use vm::{
    errors::*,
    gas_schedule::{
        words_in, AbstractMemorySize, GasAlgebra, GasCarrier, CONST_SIZE, REFERENCE_SIZE,
        STRUCT_SIZE,
    },
    vm_string::VMString,
    IndexKind,
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

/// Runtime representation of a value.
///
/// `ValueImpl` is a single enum that describes all possible values expressed in Move and those
/// internal to the runtime (e.g. `Invalid`).
/// `ValueImpl` is private to this module and wrapper types are used to limit the values available
/// in different context.
/// For instance, `[Value]` is a wrapper of `ValueImpl` that does not allow `Invalid` to be
/// represented. In that sense instances of `[Value]` are what gets pushed/popped on the
/// stack and effectively the values representable in Move.
#[derive(PartialEq, Eq, Debug, Clone)]
enum ValueImpl {
    /// Locals are invalid on entry of a function and when moved out.
    Invalid,
    // Primitive types
    U64(u64),
    Address(AccountAddress),
    Bool(bool),
    ByteArray(ByteArray),
    String(VMString),

    /// A struct in Move.
    Struct(Struct),

    /// A native struct
    NativeStruct(NativeStructValue),

    /// Reference to a local.
    Reference(Reference),
    /// Global reference into storage.
    GlobalRef(GlobalRef),
    /// A reference to a local.
    ///
    /// This value is used to promote a value into a reference lazily when borrowing a local.
    PromotedReference(Reference),
}

/// A Move value.
///
/// `Value` is just a wrapper type around `[ValueImpl]` which allows only Move types.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Value(ValueImpl);

/// Internal representation for a reference or a mutable value.
/// This is quite a core type for the mechanics of references.
#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct MutVal(Rc<RefCell<ValueImpl>>);

/// A struct in Move.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Struct(Vec<MutVal>);

/// External representation for a reference.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Reference(MutVal);

/// The locals (vector of values) for a `Frame`.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Locals(Vec<ValueImpl>);

/// The wrapper for all things reference in the VM.
/// Right now we have 2 kind of references: local and global.
/// This enum wraps both and offers a common API.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ReferenceValue {
    Reference(Reference),
    GlobalRef(GlobalRef),
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
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct GlobalRef {
    root: Rc<RefCell<RootAccessPath>>,
    reference: MutVal,
}

// All implementation here is private to this module and the effective logic in
// working with values in the VM.
impl ValueImpl {
    fn into_value(self) -> VMResult<Value> {
        match self {
            ValueImpl::Invalid => Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR)),
            ValueImpl::PromotedReference(reference) => reference.into_value(),
            _ => Ok(Value(self)),
        }
    }

    fn copy_value(&self) -> VMResult<Value> {
        match self {
            ValueImpl::Invalid => Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR)),
            ValueImpl::PromotedReference(reference) => reference.copy_value(),
            _ => Ok(Value(self.clone())),
        }
    }

    fn borrow_field(&self, field_offset: usize) -> VMResult<Value> {
        match self {
            ValueImpl::Struct(s) => s.get_field_reference(field_offset),
            _ => Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR)),
        }
    }

    fn size(&self) -> AbstractMemorySize<GasCarrier> {
        match self {
            ValueImpl::Invalid | ValueImpl::U64(_) | ValueImpl::Bool(_) => *CONST_SIZE,
            ValueImpl::Address(_) => AbstractMemorySize::new(ADDRESS_LENGTH as u64),
            // Possible debate topic: Should we charge based upon the size of the string.
            // At this moment, we take the view that you should be charged as though you are
            // copying the string onto the stack here. This doesn't replicate
            // the semantics that we utilize currently, but this string may
            // need to be copied at some later time, so we need to charge based
            // upon the size of the memory that will possibly need to be accessed.
            ValueImpl::String(s) => words_in(AbstractMemorySize::new(s.len() as u64)),
            ValueImpl::ByteArray(key) => AbstractMemorySize::new(key.len() as u64),
            ValueImpl::Struct(s) => s.size(),
            ValueImpl::NativeStruct(s) => s.size(),
            ValueImpl::Reference(reference) => reference.size(),
            ValueImpl::GlobalRef(reference) => reference.size(),
            ValueImpl::PromotedReference(reference) => reference.0.size(),
        }
    }

    // Structural equality for Move values
    fn equals(&self, v2: &ValueImpl) -> VMResult<bool> {
        match (self, v2) {
            // TODO: this does not look right to me....
            (ValueImpl::Invalid, ValueImpl::Invalid) => Ok(true),
            // values
            (ValueImpl::U64(u1), ValueImpl::U64(u2)) => Ok(u1 == u2),
            (ValueImpl::Bool(b1), ValueImpl::Bool(b2)) => Ok(b1 == b2),
            (ValueImpl::Address(a1), ValueImpl::Address(a2)) => Ok(a1 == a2),
            (ValueImpl::ByteArray(ba1), ValueImpl::ByteArray(ba2)) => Ok(ba1 == ba2),
            (ValueImpl::String(s1), ValueImpl::String(s2)) => Ok(s1 == s2),
            (ValueImpl::Struct(s1), ValueImpl::Struct(s2)) => s1.equals(s2),
            // references
            (ValueImpl::Reference(ref1), ValueImpl::Reference(ref2)) => ref1.equals(ref2),
            (ValueImpl::GlobalRef(gr1), ValueImpl::GlobalRef(gr2)) => gr1.equals(gr2),
            (ValueImpl::GlobalRef(gr), ValueImpl::Reference(reference)) => gr.equals_ref(reference),
            (ValueImpl::Reference(reference), ValueImpl::GlobalRef(gr)) => gr.equals_ref(reference),
            // Should we allow comparing native structs?
            _ => Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR)),
        }
    }

    /// Normal code should always know what type this value has. This is made available only for
    /// tests.
    #[allow(non_snake_case)]
    #[doc(hidden)]
    fn to_type_FOR_TESTING(&self) -> Type {
        match self {
            ValueImpl::Invalid => unreachable!("Cannot ask type of invalid location"),
            ValueImpl::U64(_) => Type::U64,
            ValueImpl::Address(_) => Type::Address,
            ValueImpl::Bool(_) => Type::Bool,
            ValueImpl::ByteArray(_) => Type::ByteArray,
            ValueImpl::String(_) => Type::String,
            ValueImpl::Struct(s) => Type::Struct(s.to_struct_def_FOR_TESTING()),
            ValueImpl::NativeStruct(v) => Type::Struct(v.to_struct_def_FOR_TESTING()),
            ValueImpl::Reference(reference) => {
                Type::Reference(Box::new(reference.to_type_FOR_TESTING()))
            }
            ValueImpl::GlobalRef(reference) => {
                Type::Reference(Box::new(reference.to_type_FOR_TESTING()))
            }
            ValueImpl::PromotedReference(reference) => reference.to_type_FOR_TESTING(),
        }
    }
}

impl Value {
    /// Private internal constructor to make a `Value` from a `ValueImpl`.
    fn new(value: ValueImpl) -> Self {
        Value(value)
    }

    /// Return a `Value` representing a `u64` in the VM.
    pub fn u64(value: u64) -> Self {
        Value(ValueImpl::U64(value))
    }

    /// Return a `Value` representing an `AccountAddress` in the VM.
    pub fn address(address: AccountAddress) -> Self {
        Value(ValueImpl::Address(address))
    }

    /// Return a `Value` representing an `bool` in the VM.
    pub fn bool(value: bool) -> Self {
        Value(ValueImpl::Bool(value))
    }

    /// Return a `Value` representing a `ByteArray` in the VM.
    pub fn byte_array(bytearray: ByteArray) -> Self {
        Value(ValueImpl::ByteArray(bytearray))
    }

    /// Return a `Value` representing a `String` in the VM.
    pub fn string(s: VMString) -> Self {
        Value(ValueImpl::String(s))
    }

    /// Return a `Value` representing a `Struct` in the VM.
    pub fn struct_(s: Struct) -> Self {
        Value(ValueImpl::Struct(s))
    }

    /// Return a `Value` representing a `Reference` in the VM.
    pub fn reference(reference: Reference) -> Self {
        Value(ValueImpl::Reference(reference))
    }

    /// Return a `Value` representing a `GlobalRef` in the VM.
    pub fn global_ref(reference: GlobalRef) -> Self {
        Value(ValueImpl::GlobalRef(reference))
    }

    pub fn native_struct(v: NativeStructValue) -> Self {
        Value(ValueImpl::NativeStruct(v))
    }

    /// Convert a Value into a `T` if the value represents a type `T`.
    pub fn value_as<T>(self) -> Option<T>
    where
        Option<T>: From<Value>,
    {
        std::convert::Into::into(self)
    }

    /// `Eq` bytecode
    pub fn equals(&self, v2: &Value) -> VMResult<bool> {
        self.0.equals(&v2.0)
    }

    /// `Neq` bytecode
    pub fn not_equals(&self, v2: &Value) -> VMResult<bool> {
        self.equals(v2).and_then(|res| Ok(!res))
    }

    // called from gas metering, revisit
    pub fn is_global_ref(&self) -> bool {
        match &self.0 {
            ValueImpl::GlobalRef(_) => true,
            _ => false,
        }
    }

    // called from cost synthesis, revisit
    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        self.0.size()
    }

    // called from cost synthesis, revisit
    pub fn as_struct_ref(&self) -> Option<&Struct> {
        match &self.0 {
            ValueImpl::Struct(s) => Some(s),
            _ => None,
        }
    }

    /// Normal code should always know what type this value has. This is made available only for
    /// tests.
    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub fn to_type_FOR_TESTING(&self) -> Type {
        self.0.to_type_FOR_TESTING()
    }
}

//
// From/Into implementation to read known values off the stack.
// A pop from the stack returns a `Value` that is owned by the caller of pop. For many opcodes
// (e.g. Add) the values popped from the stack are expected to be u64 and should fail otherwise.
//

impl From<Value> for Option<u64> {
    fn from(value: Value) -> Option<u64> {
        match value.0 {
            ValueImpl::U64(i) => Some(i),
            _ => None,
        }
    }
}

impl From<Value> for Option<bool> {
    fn from(value: Value) -> Option<bool> {
        match value.0 {
            ValueImpl::Bool(b) => Some(b),
            _ => None,
        }
    }
}

impl From<Value> for Option<AccountAddress> {
    fn from(value: Value) -> Option<AccountAddress> {
        match value.0 {
            ValueImpl::Address(address) => Some(address),
            _ => None,
        }
    }
}

impl From<Value> for Option<ByteArray> {
    fn from(value: Value) -> Option<ByteArray> {
        match value.0 {
            ValueImpl::ByteArray(byte_array) => Some(byte_array),
            _ => None,
        }
    }
}

impl From<Value> for Option<VMString> {
    fn from(value: Value) -> Option<VMString> {
        match value.0 {
            ValueImpl::String(s) => Some(s),
            _ => None,
        }
    }
}

impl From<Value> for Option<Struct> {
    fn from(value: Value) -> Option<Struct> {
        match value.0 {
            ValueImpl::Struct(s) => Some(s),
            _ => None,
        }
    }
}

impl From<Value> for Option<ReferenceValue> {
    fn from(value: Value) -> Option<ReferenceValue> {
        match value.0 {
            ValueImpl::Reference(reference) => Some(ReferenceValue::Reference(reference)),
            ValueImpl::GlobalRef(reference) => Some(ReferenceValue::GlobalRef(reference)),
            _ => None,
        }
    }
}

impl From<Value> for Option<Reference> {
    fn from(value: Value) -> Option<Reference> {
        match value.0 {
            ValueImpl::Reference(reference) => Some(reference),
            _ => None,
        }
    }
}

impl From<Value> for Option<GlobalRef> {
    fn from(value: Value) -> Option<GlobalRef> {
        match value.0 {
            ValueImpl::GlobalRef(reference) => Some(reference),
            _ => None,
        }
    }
}

impl MutVal {
    pub(crate) fn new(v: Value) -> Self {
        MutVal(Rc::new(RefCell::new(v.0)))
    }

    fn peek(&self) -> Ref<ValueImpl> {
        self.0.borrow()
    }

    fn into_value(self) -> VMResult<Value> {
        match Rc::try_unwrap(self.0) {
            Ok(cell) => Ok(Value::new(cell.into_inner())),
            Err(_) => Err(VMStatus::new(StatusCode::LOCAL_REFERENCE_ERROR)),
        }
    }

    fn copy_value(&self) -> VMResult<Value> {
        self.peek().copy_value()
    }

    pub(crate) fn size(&self) -> AbstractMemorySize<GasCarrier> {
        self.peek().size()
    }

    fn borrow_field(&self, field_offset: usize) -> VMResult<Value> {
        self.peek().borrow_field(field_offset)
    }

    fn write_value(self, value: Value) {
        self.0.replace(value.0);
    }

    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub(crate) fn to_type_FOR_TESTING(&self) -> Type {
        self.peek().to_type_FOR_TESTING()
    }

    fn equals(&self, v2: &MutVal) -> VMResult<bool> {
        self.peek().equals(&v2.peek())
    }

    fn mutate_native_struct<T, F>(&self, op: F) -> Option<T>
    where
        F: FnOnce(&mut NativeStructValue) -> Option<T>,
    {
        match &mut *self.0.borrow_mut() {
            ValueImpl::NativeStruct(s) => op(s),
            _ => None,
        }
    }

    fn read_native_struct<T, F>(&self, op: F) -> Option<T>
    where
        F: FnOnce(&NativeStructValue) -> Option<T>,
    {
        match &*self.0.borrow_mut() {
            ValueImpl::NativeStruct(s) => op(s),
            _ => None,
        }
    }
}

impl Struct {
    /// Creates a struct from a vector of `Value`s.
    pub fn new(values: Vec<Value>) -> Self {
        let mut fields = vec![];
        for value in values {
            fields.push(MutVal::new(value));
        }
        Struct(fields)
    }

    /// Called by `Unpack` to fetch all fields out of the struct being unpacked.
    pub fn get_field_value(&self, field_offset: usize) -> VMResult<Value> {
        if let Some(field_ref) = self.0.get(field_offset) {
            field_ref.copy_value()
        } else {
            Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR))
        }
    }

    // Public because of gas synthesis - review.
    pub fn get_field_reference(&self, field_offset: usize) -> VMResult<Value> {
        if let Some(field_ref) = self.0.get(field_offset) {
            Ok(Value::reference(Reference(field_ref.clone())))
        } else {
            Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR))
        }
    }

    // Invoked by gas metering to determine the size of the struct. (review)
    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        self.0
            .iter()
            .fold(*STRUCT_SIZE, |acc, vl| acc.map2(vl.size(), Add::add))
    }

    fn equals(&self, s2: &Struct) -> VMResult<bool> {
        if self.0.len() != s2.0.len() {
            return Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR));
        }
        for (v1, v2) in self.0.iter().zip(&s2.0) {
            if !v1.equals(v2)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Normal code should always know what type this value has. This is made available only for
    /// tests.
    #[allow(non_snake_case)]
    #[doc(hidden)]
    fn to_struct_def_FOR_TESTING(&self) -> StructDef {
        let fields = self.0.iter().map(MutVal::to_type_FOR_TESTING).collect();
        StructDef::new(fields)
    }
}

// Private API for a `Reference`. It is jut a pass through layer. All those should disappear
// once compiled
impl Reference {
    // called from cost synthesis, revisit
    pub fn new(value: Value) -> Self {
        Reference(MutVal::new(value))
    }

    pub(crate) fn new_from_cell(val: MutVal) -> Self {
        Reference(val)
    }

    fn into_value(self) -> VMResult<Value> {
        self.0.into_value()
    }

    fn copy_value(&self) -> VMResult<Value> {
        self.0.copy_value()
    }

    fn size(&self) -> AbstractMemorySize<GasCarrier> {
        words_in(*REFERENCE_SIZE)
    }

    fn borrow_field(&self, field_offset: usize) -> VMResult<Value> {
        self.0.borrow_field(field_offset)
    }

    fn write_value(self, value: Value) {
        self.0.write_value(value);
    }

    fn equals(&self, ref2: &Reference) -> VMResult<bool> {
        self.0.equals(&ref2.0)
    }

    /// Normal code should always know what type this value has. This is made available only for
    /// tests.
    #[allow(non_snake_case)]
    #[doc(hidden)]
    fn to_type_FOR_TESTING(&self) -> Type {
        self.0.to_type_FOR_TESTING()
    }

    fn mutate_native_struct<T, F>(&self, op: F) -> Option<T>
    where
        F: FnOnce(&mut NativeStructValue) -> Option<T>,
    {
        self.0.mutate_native_struct(op)
    }

    fn read_native_struct<T, F>(&self, op: F) -> Option<T>
    where
        F: FnOnce(&NativeStructValue) -> Option<T>,
    {
        self.0.read_native_struct(op)
    }
}

/// Implementation for reference opcodes.
///
/// A reference in the runtime can have different shapes so any time a reference is expected
/// a `ReferenceValue` is created by popping whatever reference is on the stack.
/// Operations on the reference (via opcodes) are then invoked on a `ReferenceValue`.
impl ReferenceValue {
    /// Create a `ReferenceValue` from a `Value` popped off the stack.
    /// Fails if the value is not a reference of some kind.
    pub fn new(value: Value) -> VMResult<Self> {
        match value.0 {
            ValueImpl::Reference(reference) => Ok(ReferenceValue::Reference(reference)),
            ValueImpl::GlobalRef(reference) => Ok(ReferenceValue::GlobalRef(reference)),
            _ => Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR)),
        }
    }

    /// Borrow a field from the reference if the reference is to a struct.
    pub fn borrow_field(self, field_offset: usize) -> VMResult<Value> {
        match self {
            ReferenceValue::GlobalRef(ref reference) => reference.borrow_field(field_offset),
            ReferenceValue::Reference(ref reference) => reference.borrow_field(field_offset),
        }
    }

    /// Read the value pointed to by the reference.
    pub fn read_ref(self) -> VMResult<Value> {
        match self {
            ReferenceValue::GlobalRef(reference) => reference.copy_value(),
            ReferenceValue::Reference(reference) => reference.copy_value(),
        }
    }

    /// Write `value` to the location pointed to by the reference.
    pub fn write_ref(self, value: Value) {
        match self {
            ReferenceValue::GlobalRef(reference) => reference.write_value(value),
            ReferenceValue::Reference(reference) => reference.write_value(value),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn mutate_native_struct<T, F>(&self, op: F) -> Option<T>
    where
        F: FnOnce(&mut NativeStructValue) -> Option<T>,
    {
        match self {
            ReferenceValue::GlobalRef(reference) => reference.mutate_native_struct(op),
            ReferenceValue::Reference(reference) => reference.mutate_native_struct(op),
        }
    }

    pub(crate) fn read_native_struct<T, F>(&self, op: F) -> Option<T>
    where
        F: FnOnce(&NativeStructValue) -> Option<T>,
    {
        match self {
            ReferenceValue::GlobalRef(reference) => reference.read_native_struct(op),
            ReferenceValue::Reference(reference) => reference.read_native_struct(op),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_native_struct_reference<F>(&self, op: F) -> Option<Value>
    where
        F: FnOnce(&NativeStructValue) -> Option<MutVal>,
    {
        match self {
            ReferenceValue::GlobalRef(reference) => reference
                .read_native_struct(op)
                .map(|v| Value::global_ref(GlobalRef::new_ref(reference, v))),
            ReferenceValue::Reference(reference) => reference
                .read_native_struct(op)
                .map(|v| Value::reference(Reference::new_from_cell(v))),
        }
    }
}

//
// Global Reference implementation - check how to move part of this code outside
// (possibly in the cache)
//

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

    #[allow(dead_code)]
    fn mark_deleted(&mut self) {
        self.status = GlobalDataStatus::DELETED;
    }
}

impl GlobalRef {
    pub fn make_root(ap: AccessPath, value: Value) -> Self {
        GlobalRef {
            root: Rc::new(RefCell::new(RootAccessPath::new(ap))),
            reference: MutVal::new(value),
        }
    }

    pub fn move_to(ap: AccessPath, value: Struct) -> Self {
        let mut root = RootAccessPath::new(ap);
        root.mark_dirty();
        GlobalRef {
            root: Rc::new(RefCell::new(root)),
            reference: MutVal::new(Value::struct_(value)),
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
                Ok(res) => Some(Value::new(res.into_inner())),
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

    pub fn move_from(&mut self) -> VMResult<Value> {
        self.root.borrow_mut().mark_deleted();
        self.reference.copy_value()
    }

    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        words_in(*REFERENCE_SIZE)
    }

    fn borrow_field(&self, field_offset: usize) -> VMResult<Value> {
        let field_ref = self
            .reference
            .borrow_field(field_offset)?
            .value_as::<Reference>()
            .unwrap()
            .0;
        Ok(Value::global_ref(GlobalRef::new_ref(self, field_ref)))
    }

    fn copy_value(&self) -> VMResult<Value> {
        let value = self.reference.copy_value()?;
        Ok(value)
    }

    fn write_value(self, value: Value) {
        self.root.borrow_mut().mark_dirty();
        self.reference.write_value(value);
    }

    fn read_native_struct<T, F>(&self, op: F) -> Option<T>
    where
        F: FnOnce(&NativeStructValue) -> Option<T>,
    {
        self.reference.read_native_struct(op)
    }

    fn mutate_native_struct<T, F>(&self, op: F) -> Option<T>
    where
        F: FnOnce(&mut NativeStructValue) -> Option<T>,
    {
        self.root.borrow_mut().mark_dirty();
        self.reference.mutate_native_struct(op)
    }

    fn equals(&self, v2: &GlobalRef) -> VMResult<bool> {
        self.reference.equals(&v2.reference)
    }

    fn equals_ref(&self, v2: &Reference) -> VMResult<bool> {
        self.reference.equals(&v2.0)
    }

    /// Normal code should always know what type this value has. This is made available only for
    /// tests.
    #[allow(non_snake_case)]
    #[doc(hidden)]
    fn to_type_FOR_TESTING(&self) -> Type {
        self.reference.to_type_FOR_TESTING()
    }
}

/// API for locals in a `Frame`.
///
/// Set up of a frame includes the allocation for the vector of locals.
/// The size of the locals is known at compile time, recorded in the binary and verified.
/// When a method is called we read the local count and initialize the vector of locals (via
/// `Locals::new`.
/// We then have 4 operations (bytecodes) that operate on locals: CopyLoc, MoveLoc, StoreLoc and
/// BorrowLoc.
/// Each opcode has an entry point here with a function named after the opcode.
impl Locals {
    /// Create a `Locals` instance containing `size` locals all initialized to Invalid.
    ///
    /// The size of the locals is fixed and immutable. There is no API to grow or shrink the
    /// locals vector.
    pub fn new(size: usize) -> Self {
        Locals(vec![ValueImpl::Invalid; size])
    }

    /// Copy the `Value` at index `idx`. Return an error if index is out of bounds.
    pub fn copy_loc(&self, idx: usize) -> VMResult<Value> {
        if let Some(local_ref) = self.0.get(idx) {
            local_ref.copy_value()
        } else {
            let msg = format!(
                "Index {} out of bounds for {} while indexing {}",
                idx,
                self.0.len(),
                IndexKind::LocalPool
            );
            Err(VMStatus::new(StatusCode::INDEX_OUT_OF_BOUNDS).with_message(msg))
        }
    }

    /// Move the `Value` at index `idx` out of the locals vector. The local is replaced with
    /// `ValueImpl::Invalid`. Return an error if index is out of bounds.
    pub fn move_loc(&mut self, idx: usize) -> VMResult<Value> {
        if let Some(local_ref) = self.0.get_mut(idx) {
            let old_local = replace(local_ref, ValueImpl::Invalid);
            old_local.into_value()
        } else {
            let msg = format!(
                "Index {} out of bounds for {} while indexing {}",
                idx,
                self.0.len(),
                IndexKind::LocalPool
            );
            Err(VMStatus::new(StatusCode::INDEX_OUT_OF_BOUNDS).with_message(msg))
        }
    }

    /// Store `value` in input in the local at index `idx`.
    /// Return an error if index is out of bounds.
    pub fn store_loc(&mut self, idx: usize, value: Value) -> VMResult<()> {
        if let Some(local_ref) = self.0.get_mut(idx) {
            replace(local_ref, value.0);
            Ok(())
        } else {
            let msg = format!(
                "Index {} out of bounds for {} while indexing {}",
                idx,
                self.0.len(),
                IndexKind::LocalPool
            );
            Err(VMStatus::new(StatusCode::INDEX_OUT_OF_BOUNDS).with_message(msg))
        }
    }

    /// Borrow the local at index `idx`.
    /// Return an error if index is out of bounds.
    ///
    /// Current implementation "promotes" a local value to a reference "on the fly".
    /// First call to borrow_global converts the value into a reference
    /// (`ValueImpl::PromotedReference`).
    pub fn borrow_loc(&mut self, idx: usize) -> VMResult<Value> {
        if let Some(local_ref) = self.0.get_mut(idx) {
            match local_ref {
                ValueImpl::GlobalRef(_) | ValueImpl::Reference(_) | ValueImpl::Invalid => {
                    Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR))
                }
                ValueImpl::PromotedReference(reference) => Ok(Value::reference(reference.clone())),
                _ => {
                    let ref_value = MutVal::new(Value::new(local_ref.clone()));
                    let new_local_ref = ValueImpl::PromotedReference(Reference(ref_value.clone()));
                    replace(local_ref, new_local_ref);
                    Ok(Value::reference(Reference(ref_value)))
                }
            }
        } else {
            let msg = format!(
                "Index {} out of bounds for {} while indexing {}",
                idx,
                self.0.len(),
                IndexKind::LocalPool
            );
            Err(VMStatus::new(StatusCode::INDEX_OUT_OF_BOUNDS).with_message(msg))
        }
    }

    // called from test, revisit and delete
    pub fn equals(&self, other: &Locals) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        for (a, b) in self.0.iter().zip(&other.0) {
            match a.equals(b) {
                Ok(res) => {
                    if !res {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }
        true
    }
}

//
// Canonical serialization and deserialization for ValueImpl (via Value)
//

impl Value {
    /// Serialize this value using `SimpleSerializer`.
    pub fn simple_serialize(&self) -> Option<Vec<u8>> {
        SimpleSerializer::<Vec<u8>>::serialize(&self.0).ok()
    }

    /// Deserialize this value using `SimpleDeserializer` and a provided struct definition.
    pub fn simple_deserialize(blob: &[u8], resource: StructDef) -> VMResult<Value> {
        let mut deserializer = SimpleDeserializer::new(blob);
        deserialize_struct(&mut deserializer, &resource)
    }
}

pub(crate) fn deserialize_value(
    deserializer: &mut SimpleDeserializer,
    ty: &Type,
) -> VMResult<Value> {
    match ty {
        Type::Bool => deserializer.decode_bool().map(Value::bool),
        Type::U64 => deserializer.decode_u64().map(Value::u64),
        Type::String => {
            if let Ok(bytes) = deserializer.decode_bytes() {
                if let Ok(s) = VMString::from_utf8(bytes) {
                    return Ok(Value::string(s));
                }
            }
            return Err(vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR));
        }
        Type::ByteArray => deserializer
            .decode_bytes()
            .map(|bytes| Value::byte_array(ByteArray::new(bytes))),
        Type::Address => deserializer
            .decode_bytes()
            .and_then(AccountAddress::try_from)
            .map(Value::address),
        Type::Struct(s_fields) => Ok(deserialize_struct(deserializer, s_fields)?),
        Type::Reference(_) | Type::MutableReference(_) | Type::TypeVariable(_) => {
            // Case TypeVariable is not possible as all type variable has to be materialized before
            // serialization.
            return Err(vm_error(Location::new(), StatusCode::INVALID_DATA));
        }
    }
    .map_err(|_| vm_error(Location::new(), StatusCode::INVALID_DATA))
}

fn deserialize_struct(
    deserializer: &mut SimpleDeserializer,
    struct_def: &StructDef,
) -> VMResult<Value> {
    match struct_def {
        StructDef::Struct(s) => {
            let mut s_vals = Vec::new();
            for field_type in s.field_definitions() {
                s_vals.push(deserialize_value(deserializer, field_type)?);
            }
            Ok(Value::struct_(Struct::new(s_vals)))
        }
        StructDef::Native(ty) => Ok(Value::native_struct(deserialize_native(deserializer, ty)?)),
    }
}

impl CanonicalSerialize for ValueImpl {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        match self {
            ValueImpl::U64(val) => {
                serializer.encode_u64(*val)?;
            }
            ValueImpl::Address(addr) => {
                // TODO: this is serializing as a vector but we want just raw bytes
                // however the AccountAddress story is a bit difficult to work with right now
                serializer.encode_bytes(addr.as_ref())?;
            }
            ValueImpl::Bool(b) => {
                serializer.encode_bool(*b)?;
            }
            ValueImpl::String(s) => {
                // TODO: must define an api for canonical serializations of string.
                // Right now we are just using Rust to serialize the string
                serializer.encode_bytes(s.as_bytes())?;
            }
            ValueImpl::Struct(vals) => {
                for mut_val in &vals.0 {
                    mut_val.peek().serialize(serializer)?;
                }
            }
            ValueImpl::ByteArray(bytearray) => {
                serializer.encode_bytes(bytearray.as_bytes())?;
            }
            ValueImpl::NativeStruct(v) => {
                serializer.encode_struct(v)?;
            }
            _ => unreachable!("invalid type to serialize"),
        }
        Ok(())
    }
}

impl CanonicalSerialize for MutVal {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        self.peek().serialize(serializer)
    }
}
