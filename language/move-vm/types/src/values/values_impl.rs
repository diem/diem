// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::types::{FatStructType, FatType},
    natives::function::NativeResult,
};
use libra_types::{
    account_address::AccountAddress,
    vm_status::{sub_status::NFE_VECTOR_ERROR_BASE, StatusCode},
};
use move_core_types::{
    gas_schedule::{
        words_in, AbstractMemorySize, GasAlgebra, GasCarrier, GasUnits, CONST_SIZE, REFERENCE_SIZE,
        STRUCT_SIZE,
    },
    value::{MoveKind, MoveKindInfo, MoveStructLayout, MoveTypeLayout},
};
use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::{self, Debug, Display},
    iter,
    mem::size_of,
    ops::Add,
    rc::Rc,
};
use vm::{
    errors::*,
    file_format::{Constant, SignatureToken},
};

/***************************************************************************************
 *
 * Internal Types
 *
 *   Internal representation of the Move value calculus. These types are abstractions
 *   over the concrete Move concepts and may carry additonal information that is not
 *   defined by the language, but required by the implementation.
 *
 **************************************************************************************/

/// Runtime representation of a Move value.
#[derive(Debug)]
enum ValueImpl {
    Invalid,

    U8(u8),
    U64(u64),
    U128(u128),
    Bool(bool),
    Address(AccountAddress),

    Container(Rc<RefCell<Container>>),

    ContainerRef(ContainerRef),
    IndexedRef(IndexedRef),
}

/// A container is a collection of values. It is used to represent data structures like a
/// Move vector or struct.
///
/// There is one general container that can be used to store an array of any values, same
/// type or not, and a few specialized flavors to offer compact memory layout for small
/// primitive types.
///
/// Except when not owned by the VM stack, a container always lives inside an Rc<RefCell<>>,
/// making it possible to be shared by references.
#[derive(Debug)]
enum Container {
    Resource(Vec<ValueImpl>),
    General(Vec<ValueImpl>),
    U8(Vec<u8>),
    U64(Vec<u64>),
    U128(Vec<u128>),
    Bool(Vec<bool>),
    Address(Vec<AccountAddress>),
}

/// A ContainerRef is a direct reference to a container, which could live either in the frame
/// or in global storage. In the latter case, it also keeps a status flag indicating whether
/// the container has been possibly modified.
#[derive(Debug)]
enum ContainerRef {
    Local(Rc<RefCell<Container>>),
    Global {
        status: Rc<RefCell<GlobalDataStatus>>,
        container: Rc<RefCell<Container>>,
    },
}

/// Status for global (on-chain) data:
/// Clean - the data was only read.
/// Dirty - the data was possibly modified.
#[derive(Debug, Clone, Copy)]
enum GlobalDataStatus {
    Clean,
    Dirty,
}

/// A Move reference pointing to an element in a container.
#[derive(Debug)]
struct IndexedRef {
    idx: usize,
    container_ref: ContainerRef,
}

/// An umbrella enum for references. It is used to hide the internals of the public type
/// Reference.
#[derive(Debug)]
enum ReferenceImpl {
    IndexedRef(IndexedRef),
    ContainerRef(ContainerRef),
}

// A reference to a signer. Clients can attempt a cast to this struct if they are
// expecting a Signer on the stavk or as an argument.
#[derive(Debug)]
pub struct SignerRef(ContainerRef);

// A reference to a vector. This is an alias for a ContainerRef for now but we may change
// it once Containers are restructured.
// It's used from vector native functions to get a reference to a vector and operate on that.
// There is an impl for VecotrRef which implements the API private to this module.
#[derive(Debug)]
pub struct VectorRef(ContainerRef);

// A vector. This is an alias for a Container for now but we may change
// it once Containers are restructured.
// It's used from vector native functions to get a vector and operate on that.
// There is an impl for Vecotr which implements the API private to this module.
#[derive(Debug)]
pub struct Vector(Rc<RefCell<Container>>);

/***************************************************************************************
 *
 * Public Types
 *
 *   Types visible from outside the module. They are almost exclusively wrappers around
 *   the internal representation, acting as public interfaces. The methods they provide
 *   closely resemble the Move concepts their names suggest: move_local, borrow_field,
 *   pack, unpack, etc.
 *
 *   They are opaque to an external caller by design -- no knowledge about the internal
 *   representation is given and they can only be manipulated via the public methods,
 *   which is to ensure no arbitratry invalid states can be created unless some crucial
 *   internal invariants are violated.
 *
 **************************************************************************************/

/// A reference to a Move struct that allows you to take a reference to one of its fields.
#[derive(Debug)]
pub struct StructRef(ContainerRef);

/// A generic Move reference that offers two functinalities: read_ref & write_ref.
#[derive(Debug)]
pub struct Reference(ReferenceImpl);

/// A Move value -- a wrapper around `ValueImpl` which can be created only through valid
/// means.
#[derive(Debug)]
pub struct Value(ValueImpl);

/// The locals for a function frame. It allows values to be read, written or taken
/// reference from.
#[derive(Debug)]
pub struct Locals(Rc<RefCell<Container>>);

/// An integer value in Move.
#[derive(Debug)]
pub enum IntegerValue {
    U8(u8),
    U64(u64),
    U128(u128),
}

/// A Move struct.
#[derive(Debug)]
pub struct Struct(Container);

/// A special value that lives in global storage.
///
/// Callers are allowed to take global references from a `GlobalValue`. A global value also contains
/// an internal flag, indicating whether the value has potentially been modified or not.
///
/// For any given value in storage, only one `GlobalValue` may exist to represent it at any time.
/// This means that:
/// * `GlobalValue` **does not** and **cannot** implement `Clone`!
/// * a borrowed reference through `borrow_global` is represented through a `&GlobalValue`.
/// * `borrow_global_mut` is also represented through a `&GlobalValue` -- the bytecode verifier
///   enforces mutability restrictions.
/// * `move_from` is represented through an owned `GlobalValue`.
#[derive(Debug)]
pub struct GlobalValue {
    status: Rc<RefCell<GlobalDataStatus>>,
    container: Rc<RefCell<Container>>,
}

/***************************************************************************************
 *
 * Misc
 *
 *   Miscellaneous helper functions.
 *
 **************************************************************************************/
impl Container {
    fn len(&self) -> usize {
        use Container::*;

        match self {
            Resource(v) | General(v) => v.len(),
            U8(v) => v.len(),
            U64(v) => v.len(),
            U128(v) => v.len(),
            Bool(v) => v.len(),
            Address(v) => v.len(),
        }
    }

    fn is_resource(&self) -> bool {
        use Container::*;

        match self {
            Resource(_) => true,
            General(_) | U8(_) | U64(_) | U128(_) | Address(_) | Bool(_) => false,
        }
    }

    fn signer(x: AccountAddress) -> Self {
        Container::Resource(vec![ValueImpl::Address(x)])
    }
}

impl ValueImpl {
    fn new_container(container: Container) -> Self {
        Self::Container(Rc::new(RefCell::new(container)))
    }

    fn is_resource(&self) -> bool {
        use ValueImpl::*;

        match self {
            Invalid | U8(_) | U64(_) | U128(_) | Address(_) | Bool(_) | ContainerRef(_)
            | IndexedRef(_) => false,

            Container(c) => c.borrow().is_resource(),
        }
    }
}

impl Value {
    // REVIEW: this allows primitive (including `Signer`) and vectors of them. Vectors can only
    // be one level (a different organization of `ValueImpl` could give us that property in an
    // efficient way). Conceptually if we serialize the value and deserialize according to
    // the argument type we would have accomplished a proper check (reference handling aside...)
    pub fn is_valid_arg(&self, sig: &SignatureToken) -> bool {
        match (sig, &self.0) {
            (SignatureToken::U8, ValueImpl::U8(_)) => true,
            (SignatureToken::U64, ValueImpl::U64(_)) => true,
            (SignatureToken::U128, ValueImpl::U128(_)) => true,
            (SignatureToken::Bool, ValueImpl::Bool(_)) => true,
            (SignatureToken::Address, ValueImpl::Address(_)) => true,
            (SignatureToken::Vector(ty), ValueImpl::Container(r)) => match (&**ty, &*r.borrow()) {
                (SignatureToken::Bool, Container::Bool(_))
                | (SignatureToken::U8, Container::U8(_))
                | (SignatureToken::U64, Container::U64(_))
                | (SignatureToken::U128, Container::U128(_))
                | (SignatureToken::Address, Container::Address(_)) => true,
                _ => false,
            },
            (
                SignatureToken::Reference(inner_ty),
                ValueImpl::ContainerRef(ContainerRef::Local(inner_ref)),
            ) => match (&**inner_ty, &*inner_ref.borrow()) {
                (SignatureToken::Signer, Container::Resource(v)) => {
                    v.len() == 1 && matches!(&v[0], ValueImpl::Address(_))
                }
                _ => true,
            },
            _ => false,
        }
    }

    pub fn is_constant_or_signer_ref(&self) -> bool {
        match &self.0 {
            ValueImpl::Bool(_)
            | ValueImpl::U8(_)
            | ValueImpl::U64(_)
            | ValueImpl::U128(_)
            | ValueImpl::Address(_) => true,
            ValueImpl::Container(r) => {
                match &*r.borrow() {
                    Container::Bool(_)
                    | Container::U8(_)
                    | Container::U64(_)
                    | Container::U128(_)
                    | Container::Address(_) => true,
                    // TODO: we need to enable vector of vectors and move Value will change
                    // to have a knowledge of vectors vs structs
                    _ => false,
                }
            }
            ValueImpl::ContainerRef(ContainerRef::Local(inner_ref)) => match &*inner_ref.borrow() {
                Container::Resource(v) => v.len() == 1 && matches!(&v[0], ValueImpl::Address(_)),
                _ => false,
            },
            _ => false,
        }
    }
}

/***************************************************************************************
 *
 * Borrows (Internal)
 *
 *   Helper functions to handle Rust borrows. When borrowing from a RefCell, we want
 *   to return an error instead of panicking.
 *
 **************************************************************************************/

fn take_unique_ownership<T: Debug>(r: Rc<RefCell<T>>) -> PartialVMResult<T> {
    match Rc::try_unwrap(r) {
        Ok(cell) => Ok(cell.into_inner()),
        Err(r) => Err(
            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message(format!("moving value {:?} with dangling references", r)),
        ),
    }
}

impl ContainerRef {
    fn borrow(&self) -> Ref<Container> {
        match self {
            Self::Local(container) | Self::Global { container, .. } => container.borrow(),
        }
    }

    fn borrow_mut(&self) -> RefMut<Container> {
        match self {
            Self::Local(container) => container.borrow_mut(),
            Self::Global { container, status } => {
                *status.borrow_mut() = GlobalDataStatus::Dirty;
                container.borrow_mut()
            }
        }
    }
}

/***************************************************************************************
 *
 * Reference Conversions (Internal)
 *
 *   Helpers to obtain a Rust reference to a value via a VM reference. Required for
 *   equalities.
 *
 **************************************************************************************/
trait VMValueRef<T> {
    fn value_ref(&self) -> PartialVMResult<&T>;
}

macro_rules! impl_vm_value_ref {
    ($ty: ty, $tc: ident) => {
        impl VMValueRef<$ty> for ValueImpl {
            fn value_ref(&self) -> PartialVMResult<&$ty> {
                match self {
                    ValueImpl::$tc(x) => Ok(x),
                    _ => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                        .with_message(format!("cannot take {:?} as &{}", self, stringify!($ty)))),
                }
            }
        }
    };
}

impl_vm_value_ref!(u8, U8);
impl_vm_value_ref!(u64, U64);
impl_vm_value_ref!(u128, U128);
impl_vm_value_ref!(bool, Bool);
impl_vm_value_ref!(AccountAddress, Address);

impl ValueImpl {
    fn as_value_ref<T>(&self) -> PartialVMResult<&T>
    where
        Self: VMValueRef<T>,
    {
        VMValueRef::value_ref(self)
    }
}

/***************************************************************************************
 *
 * Copy Value
 *
 *   Implementation of Move copy. Extra care needs to be taken when copying references.
 *   It is intentional we avoid implementing the standard library trait Clone, to prevent
 *   surprising behaviors from happening.
 *
 **************************************************************************************/
impl ValueImpl {
    fn copy_value(&self) -> PartialVMResult<Self> {
        use ValueImpl::*;

        Ok(match self {
            Invalid => Invalid,

            U8(x) => U8(*x),
            U64(x) => U64(*x),
            U128(x) => U128(*x),
            Bool(x) => Bool(*x),
            Address(x) => Address(*x),

            ContainerRef(r) => ContainerRef(r.copy_value()),
            IndexedRef(r) => IndexedRef(r.copy_value()),

            // When cloning a container, we need to make sure we make a deep
            // copy of the data instead of a shallow copy of the Rc.
            Container(c) => Container(Rc::new(RefCell::new(c.borrow().copy_value()?))),
        })
    }
}

impl Container {
    fn copy_value(&self) -> PartialVMResult<Self> {
        use Container::*;

        Ok(match self {
            General(v) => General(
                v.iter()
                    .map(|x| x.copy_value())
                    .collect::<PartialVMResult<_>>()?,
            ),
            Resource(_) => {
                return Err(
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message("copying a resource".to_string()),
                )
            }
            U8(v) => U8(v.clone()),
            U64(v) => U64(v.clone()),
            U128(v) => U128(v.clone()),
            Bool(v) => Bool(v.clone()),
            Address(v) => Address(v.clone()),
        })
    }
}

impl IndexedRef {
    fn copy_value(&self) -> Self {
        Self {
            idx: self.idx,
            container_ref: self.container_ref.copy_value(),
        }
    }
}

impl ContainerRef {
    fn copy_value(&self) -> Self {
        match self {
            Self::Local(container) => Self::Local(Rc::clone(container)),
            Self::Global { status, container } => Self::Global {
                status: Rc::clone(status),
                container: Rc::clone(container),
            },
        }
    }
}

impl Value {
    pub fn copy_value(&self) -> PartialVMResult<Self> {
        Ok(Self(self.0.copy_value()?))
    }
}

/***************************************************************************************
 *
 * Equality
 *
 *   Equality tests of Move values. Errors are raised when types mismatch.
 *
 *   It is intented to NOT use or even implement the standard library traits Eq and
 *   Partial Eq due to:
 *     1. They do not allow errors to be returned.
 *     2. They can be invoked without the user being noticed thanks to operator
 *        overloading.
 *
 *   Eq and Partial Eq must also NOT be derived for the reasons above plus that the
 *   derived implementation differs from the semantics we want.
 *
 **************************************************************************************/

impl ValueImpl {
    fn equals(&self, other: &Self) -> PartialVMResult<bool> {
        use ValueImpl::*;

        let res = match (self, other) {
            (U8(l), U8(r)) => l == r,
            (U64(l), U64(r)) => l == r,
            (U128(l), U128(r)) => l == r,
            (Bool(l), Bool(r)) => l == r,
            (Address(l), Address(r)) => l == r,

            (Container(l), Container(r)) => l.borrow().equals(&*r.borrow())?,

            (ContainerRef(l), ContainerRef(r)) => l.equals(r)?,
            (IndexedRef(l), IndexedRef(r)) => l.equals(r)?,

            _ => {
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                    .with_message(format!("cannot compare values: {:?}, {:?}", self, other)))
            }
        };

        Ok(res)
    }
}

impl Container {
    fn equals(&self, other: &Self) -> PartialVMResult<bool> {
        use Container::*;

        let res = match (self, other) {
            (Resource(l), Resource(r)) | (General(l), General(r)) => {
                if l.len() != r.len() {
                    return Ok(false);
                }
                for (v1, v2) in l.iter().zip(r.iter()) {
                    if !v1.equals(v2)? {
                        return Ok(false);
                    }
                }
                true
            }
            (U8(l), U8(r)) => l == r,
            (U64(l), U64(r)) => l == r,
            (U128(l), U128(r)) => l == r,
            (Bool(l), Bool(r)) => l == r,
            (Address(l), Address(r)) => l == r,
            _ => {
                return Err(
                    PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(format!(
                        "cannot compare container values: {:?}, {:?}",
                        self, other
                    )),
                )
            }
        };

        Ok(res)
    }
}

impl ContainerRef {
    fn equals(&self, other: &Self) -> PartialVMResult<bool> {
        self.borrow().equals(&*other.borrow())
    }
}

impl IndexedRef {
    fn equals(&self, other: &Self) -> PartialVMResult<bool> {
        use Container::*;

        let res = match (
            &*self.container_ref.borrow(),
            &*other.container_ref.borrow(),
        ) {
            (Resource(v1), Resource(v2))
            | (General(v1), Resource(v2))
            | (Resource(v1), General(v2))
            | (General(v1), General(v2)) => v1[self.idx].equals(&v2[other.idx])?,

            (U8(v1), U8(v2)) => v1[self.idx] == v2[other.idx],
            (U64(v1), U64(v2)) => v1[self.idx] == v2[other.idx],
            (U128(v1), U128(v2)) => v1[self.idx] == v2[other.idx],
            (Bool(v1), Bool(v2)) => v1[self.idx] == v2[other.idx],
            (Address(v1), Address(v2)) => v1[self.idx] == v2[other.idx],

            // Equality between a generic and a specialized container.
            (Resource(v1), U8(v2)) | (General(v1), U8(v2)) => {
                *v1[self.idx].as_value_ref::<u8>()? == v2[other.idx]
            }
            (U8(v1), Resource(v2)) | (U8(v1), General(v2)) => {
                v1[self.idx] == *v2[other.idx].as_value_ref::<u8>()?
            }

            (Resource(v1), U64(v2)) | (General(v1), U64(v2)) => {
                *v1[self.idx].as_value_ref::<u64>()? == v2[other.idx]
            }
            (U64(v1), General(v2)) | (U64(v1), Resource(v2)) => {
                v1[self.idx] == *v2[other.idx].as_value_ref::<u64>()?
            }

            (General(v1), U128(v2)) | (Resource(v1), U128(v2)) => {
                *v1[self.idx].as_value_ref::<u128>()? == v2[other.idx]
            }
            (U128(v1), General(v2)) | (U128(v1), Resource(v2)) => {
                v1[self.idx] == *v2[other.idx].as_value_ref::<u128>()?
            }

            (General(v1), Bool(v2)) | (Resource(v1), Bool(v2)) => {
                *v1[self.idx].as_value_ref::<bool>()? == v2[other.idx]
            }
            (Bool(v1), General(v2)) | (Bool(v1), Resource(v2)) => {
                v1[self.idx] == *v2[other.idx].as_value_ref::<bool>()?
            }

            (General(v1), Address(v2)) | (Resource(v1), Address(v2)) => {
                *v1[self.idx].as_value_ref::<AccountAddress>()? == v2[other.idx]
            }
            (Address(v1), General(v2)) | (Address(v1), Resource(v2)) => {
                v1[self.idx] == *v2[other.idx].as_value_ref::<AccountAddress>()?
            }

            // All other combinations are illegal.
            _ => {
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                    .with_message(format!("cannot compare references {:?}, {:?}", self, other)))
            }
        };
        Ok(res)
    }
}

impl Value {
    pub fn equals(&self, other: &Self) -> PartialVMResult<bool> {
        self.0.equals(&other.0)
    }
}

/***************************************************************************************
 *
 * Read Ref
 *
 *   Implementation of the Move operation read ref.
 *
 **************************************************************************************/

impl ContainerRef {
    fn read_ref(self) -> PartialVMResult<Value> {
        Ok(Value(ValueImpl::new_container(self.borrow().copy_value()?)))
    }
}

impl IndexedRef {
    fn read_ref(self) -> PartialVMResult<Value> {
        use Container::*;

        let res = match &*self.container_ref.borrow() {
            Resource(v) | General(v) => v[self.idx].copy_value()?,
            U8(v) => ValueImpl::U8(v[self.idx]),
            U64(v) => ValueImpl::U64(v[self.idx]),
            U128(v) => ValueImpl::U128(v[self.idx]),
            Bool(v) => ValueImpl::Bool(v[self.idx]),
            Address(v) => ValueImpl::Address(v[self.idx]),
        };

        Ok(Value(res))
    }
}

impl ReferenceImpl {
    fn read_ref(self) -> PartialVMResult<Value> {
        match self {
            Self::ContainerRef(r) => r.read_ref(),
            Self::IndexedRef(r) => r.read_ref(),
        }
    }
}

impl StructRef {
    pub fn read_ref(self) -> PartialVMResult<Value> {
        self.0.read_ref()
    }
}

impl Reference {
    pub fn read_ref(self) -> PartialVMResult<Value> {
        self.0.read_ref()
    }
}

/***************************************************************************************
 *
 * Write Ref
 *
 *   Implementation of the Move operation write ref.
 *
 **************************************************************************************/

impl ContainerRef {
    fn write_ref(self, v: Value) -> PartialVMResult<()> {
        match v.0 {
            ValueImpl::Container(r) => {
                let mut v = self.borrow_mut();
                if v.is_resource() {
                    return Err(
                        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                            .with_message(
                                "resource implicitly destroyed via ContainerRef::write_ref"
                                    .to_string(),
                            ),
                    );
                }
                *v = take_unique_ownership(r)?
                // TODO: can we simply take the Rc?
            }
            _ => {
                return Err(
                    PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(format!(
                        "cannot write value {:?} to container ref {:?}",
                        v, self
                    )),
                )
            }
        }
        Ok(())
    }
}

impl IndexedRef {
    fn write_ref(self, x: Value) -> PartialVMResult<()> {
        match &x.0 {
            ValueImpl::IndexedRef(_)
            | ValueImpl::ContainerRef(_)
            | ValueImpl::Invalid
            | ValueImpl::Container(_) => {
                return Err(
                    PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(format!(
                        "cannot write value {:?} to indexed ref {:?}",
                        x, self
                    )),
                )
            }
            _ => (),
        }

        match (&mut *self.container_ref.borrow_mut(), &x.0) {
            (Container::Resource(v), _) | (Container::General(v), _) => {
                // TODO: do we need to check this for Container::General?
                if v[self.idx].is_resource() {
                    return Err(
                        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                            .with_message(
                                "resource implicitly destroyed via IndexedRef::write_ref"
                                    .to_string(),
                            ),
                    );
                }
                v[self.idx] = x.0;
            }
            (Container::U8(v), ValueImpl::U8(x)) => v[self.idx] = *x,
            (Container::U64(v), ValueImpl::U64(x)) => v[self.idx] = *x,
            (Container::U128(v), ValueImpl::U128(x)) => v[self.idx] = *x,
            (Container::Bool(v), ValueImpl::Bool(x)) => v[self.idx] = *x,
            _ => {
                return Err(
                    PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(format!(
                        "cannot write value {:?} to indexed ref {:?}",
                        x, self
                    )),
                )
            }
        }
        Ok(())
    }
}

impl ReferenceImpl {
    fn write_ref(self, x: Value) -> PartialVMResult<()> {
        match self {
            Self::ContainerRef(r) => r.write_ref(x),
            Self::IndexedRef(r) => r.write_ref(x),
        }
    }
}

impl Reference {
    pub fn write_ref(self, x: Value) -> PartialVMResult<()> {
        self.0.write_ref(x)
    }
}

/***************************************************************************************
 *
 * Borrows (Move)
 *
 *   Implementation of borrowing in Move: borrow field, borrow local and infrastructure
 *   to support borrowing an element from a vector.
 *
 **************************************************************************************/

impl ContainerRef {
    fn borrow_elem(&self, idx: usize) -> PartialVMResult<ValueImpl> {
        let r = self.borrow();

        if idx >= r.len() {
            return Err(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(
                    format!(
                        "index out of bounds when borrowing container element: got: {}, len: {}",
                        idx,
                        r.len()
                    ),
                ),
            );
        }

        let res = match &*r {
            Container::Resource(v) | Container::General(v) => match &v[idx] {
                // TODO: check for the impossible combinations.
                ValueImpl::Container(container) => {
                    let r = match self {
                        Self::Local(_) => Self::Local(Rc::clone(container)),
                        Self::Global { status, .. } => Self::Global {
                            status: Rc::clone(status),
                            container: Rc::clone(container),
                        },
                    };
                    ValueImpl::ContainerRef(r)
                }
                _ => ValueImpl::IndexedRef(IndexedRef {
                    idx,
                    container_ref: self.copy_value(),
                }),
            },

            Container::U8(_)
            | Container::U64(_)
            | Container::U128(_)
            | Container::Address(_)
            | Container::Bool(_) => ValueImpl::IndexedRef(IndexedRef {
                idx,
                container_ref: self.copy_value(),
            }),
        };

        Ok(res)
    }
}

impl StructRef {
    pub fn borrow_field(&self, idx: usize) -> PartialVMResult<Value> {
        Ok(Value(self.0.borrow_elem(idx)?))
    }
}

impl Locals {
    pub fn borrow_loc(&self, idx: usize) -> PartialVMResult<Value> {
        // TODO: this is very similar to SharedContainer::borrow_elem. Find a way to
        // reuse that code?

        let r = self.0.borrow();

        if idx >= r.len() {
            return Err(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(
                    format!(
                        "index out of bounds when borrowing local: got: {}, len: {}",
                        idx,
                        r.len()
                    ),
                ),
            );
        }

        match &*r {
            Container::General(v) => match &v[idx] {
                ValueImpl::Container(r) => Ok(Value(ValueImpl::ContainerRef(ContainerRef::Local(
                    Rc::clone(r),
                )))),

                ValueImpl::U8(_)
                | ValueImpl::U64(_)
                | ValueImpl::U128(_)
                | ValueImpl::Bool(_)
                | ValueImpl::Address(_) => Ok(Value(ValueImpl::IndexedRef(IndexedRef {
                    container_ref: ContainerRef::Local(Rc::clone(&self.0)),
                    idx,
                }))),

                ValueImpl::ContainerRef(_) | ValueImpl::Invalid | ValueImpl::IndexedRef(_) => Err(
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!("cannot borrow local {:?}", &v[idx])),
                ),
            },
            Container::Resource(_)
            | Container::U8(_)
            | Container::U64(_)
            | Container::U128(_)
            | Container::Address(_)
            | Container::Bool(_) => Err(PartialVMError::new(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            )
            .with_message(format!("bad container for locals: {:?}", &*r))),
        }
    }
}

impl SignerRef {
    pub fn borrow_signer(&self) -> PartialVMResult<Value> {
        Ok(Value(self.0.borrow_elem(0)?))
    }
}

/***************************************************************************************
 *
 * Locals
 *
 *   Public APIs for Locals to support reading, writing and moving of values.
 *
 **************************************************************************************/
impl Locals {
    pub fn new(n: usize) -> Self {
        Self(Rc::new(RefCell::new(Container::General(
            iter::repeat_with(|| ValueImpl::Invalid).take(n).collect(),
        ))))
    }

    pub fn check_resources_for_return(&self) -> PartialVMResult<()> {
        match &*self.0.borrow() {
            Container::General(vals) => {
                for v in vals {
                    if v.is_resource() {
                        return Err(PartialVMError::new(
                            StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
                        )
                        .with_message(
                            "resource implicitly destroyed when returnining".to_string(),
                        ));
                    }
                }
                Ok(())
            }

            Container::Resource(_)
            | Container::U8(_)
            | Container::U64(_)
            | Container::U128(_)
            | Container::Address(_)
            | Container::Bool(_) => unreachable!(),
        }
    }

    pub fn copy_loc(&self, idx: usize) -> PartialVMResult<Value> {
        let r = self.0.borrow();
        let v = match &*r {
            Container::General(v) => v,

            Container::Resource(_)
            | Container::U8(_)
            | Container::U64(_)
            | Container::U128(_)
            | Container::Address(_)
            | Container::Bool(_) => unreachable!(),
        };

        match v.get(idx) {
            Some(ValueImpl::Invalid) => Err(PartialVMError::new(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            )
            .with_message(format!("cannot copy invalid value at index {}", idx))),
            Some(v) => Ok(Value(v.copy_value()?)),
            None => Err(
                PartialVMError::new(StatusCode::VERIFIER_INVARIANT_VIOLATION).with_message(
                    format!("local index out of bounds: got {}, len: {}", idx, v.len()),
                ),
            ),
        }
    }

    fn swap_loc(&mut self, idx: usize, x: Value) -> PartialVMResult<Value> {
        let mut r = self.0.borrow_mut();
        let v = match &mut *r {
            Container::General(v) => v,

            Container::Resource(_)
            | Container::U8(_)
            | Container::U64(_)
            | Container::U128(_)
            | Container::Address(_)
            | Container::Bool(_) => unreachable!(),
        };

        match v.get_mut(idx) {
            Some(v) => {
                if let ValueImpl::Container(r) = v {
                    if Rc::strong_count(r) > 1 {
                        return Err(PartialVMError::new(
                            StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
                        )
                        .with_message("moving container with dangling references".to_string()));
                    }
                }
                Ok(Value(std::mem::replace(v, x.0)))
            }
            None => Err(
                PartialVMError::new(StatusCode::VERIFIER_INVARIANT_VIOLATION).with_message(
                    format!("local index out of bounds: got {}, len: {}", idx, v.len()),
                ),
            ),
        }
    }

    pub fn move_loc(&mut self, idx: usize) -> PartialVMResult<Value> {
        match self.swap_loc(idx, Value(ValueImpl::Invalid))? {
            Value(ValueImpl::Invalid) => Err(PartialVMError::new(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            )
            .with_message(format!("cannot move invalid value at index {}", idx))),
            v => Ok(v),
        }
    }

    pub fn store_loc(&mut self, idx: usize, x: Value) -> PartialVMResult<()> {
        if self.swap_loc(idx, x)?.0.is_resource() {
            return Err(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message("resource implicitly destroyed via store_loc".to_string()),
            );
        }
        Ok(())
    }
}

/***************************************************************************************
 *
 * Public Value Constructors
 *
 *   Constructors to allow values to be created outside this module.
 *
 **************************************************************************************/
impl Value {
    pub fn u8(x: u8) -> Self {
        Self(ValueImpl::U8(x))
    }

    pub fn u64(x: u64) -> Self {
        Self(ValueImpl::U64(x))
    }

    pub fn u128(x: u128) -> Self {
        Self(ValueImpl::U128(x))
    }

    pub fn bool(x: bool) -> Self {
        Self(ValueImpl::Bool(x))
    }

    pub fn address(x: AccountAddress) -> Self {
        Self(ValueImpl::Address(x))
    }

    pub fn signer(x: AccountAddress) -> Self {
        Self(ValueImpl::new_container(Container::signer(x)))
    }

    /// Create a "unowned" reference to a signer value (&signer) for populating the &signer in a
    /// transaction script
    pub fn transaction_argument_signer_reference(x: AccountAddress) -> Self {
        Self(ValueImpl::ContainerRef(ContainerRef::Local(Rc::new(
            RefCell::new(Container::signer(x)),
        ))))
    }

    pub fn struct_(s: Struct) -> Self {
        Self(ValueImpl::new_container(s.0))
    }

    // TODO: consider whether we want to replace these with fn vector(v: Vec<Value>).
    pub fn vector_u8(it: impl IntoIterator<Item = u8>) -> Self {
        Self(ValueImpl::new_container(Container::U8(
            it.into_iter().collect(),
        )))
    }

    pub fn vector_u64(it: impl IntoIterator<Item = u64>) -> Self {
        Self(ValueImpl::new_container(Container::U64(
            it.into_iter().collect(),
        )))
    }

    pub fn vector_u128(it: impl IntoIterator<Item = u128>) -> Self {
        Self(ValueImpl::new_container(Container::U128(
            it.into_iter().collect(),
        )))
    }

    pub fn vector_bool(it: impl IntoIterator<Item = bool>) -> Self {
        Self(ValueImpl::new_container(Container::Bool(
            it.into_iter().collect(),
        )))
    }

    pub fn vector_address(it: impl IntoIterator<Item = AccountAddress>) -> Self {
        Self(ValueImpl::new_container(Container::Address(
            it.into_iter().collect(),
        )))
    }

    pub fn vector_general(it: impl IntoIterator<Item = Value>) -> Self {
        Self(ValueImpl::new_container(Container::General(
            it.into_iter().map(|v| v.0).collect(),
        )))
    }
}

/***************************************************************************************
 *
 * Casting
 *
 *   Due to the public value types being opaque to an external user, the following
 *   public APIs are required to enable conversion between types in order to gain access
 *   to specific operations certain more refined types offer.
 *   For example, one must convert a `Value` to a `Struct` before unpack can be called.
 *
 *   It is expected that the caller will keep track of the invariants and guarantee
 *   the conversion will succeed. An error will be raised in case of a violation.
 *
 **************************************************************************************/
pub trait VMValueCast<T> {
    fn cast(self) -> PartialVMResult<T>;
}

macro_rules! impl_vm_value_cast {
    ($ty: ty, $tc: ident) => {
        impl VMValueCast<$ty> for Value {
            fn cast(self) -> PartialVMResult<$ty> {
                match self.0 {
                    ValueImpl::$tc(x) => Ok(x),
                    v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                        .with_message(format!("cannot cast {:?} to {}", v, stringify!($ty)))),
                }
            }
        }
    };
}

impl_vm_value_cast!(u8, U8);
impl_vm_value_cast!(u64, U64);
impl_vm_value_cast!(u128, U128);
impl_vm_value_cast!(bool, Bool);
impl_vm_value_cast!(AccountAddress, Address);
impl_vm_value_cast!(ContainerRef, ContainerRef);
impl_vm_value_cast!(IndexedRef, IndexedRef);

impl VMValueCast<IntegerValue> for Value {
    fn cast(self) -> PartialVMResult<IntegerValue> {
        match self.0 {
            ValueImpl::U8(x) => Ok(IntegerValue::U8(x)),
            ValueImpl::U64(x) => Ok(IntegerValue::U64(x)),
            ValueImpl::U128(x) => Ok(IntegerValue::U128(x)),
            v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot cast {:?} to integer", v,))),
        }
    }
}

impl VMValueCast<Reference> for Value {
    fn cast(self) -> PartialVMResult<Reference> {
        match self.0 {
            ValueImpl::ContainerRef(r) => Ok(Reference(ReferenceImpl::ContainerRef(r))),
            ValueImpl::IndexedRef(r) => Ok(Reference(ReferenceImpl::IndexedRef(r))),
            v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot cast {:?} to reference", v,))),
        }
    }
}

impl VMValueCast<Container> for Value {
    fn cast(self) -> PartialVMResult<Container> {
        match self.0 {
            ValueImpl::Container(r) => take_unique_ownership(r),
            v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot cast {:?} to container", v,))),
        }
    }
}

impl VMValueCast<Vec<Value>> for Value {
    fn cast(self) -> PartialVMResult<Vec<Value>> {
        match self.0 {
            ValueImpl::Container(r) => Ok(match take_unique_ownership(r)? {
                Container::Resource(vs) | Container::General(vs) => {
                    vs.into_iter().map(Value).collect()
                }
                Container::U8(vs) => vs.into_iter().map(Value::u8).collect(),
                Container::U64(vs) => vs.into_iter().map(Value::u64).collect(),
                Container::U128(vs) => vs.into_iter().map(Value::u128).collect(),
                Container::Bool(vs) => vs.into_iter().map(Value::bool).collect(),
                Container::Address(vs) => vs.into_iter().map(Value::address).collect(),
            }),
            v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot cast {:?} to vector of values", v,))),
        }
    }
}

impl VMValueCast<Struct> for Value {
    fn cast(self) -> PartialVMResult<Struct> {
        match self.0 {
            ValueImpl::Container(r) => Ok(Struct(take_unique_ownership(r)?)),
            v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot cast {:?} to struct", v,))),
        }
    }
}

impl VMValueCast<StructRef> for Value {
    fn cast(self) -> PartialVMResult<StructRef> {
        Ok(StructRef(VMValueCast::cast(self)?))
    }
}

impl VMValueCast<Vec<u8>> for Value {
    fn cast(self) -> PartialVMResult<Vec<u8>> {
        match self.0 {
            ValueImpl::Container(r) => match take_unique_ownership(r)? {
                Container::U8(v) => Ok(v),
                v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                    .with_message(format!("cannot cast {:?} to vector<u8>", v,))),
            },
            v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot cast {:?} to vector<u8>", v,))),
        }
    }
}

impl VMValueCast<SignerRef> for Value {
    fn cast(self) -> PartialVMResult<SignerRef> {
        match self.0 {
            ValueImpl::ContainerRef(r) => Ok(SignerRef(r)),
            v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot cast {:?} to Signer reference", v,))),
        }
    }
}

impl VMValueCast<VectorRef> for Value {
    fn cast(self) -> PartialVMResult<VectorRef> {
        match self.0 {
            ValueImpl::ContainerRef(r) => Ok(VectorRef(r)),
            v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot cast {:?} to vector reference", v,))),
        }
    }
}

impl VMValueCast<Vector> for Value {
    fn cast(self) -> PartialVMResult<Vector> {
        match self.0 {
            ValueImpl::Container(c) => Ok(Vector(c)),
            v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot cast {:?} to vector", v,))),
        }
    }
}

impl Value {
    pub fn value_as<T>(self) -> PartialVMResult<T>
    where
        Self: VMValueCast<T>,
    {
        VMValueCast::cast(self)
    }
}

impl VMValueCast<u8> for IntegerValue {
    fn cast(self) -> PartialVMResult<u8> {
        match self {
            Self::U8(x) => Ok(x),
            v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot cast {:?} to u8", v,))),
        }
    }
}

impl VMValueCast<u64> for IntegerValue {
    fn cast(self) -> PartialVMResult<u64> {
        match self {
            Self::U64(x) => Ok(x),
            v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot cast {:?} to u64", v,))),
        }
    }
}

impl VMValueCast<u128> for IntegerValue {
    fn cast(self) -> PartialVMResult<u128> {
        match self {
            Self::U128(x) => Ok(x),
            v => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot cast {:?} to u128", v,))),
        }
    }
}

impl IntegerValue {
    pub fn value_as<T>(self) -> PartialVMResult<T>
    where
        Self: VMValueCast<T>,
    {
        VMValueCast::cast(self)
    }
}

/***************************************************************************************
 *
 * Integer Operations
 *
 *   Arithmetic operations and conversions for integer values.
 *
 **************************************************************************************/
impl IntegerValue {
    pub fn add_checked(self, other: Self) -> PartialVMResult<Self> {
        use IntegerValue::*;
        let res = match (self, other) {
            (U8(l), U8(r)) => u8::checked_add(l, r).map(IntegerValue::U8),
            (U64(l), U64(r)) => u64::checked_add(l, r).map(IntegerValue::U64),
            (U128(l), U128(r)) => u128::checked_add(l, r).map(IntegerValue::U128),
            (l, r) => {
                let msg = format!("Cannot add {:?} and {:?}", l, r);
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
            }
        };
        res.ok_or_else(|| PartialVMError::new(StatusCode::ARITHMETIC_ERROR))
    }

    pub fn sub_checked(self, other: Self) -> PartialVMResult<Self> {
        use IntegerValue::*;
        let res = match (self, other) {
            (U8(l), U8(r)) => u8::checked_sub(l, r).map(IntegerValue::U8),
            (U64(l), U64(r)) => u64::checked_sub(l, r).map(IntegerValue::U64),
            (U128(l), U128(r)) => u128::checked_sub(l, r).map(IntegerValue::U128),
            (l, r) => {
                let msg = format!("Cannot sub {:?} from {:?}", r, l);
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
            }
        };
        res.ok_or_else(|| PartialVMError::new(StatusCode::ARITHMETIC_ERROR))
    }

    pub fn mul_checked(self, other: Self) -> PartialVMResult<Self> {
        use IntegerValue::*;
        let res = match (self, other) {
            (U8(l), U8(r)) => u8::checked_mul(l, r).map(IntegerValue::U8),
            (U64(l), U64(r)) => u64::checked_mul(l, r).map(IntegerValue::U64),
            (U128(l), U128(r)) => u128::checked_mul(l, r).map(IntegerValue::U128),
            (l, r) => {
                let msg = format!("Cannot mul {:?} and {:?}", l, r);
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
            }
        };
        res.ok_or_else(|| PartialVMError::new(StatusCode::ARITHMETIC_ERROR))
    }

    pub fn div_checked(self, other: Self) -> PartialVMResult<Self> {
        use IntegerValue::*;
        let res = match (self, other) {
            (U8(l), U8(r)) => u8::checked_div(l, r).map(IntegerValue::U8),
            (U64(l), U64(r)) => u64::checked_div(l, r).map(IntegerValue::U64),
            (U128(l), U128(r)) => u128::checked_div(l, r).map(IntegerValue::U128),
            (l, r) => {
                let msg = format!("Cannot div {:?} by {:?}", l, r);
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
            }
        };
        res.ok_or_else(|| PartialVMError::new(StatusCode::ARITHMETIC_ERROR))
    }

    pub fn rem_checked(self, other: Self) -> PartialVMResult<Self> {
        use IntegerValue::*;
        let res = match (self, other) {
            (U8(l), U8(r)) => u8::checked_rem(l, r).map(IntegerValue::U8),
            (U64(l), U64(r)) => u64::checked_rem(l, r).map(IntegerValue::U64),
            (U128(l), U128(r)) => u128::checked_rem(l, r).map(IntegerValue::U128),
            (l, r) => {
                let msg = format!("Cannot rem {:?} by {:?}", l, r);
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
            }
        };
        res.ok_or_else(|| PartialVMError::new(StatusCode::ARITHMETIC_ERROR))
    }

    pub fn bit_or(self, other: Self) -> PartialVMResult<Self> {
        use IntegerValue::*;
        Ok(match (self, other) {
            (U8(l), U8(r)) => IntegerValue::U8(l | r),
            (U64(l), U64(r)) => IntegerValue::U64(l | r),
            (U128(l), U128(r)) => IntegerValue::U128(l | r),
            (l, r) => {
                let msg = format!("Cannot bit_or {:?} and {:?}", l, r);
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
            }
        })
    }

    pub fn bit_and(self, other: Self) -> PartialVMResult<Self> {
        use IntegerValue::*;
        Ok(match (self, other) {
            (U8(l), U8(r)) => IntegerValue::U8(l & r),
            (U64(l), U64(r)) => IntegerValue::U64(l & r),
            (U128(l), U128(r)) => IntegerValue::U128(l & r),
            (l, r) => {
                let msg = format!("Cannot bit_and {:?} and {:?}", l, r);
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
            }
        })
    }

    pub fn bit_xor(self, other: Self) -> PartialVMResult<Self> {
        use IntegerValue::*;
        Ok(match (self, other) {
            (U8(l), U8(r)) => IntegerValue::U8(l ^ r),
            (U64(l), U64(r)) => IntegerValue::U64(l ^ r),
            (U128(l), U128(r)) => IntegerValue::U128(l ^ r),
            (l, r) => {
                let msg = format!("Cannot bit_xor {:?} and {:?}", l, r);
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
            }
        })
    }

    pub fn shl_checked(self, n_bits: u8) -> PartialVMResult<Self> {
        use IntegerValue::*;

        Ok(match self {
            U8(x) => {
                if n_bits >= 8 {
                    return Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR));
                }
                IntegerValue::U8(x << n_bits)
            }
            U64(x) => {
                if n_bits >= 64 {
                    return Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR));
                }
                IntegerValue::U64(x << n_bits)
            }
            U128(x) => {
                if n_bits >= 128 {
                    return Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR));
                }
                IntegerValue::U128(x << n_bits)
            }
        })
    }

    pub fn shr_checked(self, n_bits: u8) -> PartialVMResult<Self> {
        use IntegerValue::*;

        Ok(match self {
            U8(x) => {
                if n_bits >= 8 {
                    return Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR));
                }
                IntegerValue::U8(x >> n_bits)
            }
            U64(x) => {
                if n_bits >= 64 {
                    return Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR));
                }
                IntegerValue::U64(x >> n_bits)
            }
            U128(x) => {
                if n_bits >= 128 {
                    return Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR));
                }
                IntegerValue::U128(x >> n_bits)
            }
        })
    }

    pub fn lt(self, other: Self) -> PartialVMResult<bool> {
        use IntegerValue::*;

        Ok(match (self, other) {
            (U8(l), U8(r)) => l < r,
            (U64(l), U64(r)) => l < r,
            (U128(l), U128(r)) => l < r,
            (l, r) => {
                let msg = format!(
                    "Cannot compare {:?} and {:?}: incompatible integer types",
                    l, r
                );
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
            }
        })
    }

    pub fn le(self, other: Self) -> PartialVMResult<bool> {
        use IntegerValue::*;

        Ok(match (self, other) {
            (U8(l), U8(r)) => l <= r,
            (U64(l), U64(r)) => l <= r,
            (U128(l), U128(r)) => l <= r,
            (l, r) => {
                let msg = format!(
                    "Cannot compare {:?} and {:?}: incompatible integer types",
                    l, r
                );
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
            }
        })
    }

    pub fn gt(self, other: Self) -> PartialVMResult<bool> {
        use IntegerValue::*;

        Ok(match (self, other) {
            (U8(l), U8(r)) => l > r,
            (U64(l), U64(r)) => l > r,
            (U128(l), U128(r)) => l > r,
            (l, r) => {
                let msg = format!(
                    "Cannot compare {:?} and {:?}: incompatible integer types",
                    l, r
                );
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
            }
        })
    }

    pub fn ge(self, other: Self) -> PartialVMResult<bool> {
        use IntegerValue::*;

        Ok(match (self, other) {
            (U8(l), U8(r)) => l >= r,
            (U64(l), U64(r)) => l >= r,
            (U128(l), U128(r)) => l >= r,
            (l, r) => {
                let msg = format!(
                    "Cannot compare {:?} and {:?}: incompatible integer types",
                    l, r
                );
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(msg));
            }
        })
    }

    pub fn into_value(self) -> Value {
        use IntegerValue::*;

        match self {
            U8(x) => Value::u8(x),
            U64(x) => Value::u64(x),
            U128(x) => Value::u128(x),
        }
    }
}

impl IntegerValue {
    pub fn cast_u8(self) -> PartialVMResult<u8> {
        use IntegerValue::*;

        match self {
            U8(x) => Ok(x),
            U64(x) => {
                if x > (std::u8::MAX as u64) {
                    Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR)
                        .with_message(format!("Cannot cast u64({}) to u8", x)))
                } else {
                    Ok(x as u8)
                }
            }
            U128(x) => {
                if x > (std::u8::MAX as u128) {
                    Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR)
                        .with_message(format!("Cannot cast u128({}) to u8", x)))
                } else {
                    Ok(x as u8)
                }
            }
        }
    }

    pub fn cast_u64(self) -> PartialVMResult<u64> {
        use IntegerValue::*;

        match self {
            U8(x) => Ok(x as u64),
            U64(x) => Ok(x),
            U128(x) => {
                if x > (std::u64::MAX as u128) {
                    Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR)
                        .with_message(format!("Cannot cast u128({}) to u64", x)))
                } else {
                    Ok(x as u64)
                }
            }
        }
    }

    pub fn cast_u128(self) -> PartialVMResult<u128> {
        use IntegerValue::*;

        Ok(match self {
            U8(x) => x as u128,
            U64(x) => x as u128,
            U128(x) => x,
        })
    }
}

/***************************************************************************************
*
* Vector
*
*   Native function imeplementations of the Vector module.
*
*   TODO: split the code into two parts:
*         1) Internal vector APIs that define & implements the core operations
             (and operations only).
*         2) Native function adapters that the dispatcher can call into. These will
*            check if arguments are valid and deal with gas metering.
*
**************************************************************************************/

pub const INDEX_OUT_OF_BOUNDS: u64 = NFE_VECTOR_ERROR_BASE + 1;
pub const POP_EMPTY_VEC: u64 = NFE_VECTOR_ERROR_BASE + 2;
pub const DESTROY_NON_EMPTY_VEC: u64 = NFE_VECTOR_ERROR_BASE + 3;

fn check_elem_layout(
    context: &impl NativeContext,
    ty: &Type,
    v: &Container,
) -> PartialVMResult<()> {
    let is_resource = context.is_resource(ty)?;

    match (ty, v) {
        (Type::U8, Container::U8(_))
        | (Type::U64, Container::U64(_))
        | (Type::U128, Container::U128(_))
        | (Type::Bool, Container::Bool(_))
        | (Type::Address, Container::Address(_))
        | (Type::Signer, Container::Resource(_))
        | (Type::Vector(_), Container::General(_)) => Ok(()),

        (Type::Struct(_), Container::General(_))
        | (Type::StructInstantiation(_, _), Container::General(_))
            if !is_resource =>
        {
            Ok(())
        }

        (Type::Struct(_), Container::Resource(_))
        | (Type::StructInstantiation(_, _), Container::Resource(_))
            if is_resource =>
        {
            Ok(())
        }

        (Type::Reference(_), _) | (Type::MutableReference(_), _) | (Type::TyParam(_), _) => Err(
            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message(format!("invalid type param for vector: {:?}", ty)),
        ),

        (Type::U8, _)
        | (Type::U64, _)
        | (Type::U128, _)
        | (Type::Bool, _)
        | (Type::Address, _)
        | (Type::Signer, _)
        | (Type::Vector(_), _)
        | (Type::Struct(_), _)
        | (Type::StructInstantiation(_, _), _) => Err(PartialVMError::new(
            StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
        )
        .with_message(format!(
            "vector elem layout mismatch, expected {:?}, got {:?}",
            ty, v
        ))),
    }
}

impl VectorRef {
    pub fn len(&self, type_param: &Type, context: &impl NativeContext) -> PartialVMResult<Value> {
        let v = self.0.borrow();
        check_elem_layout(context, type_param, &*v)?;

        let len = match &*v {
            Container::U8(v) => v.len(),
            Container::U64(v) => v.len(),
            Container::U128(v) => v.len(),
            Container::Bool(v) => v.len(),
            Container::Address(v) => v.len(),
            Container::Resource(v) | Container::General(v) => v.len(),
        };
        Ok(Value::u64(len as u64))
    }

    pub fn push_back(
        &self,
        e: Value,
        type_param: &Type,
        context: &impl NativeContext,
    ) -> PartialVMResult<()> {
        let mut v = self.0.borrow_mut();
        check_elem_layout(context, type_param, &*v)?;

        match &mut *v {
            Container::U8(v) => v.push(e.value_as()?),
            Container::U64(v) => v.push(e.value_as()?),
            Container::U128(v) => v.push(e.value_as()?),
            Container::Bool(v) => v.push(e.value_as()?),
            Container::Address(v) => v.push(e.value_as()?),
            Container::Resource(v) | Container::General(v) => v.push(e.0),
        }
        Ok(())
    }

    pub fn borrow_elem(
        &self,
        idx: usize,
        cost: GasUnits<GasCarrier>,
        type_param: &Type,
        context: &impl NativeContext,
    ) -> PartialVMResult<NativeResult> {
        let v = self.0.borrow();
        check_elem_layout(context, type_param, &*v)?;
        if idx >= v.len() {
            return Ok(NativeResult::err(
                cost,
                PartialVMError::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(INDEX_OUT_OF_BOUNDS),
            ));
        }
        Ok(NativeResult::ok(
            cost,
            vec![Value(self.0.borrow_elem(idx)?)],
        ))
    }

    pub fn pop(
        &self,
        cost: GasUnits<GasCarrier>,
        type_param: &Type,
        context: &impl NativeContext,
    ) -> PartialVMResult<NativeResult> {
        let mut v = self.0.borrow_mut();
        check_elem_layout(context, type_param, &*v)?;

        macro_rules! err_pop_empty_vec {
            () => {
                return Ok(NativeResult::err(
                    cost,
                    PartialVMError::new(StatusCode::NATIVE_FUNCTION_ERROR)
                        .with_sub_status(POP_EMPTY_VEC),
                ));
            };
        }

        let res = match &mut *v {
            Container::U8(v) => match v.pop() {
                Some(x) => Value::u8(x),
                None => err_pop_empty_vec!(),
            },
            Container::U64(v) => match v.pop() {
                Some(x) => Value::u64(x),
                None => err_pop_empty_vec!(),
            },
            Container::U128(v) => match v.pop() {
                Some(x) => Value::u128(x),
                None => err_pop_empty_vec!(),
            },
            Container::Bool(v) => match v.pop() {
                Some(x) => Value::bool(x),
                None => err_pop_empty_vec!(),
            },
            Container::Address(v) => match v.pop() {
                Some(x) => Value::address(x),
                None => err_pop_empty_vec!(),
            },

            Container::Resource(v) | Container::General(v) => match v.pop() {
                Some(x) => Value(x),
                None => err_pop_empty_vec!(),
            },
        };

        Ok(NativeResult::ok(cost, vec![res]))
    }

    pub fn swap(
        &self,
        idx1: usize,
        idx2: usize,
        cost: GasUnits<GasCarrier>,
        type_param: &Type,
        context: &impl NativeContext,
    ) -> PartialVMResult<NativeResult> {
        let mut v = self.0.borrow_mut();
        check_elem_layout(context, type_param, &*v)?;

        macro_rules! swap {
            ($v: ident) => {{
                if idx1 >= $v.len() || idx2 >= $v.len() {
                    return Ok(NativeResult::err(
                        cost,
                        PartialVMError::new(StatusCode::NATIVE_FUNCTION_ERROR)
                            .with_sub_status(INDEX_OUT_OF_BOUNDS),
                    ));
                }
                $v.swap(idx1, idx2);
            }};
        }

        match &mut *v {
            Container::U8(v) => swap!(v),
            Container::U64(v) => swap!(v),
            Container::U128(v) => swap!(v),
            Container::Bool(v) => swap!(v),
            Container::Address(v) => swap!(v),
            Container::Resource(v) | Container::General(v) => swap!(v),
        }

        Ok(NativeResult::ok(cost, vec![]))
    }
}

impl Vector {
    pub fn empty(
        cost: GasUnits<GasCarrier>,
        type_param: &Type,
        context: &impl NativeContext,
    ) -> PartialVMResult<NativeResult> {
        let container = match type_param {
            Type::U8 => Value::vector_u8(iter::empty::<u8>()),
            Type::U64 => Value::vector_u64(iter::empty::<u64>()),
            Type::U128 => Value::vector_u128(iter::empty::<u128>()),
            Type::Bool => Value::vector_bool(iter::empty::<bool>()),
            Type::Address => Value::vector_address(iter::empty::<AccountAddress>()),

            Type::Signer => Value(ValueImpl::new_container(Container::Resource(vec![]))),

            Type::Vector(_) | Type::Struct(_) | Type::StructInstantiation(_, _) => {
                if context.is_resource(type_param)? {
                    Value(ValueImpl::new_container(Container::Resource(vec![])))
                } else {
                    Value(ValueImpl::new_container(Container::General(vec![])))
                }
            }

            Type::Reference(_) | Type::MutableReference(_) | Type::TyParam(_) => {
                return Err(
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!("invalid type param for vector: {:?}", type_param)),
                )
            }
        };

        Ok(NativeResult::ok(cost, vec![container]))
    }

    pub fn destroy_empty(
        self,
        cost: GasUnits<GasCarrier>,
        type_param: &Type,
        context: &impl NativeContext,
    ) -> PartialVMResult<NativeResult> {
        let v = self.0.borrow();
        check_elem_layout(context, type_param, &*v)?;

        let is_empty = match &*v {
            Container::U8(v) => v.is_empty(),
            Container::U64(v) => v.is_empty(),
            Container::U128(v) => v.is_empty(),
            Container::Bool(v) => v.is_empty(),
            Container::Address(v) => v.is_empty(),

            Container::Resource(v) | Container::General(v) => v.is_empty(),
        };

        if is_empty {
            Ok(NativeResult::ok(cost, vec![]))
        } else {
            Ok(NativeResult::err(
                cost,
                PartialVMError::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(DESTROY_NON_EMPTY_VEC),
            ))
        }
    }
}

/***************************************************************************************
 *
 * Gas
 *
 *   Abstract memory sizes of the VM values.
 *
 **************************************************************************************/

impl Container {
    fn size(&self) -> AbstractMemorySize<GasCarrier> {
        match self {
            Self::Resource(v) | Self::General(v) => v
                .iter()
                .fold(STRUCT_SIZE, |acc, v| acc.map2(v.size(), Add::add)),
            Self::U8(v) => AbstractMemorySize::new((v.len() * size_of::<u8>()) as u64),
            Self::U64(v) => AbstractMemorySize::new((v.len() * size_of::<u64>()) as u64),
            Self::U128(v) => AbstractMemorySize::new((v.len() * size_of::<u128>()) as u64),
            Self::Bool(v) => AbstractMemorySize::new((v.len() * size_of::<bool>()) as u64),
            Self::Address(v) => {
                AbstractMemorySize::new((v.len() * size_of::<AccountAddress>()) as u64)
            }
        }
    }
}

impl ContainerRef {
    fn size(&self) -> AbstractMemorySize<GasCarrier> {
        words_in(REFERENCE_SIZE)
    }
}

impl IndexedRef {
    fn size(&self) -> AbstractMemorySize<GasCarrier> {
        words_in(REFERENCE_SIZE)
    }
}

impl ValueImpl {
    fn size(&self) -> AbstractMemorySize<GasCarrier> {
        use ValueImpl::*;

        match self {
            Invalid | U8(_) | U64(_) | U128(_) | Bool(_) => CONST_SIZE,
            Address(_) => AbstractMemorySize::new(AccountAddress::LENGTH as u64),
            ContainerRef(r) => r.size(),
            IndexedRef(r) => r.size(),
            // TODO: in case the borrow fails the VM will panic.
            Container(r) => r.borrow().size(),
        }
    }
}

impl Struct {
    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        self.0.size()
    }
}

impl Value {
    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        self.0.size()
    }
}

impl ReferenceImpl {
    fn size(&self) -> AbstractMemorySize<GasCarrier> {
        match self {
            Self::ContainerRef(r) => r.size(),
            Self::IndexedRef(r) => r.size(),
        }
    }
}

impl Reference {
    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        self.0.size()
    }
}

impl GlobalValue {
    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        // TODO: should it be self.container.borrow().size()
        words_in(REFERENCE_SIZE)
    }
}

/***************************************************************************************
 *
 * Struct Operations
 *
 *   Public APIs for Struct.
 *
 **************************************************************************************/
impl Struct {
    pub fn pack<I: IntoIterator<Item = Value>>(vals: I, is_resource: bool) -> Self {
        let v = vals.into_iter().map(|v| v.0).collect();
        Self(if is_resource {
            Container::Resource(v)
        } else {
            Container::General(v)
        })
    }

    pub fn unpack(self) -> PartialVMResult<impl Iterator<Item = Value>> {
        match self.0 {
            Container::Resource(v) | Container::General(v) => Ok(v.into_iter().map(Value)),
            Container::U8(_)
            | Container::U64(_)
            | Container::U128(_)
            | Container::Bool(_)
            | Container::Address(_) => Err(PartialVMError::new(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            )
            .with_message("not a struct".to_string())),
        }
    }
}

/***************************************************************************************
 *
 * Global Value Operations
 *
 *   Public APIs for GlobalValue. They allow global values to be created from external
 *   source (a.k.a. storage), and references to be taken from them. At the end of the
 *   transaction execution the dirty ones can be identified and wrote back to storage.
 *
 **************************************************************************************/
impl GlobalValue {
    pub fn new(v: Value) -> PartialVMResult<Self> {
        match v.0 {
            ValueImpl::Container(container) => {
                // TODO: check strong count?
                Ok(Self {
                    status: Rc::new(RefCell::new(GlobalDataStatus::Clean)),
                    container,
                })
            }
            v => Err(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message(format!("cannot create global ref from {:?}", v)),
            ),
        }
    }

    pub fn borrow_global(&self) -> PartialVMResult<Value> {
        Ok(Value(ValueImpl::ContainerRef(ContainerRef::Global {
            status: Rc::clone(&self.status),
            container: Rc::clone(&self.container),
        })))
    }

    pub fn mark_dirty(&self) -> PartialVMResult<()> {
        *self.status.borrow_mut() = GlobalDataStatus::Dirty;
        Ok(())
    }

    pub fn is_clean(&self) -> PartialVMResult<bool> {
        match &*self.status.borrow() {
            GlobalDataStatus::Clean => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn is_dirty(&self) -> PartialVMResult<bool> {
        match &*self.status.borrow() {
            GlobalDataStatus::Dirty => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn into_owned_struct(self) -> PartialVMResult<Struct> {
        Ok(Struct(take_unique_ownership(self.container)?))
    }
}

/***************************************************************************************
*
* Display
*
*   Implementation of the Display trait for VM Values. These are supposed to be more
*   friendly & readable than the generated Debug dump.
*
**************************************************************************************/

impl Display for ValueImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Invalid => write!(f, "Invalid"),

            Self::U8(x) => write!(f, "U8({})", x),
            Self::U64(x) => write!(f, "U64({})", x),
            Self::U128(x) => write!(f, "U128({})", x),
            Self::Bool(x) => write!(f, "{}", x),
            Self::Address(addr) => write!(f, "Address({})", addr.short_str()),

            Self::Container(r) => write!(f, "Container({})", &*r.borrow()),

            Self::ContainerRef(r) => write!(f, "{}", r),
            Self::IndexedRef(r) => write!(f, "{}", r),
        }
    }
}

fn display_list_of_items<T, I>(items: I, f: &mut fmt::Formatter) -> fmt::Result
where
    T: Display,
    I: IntoIterator<Item = T>,
{
    write!(f, "[")?;
    let mut items = items.into_iter();
    if let Some(x) = items.next() {
        write!(f, "{}", x)?;
        for x in items {
            write!(f, ", {}", x)?;
        }
    }
    write!(f, "]")
}

impl Display for ContainerRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: this could panic.
        match self {
            Self::Local(r) => write!(f, "({}, {})", Rc::strong_count(r), &*r.borrow()),
            Self::Global { status, container } => write!(
                f,
                "({:?}, {}, {})",
                &*status.borrow(),
                Rc::strong_count(container),
                &*container.borrow()
            ),
        }
    }
}

impl Display for IndexedRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}<{}>", self.container_ref, self.idx)
    }
}

impl Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Resource(v) | Self::General(v) => display_list_of_items(v, f),
            Self::U8(v) => display_list_of_items(v, f),
            Self::U64(v) => display_list_of_items(v, f),
            Self::U128(v) => display_list_of_items(v, f),
            Self::Bool(v) => display_list_of_items(v, f),
            Self::Address(v) => display_list_of_items(v, f),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Display for Locals {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: this could panic.
        match &*self.0.borrow() {
            Container::General(v) => write!(
                f,
                "{}",
                v.iter()
                    .enumerate()
                    .map(|(idx, val)| format!("[{}] {}", idx, val))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),

            Container::Resource(_)
            | Container::U8(_)
            | Container::U64(_)
            | Container::U128(_)
            | Container::Bool(_)
            | Container::Address(_) => unreachable!(),
        }
    }
}

#[allow(dead_code)]
pub mod debug {
    use super::*;
    use std::fmt::Write;

    fn print_value_impl<B: Write>(
        buf: &mut B,
        ty: &FatType,
        val: &ValueImpl,
    ) -> PartialVMResult<()> {
        match (ty, val) {
            (FatType::U8, ValueImpl::U8(x)) => debug_write!(buf, "{}u8", x),
            (FatType::U64, ValueImpl::U64(x)) => debug_write!(buf, "{}u64", x),
            (FatType::U128, ValueImpl::U128(x)) => debug_write!(buf, "{}u128", x),
            (FatType::Bool, ValueImpl::Bool(x)) => debug_write!(buf, "{}", x),
            (FatType::Address, ValueImpl::Address(x)) => debug_write!(buf, "{}", x),

            (FatType::Vector(elem_ty), ValueImpl::Container(r)) => {
                print_vector(buf, elem_ty, &*r.borrow())
            }

            (FatType::Struct(struct_ty), ValueImpl::Container(r)) => {
                print_struct(buf, struct_ty, &*r.borrow())
            }

            (FatType::MutableReference(val_ty), ValueImpl::ContainerRef(r)) => {
                debug_write!(buf, "(&mut) ")?;
                print_container_ref(buf, val_ty, r)
            }
            (FatType::Reference(val_ty), ValueImpl::ContainerRef(r)) => {
                debug_write!(buf, "(&) ")?;
                print_container_ref(buf, val_ty, r)
            }

            (FatType::MutableReference(val_ty), ValueImpl::IndexedRef(r)) => {
                debug_write!(buf, "(&mut) ")?;
                print_indexed_ref(buf, val_ty, r)
            }
            (FatType::Reference(val_ty), ValueImpl::IndexedRef(r)) => {
                debug_write!(buf, "(&) ")?;
                print_indexed_ref(buf, val_ty, r)
            }

            (_, ValueImpl::Invalid) => debug_write!(buf, "(invalid)"),

            _ => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!("cannot print value {:?} as type {:?}", val, ty))),
        }
    }

    fn print_vector<B: Write>(
        buf: &mut B,
        elem_ty: &FatType,
        v: &Container,
    ) -> PartialVMResult<()> {
        macro_rules! print_vector {
            ($v: expr, $suffix: expr) => {{
                let suffix = &$suffix;
                debug_write!(buf, "[")?;
                let mut it = $v.iter();
                if let Some(x) = it.next() {
                    debug_write!(buf, "{}{}", x, suffix)?;
                    for x in it {
                        debug_write!(buf, ", {}{}", x, suffix)?;
                    }
                }
                debug_write!(buf, "]")
            }};
        }

        match (elem_ty, v) {
            (FatType::U8, Container::U8(v)) => print_vector!(v, "u8"),
            (FatType::U64, Container::U64(v)) => print_vector!(v, "u64"),
            (FatType::U128, Container::U128(v)) => print_vector!(v, "u128"),
            (FatType::Bool, Container::Bool(v)) => print_vector!(v, ""),
            (FatType::Address, Container::Address(v)) => print_vector!(v, ""),

            (FatType::Struct(_), Container::Resource(v))
            | (FatType::Struct(_), Container::General(v)) => {
                debug_write!(buf, "[")?;
                let mut it = v.iter();
                if let Some(x) = it.next() {
                    print_value_impl(buf, elem_ty, x)?;
                    for x in it {
                        debug_write!(buf, ", ")?;
                        print_value_impl(buf, elem_ty, x)?;
                    }
                }
                debug_write!(buf, "]")
            }

            _ => Err(
                PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(format!(
                    "cannot print container {:?} as vector with element type {:?}",
                    v, elem_ty
                )),
            ),
        }
    }

    fn print_struct<B: Write>(
        buf: &mut B,
        struct_ty: &FatStructType,
        s: &Container,
    ) -> PartialVMResult<()> {
        let v = match s {
            Container::Resource(v) | Container::General(v) => v,
            _ => {
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                    .with_message(format!("invalid container {:?} as struct", s)))
            }
        };
        let layout = &struct_ty.layout;
        if layout.len() != v.len() {
            return Err(
                PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(format!(
                    "cannot print container {:?} as struct type {:?}, expected {} fields, got {}",
                    v,
                    struct_ty,
                    layout.len(),
                    v.len()
                )),
            );
        }
        debug_write!(buf, "{}::{} {{ ", struct_ty.module, struct_ty.name)?;
        let mut it = layout.iter().zip(v.iter());
        if let Some((ty, val)) = it.next() {
            print_value_impl(buf, ty, val)?;
            for (ty, val) in it {
                debug_write!(buf, ", ")?;
                print_value_impl(buf, ty, val)?;
            }
        }
        debug_write!(buf, " }}")
    }

    fn print_container_ref<B: Write>(
        buf: &mut B,
        val_ty: &FatType,
        r: &ContainerRef,
    ) -> PartialVMResult<()> {
        match val_ty {
            FatType::Vector(elem_ty) => print_vector(buf, elem_ty, &*r.borrow()),
            FatType::Struct(struct_ty) => print_struct(buf, struct_ty, &*r.borrow()),
            _ => Err(
                PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR).with_message(format!(
                    "cannot print container {:?} as type {:?}",
                    &*r.borrow(),
                    val_ty
                )),
            ),
        }
    }

    fn print_indexed_ref<B: Write>(
        buf: &mut B,
        val_ty: &FatType,
        r: &IndexedRef,
    ) -> PartialVMResult<()> {
        macro_rules! print_vector_elem {
            ($v: expr, $idx: expr, $suffix: expr) => {
                match $v.get($idx) {
                    Some(x) => debug_write!(buf, "{}{}", x, $suffix),
                    None => Err(
                        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                            .with_message("ref index out of bounds".to_string()),
                    ),
                }
            };
        }

        let idx = r.idx;
        match (val_ty, &*r.container_ref.borrow()) {
            (FatType::U8, Container::U8(v)) => print_vector_elem!(v, idx, "u8"),
            (FatType::U64, Container::U64(v)) => print_vector_elem!(v, idx, "u64"),
            (FatType::U128, Container::U128(v)) => print_vector_elem!(v, idx, "u128"),
            (FatType::Bool, Container::Bool(v)) => print_vector_elem!(v, idx, ""),

            (FatType::U8, Container::Resource(v))
            | (FatType::U64, Container::Resource(v))
            | (FatType::U128, Container::Resource(v))
            | (FatType::Bool, Container::Resource(v))
            | (FatType::Address, Container::Resource(v))
            | (FatType::U8, Container::General(v))
            | (FatType::U64, Container::General(v))
            | (FatType::U128, Container::General(v))
            | (FatType::Bool, Container::General(v))
            | (FatType::Address, Container::General(v)) => match v.get(idx) {
                Some(val) => print_value_impl(buf, val_ty, val),
                None => Err(
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message("ref index out of bounds".to_string()),
                ),
            },

            (_, container) => Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message(format!(
                    "cannot print element {} of container {:?} as {:?}",
                    idx, container, val_ty
                ))),
        }
    }

    pub fn print_reference<B: Write>(
        buf: &mut B,
        val_ty: &FatType,
        r: &Reference,
    ) -> PartialVMResult<()> {
        match &r.0 {
            ReferenceImpl::ContainerRef(r) => print_container_ref(buf, val_ty, r),
            ReferenceImpl::IndexedRef(r) => print_indexed_ref(buf, val_ty, r),
        }
    }

    pub fn print_locals<B: Write>(
        buf: &mut B,
        tys: &[FatType],
        locals: &Locals,
    ) -> PartialVMResult<()> {
        match &*locals.0.borrow() {
            Container::General(v) => {
                // TODO: The number of spaces in the indent is currently hard coded.
                // Plan is to switch to the pretty crate for pretty printing.
                if tys.is_empty() {
                    debug_writeln!(buf, "            (none) ")?;
                } else {
                    for (idx, (ty, val)) in tys.iter().zip(v.iter()).enumerate() {
                        debug_write!(buf, "            [{}] ", idx)?;
                        print_value_impl(buf, ty, val)?;
                        debug_writeln!(buf)?;
                    }
                }
                Ok(())
            }

            Container::Resource(_)
            | Container::U8(_)
            | Container::U64(_)
            | Container::U128(_)
            | Container::Bool(_)
            | Container::Address(_) => unreachable!(),
        }
    }
}

/***************************************************************************************
 *
 * Serialization & Deserialization
 *
 *   LCS implementation for VM values. Note although values are represented as Rust
 *   enums that carry type info in the tags, we should NOT rely on them for
 *   serialization:
 *     1) Depending on the specific internal representation, it may be impossible to
 *        reconstruct the layout from a value. For example, one cannot tell if a general
 *        container is a struct or a value.
 *     2) Even if 1) is not a problem at a certain time, we may change to a different
 *        internal representation that breaks the 1-1 mapping. Extremely speaking, if
 *        we switch to untagged unions one day, none of the type info will be carried
 *        by the value.
 *
 *   Therefore the appropriate & robust way to implement serialization & deserialization
 *   is to involve an explicit representation of the type layout.
 *
 **************************************************************************************/
use crate::{loaded_data::runtime_types::Type, natives::function::NativeContext};
use serde::{
    de::Error as DeError,
    ser::{Error as SerError, SerializeSeq, SerializeTuple},
    Deserialize,
};

impl Value {
    pub fn simple_deserialize_fat(blob: &[u8], ty: &FatType) -> PartialVMResult<Value> {
        let (kind_info, layout) = ty.layout_and_kind_info()?;
        Self::simple_deserialize(blob, &kind_info, &layout)
    }

    pub fn simple_serialize_fat(&self, ty: &FatType) -> Option<Vec<u8>> {
        let (_, layout) = ty.layout_and_kind_info().ok()?;
        self.simple_serialize(&layout)
    }

    pub fn simple_deserialize(
        blob: &[u8],
        kind_info: &MoveKindInfo,
        layout: &MoveTypeLayout,
    ) -> PartialVMResult<Value> {
        lcs::from_bytes_seed(SeedWrapper { kind_info, layout }, blob)
            .map_err(|e| PartialVMError::new(StatusCode::INVALID_DATA).with_message(e.to_string()))
    }

    pub fn simple_serialize(&self, layout: &MoveTypeLayout) -> Option<Vec<u8>> {
        lcs::to_bytes(&AnnotatedValue {
            layout,
            val: &self.0,
        })
        .ok()
    }
}

impl Struct {
    pub fn simple_deserialize_fat(blob: &[u8], ty: &FatStructType) -> PartialVMResult<Struct> {
        let ((is_resource, field_kinds), struct_layout) = ty.layout_and_kind_info()?;
        lcs::from_bytes_seed(
            SeedWrapper {
                kind_info: (is_resource, field_kinds.as_slice()),
                layout: &struct_layout,
            },
            blob,
        )
        .map_err(|e| PartialVMError::new(StatusCode::INVALID_DATA).with_message(e.to_string()))
    }

    pub fn simple_serialize_fat(&self, ty: &FatStructType) -> Option<Vec<u8>> {
        let (_, struct_layout) = ty.layout_and_kind_info().ok()?;
        lcs::to_bytes(&AnnotatedValue {
            layout: &struct_layout,
            val: &self.0,
        })
        .ok()
    }

    pub fn simple_deserialize(
        blob: &[u8],
        is_resource: bool,
        field_kinds: &[MoveKindInfo],
        layout: &MoveStructLayout,
    ) -> PartialVMResult<Struct> {
        lcs::from_bytes_seed(
            SeedWrapper {
                kind_info: (MoveKind::from_bool(is_resource), field_kinds),
                layout,
            },
            blob,
        )
        .map_err(|e| PartialVMError::new(StatusCode::INVALID_DATA).with_message(e.to_string()))
    }

    pub fn simple_serialize(&self, layout: &MoveStructLayout) -> Option<Vec<u8>> {
        lcs::to_bytes(&AnnotatedValue {
            layout,
            val: &self.0,
        })
        .ok()
    }
}

struct AnnotatedValue<'a, 'b, T1, T2> {
    layout: &'a T1,
    val: &'b T2,
}

fn invariant_violation<S: serde::Serializer>(message: String) -> S::Error {
    S::Error::custom(
        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(message),
    )
}

impl<'a, 'b> serde::Serialize for AnnotatedValue<'a, 'b, MoveTypeLayout, ValueImpl> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match (self.layout, self.val) {
            (MoveTypeLayout::U8, ValueImpl::U8(x)) => serializer.serialize_u8(*x),
            (MoveTypeLayout::U64, ValueImpl::U64(x)) => serializer.serialize_u64(*x),
            (MoveTypeLayout::U128, ValueImpl::U128(x)) => serializer.serialize_u128(*x),
            (MoveTypeLayout::Bool, ValueImpl::Bool(x)) => serializer.serialize_bool(*x),
            (MoveTypeLayout::Address, ValueImpl::Address(x)) => x.serialize(serializer),

            (MoveTypeLayout::Struct(struct_layout), ValueImpl::Container(r)) => {
                let r = r.borrow();
                (AnnotatedValue {
                    layout: struct_layout,
                    val: &*r,
                })
                .serialize(serializer)
            }

            (MoveTypeLayout::Vector(layout), ValueImpl::Container(r)) => {
                let layout = &**layout;
                match (layout, &*r.borrow()) {
                    (MoveTypeLayout::U8, Container::U8(v)) => v.serialize(serializer),
                    (MoveTypeLayout::U64, Container::U64(v)) => v.serialize(serializer),
                    (MoveTypeLayout::U128, Container::U128(v)) => v.serialize(serializer),
                    (MoveTypeLayout::Bool, Container::Bool(v)) => v.serialize(serializer),
                    (MoveTypeLayout::Address, Container::Address(v)) => v.serialize(serializer),

                    (_, Container::Resource(v)) | (_, Container::General(v)) => {
                        let mut t = serializer.serialize_seq(Some(v.len()))?;
                        for val in v {
                            t.serialize_element(&AnnotatedValue { layout, val })?;
                        }
                        t.end()
                    }

                    (layout, container) => Err(invariant_violation::<S>(format!(
                        "cannot serialize container {:?} as {:?}",
                        container, layout
                    ))),
                }
            }

            (MoveTypeLayout::Signer, ValueImpl::Container(r)) => match &*r.borrow() {
                Container::Resource(v) if v.len() == 1 => (AnnotatedValue {
                    layout: &MoveTypeLayout::Address,
                    val: &v[0],
                })
                .serialize(serializer),

                container => Err(invariant_violation::<S>(format!(
                    "cannot serialize container {:?} as a signer",
                    container
                ))),
            },

            (ty, val) => Err(invariant_violation::<S>(format!(
                "cannot serialize value {:?} as {:?}",
                val, ty
            ))),
        }
    }
}

impl<'a, 'b> serde::Serialize for AnnotatedValue<'a, 'b, MoveStructLayout, Container> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let values = match &self.val {
            Container::Resource(v) | Container::General(v) => v,

            Container::U8(_)
            | Container::U64(_)
            | Container::U128(_)
            | Container::Bool(_)
            | Container::Address(_) => {
                return Err(invariant_violation::<S>(format!(
                    "cannot serialize container value {:?} as {:?}",
                    self.val, self.layout
                )));
            }
        };
        let fields = self.layout.fields();
        if fields.len() != values.len() {
            return Err(invariant_violation::<S>(format!(
                "cannot serialize struct value {:?} as {:?} -- number of fields mismatch",
                self.val, self.layout
            )));
        }
        let mut t = serializer.serialize_tuple(values.len())?;
        for (field_layout, val) in fields.iter().zip(values.iter()) {
            t.serialize_element(&AnnotatedValue {
                layout: field_layout,
                val,
            })?;
        }
        t.end()
    }
}

#[derive(Clone)]
struct SeedWrapper<K, L> {
    kind_info: K,
    layout: L,
}

impl<'d> serde::de::DeserializeSeed<'d> for SeedWrapper<&MoveKindInfo, &MoveTypeLayout> {
    type Value = Value;

    fn deserialize<D: serde::de::Deserializer<'d>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        use MoveKindInfo as K;
        use MoveTypeLayout as L;

        match (self.kind_info, self.layout) {
            (K::Base(MoveKind::Copyable), L::Bool) => {
                bool::deserialize(deserializer).map(Value::bool)
            }
            (K::Base(MoveKind::Copyable), L::U8) => u8::deserialize(deserializer).map(Value::u8),
            (K::Base(MoveKind::Copyable), L::U64) => u64::deserialize(deserializer).map(Value::u64),
            (K::Base(MoveKind::Copyable), L::U128) => {
                u128::deserialize(deserializer).map(Value::u128)
            }
            (K::Base(MoveKind::Copyable), L::Address) => {
                AccountAddress::deserialize(deserializer).map(Value::address)
            }
            (K::Base(MoveKind::Resource), L::Signer) => {
                AccountAddress::deserialize(deserializer).map(Value::signer)
            }

            (K::Struct(k, field_kinds), L::Struct(struct_layout)) => Ok(Value::struct_(
                SeedWrapper {
                    kind_info: (*k, field_kinds.as_slice()),
                    layout: struct_layout,
                }
                .deserialize(deserializer)?,
            )),

            (K::Vector(k, elem_k), L::Vector(layout)) => {
                let container = match &**layout {
                    L::U8 => Container::U8(Vec::deserialize(deserializer)?),
                    L::U64 => Container::U64(Vec::deserialize(deserializer)?),
                    L::U128 => Container::U128(Vec::deserialize(deserializer)?),
                    L::Bool => Container::Bool(Vec::deserialize(deserializer)?),
                    L::Address => Container::Address(Vec::deserialize(deserializer)?),
                    layout => {
                        let v =
                            deserializer.deserialize_seq(VectorElementVisitor(SeedWrapper {
                                kind_info: elem_k,
                                layout,
                            }))?;
                        if k.is_resource() {
                            Container::Resource(v)
                        } else {
                            Container::General(v)
                        }
                    }
                };
                Ok(Value(ValueImpl::Container(Rc::new(RefCell::new(
                    container,
                )))))
            }

            _ => Err(D::Error::custom(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(
                    format!(
                        "cannot deserialize as ({:?}, {:?})",
                        self.kind_info, self.layout
                    ),
                ),
            )),
        }
    }
}

impl<'d> serde::de::DeserializeSeed<'d>
    for SeedWrapper<(MoveKind, &[MoveKindInfo]), &MoveStructLayout>
{
    type Value = Struct;

    fn deserialize<D: serde::de::Deserializer<'d>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        let (k, field_kinds) = self.kind_info;
        let field_layouts = self.layout.fields();
        let fields = deserializer.deserialize_tuple(
            field_layouts.len(),
            StructFieldVisitor(field_kinds, field_layouts),
        )?;
        Ok(Struct::pack(fields, k.is_resource()))
    }
}

struct VectorElementVisitor<'a>(SeedWrapper<&'a MoveKindInfo, &'a MoveTypeLayout>);

impl<'d, 'a> serde::de::Visitor<'d> for VectorElementVisitor<'a> {
    type Value = Vec<ValueImpl>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Vector")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'d>,
    {
        let mut vals = Vec::new();
        while let Some(elem) = seq.next_element_seed(self.0.clone())? {
            vals.push(elem.0)
        }
        Ok(vals)
    }
}

struct StructFieldVisitor<'a>(&'a [MoveKindInfo], &'a [MoveTypeLayout]);

impl<'d, 'a> serde::de::Visitor<'d> for StructFieldVisitor<'a> {
    type Value = Vec<Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Struct")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'d>,
    {
        let mut val = Vec::new();
        for (i, (field_kind, field_layout)) in self.0.iter().zip(self.1.iter()).enumerate() {
            if let Some(elem) = seq.next_element_seed(SeedWrapper {
                kind_info: field_kind,
                layout: field_layout,
            })? {
                val.push(elem)
            } else {
                return Err(A::Error::invalid_length(i, &self));
            }
        }
        Ok(val)
    }
}

/***************************************************************************************
*
* Constants
*
*   Implementation of deseserialization of constant data into a runtime value
*
**************************************************************************************/

impl Value {
    fn constant_sig_token_to_layout(
        constant_signature: &SignatureToken,
    ) -> Option<(MoveKindInfo, MoveTypeLayout)> {
        use MoveKindInfo as K;
        use MoveTypeLayout as L;
        use SignatureToken as S;

        Some(match constant_signature {
            S::Bool => (K::Base(MoveKind::Copyable), L::Bool),
            S::U8 => (K::Base(MoveKind::Copyable), L::U8),
            S::U64 => (K::Base(MoveKind::Copyable), L::U64),
            S::U128 => (K::Base(MoveKind::Copyable), L::U128),
            S::Address => (K::Base(MoveKind::Copyable), L::Address),
            S::Signer => return None,
            S::Vector(inner) => {
                let (k, l) = Self::constant_sig_token_to_layout(inner)?;
                (
                    MoveKindInfo::Vector(k.kind(), Box::new(k)),
                    L::Vector(Box::new(l)),
                )
            }
            // Not yet supported
            S::Struct(_) | S::StructInstantiation(_, _) => return None,
            // Not allowed/Not meaningful
            S::TypeParameter(_) | S::Reference(_) | S::MutableReference(_) => return None,
        })
    }

    pub fn deserialize_constant(constant: &Constant) -> Option<Value> {
        let (kind_info, layout) = Self::constant_sig_token_to_layout(&constant.type_)?;
        Value::simple_deserialize(&constant.data, &kind_info, &layout).ok()
    }
}

/***************************************************************************************
 *
 * Prop Testing
 *
 *   Random generation of values that fit into a given layout.
 *
 **************************************************************************************/
#[cfg(feature = "fuzzing")]
pub mod prop {
    use super::*;
    use move_core_types::value::{MoveStruct, MoveValue};
    use proptest::{collection::vec, prelude::*};

    pub fn value_strategy_with_layout(layout: &FatType) -> impl Strategy<Value = Value> {
        match layout {
            FatType::U8 => any::<u8>().prop_map(Value::u8).boxed(),
            FatType::U64 => any::<u64>().prop_map(Value::u64).boxed(),
            FatType::U128 => any::<u128>().prop_map(Value::u128).boxed(),
            FatType::Bool => any::<bool>().prop_map(Value::bool).boxed(),
            FatType::Address => any::<AccountAddress>().prop_map(Value::address).boxed(),
            FatType::Signer => any::<AccountAddress>().prop_map(Value::signer).boxed(),

            FatType::Vector(layout) => match &**layout {
                FatType::U8 => vec(any::<u8>(), 0..10)
                    .prop_map(|vals| Value(ValueImpl::new_container(Container::U8(vals))))
                    .boxed(),
                FatType::U64 => vec(any::<u64>(), 0..10)
                    .prop_map(|vals| Value(ValueImpl::new_container(Container::U64(vals))))
                    .boxed(),
                FatType::U128 => vec(any::<u128>(), 0..10)
                    .prop_map(|vals| Value(ValueImpl::new_container(Container::U128(vals))))
                    .boxed(),
                FatType::Bool => vec(any::<bool>(), 0..10)
                    .prop_map(|vals| Value(ValueImpl::new_container(Container::Bool(vals))))
                    .boxed(),
                FatType::Address => vec(any::<AccountAddress>(), 0..10)
                    .prop_map(|vals| Value(ValueImpl::new_container(Container::Address(vals))))
                    .boxed(),
                layout => {
                    if layout.is_resource().unwrap() {
                        vec(value_strategy_with_layout(layout), 0..10)
                            .prop_map(|vals| {
                                Value(ValueImpl::new_container(Container::Resource(
                                    vals.into_iter().map(|val| val.0).collect(),
                                )))
                            })
                            .boxed()
                    } else {
                        vec(value_strategy_with_layout(layout), 0..10)
                            .prop_map(|vals| {
                                Value(ValueImpl::new_container(Container::General(
                                    vals.into_iter().map(|val| val.0).collect(),
                                )))
                            })
                            .boxed()
                    }
                }
            },

            FatType::Struct(struct_ty) => {
                let is_resource = struct_ty.is_resource;
                struct_ty
                    .layout
                    .iter()
                    .map(|layout| value_strategy_with_layout(layout))
                    .collect::<Vec<_>>()
                    .prop_map(move |vals| Value::struct_(Struct::pack(vals, is_resource)))
                    .boxed()
            }

            FatType::Reference(..) | FatType::MutableReference(..) => {
                panic!("cannot generate references for prop tests")
            }

            FatType::TyParam(..) => panic!("cannot generate type params for prop tests"),
        }
    }

    pub fn layout_and_value_strategy() -> impl Strategy<Value = (FatType, Value)> {
        any::<FatType>().no_shrink().prop_flat_map(|layout| {
            let value_strategy = value_strategy_with_layout(&layout);
            (Just(layout), value_strategy)
        })
    }

    impl ValueImpl {
        pub fn as_move_value(&self, ty: &FatType) -> MoveValue {
            match (ty, &self) {
                (FatType::U8, ValueImpl::U8(x)) => MoveValue::U8(*x),
                (FatType::U64, ValueImpl::U64(x)) => MoveValue::U64(*x),
                (FatType::U128, ValueImpl::U128(x)) => MoveValue::U128(*x),
                (FatType::Bool, ValueImpl::Bool(x)) => MoveValue::Bool(*x),
                (FatType::Address, ValueImpl::Address(x)) => MoveValue::Address(*x),

                (FatType::Struct(ty), ValueImpl::Container(r)) => match &*r.borrow() {
                    Container::Resource(v) | Container::General(v) => {
                        let mut fields = vec![];
                        for (v, field_ty) in v.iter().zip(ty.layout.iter()) {
                            fields.push(v.as_move_value(field_ty));
                        }
                        MoveValue::Struct(MoveStruct::new(fields))
                    }
                    _ => panic!(
                        "Unexpected non-general container while converting struct: {:?}",
                        ty
                    ),
                },

                (FatType::Vector(inner_ty), ValueImpl::Container(r)) => {
                    MoveValue::Vector(match &*r.borrow() {
                        Container::U8(v) => v.iter().map(|u| MoveValue::U8(*u)).collect(),
                        Container::U64(v) => v.iter().map(|u| MoveValue::U64(*u)).collect(),
                        Container::U128(v) => v.iter().map(|u| MoveValue::U128(*u)).collect(),
                        Container::Bool(b) => b.iter().map(|u| MoveValue::Bool(*u)).collect(),
                        Container::Address(v) => v.iter().map(|u| MoveValue::Address(*u)).collect(),
                        Container::Resource(v) | Container::General(v) => v
                            .iter()
                            .map(|v| v.as_move_value(inner_ty.as_ref()))
                            .collect(),
                    })
                }

                (FatType::Signer, ValueImpl::Container(r)) => {
                    MoveValue::Signer(match &*r.borrow() {
                        Container::Resource(v) if v.len() == 1 => match &v[0] {
                            ValueImpl::Address(a) => *a,
                            v => panic!("Unexpected non-address while converting signer: {:?}", v),
                        },
                        c => panic!("Unexpected container while converting signer: {:?}", c),
                    })
                }

                (ty, val) => panic!("Cannot convert value {:?} as {:?}", val, ty),
            }
        }
    }

    impl Value {
        pub fn as_move_value(&self, ty: &FatType) -> MoveValue {
            self.0.as_move_value(ty)
        }
    }
}
