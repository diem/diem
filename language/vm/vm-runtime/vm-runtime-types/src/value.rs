// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::{struct_def::StructDef, types::Type},
    native_structs::{
        def::NativeStructTag, vector::NativeVector, NativeStructType, NativeStructValue,
    },
};
use libra_types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
    byte_array::ByteArray,
    vm_error::{StatusCode, VMStatus},
};
use serde::{de, Deserialize, Serialize};
use std::{
    cell::{Ref, RefCell},
    fmt,
    mem::replace,
    ops::Add,
    rc::Rc,
};
use vm::file_format::SignatureToken;
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
#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
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
            ValueImpl::Invalid => {
                let msg = "Cannot cast an INVALID value".to_string();
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
            ValueImpl::PromotedReference(reference) => reference.into_value(),
            _ => Ok(Value(self)),
        }
    }

    fn copy_value(&self) -> VMResult<Value> {
        match self {
            ValueImpl::Invalid => {
                let msg = "Cannot cast an INVALID value".to_string();
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
            ValueImpl::PromotedReference(reference) => reference.copy_value(),
            _ => Ok(Value(self.clone())),
        }
    }

    fn borrow_field(&self, field_offset: usize) -> VMResult<Value> {
        match self {
            ValueImpl::Struct(s) => s.get_field_reference(field_offset),
            _ => {
                let msg = format!("Borrow field must be called on a Struct, found {:?}", self);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
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
            _ => {
                let msg = format!("Invalid equality called between {:?} and {:?}", self, v2);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
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

    fn pretty_string(&self) -> String {
        match self {
            ValueImpl::Invalid => "Invalid".to_string(),
            ValueImpl::U64(i) => format!("U64({})", i),
            ValueImpl::Address(addr) => format!("Address({})", addr.short_str()),
            ValueImpl::Bool(b) => format!("Bool({})", b),
            ValueImpl::ByteArray(ba) => format!("ByteArray({})", ba),
            ValueImpl::String(s) => format!("String({})", s),
            ValueImpl::Struct(s) => format!("Struct({})", s.pretty_string()),
            ValueImpl::NativeStruct(v) => format!("NativeStruct({:?})", v),
            ValueImpl::Reference(reference) => format!("Reference({})", reference.pretty_string()),
            ValueImpl::GlobalRef(reference) => format!("GlobalRef({})", reference.pretty_string()),
            ValueImpl::PromotedReference(reference) => {
                format!("PromotedReference({})", reference.pretty_string())
            }
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
    pub fn value_as<T>(self) -> VMResult<T>
    where
        VMResult<T>: From<Value>,
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

    pub fn pretty_string(&self) -> String {
        self.0.pretty_string()
    }

    pub fn is_valid_script_arg(&self, sig: &SignatureToken) -> bool {
        match (sig, &self.0) {
            (SignatureToken::U64, ValueImpl::U64(_)) => true,
            (SignatureToken::Address, ValueImpl::Address(_)) => true,
            (SignatureToken::ByteArray, ValueImpl::ByteArray(_)) => true,
            (SignatureToken::String, ValueImpl::String(_)) => true,
            _ => false,
        }
    }
}

//
// From/Into implementation to read known values off the stack.
// A pop from the stack returns a `Value` that is owned by the caller of pop. For many opcodes
// (e.g. Add) the values popped from the stack are expected to be u64 and should fail otherwise.
//

impl From<Value> for VMResult<u64> {
    fn from(value: Value) -> VMResult<u64> {
        match value.0 {
            ValueImpl::U64(i) => Ok(i),
            _ => {
                let msg = format!("Cannot cast {:?} to u64", value);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }
}

impl From<Value> for VMResult<bool> {
    fn from(value: Value) -> VMResult<bool> {
        match value.0 {
            ValueImpl::Bool(b) => Ok(b),
            _ => {
                let msg = format!("Cannot cast {:?} to bool", value);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }
}

impl From<Value> for VMResult<AccountAddress> {
    fn from(value: Value) -> VMResult<AccountAddress> {
        match value.0 {
            ValueImpl::Address(address) => Ok(address),
            _ => {
                let msg = format!("Cannot cast {:?} to Address", value);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }
}

impl From<Value> for VMResult<ByteArray> {
    fn from(value: Value) -> VMResult<ByteArray> {
        match value.0 {
            ValueImpl::ByteArray(byte_array) => Ok(byte_array),
            _ => {
                let msg = format!("Cannot cast {:?} to ByteArray", value);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }
}

impl From<Value> for VMResult<VMString> {
    fn from(value: Value) -> VMResult<VMString> {
        match value.0 {
            ValueImpl::String(s) => Ok(s),
            _ => {
                let msg = format!("Cannot cast {:?} to String", value);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }
}

impl From<Value> for VMResult<Struct> {
    fn from(value: Value) -> VMResult<Struct> {
        match value.0 {
            ValueImpl::Struct(s) => Ok(s),
            _ => {
                let msg = format!("Cannot cast {:?} to Struct", value);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }
}

impl From<Value> for VMResult<NativeStructValue> {
    fn from(value: Value) -> VMResult<NativeStructValue> {
        match value.0 {
            ValueImpl::NativeStruct(s) => Ok(s),
            _ => {
                let msg = format!("Cannot cast {:?} to NativeStructValue", value);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }
}

impl From<Value> for VMResult<ReferenceValue> {
    fn from(value: Value) -> VMResult<ReferenceValue> {
        match value.0 {
            ValueImpl::Reference(reference) => Ok(ReferenceValue::Reference(reference)),
            ValueImpl::GlobalRef(reference) => Ok(ReferenceValue::GlobalRef(reference)),
            _ => {
                let msg = format!("Cannot cast {:?} to ReferenceValue", value);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }
}

impl From<Value> for VMResult<Reference> {
    fn from(value: Value) -> VMResult<Reference> {
        match value.0 {
            ValueImpl::Reference(reference) => Ok(reference),
            _ => {
                let msg = format!("Cannot cast {:?} to Reference", value);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }
}

impl From<Value> for VMResult<GlobalRef> {
    fn from(value: Value) -> VMResult<GlobalRef> {
        match value.0 {
            ValueImpl::GlobalRef(reference) => Ok(reference),
            _ => {
                let msg = format!("Cannot cast {:?} to GlobalReference", value);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
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

    pub(crate) fn into_value(self) -> VMResult<Value> {
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

    fn mutate_native_struct<T, F>(&self, op: F) -> VMResult<T>
    where
        F: FnOnce(&mut NativeStructValue) -> VMResult<T>,
    {
        match &mut *self.0.borrow_mut() {
            ValueImpl::NativeStruct(s) => op(s),
            _ => {
                let msg = format!("Cannot cast {:?} to NativeStruct", self);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }

    fn read_native_struct<T, F>(&self, op: F) -> VMResult<T>
    where
        F: FnOnce(&NativeStructValue) -> VMResult<T>,
    {
        match &*self.0.borrow_mut() {
            ValueImpl::NativeStruct(s) => op(s),
            _ => {
                let msg = format!("Cannot cast {:?} to NativeStruct", self);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }

    fn pretty_string(&self) -> String {
        self.peek().pretty_string()
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
            let msg = format!("Invalid field at index {} for {:?}", field_offset, self);
            Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
        }
    }

    // Public because of gas synthesis - review.
    pub fn get_field_reference(&self, field_offset: usize) -> VMResult<Value> {
        if let Some(field_ref) = self.0.get(field_offset) {
            Ok(Value::reference(Reference(field_ref.clone())))
        } else {
            let msg = format!("Invalid field at index {} for {:?}", field_offset, self);
            Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
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
            let msg = format!("Equals on different types {:?} for {:?}", self, s2);
            return Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
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

    fn pretty_string(&self) -> String {
        let mut st = "".to_string();
        for field in &self.0 {
            st.push_str(format!("{}, ", field.pretty_string()).as_str());
        }
        st
    }
}

// Private API for a `Reference`. It is jut a pass through layer. All those should disappear
// once compiled
impl Reference {
    // called from cost synthesis, revisit
    pub fn new(value: Value) -> Self {
        Reference(MutVal::new(value))
    }

    fn new_from_cell(val: MutVal) -> Self {
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

    fn mutate_native_struct<T, F>(&self, op: F) -> VMResult<T>
    where
        F: FnOnce(&mut NativeStructValue) -> VMResult<T>,
    {
        self.0.mutate_native_struct(op)
    }

    fn read_native_struct<T, F>(&self, op: F) -> VMResult<T>
    where
        F: FnOnce(&NativeStructValue) -> VMResult<T>,
    {
        self.0.read_native_struct(op)
    }

    fn pretty_string(&self) -> String {
        self.0.pretty_string()
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
            _ => {
                let msg = format!("ReferenceValue must be built from a reference {:?}", value);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
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
    pub(crate) fn mutate_native_struct<T, F>(&self, op: F) -> VMResult<T>
    where
        F: FnOnce(&mut NativeStructValue) -> VMResult<T>,
    {
        match self {
            ReferenceValue::GlobalRef(reference) => reference.mutate_native_struct(op),
            ReferenceValue::Reference(reference) => reference.mutate_native_struct(op),
        }
    }

    pub(crate) fn read_native_struct<T, F>(&self, op: F) -> VMResult<T>
    where
        F: FnOnce(&NativeStructValue) -> VMResult<T>,
    {
        match self {
            ReferenceValue::GlobalRef(reference) => reference.read_native_struct(op),
            ReferenceValue::Reference(reference) => reference.read_native_struct(op),
        }
    }

    pub(crate) fn get_native_struct_reference<F>(&self, op: F) -> VMResult<Value>
    where
        F: FnOnce(&NativeStructValue) -> VMResult<MutVal>,
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

    pub fn mark_dirty(&mut self) {
        self.status = GlobalDataStatus::DIRTY;
    }

    #[allow(dead_code)]
    fn mark_deleted(&mut self) {
        self.status = GlobalDataStatus::DELETED;
    }

    fn pretty_string(&self) -> String {
        format!("{:?}, {:?}", self.status, self.ap)
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

    fn read_native_struct<T, F>(&self, op: F) -> VMResult<T>
    where
        F: FnOnce(&NativeStructValue) -> VMResult<T>,
    {
        self.reference.read_native_struct(op)
    }

    fn mutate_native_struct<T, F>(&self, op: F) -> VMResult<T>
    where
        F: FnOnce(&mut NativeStructValue) -> VMResult<T>,
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

    fn pretty_string(&self) -> String {
        format!(
            "({}, {}), {}",
            Rc::strong_count(&self.root),
            self.root.borrow().pretty_string(),
            self.reference.pretty_string()
        )
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
                    let msg = format!(
                        "BorrowLoc on an invalid local {:?} at index {}",
                        local_ref, idx
                    );
                    Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
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

    pub fn pretty_string(&self) -> String {
        let mut locals = "".to_string();
        for (i, local) in self.0.iter().enumerate() {
            locals.push_str(format!("[{}]: {}\n", i, local.pretty_string()).as_str());
        }
        locals
    }
}

//
// Canonical serialization and deserialization for ValueImpl (via Value)
//

impl Value {
    /// Serialize this value using `SimpleSerializer`.
    pub fn simple_serialize(&self) -> Option<Vec<u8>> {
        lcs::to_bytes(&self.0).ok()
    }

    /// Deserialize this value using `lcs::Deserializer` and a provided struct definition.
    pub fn simple_deserialize(blob: &[u8], resource: StructDef) -> VMResult<Value> {
        lcs::from_bytes_seed(&resource, blob)
            .map_err(|e| VMStatus::new(StatusCode::INVALID_DATA).with_message(e.to_string()))
    }
}

impl<'de> de::DeserializeSeed<'de> for &StructDef {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use de::Error;

        struct StructVisitor<'a>(&'a [Type]);
        impl<'de, 'a> de::Visitor<'de> for StructVisitor<'a> {
            type Value = Struct;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("Struct")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut val = Vec::new();

                for (i, field_type) in self.0.iter().enumerate() {
                    if let Some(elem) = seq.next_element_seed(field_type)? {
                        val.push(elem)
                    } else {
                        return Err(A::Error::invalid_length(i, &self));
                    }
                }
                Ok(Struct::new(val))
            }
        }

        match self {
            StructDef::Struct(s) => {
                let fields = s.field_definitions();
                Ok(Value::struct_(
                    deserializer.deserialize_tuple(fields.len(), StructVisitor(fields))?,
                ))
            }
            StructDef::Native(ty) => Ok(Value::native_struct(ty.deserialize(deserializer)?)),
        }
    }
}

impl<'de> de::DeserializeSeed<'de> for &Type {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use de::Error;

        match self {
            Type::Bool => bool::deserialize(deserializer).map(Value::bool),
            Type::U64 => u64::deserialize(deserializer).map(Value::u64),
            Type::String => VMString::deserialize(deserializer).map(Value::string),
            Type::ByteArray => ByteArray::deserialize(deserializer).map(Value::byte_array),
            Type::Address => AccountAddress::deserialize(deserializer).map(Value::address),
            Type::Struct(s_fields) => s_fields.deserialize(deserializer),
            Type::Reference(_) | Type::MutableReference(_) | Type::TypeVariable(_) => {
                // Case TypeVariable is not possible as all type variable has to be materialized
                // before serialization.
                Err(D::Error::custom(
                    VMStatus::new(StatusCode::INVALID_DATA)
                        .with_message(format!("Value type {:?} not possible", self)),
                ))
            }
        }
    }
}

impl<'de> de::DeserializeSeed<'de> for &NativeStructType {
    type Value = NativeStructValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use de::Error;

        struct NativeVectorVisitor<'a>(&'a Type);
        impl<'de, 'a> de::Visitor<'de> for NativeVectorVisitor<'a> {
            type Value = NativeVector;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("NativeVector")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut val = Vec::new();
                while let Some(elem) = seq.next_element_seed(self.0)? {
                    val.push(MutVal::new(elem))
                }
                Ok(NativeVector(val))
            }
        }

        match self.tag {
            NativeStructTag::Vector => {
                if self.type_actuals().len() != 1 {
                    return Err(D::Error::custom(
                        VMStatus::new(StatusCode::DATA_FORMAT_ERROR)
                            .with_message("NaitiveVector must have uniform types".into()),
                    ));
                };
                let elem_type = &self.type_actuals()[0];
                Ok(NativeStructValue::Vector(
                    deserializer.deserialize_seq(NativeVectorVisitor(elem_type))?,
                ))
            }
        }
    }
}

impl Serialize for ValueImpl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTuple;
        match self {
            ValueImpl::U64(val) => serializer.serialize_u64(*val),
            ValueImpl::Address(addr) => {
                // TODO: this is serializing as a vector but we want just raw bytes
                // however the AccountAddress story is a bit difficult to work with right now
                addr.serialize(serializer)
            }
            ValueImpl::Bool(b) => serializer.serialize_bool(*b),
            ValueImpl::String(s) => {
                // TODO: must define an api for canonical serializations of string.
                // Right now we are just using Rust to serialize the string
                serializer.serialize_bytes(s.as_bytes())
            }
            ValueImpl::Struct(vals) => {
                let mut t = serializer.serialize_tuple(vals.0.len())?;
                for mut_val in &vals.0 {
                    t.serialize_element(&mut_val)?;
                }
                t.end()
            }
            ValueImpl::ByteArray(bytearray) => bytearray.serialize(serializer),
            ValueImpl::NativeStruct(v) => v.serialize(serializer),
            _ => unreachable!("invalid type to serialize"),
        }
    }
}
