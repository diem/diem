// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file contains an abstraction of concrete *access paths*, which are canonical names for a particular cell in
//! memory. Some examples of concrete paths are:
//! * `0x7/M::T/f` (i.e., the field `f` of the `M::T` resource stored at address `0x7`
//! * `Formal(0)/[2]` (i.e., the value stored at index 2 of the array bound the 0th formal of the current procedure)
//! An abstract path is similar; it consists of the following components:
//! * A *root*, which is either an abstract address or a local
//! * Zero or more *offsets*, where an offset is a field, an unknown vector index, or an abstract struct type
//!
//! Abstract addresses are a set containing constants and abstract access paths read from the environment. For example,
//! in the following Move code:
//!```ignore
//! struct S { f: u64 }
//!
//! fun foo(x: S) {
//!   let a = if (*) { 0x1 } else { *&x.f }
//!    ... // program point 1
//! }
//!```
//!, the value of `a` will be `{ 0x1, Footprint(x/f) }` at program point 1.

use crate::dataflow_domains::{AbstractDomain, SetDomain};
use move_core_types::{account_address::AccountAddress, language_storage::StructTag};
use move_model::{
    ast::TempIndex,
    model::{FunctionEnv, GlobalEnv, ModuleId, QualifiedId, StructId},
    ty::{PrimitiveType, Type, TypeDisplayContext},
};
use num::BigUint;
use std::{
    fmt,
    fmt::{Debug, Formatter},
};

type Address = BigUint;

// =================================================================================================
// Data Model

/// Fully qualified type identifier `base` bound to type actuals `types`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AbsStructType {
    /// Module ID and struct ID
    base: QualifiedId<StructId>,
    /// Instantiation of generic type parameters
    types: Vec<Type>,
}

/// Building block for abstraction of addresses
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Addr {
    /// Account address constant
    Constant(Address),
    /// Account address read from given access path. This represents the value read from the given path at the beginning of
    /// the current function
    Footprint(AccessPath),
}

/// Abstraction of an address: non-empty set of constant or footprint address values
pub type AbsAddr = SetDomain<Addr>;

/// Abstraction of a key of type `addr`::`ty` in global storage
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalKey {
    /// Account address of key
    addr: AbsAddr,
    /// Type of key
    ty: AbsStructType,
}

/// Root of an access path: a global, local, or return variable
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Root {
    /// A key in global storage
    Global(GlobalKey), // TODO: this could (and maybe should) be AbsAddr + Offset::Global
    /// A formal parameter
    Formal(TempIndex),
    /// A local variable
    Local(TempIndex),
    /// A return variable
    Return(usize),
}

/// Offset of an access path: either a field, vector index, or global key
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Offset {
    /// Index into contents of a struct by field offset
    Field(usize),
    /// Unknown index into a vector
    VectorIndex,
    /// A type index into global storage. Only follows a field or vector index of type address
    Global(AbsStructType),
}

/// A unique identifier for a memory cell: root followed by zero or more offsets
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccessPath {
    root: Root,
    offsets: Vec<Offset>,
}

// =================================================================================================
// Abstract domain operations

/// Trait for a domain that can be viewed as a partial map from access paths to values
/// and values can be deleted using their access paths
pub trait AccessPathMap<T: AbstractDomain> {
    fn get_access_path(&self, ap: AccessPath) -> Option<&T>;
    fn remove_access_path(&mut self, ap: AccessPath) -> Option<T>;
}

/// Trait for an abstract domain that can represent footprint values
pub trait FootprintDomain: AbstractDomain + Clone + Debug + PartialEq + Sized {
    /// Create a footprint value for access path `ap`
    fn make_footprint(ap: AccessPath) -> Option<Self>;
}

impl Addr {
    /// Create a constant address from concrete address `a`
    pub fn constant(a: Address) -> Self {
        Addr::Constant(a)
    }

    /// Create a footprint address from access path `ap`
    pub fn footprint(ap: AccessPath) -> Self {
        assert!(
            !ap.root.is_return(),
            "Attempting to create footprint from return access path"
        );
        assert!(
            !ap.root.is_local(),
            "Attempting to create footprint from local access path"
        );

        Self::Footprint(ap)
    }

    /// Return `true` if `self` is a constant
    pub fn is_constant(&self) -> bool {
        match self {
            Self::Constant(_) => true,
            Self::Footprint(_) => false,
        }
    }

    /// Convert this address-typed abstract value A into an access path A/mid::sid::types
    pub fn add_struct_offset(self, mid: &ModuleId, sid: StructId, types: Vec<Type>) -> AccessPath {
        match self {
            Self::Footprint(mut ap) => {
                // TODO: assert type address?
                ap.add_offset(Offset::global(mid, sid, types));
                ap
            }
            Self::Constant(a) => AccessPath::new_address_constant(a, mid, sid, types),
        }
    }

    /// Convert `self` into a concrete `AccountAddress` if it is a constant. Returns `None`
    /// otherwise.
    pub fn get_concrete_address(&self) -> Option<AccountAddress> {
        match self {
            Addr::Constant(a) => Some(move_model::big_uint_to_addr(a)),
            Addr::Footprint(_) => None,
        }
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a FunctionEnv) -> AddrDisplay<'a> {
        AddrDisplay { addr: self, env }
    }
}

impl AbsAddr {
    /// Create a constant address from concrete address `a`
    pub fn constant(a: Address) -> Self {
        SetDomain::singleton(Addr::Constant(a))
    }

    /// Create a footprint address from access path `ap`
    pub fn footprint(ap: AccessPath) -> Self {
        SetDomain::singleton(Addr::Footprint(ap))
    }

    /// Create a footprint address read from formal `temp_index`
    pub fn formal(formal_index: TempIndex, func_env: &FunctionEnv) -> Self {
        assert!(
            func_env.is_parameter(formal_index),
            "Attempting to create formal from local index"
        );
        Self::footprint(AccessPath::from_index(formal_index, func_env))
    }

    /// Return `true` if `self` is a constant
    pub fn is_constant(&self) -> bool {
        self.iter().all(|a| a.is_constant())
    }

    /// Return `true` if `self` consists only of statically known constants
    pub fn is_statically_known(&self) -> bool {
        for a in self.iter() {
            if !a.is_constant() {
                return false;
            }
        }
        true
    }

    /// Substitute all occurences of Footprint(ap) in `self` by resolving the accesss path
    /// `ap` in `sub_map`
    pub fn substitute_footprint(
        &mut self,
        actuals: &[TempIndex],
        type_actuals: &[Type],
        func_env: &FunctionEnv,
        sub_map: &dyn AccessPathMap<AbsAddr>,
    ) {
        let mut acc = SetDomain::default();
        for a in self.iter() {
            match a {
                Addr::Footprint(ap) => {
                    acc.join(&ap.substitute_footprint(actuals, type_actuals, func_env, sub_map));
                }
                c => {
                    acc.insert(c.clone());
                }
            }
        }
        *self = acc
    }

    /// Return a new abstract address by adding the offset `mid::sid<types>` to each element
    /// of `self`
    pub fn add_struct_offset(self, mid: &ModuleId, sid: StructId, types: Vec<Type>) -> Self {
        let mut acc = Self::default();
        for v in self.into_iter() {
            acc.insert(Addr::Footprint(v.add_struct_offset(
                mid,
                sid,
                types.clone(),
            )));
        }
        acc
    }

    /// Return a new abstract address by adding the offset `offset` to each element of `self`
    pub fn add_offset(&self, offset: Offset) -> Self {
        let mut extended_aps: AbsAddr = AbsAddr::default();
        for p in self.iter() {
            match p {
                Addr::Footprint(ap) => {
                    let mut extended_ap = ap.clone();
                    extended_ap.add_offset(offset.clone());
                    extended_aps.insert(Addr::Footprint(extended_ap));
                }
                Addr::Constant(c) => {
                    panic!(
                        "Type error: address constant {:?} as base for offset {:?}",
                        c, offset
                    )
                }
            }
        }
        extended_aps
    }

    /// Produce a new version of `self` with `prefix` prepended to each footprint
    /// value
    pub fn prepend(self, prefix: AccessPath) -> Self {
        let mut acc = Self::default();
        for v in self.into_iter() {
            match v {
                Addr::Footprint(ap) => {
                    let mut new_ap = ap.clone();
                    new_ap.prepend(prefix.clone());
                    acc.insert(Addr::Footprint(new_ap));
                }
                a => {
                    acc.insert(a);
                }
            }
        }
        acc
    }

    /// return an iterator over the footprint paths in `self`
    pub fn footprint_paths(&self) -> impl Iterator<Item = &AccessPath> {
        self.iter().filter_map(|a| match a {
            Addr::Footprint(ap) => Some(ap),
            Addr::Constant(_) => None,
        })
    }

    /// Return an iterator over the concrete addresses in `self`
    pub fn get_concrete_addresses(&self) -> Vec<AccountAddress> {
        self.iter().flat_map(|a| a.get_concrete_address()).collect()
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a FunctionEnv) -> AbsAddrDisplay<'a> {
        AbsAddrDisplay { addr: self, env }
    }
}

impl FootprintDomain for AbsAddr {
    fn make_footprint(ap: AccessPath) -> Option<Self> {
        if !ap.root.is_return() && !ap.root.is_local() {
            Some(AbsAddr::footprint(ap))
        } else {
            None
        }
    }
}

impl From<&AccountAddress> for AbsAddr {
    fn from(addr: &AccountAddress) -> Self {
        Self::constant(move_model::addr_to_big_uint(addr))
    }
}

impl GlobalKey {
    pub fn new(addr: AbsAddr, mid: &ModuleId, sid: StructId, types: Vec<Type>) -> Self {
        Self {
            addr,
            ty: AbsStructType::new(mid, sid, types),
        }
    }

    /// Create a constant `GlobalKey` using constant `addr` and type `ty`
    pub fn constant(addr: BigUint, ty: AbsStructType) -> Self {
        Self {
            addr: AbsAddr::constant(addr),
            ty,
        }
    }

    /// Return the abstract address associated with `self`
    pub fn address(&self) -> &AbsAddr {
        &self.addr
    }

    /// Return the abstract struct type associated with `self`
    pub fn struct_type(&self) -> &AbsStructType {
        &self.ty
    }

    /// Return true if the address and type parameters of this global key are known statically
    pub fn is_statically_known(&self) -> bool {
        self.addr.is_statically_known() && self.ty.is_closed()
    }

    /// Substitute all occurences of Footprint(ap) in `self.addr` by resolving the accesss path
    /// `ap` in `sub_map`.
    pub fn substitute_footprint(
        &mut self,
        actuals: &[TempIndex],
        type_actuals: &[Type],
        func_env: &FunctionEnv,
        sub_map: &dyn AccessPathMap<AbsAddr>,
    ) {
        self.addr
            .substitute_footprint(actuals, type_actuals, func_env, sub_map);
        self.ty.substitute_footprint(type_actuals);
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a FunctionEnv) -> GlobalKeyDisplay<'a> {
        GlobalKeyDisplay { g: self, env }
    }
}

impl Root {
    /// Create a `Root` from local index `index`
    pub fn from_index(index: TempIndex, fun: &FunctionEnv) -> Self {
        if fun.is_parameter(index) {
            Root::Formal(index)
        } else {
            Root::Local(index)
        }
    }

    /// Create a `Root` from global storage key `key`
    pub fn global(key: GlobalKey) -> Self {
        Root::Global(key)
    }

    /// Create a `Root` from return variable `index`
    pub fn ret(index: usize) -> Self {
        Root::Return(index)
    }

    /// Return the type of `self` in `fun`
    pub fn get_type(&self, fun: &FunctionEnv) -> Type {
        match self {
            Self::Global(g) => g.ty.get_type(),
            Self::Formal(i) => fun.get_local_type(*i),
            Self::Local(i) => {
                if *i < fun.get_local_count() {
                    fun.get_local_type(*i)
                } else {
                    // temporary local generated by stackless bytecode. use dummy type
                    Type::Error
                }
            }
            Self::Return(i) => fun.get_return_type(*i),
        }
    }

    /// Return true if this variable is a formal parameter
    pub fn is_formal(&self) -> bool {
        matches!(self, Self::Formal(_))
    }

    /// Return `true` if this variable is a return value
    pub fn is_return(&self) -> bool {
        matches!(self, Self::Return(_))
    }

    /// Return `true` if this variable is a lol
    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local(_))
    }

    /// Return true if `self` can be determined statically
    pub fn is_statically_known(&self) -> bool {
        match self {
            Self::Local(..) | Self::Return(..) | Self::Formal(..) => false,
            Self::Global(g) => g.is_statically_known(),
        }
    }

    /// Replace all footprint paths in `self` using `actuals` and `sub_map`.
    /// Bind free type variables to `type_actuals`.
    pub fn substitute_footprint(
        &mut self,
        actuals: &[TempIndex],
        type_actuals: &[Type],
        func_env: &FunctionEnv,
        sub_map: &dyn AccessPathMap<AbsAddr>,
    ) {
        match self {
            Self::Global(g) => g.substitute_footprint(actuals, type_actuals, func_env, sub_map),
            Self::Formal(_) | Self::Local(_) | Self::Return(_) => (),
        }
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a FunctionEnv) -> RootDisplay<'a> {
        RootDisplay { root: self, env }
    }
}

impl Offset {
    pub fn global(mid: &ModuleId, sid: StructId, types: Vec<Type>) -> Self {
        Offset::Global(AbsStructType::new(mid, sid, types))
    }

    pub fn field(f: usize) -> Self {
        Offset::Field(f)
    }

    /// Get the type of offset `base`/`self` in function `fun`
    pub fn get_type(&self, base: &Type, env: &GlobalEnv) -> Type {
        match (base.skip_reference(), self) {
            (Type::Struct(mid, sid, types), Offset::Field(f)) => {
                let field_type = env
                    .get_module(*mid)
                    .get_struct(*sid)
                    .get_field_by_offset(*f)
                    .get_type();
                field_type.instantiate(types)
            }
            (Type::Vector(t), Offset::VectorIndex) => (*t.clone()),
            (Type::Primitive(PrimitiveType::Address), Offset::Global(s)) => s.get_type(),
            (Type::Primitive(PrimitiveType::Signer), Offset::Global(s)) => {
                // we conflate address and signer, so this can happen
                s.get_type()
            }
            (Type::Error, _) => {
                // couldn't infer the type of `base`. propagate the error
                Type::Error
            }
            _ => {
                panic!(
                    "get_type warning: Invalid base type {} for offset {:?} in get_type",
                    base.display(&move_model::ty::TypeDisplayContext::WithEnv {
                        env,
                        type_param_names: None
                    }),
                    self,
                )
            }
        }
    }

    /// Bind free type variables in `self` to `type_actuals`
    pub fn substitute_footprint(&mut self, type_actuals: &[Type]) {
        match self {
            Offset::Global(g) => g.substitute_footprint(type_actuals),
            Offset::Field(..) | Offset::VectorIndex => (),
        }
    }

    /// Return true if this offset is the same in all concrete program executions
    pub fn is_statically_known(&self) -> bool {
        use Offset::*;
        match self {
            Field(..) => true,
            Global(..) // Note: even if `g.is_statically_known()`, we should return
		// false here because `g` will always have a successor field that
		// is an offset determined at runtime
		| VectorIndex => false,
        }
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, base_type: &'a Type, env: &'a GlobalEnv) -> OffsetDisplay<'a> {
        OffsetDisplay {
            offset: self,
            base_type,
            env,
        }
    }
}

impl AccessPath {
    pub fn new(root: Root, offsets: Vec<Offset>) -> Self {
        AccessPath { root, offsets }
    }

    pub fn new_root(root: Root) -> Self {
        AccessPath {
            root,
            offsets: vec![],
        }
    }

    pub fn new_global(addr: AbsAddr, mid: &ModuleId, sid: StructId, types: Vec<Type>) -> Self {
        Self::new_root(Root::Global(GlobalKey::new(addr, mid, sid, types)))
    }

    pub fn new_address_constant(
        addr: BigUint,
        mid: &ModuleId,
        sid: StructId,
        types: Vec<Type>,
    ) -> Self {
        Self::new_global(AbsAddr::constant(addr), mid, sid, types)
    }

    pub fn new_global_constant(addr: BigUint, ty: AbsStructType) -> Self {
        Self::new_root(Root::Global(GlobalKey::constant(addr, ty)))
    }

    pub fn from_index(i: TempIndex, func_env: &FunctionEnv) -> Self {
        Self::new_root(Root::from_index(i, func_env))
    }

    /// Unpack `self` into its root and offsets
    pub fn into(self) -> (Root, Vec<Offset>) {
        (self.root, self.offsets)
    }

    pub fn root(&self) -> &Root {
        &self.root
    }

    pub fn offsets(&self) -> &[Offset] {
        &self.offsets
    }

    /// extend this access path by adding offset `o` to the end
    pub fn add_offset(&mut self, o: Offset) {
        self.offsets.push(o)
    }

    /// Return the type of this access path
    pub fn get_type(&self, fun: &FunctionEnv) -> Type {
        let mut ty = self.root.get_type(fun);
        for offset in &self.offsets {
            let offset_ty = offset.get_type(&ty, fun.module_env.env);
            ty = offset_ty;
        }
        ty
    }

    /// prepend `prefix` to self by swapping `self`'s root for prefix.root and
    /// replacing `self`'s accesses with prefix.accesses :: self.accesses
    pub fn prepend(&mut self, prefix: Self) {
        // TODO: assert root is a formal
        self.root = prefix.root;
        let mut suffix_offsets = self.offsets.clone();
        self.offsets = prefix.offsets;
        self.offsets.append(&mut suffix_offsets)
    }

    /// Construct a new abstract address by prepending the addresses in `addrs` to `self`
    pub fn prepend_addrs(&self, addrs: &AbsAddr) -> AbsAddr {
        let mut acc = AbsAddr::default();
        for a in addrs.iter() {
            match a {
                Addr::Footprint(ap) => {
                    let mut new_ap = self.clone();
                    new_ap.prepend(ap.clone());
                    acc.insert(Addr::footprint(new_ap));
                }
                Addr::Constant(c) => {
                    if self.offsets.is_empty() {
                        acc.insert(Addr::constant(c.clone()));
                    } else {
                        // access path with constant base and offsets (e.g., 0x1/M::S/f/g)
                        // normalize by converting into a path with a global base instead
                        match &self.offsets[0] {
                            Offset::Global(struct_type) => {
                                let root = Root::Global(GlobalKey::constant(
                                    c.clone(),
                                    struct_type.clone(),
                                ));
                                let mut new_offsets = vec![];
                                for v in self.offsets[1..].iter() {
                                    new_offsets.push(v.clone())
                                }
                                acc.insert(Addr::footprint(AccessPath::new(root, new_offsets)));
                            }
                            _ => panic!(
                                "Invariant violation: constant root with bad offsets {:?}",
                                self.offsets
                            ),
                        }
                    }
                }
            }
        }
        acc
    }

    /// Replace all footprint paths in `self` using `actuals` and `sub_map`.
    /// Bind free type variables to `type_actuals`.
    pub fn substitute_footprint(
        &self,
        actuals: &[TempIndex],
        type_actuals: &[Type],
        func_env: &FunctionEnv,
        sub_map: &dyn AccessPathMap<AbsAddr>,
    ) -> AbsAddr {
        let mut new_offsets = self.offsets.clone();
        new_offsets.iter_mut().for_each(|o| {
            o.substitute_footprint(type_actuals);
        });
        match &self.root {
            Root::Formal(i) => {
                let temp_index = actuals[*i];
                let caller_root = Root::from_index(temp_index, func_env);
                let mut results = AbsAddr::default();

                // In this loop, we lookup the access path in the submap,
                // starting with the root of the caller and adding the
                // offsets of the current access path.
                // When a result is found, we accumulate to results
                // We need to try with the offsets to handle cases such as
                // Formal(1): { data: None, children: { Field(0): { data: Footprint(AccessPath { root: Formal(0), offsets: [] }) } }
                // where simply reading Formal(1) would yield None
                // but Formal(1)/0 gives the mapping to Formal(0)
                for i in 0..=new_offsets.len() {
                    let caller_offsets = Vec::from(&new_offsets[0..i]);
                    let ap = AccessPath::new(caller_root.clone(), caller_offsets.clone());
                    if let Some(addrs) = sub_map.get_access_path(ap) {
                        // We need to adjust the offset from the current access path
                        // to avoid duplicating it (i.e. Formal(0)/0/0 instead of Formal(0)/0)
                        let callee_offsets = Vec::from(&new_offsets[i..new_offsets.len()]);
                        let new_addrs =
                            AccessPath::new(self.root.clone(), callee_offsets).prepend_addrs(addrs);
                        results.join(&new_addrs);
                    }
                }
                results
            }
            Root::Global(g) => {
                let mut new_g = g.clone();
                new_g.substitute_footprint(actuals, type_actuals, func_env, sub_map);
                AbsAddr::singleton(Addr::footprint(AccessPath::new(
                    Root::Global(new_g),
                    new_offsets,
                )))
            }
            Root::Local(_) | Root::Return(_) => AbsAddr::default(),
        }
    }

    /// Return true if `self` can be converted to a compact set of concrete access paths.
    /// Returns false if (e.g.) `self` contains an global root with an unbound
    /// address/type parameter, a global offset, or a vector index offset.
    pub fn is_statically_known(&self) -> bool {
        self.root.is_statically_known() && {
            for offset in &self.offsets {
                if !offset.is_statically_known() {
                    return false;
                }
            }
            true
        }
    }

    /// Return `true` if `self` has no unbound address or type variables (i.e., the type variables
    /// and addresses in `self.root` are statically known and `self` has no `Global` offsets.
    /// This function is the same as `is_statically_known` except that `is_statically_known` returns
    /// `false` if `self` has `Vector` offsets, but this function will not.
    pub fn all_addresses_types_bound(&self) -> bool {
        self.root.is_statically_known() && {
            for offset in &self.offsets {
                if matches!(offset, Offset::Global(_)) {
                    return false;
                }
            }
            true
        }
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a FunctionEnv) -> AccessPathDisplay<'a> {
        AccessPathDisplay { ap: self, env }
    }
}

impl AbsStructType {
    pub fn new(mid: &ModuleId, sid: StructId, types: Vec<Type>) -> Self {
        AbsStructType {
            base: mid.qualified(sid),
            types,
        }
    }

    /// Return the concrete type of `self`
    pub fn get_type(&self) -> Type {
        Type::Struct(self.base.module_id, self.base.id, self.types.clone())
    }

    /// If this `self` is closed, convert it to a `StructTag`. Return
    /// `None` otherwise
    pub fn get_type_tag(&self, env: &GlobalEnv) -> Option<StructTag> {
        self.get_type().into_struct_tag(env)
    }

    /// Substitue the open types in self.types with caller `type_actuals`
    pub fn substitute_footprint(&mut self, type_actuals: &[Type]) {
        for t in self.types.iter_mut() {
            *t = t.instantiate(type_actuals)
        }
    }

    /// Returns a normalized representation of this type if it closed,
    /// None if it is open
    pub fn normalize(&self, env: &GlobalEnv) -> Option<StructTag> {
        self.get_type().into_struct_tag(env)
    }

    /// Return `true` if `self` has no type variables or if all of `self`'s type variables are bound
    pub fn is_closed(&self) -> bool {
        for t in &self.types {
            if t.is_open() {
                return false;
            }
        }
        true
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a GlobalEnv) -> AbsStructTypeDisplay<'a> {
        AbsStructTypeDisplay { s: self, env }
    }
}

// =================================================================================================
// Formatting

pub struct AbsStructTypeDisplay<'a> {
    s: &'a AbsStructType,
    env: &'a GlobalEnv,
}

impl<'a> fmt::Display for AbsStructTypeDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.s.normalize(self.env) {
            Some(t) => {
                write!(f, "{}", t)
            }
            None => {
                let tctx = TypeDisplayContext::WithEnv {
                    env: self.env,
                    type_param_names: None,
                };
                let dummy_type =
                    Type::Struct(self.s.base.module_id, self.s.base.id, self.s.types.clone());
                write!(f, "{}", dummy_type.display(&tctx))
            }
        }
    }
}

pub struct AddrDisplay<'a> {
    addr: &'a Addr,
    env: &'a FunctionEnv<'a>,
}

impl<'a> fmt::Display for AddrDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.addr {
            Addr::Constant(a) => write!(f, "{:#x}", a),
            Addr::Footprint(ap) => write!(f, "{}", ap.display(self.env)),
        }
    }
}

pub struct AbsAddrDisplay<'a> {
    addr: &'a AbsAddr,
    env: &'a FunctionEnv<'a>,
}

impl<'a> fmt::Display for AbsAddrDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.addr.len() == 1 {
            write!(f, "{}", self.addr.iter().next().unwrap().display(self.env))
        } else {
            f.write_str("{")?;
            for a in self.addr.iter() {
                write!(f, "{}", a.display(self.env))?;
                // TODO: nice comma-separated list
                f.write_str(", ")?;
            }
            f.write_str("}")
        }
    }
}

pub struct GlobalKeyDisplay<'a> {
    g: &'a GlobalKey,
    env: &'a FunctionEnv<'a>,
}

impl<'a> fmt::Display for GlobalKeyDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}",
            self.g.addr.display(self.env),
            self.g.ty.display(self.env.module_env.env)
        )
    }
}

pub struct RootDisplay<'a> {
    root: &'a Root,
    env: &'a FunctionEnv<'a>,
}

impl<'a> fmt::Display for RootDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.root {
            Root::Global(g) => write!(f, "{}", g.display(self.env)),
            Root::Local(i) => write!(f, "Loc({})", i), // TODO: print name if available
            Root::Formal(i) => write!(f, "Formal({})", i), // TODO: print name if available
            Root::Return(i) => write!(f, "Ret({})", i),
        }
    }
}

pub struct OffsetDisplay<'a> {
    offset: &'a Offset,
    base_type: &'a Type,
    env: &'a GlobalEnv,
}

impl<'a> fmt::Display for OffsetDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Offset::*;
        match self.offset {
            Field(fld) => match self.base_type.skip_reference() {
                Type::Struct(mid, sid, _types) => f.write_str(
                    self.env
                        .get_module(*mid)
                        .get_struct(*sid)
                        .get_field_by_offset(*fld)
                        .get_identifier()
                        .expect("identifier defined")
                        .as_str(),
                ),
                Type::Error => {
                    // this happens when we are trying to print an access path rooted in a stackless
                    // bytecode-generated local. we do not know the type of this local, so we cannot
                    // infer the type of its fields either. just print the index
                    write!(f, "{}", *fld)
                }
                _ => {
                    panic!(
                        "Warning: Invalid base type {:?} for field offset {:?}",
                        self.base_type, self.offset
                    );
                }
            },
            VectorIndex => f.write_str("[_]"),
            Global(g) => write!(f, "{}", g.display(self.env)),
        }
    }
}

pub struct AccessPathDisplay<'a> {
    ap: &'a AccessPath,
    env: &'a FunctionEnv<'a>,
}

impl<'a> fmt::Display for AccessPathDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let genv = self.env.module_env.env;
        write!(f, "{}", self.ap.root.display(self.env))?;
        let mut root_ty = self.ap.root.get_type(self.env);
        for offset in &self.ap.offsets {
            f.write_str("/")?;
            write!(f, "{}", offset.display(&root_ty, genv))?;
            let offset_ty = offset.get_type(&root_ty, genv);
            root_ty = offset_ty;
        }
        Ok(())
    }
}
