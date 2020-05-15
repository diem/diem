// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Contains types and related functions.

use crate::{
    ast::QualifiedSymbol,
    env::{GlobalEnv, ModuleId, StructEnv, StructId},
    symbol::{Symbol, SymbolPool},
};
use std::{collections::BTreeMap, fmt, fmt::Formatter};

/// Represents a type.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Type {
    Primitive(PrimitiveType),
    Tuple(Vec<Type>),
    Vector(Box<Type>),
    Struct(ModuleId, StructId, Vec<Type>),
    TypeParameter(u16),

    // Types only appearing in programs.
    Reference(bool, Box<Type>),

    // Types only appearing in specifications
    Fun(Vec<Type>, Box<Type>),

    // Temporary types used during type checking
    Error,
    Var(u16),
}

pub const BOOL_TYPE: Type = Type::Primitive(PrimitiveType::Bool);
pub const NUM_TYPE: Type = Type::Primitive(PrimitiveType::Num);

/// Represents a primitive (builtin) type.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum PrimitiveType {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    Addresses, // set of addresses, for quantification
    // Types only appearing in specifications
    Num,
    Range,
}

/// A type substitution.
#[derive(Debug, Clone)]
pub struct Substitution {
    subs: BTreeMap<u16, Type>,
}

/// Represents an type error resulting from unification.
pub struct TypeError {
    pub message: String,
}

impl TypeError {
    fn new(msg: impl Into<String>) -> Self {
        TypeError {
            message: msg.into(),
        }
    }
}

impl Type {
    pub fn new_prim(p: PrimitiveType) -> Type {
        Type::Primitive(p)
    }

    /// Determines whether this is a reference.
    pub fn is_reference(&self) -> bool {
        matches!(self, Type::Reference(_, _))
    }

    /// Determines whether this is a mutable reference.
    pub fn is_mutable_reference(&self) -> bool {
        if let Type::Reference(true, _) = self {
            true
        } else {
            false
        }
    }

    /// Determines whether this is an immutable reference.
    pub fn is_immutable_reference(&self) -> bool {
        if let Type::Reference(false, _) = self {
            true
        } else {
            false
        }
    }

    /// Returns true if this is any number type.
    pub fn is_number(&self) -> bool {
        if let Type::Primitive(p) = self {
            if let PrimitiveType::U8
            | PrimitiveType::U64
            | PrimitiveType::U128
            | PrimitiveType::Num = p
            {
                return true;
            }
        }
        false
    }

    /// If this is a struct type, replace the type instantiation.
    pub fn replace_struct_instantiation(&self, inst: &[Type]) -> Type {
        match self {
            Type::Struct(mid, sid, _) => Type::Struct(*mid, *sid, inst.to_vec()),
            _ => self.clone(),
        }
    }

    /// If this is a struct type, return the associated struct env and type parameters.
    pub fn get_struct<'env>(
        &'env self,
        env: &'env GlobalEnv,
    ) -> Option<(StructEnv<'env>, &'env [Type])> {
        if let Type::Struct(module_idx, struct_idx, params) = self {
            Some((env.get_module(*module_idx).into_struct(*struct_idx), params))
        } else {
            None
        }
    }

    /// Instantiates type parameters in this type.
    pub fn instantiate(&self, params: &[Type]) -> Type {
        if params.is_empty() {
            self.clone()
        } else {
            self.replace(Some(params), None)
        }
    }

    /// A helper function to do replacement of type parameters and/or type variables.
    fn replace(&self, params: Option<&[Type]>, subs: Option<&Substitution>) -> Type {
        let replace_vec = |types: &[Type]| types.iter().map(|t| t.replace(params, subs)).collect();
        match self {
            Type::TypeParameter(i) => {
                if let Some(ps) = params {
                    ps[*i as usize].clone()
                } else {
                    self.clone()
                }
            }
            Type::Var(i) => {
                if let Some(s) = subs {
                    if let Some(s) = s.subs.get(i) {
                        // Recursively call replacement again here, in case the substitution s
                        // refers to type variables.
                        // TODO: a more efficient approach is to maintain that type assignments
                        // are always fully specialized w.r.t. to the substitution.
                        s.replace(params, subs)
                    } else {
                        self.clone()
                    }
                } else {
                    self.clone()
                }
            }
            Type::Reference(is_mut, bt) => {
                Type::Reference(*is_mut, Box::new(bt.replace(params, subs)))
            }
            Type::Struct(mid, sid, args) => Type::Struct(*mid, *sid, replace_vec(args)),
            Type::Fun(args, result) => {
                Type::Fun(replace_vec(args), Box::new(result.replace(params, subs)))
            }
            Type::Tuple(args) => Type::Tuple(replace_vec(args)),
            Type::Vector(et) => Type::Vector(Box::new(et.replace(params, subs))),
            _ => self.clone(),
        }
    }

    /// Checks whether this type contains a type for which the predicate is true.
    pub fn contains<P>(&self, p: &P) -> bool
    where
        P: Fn(&Type) -> bool,
    {
        if p(self) {
            true
        } else {
            let contains_vec = |ts: &[Type]| ts.iter().any(p);
            match self {
                Type::Reference(_, bt) => bt.contains(p),
                Type::Struct(_, _, args) => contains_vec(args),
                Type::Fun(args, result) => contains_vec(args) || result.contains(p),
                Type::Tuple(args) => contains_vec(args),
                Type::Vector(et) => et.contains(p),
                _ => false,
            }
        }
    }

    /// Returns true if this type is incomplete, i.e. contains any type variables.
    pub fn is_incomplete(&self) -> bool {
        use Type::*;
        match self {
            Var(_) => true,
            Tuple(ts) => ts.iter().any(|t| t.is_incomplete()),
            Fun(ts, r) => ts.iter().any(|t| t.is_incomplete()) || r.is_incomplete(),
            Struct(_, _, ts) => ts.iter().any(|t| t.is_incomplete()),
            Vector(et) => et.is_incomplete(),
            _ => false,
        }
    }
}

impl Substitution {
    /// Creates a new substitution.
    pub fn new() -> Self {
        Self {
            subs: BTreeMap::new(),
        }
    }

    /// Specializes the type, substituting all variables bound in this substitution.
    pub fn specialize(&self, t: &Type) -> Type {
        t.replace(None, Some(self))
    }

    /// Unify two types, returning the unified type.
    ///
    /// This currently implements the following notion of type compatibility:
    ///
    /// - References are dropped (i.e. &T and T are compatible)
    /// - All integer types are compatible.
    ///
    /// The substitution will be refined by variable assignments as needed to perform
    /// unification. If unification fails, the substitution will be in some intermediate state;
    /// to implement transactional unification, the substitution must be cloned before calling
    /// this.
    ///
    /// The passed `display_context` is needed for visualization of types on unification errors.
    pub fn unify<'a>(
        &mut self,
        display_context: &'a TypeDisplayContext<'a>,
        t1: &Type,
        t2: &Type,
    ) -> Result<Type, TypeError> {
        // If any of the arguments is a reference, drop it for unification, but ensure
        // it is put back since we need to maintain this information for later phases.
        if let Type::Reference(is_mut, bt1) = t1 {
            // Avoid creating nested references.
            let t2 = if let Type::Reference(_, bt2) = t2 {
                bt2.as_ref()
            } else {
                t2
            };
            return Ok(Type::Reference(
                *is_mut,
                Box::new(self.unify(display_context, bt1.as_ref(), t2)?),
            ));
        }
        if let Type::Reference(is_mut, bt2) = t2 {
            return Ok(Type::Reference(
                *is_mut,
                Box::new(self.unify(display_context, t1, bt2.as_ref())?),
            ));
        }

        // Substitute or assign variables.
        if let Some(rt) = self.try_substitute_or_assign(display_context, false, &t1, &t2)? {
            return Ok(rt);
        }
        if let Some(rt) = self.try_substitute_or_assign(display_context, true, &t2, &t1)? {
            return Ok(rt);
        }

        // Accept any error type.
        if t1 == &Type::Error {
            return Ok(t2.clone());
        }
        if t2 == &Type::Error {
            return Ok(t1.clone());
        }

        // All number types are currently compatible.
        if t1.is_number() && t2.is_number() {
            return Ok(t1.clone());
        }

        // Unify matching structured types.
        match (t1, t2) {
            (Type::Primitive(p1), Type::Primitive(p2)) => {
                if p1 == p2 {
                    return Ok(t1.clone());
                }
            }
            (Type::TypeParameter(idx1), Type::TypeParameter(idx2)) => {
                if idx1 == idx2 {
                    return Ok(t1.clone());
                }
            }
            (Type::Tuple(ts1), Type::Tuple(ts2)) => {
                return Ok(Type::Tuple(self.unify_vec(
                    display_context,
                    ts1,
                    ts2,
                    "tuples",
                )?));
            }
            (Type::Fun(ts1, r1), Type::Fun(ts2, r2)) => {
                return Ok(Type::Fun(
                    self.unify_vec(display_context, ts1, ts2, "functions")?,
                    Box::new(self.unify(display_context, &*r1, &*r2)?),
                ));
            }
            (Type::Struct(m1, s1, ts1), Type::Struct(m2, s2, ts2)) => {
                if m1 == m2 && s1 == s2 {
                    return Ok(Type::Struct(
                        *m1,
                        *s1,
                        self.unify_vec(display_context, ts1, ts2, "structs")?,
                    ));
                }
            }
            (Type::Vector(e1), Type::Vector(e2)) => {
                return self.unify(display_context, &*e1, &*e2);
            }
            _ => {}
        }

        Err(TypeError::new(format!(
            "expected `{}` but found `{}`",
            self.specialize(&t2).display(display_context),
            self.specialize(&t1).display(display_context),
        )))
    }

    /// Helper to unify two type vectors.
    fn unify_vec<'a>(
        &mut self,
        display_context: &'a TypeDisplayContext<'a>,
        ts1: &[Type],
        ts2: &[Type],
        item_name: &str,
    ) -> Result<Vec<Type>, TypeError> {
        if ts1.len() != ts2.len() {
            return Err(TypeError::new(format!(
                "{} have different arity ({} != {})",
                item_name,
                ts1.len(),
                ts2.len()
            )));
        }
        let mut rs = vec![];
        for i in 0..ts1.len() {
            rs.push(self.unify(display_context, &ts1[i], &ts2[i])?);
        }
        Ok(rs)
    }

    /// Tries to substitute or assign a variable. Returned option is Some if unification
    /// was performed, None if not.
    fn try_substitute_or_assign(
        &mut self,
        display_context: &TypeDisplayContext,
        swapped: bool,
        t1: &Type,
        t2: &Type,
    ) -> Result<Option<Type>, TypeError> {
        if let Type::Var(v1) = t1 {
            if let Some(s1) = self.subs.get(&v1).cloned() {
                return if swapped {
                    // Place the type terms in the right order again, so we
                    // get the 'expected vs actual' direction right.
                    Ok(Some(self.unify(display_context, t2, &s1)?))
                } else {
                    Ok(Some(self.unify(display_context, &s1, t2)?))
                };
            }
            let is_t1_var = |t: &Type| {
                if let Type::Var(v2) = t {
                    v1 == v2
                } else {
                    false
                }
            };
            // Skip the cycle check if we are unifying the same two variables.
            if is_t1_var(t2) {
                return Ok(Some(t1.clone()));
            }
            // Cycle check.
            if !t2.contains(&is_t1_var) {
                self.subs.insert(*v1, t2.clone());
                Ok(Some(t2.clone()))
            } else {
                // It is not clear to me whether this can ever occur given we do no global
                // unification with recursion, but to be on the save side, we have it.
                Err(TypeError::new(
                    "[internal] type unification cycle check failed. Try to annotate types.",
                ))
            }
        } else {
            Ok(None)
        }
    }
}

impl Default for Substitution {
    fn default() -> Self {
        Self::new()
    }
}

/// Data providing context for displaying types.
pub enum TypeDisplayContext<'a> {
    WithoutEnv {
        symbol_pool: &'a SymbolPool,
        reverse_struct_table: &'a BTreeMap<(ModuleId, StructId), QualifiedSymbol>,
    },
    WithEnv {
        env: &'a GlobalEnv,
        type_param_names: Option<Vec<Symbol>>,
    },
}

/// Helper for type displays.
pub struct TypeDisplay<'a> {
    type_: &'a Type,
    context: &'a TypeDisplayContext<'a>,
}

impl Type {
    pub fn display<'a>(&'a self, context: &'a TypeDisplayContext<'a>) -> TypeDisplay<'a> {
        TypeDisplay {
            type_: self,
            context,
        }
    }
}

impl<'a> fmt::Display for TypeDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Type::*;
        let comma_list = |f: &mut Formatter<'_>, ts: &[Type]| {
            let mut first = true;
            for t in ts {
                if first {
                    first = false
                } else {
                    f.write_str(", ")?;
                }
                write!(f, "{}", t.display(self.context))?;
            }
            Ok(())
        };
        match self.type_ {
            Primitive(p) => write!(f, "{}", p),
            Tuple(ts) => {
                f.write_str("(")?;
                comma_list(f, ts)?;
                f.write_str(")")
            }
            Vector(t) => write!(f, "vector<{}>", t.display(self.context)),
            Fun(ts, t) => {
                f.write_str("|")?;
                comma_list(f, ts)?;
                f.write_str("|")?;
                write!(f, "{}", t.display(self.context))
            }
            Struct(mid, sid, ts) => {
                match self.context {
                    TypeDisplayContext::WithoutEnv {
                        symbol_pool,
                        reverse_struct_table,
                    } => {
                        if let Some(sym) = reverse_struct_table.get(&(*mid, *sid)) {
                            write!(f, "{}", sym.display(symbol_pool))?;
                        } else {
                            f.write_str("??unknown??")?;
                        }
                    }
                    TypeDisplayContext::WithEnv { env, .. } => {
                        let func_env = env.get_module(*mid).into_struct(*sid);
                        write!(
                            f,
                            "{}::{}",
                            func_env.module_env.get_name().display(env.symbol_pool()),
                            func_env.get_name().display(env.symbol_pool())
                        )?;
                    }
                }
                if !ts.is_empty() {
                    f.write_str("<")?;
                    comma_list(f, ts)?;
                    f.write_str(">")?;
                }
                Ok(())
            }
            Reference(is_mut, t) => {
                f.write_str("&")?;
                if *is_mut {
                    f.write_str("mut ")?;
                }
                write!(f, "{}", t.display(self.context))
            }
            TypeParameter(idx) => {
                if let TypeDisplayContext::WithEnv {
                    env,
                    type_param_names: Some(names),
                } = self.context
                {
                    let idx = *idx as usize;
                    if idx < names.len() {
                        write!(f, "{}", names[idx].display(env.symbol_pool()))
                    } else {
                        write!(f, "#{}", idx)
                    }
                } else {
                    write!(f, "#{}", idx)
                }
            }
            Var(idx) => write!(f, "?{}", idx),
            Error => f.write_str("?error"),
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use PrimitiveType::*;
        match self {
            Bool => f.write_str("bool"),
            U8 => f.write_str("u8"),
            U64 => f.write_str("u64"),
            U128 => f.write_str("u128"),
            Address => f.write_str("address"),
            Signer => f.write_str("signer"),
            Addresses => f.write_str("addresses"),
            Range => f.write_str("range"),
            Num => f.write_str("num"),
        }
    }
}
