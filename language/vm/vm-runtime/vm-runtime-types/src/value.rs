// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::{struct_def::StructDef, types::Type},
    native_structs::{
        def::{NativeStructTag, NativeStructType},
        vector::{NativeVector, VectorElemRef},
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
    cell::{Ref, RefCell, RefMut},
    fmt,
    mem::replace,
    ops::Add,
    rc::Rc,
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
#[derive(Debug, Clone)]
pub(crate) enum ValueImpl {
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

    /// A native vector.
    Vector(NativeVector),

    /// A direct reference to a value behind a RefCell.
    DirectRef(DirectRef),

    /// A reference to a vector element.
    VectorElemRef(VectorElemRef),

    /// A reference to a local.
    ///
    /// This value is used to promote a value into a reference lazily when borrowing a local.
    PromotedReference(MutVal),
}

/// A Move value.
///
/// `Value` is just a wrapper type around `[ValueImpl]` which allows only Move types.
#[derive(Debug, Clone)]
pub struct Value(pub(crate) ValueImpl);

/// Internal representation for a mutable value.
/// This is quite a core type for the mechanics of references.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct MutVal(pub(crate) Rc<RefCell<ValueImpl>>);

/// A struct in Move.
#[derive(Debug, Clone)]
pub struct Struct(Vec<MutVal>);

/// The locals (vector of values) for a `Frame`.
#[derive(Debug, Clone)]
pub struct Locals(Vec<ValueImpl>);

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

/// Wrapper of a reference to the in-memory rust representation of a Move value behind
/// a `RefCell`.
///
/// This acts as a common interface for reading values, which may have different
/// runtime representations under specialization. (Consider a bool stored in locals
/// vs a bool in a native vector.)
///
/// Note: this is a very thin layer of abstraction that will live only on the stack,
/// which is to be differentiated from the Move references.
#[derive(Debug)]
pub(crate) enum VMRef<'a> {
    U64(Ref<'a, u64>),
    Address(Ref<'a, AccountAddress>),
    Bool(Ref<'a, bool>),
    ByteArray(Ref<'a, ByteArray>),
    Struct(Ref<'a, Struct>),
    Vector(Ref<'a, NativeVector>),
}

/// Wrapper of a mutable reference to the in-memory rust representation of a Move value
/// behind a `RefCell`.
///
/// Similar to `VMRef`, but with the additional ability to mutate the value referenced.
#[derive(Debug)]
pub(crate) enum VMRefMut<'a> {
    U64(RefMut<'a, u64>),
    Address(RefMut<'a, AccountAddress>),
    Bool(RefMut<'a, bool>),
    ByteArray(RefMut<'a, ByteArray>),
    Struct(RefMut<'a, Struct>),
    Vector(RefMut<'a, NativeVector>),
}

macro_rules! fn_ref_equals {
    ($ref_ty: ident) => {
        #[allow(dead_code)]
        pub(crate) fn equals(&self, other: &Self) -> VMResult<bool> {
            use $ref_ty::*;

            match (self, other) {
                (U64(r1), U64(r2)) => Ok(**r1 == **r2),
                (Address(r1), Address(r2)) => Ok(**r1 == **r2),
                (Bool(r1), Bool(r2)) => Ok(**r1 == **r2),
                (ByteArray(r1), ByteArray(r2)) => Ok(**r1 == **r2),
                (Struct(r1), Struct(r2)) => r1.equals(r2),
                (Vector(_), Vector(_)) => Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR)
                    .with_message(format!("Equality between vectors aren't allowed"))),
                (r1, r2) => Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR)
                    .with_message(format!("Invalid equality between {:?} and {:?}", r1, r2))),
            }
        }
    };
}

macro_rules! fn_ref_copy_value {
    ($ref_ty: ident) => {
        #[allow(dead_code)]
        pub(crate) fn copy_value(&self) -> Value {
            use $ref_ty::*;

            match self {
                U64(r) => Value::u64(**r),
                Address(r) => Value::address(**r),
                Bool(r) => Value::bool(**r),
                ByteArray(r) => Value::byte_array((*r).clone()),
                Struct(r) => Value::struct_((*r).clone()),
                Vector(r) => Value::native_vector((*r).clone()),
            }
        }
    };
}

impl<'a> VMRef<'a> {
    pub(crate) fn from_value_impl_ref(r: Ref<'a, ValueImpl>) -> VMResult<Self> {
        macro_rules! conv {
            ($ty: ident, $tc: ident, $r: expr) => {{
                let raw = ($r) as *const $ty;
                Ok(VMRef::$tc(Ref::map(r, |_| unsafe { &*raw })))
            }};
        }

        match &*r {
            ValueImpl::Bool(x) => conv!(bool, Bool, x),
            ValueImpl::U64(x) => conv!(u64, U64, x),
            ValueImpl::Address(x) => conv!(AccountAddress, Address, x),
            ValueImpl::ByteArray(x) => conv!(ByteArray, ByteArray, x),
            ValueImpl::Struct(x) => conv!(Struct, Struct, x),
            ValueImpl::Vector(x) => conv!(NativeVector, Vector, x),
            v => {
                let msg = format!("Cannot borrow {:?}", v);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }

    fn_ref_equals!(VMRef);
    fn_ref_copy_value!(VMRef);

    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub(crate) fn to_type_FOR_TESTING(&self) -> Type {
        use VMRef::*;

        match self {
            U64(_) => Type::U64,
            Address(_) => Type::Address,
            Bool(_) => Type::Bool,
            ByteArray(_) => Type::ByteArray,
            Struct(r) => Type::Struct((*r).to_struct_def_FOR_TESTING()),
            Vector(r) => Type::Struct((*r).to_struct_def_FOR_TESTING()),
        }
    }
}

impl<'a> VMRefMut<'a> {
    pub(crate) fn from_value_impl_ref_mut(mut r: RefMut<'a, ValueImpl>) -> VMResult<Self> {
        macro_rules! conv {
            ($ty: ident, $tc: ident, $r: expr) => {{
                let raw = ($r) as *mut $ty;
                Ok(VMRefMut::$tc(RefMut::map(r, |_| unsafe { &mut *raw })))
            }};
        }

        match &mut *r {
            ValueImpl::Bool(x) => conv!(bool, Bool, x),
            ValueImpl::U64(x) => conv!(u64, U64, x),
            ValueImpl::Address(x) => conv!(AccountAddress, Address, x),
            ValueImpl::ByteArray(x) => conv!(ByteArray, ByteArray, x),
            ValueImpl::Struct(x) => conv!(Struct, Struct, x),
            ValueImpl::Vector(x) => conv!(NativeVector, Vector, x),
            v => {
                let msg = format!("Cannot borrow {:?}", v);
                Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
            }
        }
    }

    pub(crate) fn write_value(&mut self, v: Value) -> VMResult<()> {
        use VMRefMut::*;

        match self {
            U64(r) => **r = v.value_as()?,
            Address(r) => **r = v.value_as()?,
            Bool(r) => **r = v.value_as()?,
            ByteArray(r) => **r = v.value_as()?,
            Struct(r) => **r = v.value_as()?,
            Vector(r) => **r = v.value_as()?,
        }

        Ok(())
    }

    fn_ref_equals!(VMRefMut);
    fn_ref_copy_value!(VMRefMut);
}

///
pub(crate) trait FromVMRef {
    fn from_vm_ref(r: VMRef) -> VMResult<Ref<Self>>;
    fn from_vm_ref_mut(r: VMRefMut) -> VMResult<RefMut<Self>>;
}

macro_rules! impl_from_vm_ref {
    ($ty: ident, $tc: ident) => {
        impl FromVMRef for $ty {
            fn from_vm_ref(r: VMRef) -> VMResult<Ref<Self>> {
                match r {
                    VMRef::$tc(r) => Ok(r),
                    _ => {
                        let msg = format!("Cannot use {:?} as {} ref", r, stringify!($ty));
                        Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
                    }
                }
            }

            fn from_vm_ref_mut(r: VMRefMut) -> VMResult<RefMut<Self>> {
                match r {
                    VMRefMut::$tc(r) => Ok(r),
                    _ => {
                        let msg = format!("Cannot use {:?} as {} ref", r, stringify!($ty));
                        Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
                    }
                }
            }
        }
    };
}
impl_from_vm_ref!(u64, U64);
impl_from_vm_ref!(bool, Bool);
impl_from_vm_ref!(AccountAddress, Address);
impl_from_vm_ref!(ByteArray, ByteArray);
impl_from_vm_ref!(Struct, Struct);
impl_from_vm_ref!(NativeVector, Vector);

/// Trait for all sorts of references in the VM, providing a unified set of APIs reading and manipulating
/// VM Values indirectly.
/// Its definition consists of two core operations: `borrow` and `borrow_mut`, with a few
/// auxiliary ones derived from them.
pub(crate) trait Referenceable {
    /// Immutably borrows the value referenced.
    fn borrow(&self) -> VMResult<VMRef>;
    /// Mutably borrows the value referenced.
    fn borrow_mut(&self) -> VMResult<VMRefMut>;

    // Immutably borrows the value referenced as type `T`.
    fn borrow_as<T>(&self) -> VMResult<Ref<T>>
    where
        T: FromVMRef,
    {
        FromVMRef::from_vm_ref(self.borrow()?)
    }

    // Mutably borrows the value referenced as type `T`.
    fn borrow_mut_as<T>(&self) -> VMResult<RefMut<T>>
    where
        T: FromVMRef,
    {
        FromVMRef::from_vm_ref_mut(self.borrow_mut()?)
    }

    // Checks if the referrenced values are equal.
    fn equals<R: Referenceable>(&self, other: &R) -> VMResult<bool> {
        let r1 = self.borrow()?;
        let r2 = other.borrow()?;
        r1.equals(&r2)
    }

    // Returns a copy of the referenced value.
    fn copy_value(&self) -> VMResult<Value> {
        Ok(self.borrow()?.copy_value())
    }

    // Write a value to the location pointed to by the reference.
    fn write_value(&self, v: Value) -> VMResult<()> {
        self.borrow_mut()?.write_value(v)
    }
}

impl Referenceable for MutVal {
    fn borrow(&self) -> VMResult<VMRef> {
        VMRef::from_value_impl_ref(self.0.borrow())
    }

    fn borrow_mut(&self) -> VMResult<VMRefMut> {
        VMRefMut::from_value_impl_ref_mut(self.0.borrow_mut())
    }
}

/// A reference to a value local to the current interpreter execution flow, or
/// in other words, not in the global storage.
///
/// It is simply a passthrough layer for `MutVal`.
#[derive(Debug, Clone)]
pub struct LocalRef(MutVal);

impl Referenceable for LocalRef {
    fn borrow(&self) -> VMResult<VMRef> {
        self.0.borrow()
    }

    fn borrow_mut(&self) -> VMResult<VMRefMut> {
        self.0.borrow_mut()
    }
}

impl LocalRef {
    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        words_in(*REFERENCE_SIZE)
    }

    fn pretty_string(&self) -> String {
        format!("LocalRef({})", self.0.pretty_string())
    }
}

/// A reference to a value in the global storage.
/// It also holds a shared reference to the root access path in order to maintain the
/// status flags.
///
/// It is worth mentioning that, any modification to the value **MUST** go through
/// `borrow_mut`, which sets the status flag to dirty, or otherwise the updated value
/// may not get written back to the storage.
#[derive(Debug, Clone)]
pub struct GlobalRef {
    val: MutVal,
    root: Rc<RefCell<RootAccessPath>>,
}

impl Referenceable for GlobalRef {
    fn borrow(&self) -> VMResult<VMRef> {
        self.val.borrow()
    }

    fn borrow_mut(&self) -> VMResult<VMRefMut> {
        self.root.borrow_mut().mark_dirty();
        self.val.borrow_mut()
    }
}

impl GlobalRef {
    pub fn make_root(ap: AccessPath, value: Value) -> Self {
        Self {
            root: Rc::new(RefCell::new(RootAccessPath::new(ap))),
            val: MutVal::new(value),
        }
    }

    pub fn move_to(ap: AccessPath, value: Struct) -> Self {
        let mut root = RootAccessPath::new(ap);
        root.mark_dirty();
        GlobalRef {
            root: Rc::new(RefCell::new(root)),
            val: MutVal::new(Value::struct_(value)),
        }
    }

    pub fn move_from(&mut self) -> VMResult<Value> {
        self.root.borrow_mut().mark_deleted();
        self.val.copy_value()
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

    // Return the resource behind the reference.
    // If the reference is not exclusively held by the cache (ref count 0) returns None
    pub fn get_data(self) -> Option<Value> {
        match Rc::try_unwrap(self.root) {
            Ok(_) => match Rc::try_unwrap(self.val.0) {
                Ok(res) => Some(Value::new(res.into_inner())),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        words_in(*REFERENCE_SIZE)
    }

    fn pretty_string(&self) -> String {
        format!(
            "GlobalRef(({}, {}), {})",
            Rc::strong_count(&self.root),
            self.root.borrow().pretty_string(),
            self.val.pretty_string()
        )
    }
}

/// A reference that holds a shared pointer to the value being referenced directly.
///
/// `DirectRef` is the only kind of reference that can be extended -- a.k.a creating
/// a reference to a sub component of the value referenced. For example, borrowing a
/// field of a struct or borrowing an element of a vector.
///
/// This means all other references in the VM are indirect -- they all contain a
/// `DirectRef` and some auxiliary info (e.g. an index) to further locate the data
/// behind the shared pointer.
#[derive(Debug, Clone)]
pub enum DirectRef {
    Local(LocalRef),
    Global(GlobalRef),
}

impl DirectRef {
    // TODO: revisit.
    pub fn from_value_for_cost_synthesis(v: Value) -> Self {
        Self::Local(LocalRef(MutVal::new(v)))
    }

    /// Creates a new `DirectRef` pointing to the given shared value, which is a component
    /// of the value the original `DirectRef` points to.
    ///
    /// If the original one is a `GlobalRef`, the root access path is inherited by the component ref.
    pub(crate) fn extend(&self, mv: MutVal) -> Self {
        match self {
            Self::Local(_) => Self::Local(LocalRef(mv)),
            Self::Global(GlobalRef { root, .. }) => Self::Global(GlobalRef {
                root: root.clone(),
                val: mv,
            }),
        }
    }

    /// Borrows a field from the reference if the reference is to a struct.
    pub fn borrow_field(&self, field_offset: usize) -> VMResult<Value> {
        let s = self.borrow_as::<Struct>()?;
        Ok(Value::direct_ref(
            self.extend(s.get_field_reference(field_offset)?),
        ))
    }

    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        match self {
            Self::Local(r) => r.size(),
            Self::Global(r) => r.size(),
        }
    }

    pub(crate) fn pretty_string(&self) -> String {
        match self {
            Self::Local(r) => r.pretty_string(),
            Self::Global(r) => r.pretty_string(),
        }
    }
}

impl Referenceable for DirectRef {
    fn borrow(&self) -> VMResult<VMRef> {
        match self {
            Self::Local(r) => r.borrow(),
            Self::Global(r) => r.borrow(),
        }
    }

    fn borrow_mut(&self) -> VMResult<VMRefMut> {
        match self {
            Self::Local(r) => r.borrow_mut(),
            Self::Global(r) => r.borrow_mut(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Reference {
    DirectRef(DirectRef),
    VectorElemRef(VectorElemRef),
}

/// The wrapper for all types of references in the VM, offering a common API for
/// applications that do not care about the specific type of the reference.
impl Reference {
    pub fn write_ref(&self, value: Value) -> VMResult<()> {
        Referenceable::write_value(self, value)
    }

    pub fn read_ref(&self) -> VMResult<Value> {
        Referenceable::copy_value(self)
    }
}

impl Referenceable for Reference {
    fn borrow(&self) -> VMResult<VMRef> {
        match self {
            Self::DirectRef(r) => r.borrow(),
            Self::VectorElemRef(r) => r.borrow(),
        }
    }

    fn borrow_mut(&self) -> VMResult<VMRefMut> {
        match self {
            Self::DirectRef(r) => r.borrow_mut(),
            Self::VectorElemRef(r) => r.borrow_mut(),
        }
    }
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

    pub(crate) fn size(&self) -> AbstractMemorySize<GasCarrier> {
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
            ValueImpl::Vector(v) => v.size(),

            ValueImpl::DirectRef(r) => r.size(),
            ValueImpl::VectorElemRef(r) => r.size(),

            ValueImpl::PromotedReference(val) => val.size(),
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
            (ValueImpl::DirectRef(r1), ValueImpl::DirectRef(r2)) => r1.equals(r2),
            (ValueImpl::DirectRef(r1), ValueImpl::VectorElemRef(r2)) => r1.equals(r2),
            (ValueImpl::VectorElemRef(r1), ValueImpl::DirectRef(r2)) => r1.equals(r2),
            (ValueImpl::VectorElemRef(r1), ValueImpl::VectorElemRef(r2)) => r1.equals(r2),

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
    pub(crate) fn to_type_FOR_TESTING(&self) -> Type {
        match self {
            ValueImpl::Invalid => unreachable!("Cannot ask type of invalid location"),
            ValueImpl::U64(_) => Type::U64,
            ValueImpl::Address(_) => Type::Address,
            ValueImpl::Bool(_) => Type::Bool,
            ValueImpl::ByteArray(_) => Type::ByteArray,
            ValueImpl::String(_) => Type::String,
            ValueImpl::Struct(s) => Type::Struct(s.to_struct_def_FOR_TESTING()),
            ValueImpl::Vector(v) => Type::Struct(v.to_struct_def_FOR_TESTING()),

            ValueImpl::DirectRef(r) => r.borrow().unwrap().to_type_FOR_TESTING(),
            ValueImpl::VectorElemRef(r) => r.borrow().unwrap().to_type_FOR_TESTING(),

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
            ValueImpl::Vector(v) => format!("NativeVector({:?})", v),

            ValueImpl::DirectRef(r) => format!("DirectRef({})", r.pretty_string()),
            ValueImpl::VectorElemRef(r) => format!("VectorElemRef({})", r.pretty_string()),

            ValueImpl::PromotedReference(reference) => {
                format!("PromotedReference({})", reference.pretty_string())
            }
        }
    }
}

impl Value {
    /// Private internal constructor to make a `Value` from a `ValueImpl`.
    pub(crate) fn new(value: ValueImpl) -> Self {
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

    /// Return a `Value` representing a direct reference in the VM.
    pub fn direct_ref(val: DirectRef) -> Self {
        Value(ValueImpl::DirectRef(val))
    }

    pub(crate) fn vector_elem_ref(r: VectorElemRef) -> Self {
        Value(ValueImpl::VectorElemRef(r))
    }

    pub(crate) fn native_vector(v: NativeVector) -> Self {
        Value(ValueImpl::Vector(v))
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

    /*// called from gas metering, revisit
    pub fn is_global_ref(&self) -> bool {
        match &self.0 {
            ValueImpl::GlobalRef(_) => true,
            _ => false,
        }
    }*/

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
}

//
// From/Into implementation to read known values off the stack.
// A pop from the stack returns a `Value` that is owned by the caller of pop. For many opcodes
// (e.g. Add) the values popped from the stack are expected to be u64 and should fail otherwise.
//
macro_rules! impl_value_cast {
    ($ty: ident, $tc: ident) => {
        impl From<Value> for VMResult<$ty> {
            fn from(value: Value) -> Self {
                match value.0 {
                    ValueImpl::$tc(x) => Ok(x),
                    _ => {
                        let msg = format!("Cannot cast {:?} to {}", value, stringify!($ty));
                        Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
                    }
                }
            }
        }
    };
}
impl_value_cast!(u64, U64);
impl_value_cast!(bool, Bool);
impl_value_cast!(AccountAddress, Address);
impl_value_cast!(ByteArray, ByteArray);
impl_value_cast!(Struct, Struct);
impl_value_cast!(NativeVector, Vector);
impl_value_cast!(DirectRef, DirectRef);
impl_value_cast!(VectorElemRef, VectorElemRef);

impl From<Value> for VMResult<Reference> {
    fn from(value: Value) -> Self {
        match value.0 {
            ValueImpl::DirectRef(r) => Ok(Reference::DirectRef(r)),
            ValueImpl::VectorElemRef(r) => Ok(Reference::VectorElemRef(r)),
            _ => {
                let msg = format!("Cannot cast {:?} to Reference", value);
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

    pub(crate) fn size(&self) -> AbstractMemorySize<GasCarrier> {
        self.peek().size()
    }

    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub(crate) fn to_type_FOR_TESTING(&self) -> Type {
        self.peek().to_type_FOR_TESTING()
    }

    fn equals(&self, v2: &MutVal) -> VMResult<bool> {
        self.peek().equals(&v2.peek())
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
    pub(crate) fn get_field_reference(&self, field_offset: usize) -> VMResult<MutVal> {
        if let Some(field_ref) = self.0.get(field_offset) {
            Ok(field_ref.clone())
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
                ValueImpl::DirectRef(_) | ValueImpl::VectorElemRef(_) | ValueImpl::Invalid => {
                    let msg = format!(
                        "BorrowLoc on an invalid local {:?} at index {}",
                        local_ref, idx
                    );
                    Err(VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg))
                }
                ValueImpl::PromotedReference(mv) => {
                    Ok(Value::direct_ref(DirectRef::Local(LocalRef(mv.clone()))))
                }
                _ => {
                    let ref_value = MutVal::new(Value::new(local_ref.clone()));
                    let new_local_ref = ValueImpl::PromotedReference(ref_value.clone());
                    replace(local_ref, new_local_ref);
                    Ok(Value::direct_ref(DirectRef::Local(LocalRef(ref_value))))
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
            StructDef::Native(ty) => Ok(ty.deserialize(deserializer)?),
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
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use de::Error;

        struct NativeVectorVisitor<'a>(&'a Type);
        impl<'de, 'a> de::Visitor<'de> for NativeVectorVisitor<'a> {
            type Value = NativeVector;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("NativeVector::General")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut val = vec![];
                while let Some(elem) = seq.next_element_seed(self.0)? {
                    val.push(elem.0)
                }
                Ok(NativeVector::General(val))
            }
        }

        struct BoolSeed;
        impl<'de> de::DeserializeSeed<'de> for &BoolSeed {
            type Value = bool;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                bool::deserialize(deserializer)
            }
        }

        struct BoolVectorVisitor;
        impl<'de, 'a> de::Visitor<'de> for BoolVectorVisitor {
            type Value = NativeVector;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("NativeVector::Bool")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut val = vec![];
                while let Some(elem) = seq.next_element_seed(&BoolSeed)? {
                    val.push(elem)
                }
                Ok(NativeVector::Bool(val))
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
                match elem_type {
                    Type::Bool => Ok(Value::native_vector(
                        deserializer.deserialize_seq(BoolVectorVisitor)?,
                    )),
                    _ => Ok(Value::native_vector(
                        deserializer.deserialize_seq(NativeVectorVisitor(elem_type))?,
                    )),
                }
            }
        }
    }
}

impl Serialize for ValueImpl {
    // TODO: serialization should be guided by the types rather than the enum tags of the values themselves.
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
            ValueImpl::Vector(v) => v.serialize(serializer),
            ValueImpl::PromotedReference(mv) => mv.0.borrow().serialize(serializer),
            _ => unreachable!("invalid type to serialize {:?}", self),
        }
    }
}
