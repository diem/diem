// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for runtime types.

use libra_types::{
    account_address::AccountAddress,
    language_storage::{StructTag, TypeTag},
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::identifier::Identifier;
use std::fmt::Write;
use vm::errors::VMResult;

use libra_types::access_path::{AccessPath, Accesses};
use serde::{Deserialize, Serialize};

/// VM representation of a struct type in Move.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzzing", derive(Eq, PartialEq))]
pub struct FatStructType {
    pub address: AccountAddress,
    pub module: Identifier,
    pub name: Identifier,
    pub is_resource: bool,
    pub ty_args: Vec<FatType>,
    pub layout: Vec<FatType>,
}

/// VM representation of a Move type that gives access to both the fully qualified
/// name and data layout of the type.
///
/// TODO: this data structure itself is intended to be used in runtime only and
/// should NOT be serialized in any form. Currently we still derive `Serialize` and
/// `Deserialize`, but this is a hack for fuzzing and should be guarded behind the
/// "fuzzing" feature flag. We should look into ways to get rid of this.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "fuzzing", derive(Eq, PartialEq))]
pub enum FatType {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Vector(Box<FatType>),
    Struct(Box<FatStructType>),
    Reference(Box<FatType>),
    MutableReference(Box<FatType>),
    TyParam(usize),
}

impl FatStructType {
    pub fn resource_path(&self) -> VMResult<Vec<u8>> {
        Ok(AccessPath::resource_access_vec(
            &self.struct_tag()?,
            &Accesses::empty(),
        ))
    }

    pub fn subst(&self, ty_args: &[FatType]) -> VMResult<FatStructType> {
        Ok(Self {
            address: self.address,
            module: self.module.clone(),
            name: self.name.clone(),
            is_resource: self.is_resource,
            ty_args: self
                .ty_args
                .iter()
                .map(|ty| ty.subst(ty_args))
                .collect::<VMResult<_>>()?,
            layout: self
                .layout
                .iter()
                .map(|ty| ty.subst(ty_args))
                .collect::<VMResult<_>>()?,
        })
    }

    pub fn struct_tag(&self) -> VMResult<StructTag> {
        let ty_args = self
            .ty_args
            .iter()
            .map(|ty| ty.type_tag())
            .collect::<VMResult<Vec<_>>>()?;
        Ok(StructTag {
            address: self.address,
            module: self.module.clone(),
            name: self.name.clone(),
            type_params: ty_args,
        })
    }

    pub fn debug_print<B: Write>(&self, buf: &mut B) -> VMResult<()> {
        debug_write!(buf, "{}::{}", self.module, self.name)?;
        let mut it = self.ty_args.iter();
        if let Some(ty) = it.next() {
            debug_write!(buf, "<")?;
            ty.debug_print(buf)?;
            for ty in it {
                debug_write!(buf, ", ")?;
                ty.debug_print(buf)?;
            }
            debug_write!(buf, ">")?;
        }
        Ok(())
    }
}

impl FatType {
    pub fn subst(&self, ty_args: &[FatType]) -> VMResult<FatType> {
        use FatType::*;

        let res = match self {
            TyParam(idx) => match ty_args.get(*idx) {
                Some(ty) => ty.clone(),
                None => {
                    return Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!(
                            "fat type substitution failed: index out of bounds -- len {} got {}",
                            ty_args.len(),
                            idx
                        )));
                }
            },

            Bool => Bool,
            U8 => U8,
            U64 => U64,
            U128 => U128,
            Address => Address,
            Vector(ty) => Vector(Box::new(ty.subst(ty_args)?)),
            Reference(ty) => Reference(Box::new(ty.subst(ty_args)?)),
            MutableReference(ty) => MutableReference(Box::new(ty.subst(ty_args)?)),

            Struct(struct_ty) => Struct(Box::new(struct_ty.subst(ty_args)?)),
        };

        Ok(res)
    }

    pub fn type_tag(&self) -> VMResult<TypeTag> {
        use FatType::*;

        let res = match self {
            Bool => TypeTag::Bool,
            U8 => TypeTag::U8,
            U64 => TypeTag::U64,
            U128 => TypeTag::U128,
            Address => TypeTag::Address,
            Vector(ty) => TypeTag::Vector(Box::new(ty.type_tag()?)),
            Struct(struct_ty) => TypeTag::Struct(struct_ty.struct_tag()?),

            ty @ Reference(_) | ty @ MutableReference(_) | ty @ TyParam(_) => {
                return Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message(format!("cannot derive type tag for {:?}", ty)))
            }
        };

        Ok(res)
    }

    pub fn is_resource(&self) -> VMResult<bool> {
        use FatType::*;

        match self {
            Bool | U8 | U64 | U128 | Address | Reference(_) | MutableReference(_) => Ok(false),
            Vector(ty) => ty.is_resource(),
            Struct(struct_ty) => Ok(struct_ty.is_resource),
            // In the VM, concrete type arguments are required for type resolution and the only place
            // uninstantiated type parameters can show up is the cache.
            //
            // Therefore `is_resource` should only be called upon types outside the cache, in which
            // case it will always succeed. (Internal invariant violation otherwise.)
            TyParam(_) => Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message("cannot check if a type parameter is a resource or not".to_string())),
        }
    }

    pub fn debug_print<B: Write>(&self, buf: &mut B) -> VMResult<()> {
        use FatType::*;

        match self {
            Bool => debug_write!(buf, "bool"),
            U8 => debug_write!(buf, "u8"),
            U64 => debug_write!(buf, "u64"),
            U128 => debug_write!(buf, "u128"),
            Address => debug_write!(buf, "address"),
            Vector(elem_ty) => {
                debug_write!(buf, "vector<")?;
                elem_ty.debug_print(buf)?;
                debug_write!(buf, ">")
            }
            Struct(struct_ty) => struct_ty.debug_print(buf),
            Reference(ty) => {
                debug_write!(buf, "&")?;
                ty.debug_print(buf)
            }
            MutableReference(ty) => {
                debug_write!(buf, "&mut ")?;
                ty.debug_print(buf)
            }
            TyParam(_) => Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message("cannot print out uninstantiated type params".to_string())),
        }
    }
}

#[cfg(feature = "fuzzing")]
pub mod prop {
    use super::*;
    use proptest::{collection::vec, prelude::*};

    impl FatType {
        /// Generate a random primitive Type, no Struct or Vector.
        pub fn single_value_strategy() -> impl Strategy<Value = Self> {
            use FatType::*;

            prop_oneof![Just(Bool), Just(U8), Just(U64), Just(U128), Just(Address),]
        }

        /// Generate a primitive Value, a Struct or a Vector.
        pub fn nested_strategy(
            depth: u32,
            desired_size: u32,
            expected_branch_size: u32,
        ) -> impl Strategy<Value = Self> {
            use FatType::*;

            let leaf = Self::single_value_strategy();
            leaf.prop_recursive(depth, desired_size, expected_branch_size, |inner| {
                prop_oneof![
                    inner
                        .clone()
                        .prop_map(|layout| FatType::Vector(Box::new(layout))),
                    (
                        any::<AccountAddress>(),
                        any::<Identifier>(),
                        any::<Identifier>(),
                        any::<bool>(),
                        vec(inner.clone(), 0..4),
                        vec(inner, 0..10)
                    )
                        .prop_map(
                            |(address, module, name, is_resource, ty_args, layout)| Struct(
                                Box::new(FatStructType {
                                    address,
                                    module,
                                    name,
                                    is_resource,
                                    ty_args,
                                    layout,
                                })
                            )
                        ),
                ]
            })
        }
    }

    impl Arbitrary for FatType {
        type Parameters = ();
        fn arbitrary_with(_args: ()) -> Self::Strategy {
            Self::nested_strategy(3, 20, 10).boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }
}
