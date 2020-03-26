// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for runtime types.

use libra_types::{
    account_address::AccountAddress,
    language_storage::{StructTag, TypeTag},
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::identifier::Identifier;
use vm::errors::VMResult;

#[cfg(feature = "fuzzing")]
use serde::{Deserialize, Serialize};

/// VM representation of a struct type in Move.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "fuzzing", derive(Serialize, Deserialize, Eq, PartialEq))]
pub struct StructType {
    pub address: AccountAddress,
    pub module: Identifier,
    pub name: Identifier,
    pub is_resource: bool,
    pub ty_args: Vec<Type>,
    pub layout: Vec<Type>,
}

/// VM representation of a Move type that gives access to both the fully qualified
/// name and data layout of the type.
///
/// TODO: this data structure itself is intended to be used in runtime only and
/// should NOT be serialized in any form. Currently we still derive `Serialize` and
/// `Deserialize`, but this is a hack for fuzzing and should be guarded behind the
/// "fuzzing" feature flag. We should look into ways to get rid of this.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "fuzzing", derive(Serialize, Deserialize, Eq, PartialEq))]
pub enum Type {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Vector(Box<Type>),
    Struct(Box<StructType>),
    Reference(Box<Type>),
    MutableReference(Box<Type>),
    TyParam(usize),
}

impl StructType {
    pub fn subst(self, ty_args: &[Type]) -> VMResult<StructType> {
        Ok(Self {
            address: self.address,
            module: self.module,
            name: self.name,
            is_resource: self.is_resource,
            ty_args: self
                .ty_args
                .into_iter()
                .map(|ty| ty.subst(ty_args))
                .collect::<VMResult<_>>()?,
            layout: self
                .layout
                .into_iter()
                .map(|ty| ty.subst(ty_args))
                .collect::<VMResult<_>>()?,
        })
    }

    pub fn into_struct_tag(self) -> VMResult<StructTag> {
        let ty_args = self
            .ty_args
            .into_iter()
            .map(|ty| ty.into_type_tag())
            .collect::<VMResult<Vec<_>>>()?;
        Ok(StructTag {
            address: self.address,
            module: self.module,
            name: self.name,
            type_params: ty_args,
        })
    }
}

impl Type {
    pub fn subst(self, ty_args: &[Type]) -> VMResult<Type> {
        use Type::*;

        let res = match self {
            TyParam(idx) => match ty_args.get(idx) {
                Some(ty) => ty.clone(),
                None => {
                    return Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!(
                            "type substitution failed: index out of bounds -- len {} got {}",
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

    pub fn into_type_tag(self) -> VMResult<TypeTag> {
        use Type::*;

        let res = match self {
            Bool => TypeTag::Bool,
            U8 => TypeTag::U8,
            U64 => TypeTag::U64,
            U128 => TypeTag::U128,
            Address => TypeTag::Address,
            Vector(ty) => TypeTag::Vector(Box::new(ty.into_type_tag()?)),
            Struct(struct_ty) => TypeTag::Struct(struct_ty.into_struct_tag()?),

            ty @ Reference(_) | ty @ MutableReference(_) | ty @ TyParam(_) => {
                return Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message(format!("cannot derive type tag for {:?}", ty)))
            }
        };

        Ok(res)
    }

    pub fn is_resource(&self) -> VMResult<bool> {
        use Type::*;

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
}

#[cfg(feature = "fuzzing")]
pub mod prop {
    use super::*;
    use proptest::{collection::vec, prelude::*};

    impl Type {
        /// Generate a random primitive Type, no Struct or Vector.
        pub fn single_value_strategy() -> impl Strategy<Value = Self> {
            use Type::*;

            prop_oneof![Just(Bool), Just(U8), Just(U64), Just(U128), Just(Address),]
        }

        /// Generate a primitive Value, a Struct or a Vector.
        pub fn nested_strategy(
            depth: u32,
            desired_size: u32,
            expected_branch_size: u32,
        ) -> impl Strategy<Value = Self> {
            use Type::*;

            let leaf = Self::single_value_strategy();
            leaf.prop_recursive(depth, desired_size, expected_branch_size, |inner| {
                prop_oneof![
                    inner
                        .clone()
                        .prop_map(|layout| Type::Vector(Box::new(layout))),
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
                                Box::new(StructType {
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

    impl Arbitrary for Type {
        type Parameters = ();
        fn arbitrary_with(_args: ()) -> Self::Strategy {
            Self::nested_strategy(3, 20, 10).boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }
}
