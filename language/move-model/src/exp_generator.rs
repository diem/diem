// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use itertools::Itertools;

use crate::{
    ast::{Exp, LocalVarDecl, Operation, QuantKind, RcExp, TempIndex, Value},
    model::{
        FieldEnv, FunctionEnv, GlobalEnv, Loc, NodeId, QualifiedId, QualifiedInstId, StructId,
    },
    symbol::Symbol,
    ty::{Type, BOOL_TYPE, NUM_TYPE},
};
use std::rc::Rc;

/// A trait that defines a generator for `Exp`.
pub trait ExpGenerator<'env> {
    /// Get the functional environment
    fn function_env(&self) -> &FunctionEnv<'env>;

    /// Get the current location
    fn get_current_loc(&self) -> Loc;

    /// Set the current location
    fn set_loc(&mut self, loc: Loc);

    /// Add a local variable with given type, return the local index.
    fn add_local(&mut self, ty: Type) -> TempIndex;

    /// Get the type of a local given at `temp` index
    fn get_local_type(&self, temp: TempIndex) -> Type;

    /// Get the global environment
    fn global_env(&self) -> &'env GlobalEnv {
        self.function_env().module_env.env
    }

    /// Sets the default location from a node id.
    fn set_loc_from_node(&mut self, node_id: NodeId) {
        let loc = self.global_env().get_node_loc(node_id);
        self.set_loc(loc);
    }

    /// Creates a new expression node id, using current default location, provided type,
    /// and optional instantiation.
    fn new_node(&self, ty: Type, inst_opt: Option<Vec<Type>>) -> NodeId {
        let node_id = self.global_env().new_node(self.get_current_loc(), ty);
        if let Some(inst) = inst_opt {
            self.global_env().set_node_instantiation(node_id, inst);
        }
        node_id
    }

    /// Allocates a new temporary.
    fn new_temp(&mut self, ty: Type) -> TempIndex {
        self.add_local(ty)
    }

    /// Make a boolean constant expression.
    fn mk_bool_const(&self, value: bool) -> RcExp {
        let node_id = self.new_node(BOOL_TYPE.clone(), None);
        Rc::new(Exp::Value(node_id, Value::Bool(value)))
    }

    /// Makes a Call expression.
    fn mk_call(&self, ty: &Type, oper: Operation, args: Vec<RcExp>) -> RcExp {
        let node_id = self.new_node(ty.clone(), None);
        Rc::new(Exp::Call(node_id, oper, args))
    }

    /// Makes a Call expression with type instantiation.
    fn mk_call_with_inst(
        &self,
        ty: &Type,
        inst: Vec<Type>,
        oper: Operation,
        args: Vec<RcExp>,
    ) -> RcExp {
        let node_id = self.new_node(ty.clone(), Some(inst));
        Rc::new(Exp::Call(node_id, oper, args))
    }

    /// Makes an if-then-else expression.
    fn mk_ite(&self, cond: Exp, if_true: Exp, if_false: Exp) -> RcExp {
        let node_id = self.new_node(self.global_env().get_node_type(if_true.node_id()), None);
        Rc::new(Exp::IfElse(
            node_id,
            Rc::new(cond),
            Rc::new(if_true),
            Rc::new(if_false),
        ))
    }

    /// Makes a Call expression with boolean result type.
    fn mk_bool_call(&self, oper: Operation, args: Vec<RcExp>) -> RcExp {
        self.mk_call(&BOOL_TYPE, oper, args)
    }

    /// Make a boolean not expression.
    fn mk_not(&self, arg: RcExp) -> RcExp {
        self.mk_bool_call(Operation::Not, vec![arg])
    }

    /// Make an equality expression.
    fn mk_eq(&self, arg1: RcExp, arg2: RcExp) -> RcExp {
        self.mk_bool_call(Operation::Eq, vec![arg1, arg2])
    }

    /// Make an identical equality expression. This is stronger than `make_equal` because
    /// it requires the exact same representation, not only interpretation.
    fn mk_identical(&self, arg1: RcExp, arg2: RcExp) -> RcExp {
        self.mk_bool_call(Operation::Identical, vec![arg1, arg2])
    }

    /// Make an and expression.
    fn mk_and(&self, arg1: RcExp, arg2: RcExp) -> RcExp {
        self.mk_bool_call(Operation::And, vec![arg1, arg2])
    }

    /// Make an or expression.
    fn mk_or(&self, arg1: RcExp, arg2: RcExp) -> RcExp {
        self.mk_bool_call(Operation::Or, vec![arg1, arg2])
    }

    /// Make an implies expression.
    fn mk_implies(&self, arg1: RcExp, arg2: RcExp) -> RcExp {
        self.mk_bool_call(Operation::Implies, vec![arg1, arg2])
    }

    /// Make a numerical expression for some of the builtin constants.
    fn mk_builtin_num_const(&self, oper: Operation) -> RcExp {
        assert!(matches!(
            oper,
            Operation::MaxU8 | Operation::MaxU64 | Operation::MaxU128
        ));
        self.mk_call(&NUM_TYPE, oper, vec![])
    }

    /// Join an iterator of boolean expressions with a boolean binary operator.
    fn mk_join_bool(&self, oper: Operation, args: impl Iterator<Item = RcExp>) -> Option<RcExp> {
        args.fold1(|a, b| self.mk_bool_call(oper.clone(), vec![a, b]))
    }

    /// Join two boolean optional expression with binary operator.
    fn mk_join_opt_bool(
        &self,
        oper: Operation,
        arg1: Option<RcExp>,
        arg2: Option<RcExp>,
    ) -> Option<RcExp> {
        match (arg1, arg2) {
            (Some(a1), Some(a2)) => Some(self.mk_bool_call(oper, vec![a1, a2])),
            (Some(a1), None) => Some(a1),
            (None, Some(a2)) => Some(a2),
            _ => None,
        }
    }

    /// Creates a quantifier over the content of a vector. The passed function `f` receives
    /// an expression representing an element of the vector and returns the quantifiers predicate;
    /// if it returns None, this function will also return None, otherwise the quantifier will be
    /// returned.
    fn mk_vector_quant_opt<F>(
        &self,
        kind: QuantKind,
        vector: RcExp,
        elem_ty: &Type,
        f: &mut F,
    ) -> Option<RcExp>
    where
        F: FnMut(RcExp) -> Option<RcExp>,
    {
        let elem = self.mk_local("$elem", elem_ty.clone());
        if let Some(body) = f(elem) {
            let range_decl = self.mk_decl(self.mk_symbol("$elem"), elem_ty.clone(), None);
            let node_id = self.new_node(BOOL_TYPE.clone(), None);
            Some(Rc::new(Exp::Quant(
                node_id,
                kind,
                vec![(range_decl, vector)],
                vec![],
                None,
                body,
            )))
        } else {
            None
        }
    }

    /// Creates a quantifier over the content of memory. The passed function `f` receives
    //  an expression representing a value in memory and returns the quantifiers predicate;
    //  if it returns None, this function will also return None.
    fn mk_mem_quant_opt<F>(
        &self,
        kind: QuantKind,
        mem: QualifiedId<StructId>,
        f: &mut F,
    ) -> Option<Exp>
    where
        F: FnMut(RcExp) -> Option<RcExp>,
    {
        // We generate `forall $val in resources<R>: INV[$val]`. The `resources<R>`
        // quantifier domain is currently only available in the internal expression language,
        // not on user level.
        let struct_env = self
            .global_env()
            .get_module(mem.module_id)
            .into_struct(mem.id);
        let type_inst = (0..struct_env.get_type_parameters().len())
            .map(|i| Type::TypeParameter(i as u16))
            .collect_vec();
        let struct_ty = Type::Struct(mem.module_id, mem.id, type_inst);
        let value = self.mk_local("$rsc", struct_ty.clone());

        if let Some(body) = f(value) {
            let resource_domain_ty = Type::ResourceDomain(mem.module_id, mem.id, None);
            let resource_domain_node_id =
                self.new_node(resource_domain_ty, Some(vec![struct_ty.clone()]));
            let resource_domain = Rc::new(Exp::Call(
                resource_domain_node_id,
                Operation::ResourceDomain,
                vec![],
            ));
            let resource_decl = self.mk_decl(self.mk_symbol("$rsc"), struct_ty, None);
            let quant_node_id = self.new_node(BOOL_TYPE.clone(), None);
            Some(Exp::Quant(
                quant_node_id,
                kind,
                vec![(resource_decl, resource_domain)],
                vec![],
                None,
                body,
            ))
        } else {
            None
        }
    }

    /// Creates a quantifier over the content of instantiated memory. The passed function `f`
    /// receives an expression representing a value in memory and returns the quantifiers predicate;
    //  if it returns None, this function will also return None.
    fn mk_inst_mem_quant_opt<F>(
        &self,
        kind: QuantKind,
        mem: &QualifiedInstId<StructId>,
        f: &mut F,
    ) -> Option<RcExp>
    where
        F: FnMut(RcExp) -> Option<RcExp>,
    {
        // We generate `forall $val in resources<R>: INV[$val]`. The `resources<R>`
        // quantifier domain is currently only available in the internal expression language,
        // not on user level.
        let struct_ty = Type::Struct(mem.module_id, mem.id, mem.inst.clone());
        let value = self.mk_local("$rsc", struct_ty.clone());

        if let Some(body) = f(value) {
            let resource_domain_ty =
                Type::ResourceDomain(mem.module_id, mem.id, Some(mem.inst.clone()));
            let resource_domain_node_id =
                self.new_node(resource_domain_ty, Some(vec![struct_ty.clone()]));
            let resource_domain = Rc::new(Exp::Call(
                resource_domain_node_id,
                Operation::ResourceDomain,
                vec![],
            ));
            let resource_decl = self.mk_decl(self.mk_symbol("$rsc"), struct_ty, None);
            let quant_node_id = self.new_node(BOOL_TYPE.clone(), None);
            Some(
                Exp::Quant(
                    quant_node_id,
                    kind,
                    vec![(resource_decl, resource_domain)],
                    vec![],
                    None,
                    body,
                )
                .into_rc(),
            )
        } else {
            None
        }
    }

    /// Makes a local variable declaration.
    fn mk_decl(&self, name: Symbol, ty: Type, binding: Option<RcExp>) -> LocalVarDecl {
        let node_id = self.new_node(ty, None);
        LocalVarDecl {
            id: node_id,
            name,
            binding,
        }
    }

    /// Makes a symbol from a string.
    fn mk_symbol(&self, str: &str) -> Symbol {
        self.global_env().symbol_pool().make(str)
    }

    /// Makes a type domain expression.
    fn mk_type_domain(&self, ty: Type) -> RcExp {
        let domain_ty = Type::TypeDomain(Box::new(ty.clone()));
        let node_id = self.new_node(domain_ty, Some(vec![ty]));
        Rc::new(Exp::Call(node_id, Operation::TypeDomain, vec![]))
    }

    /// Makes an expression which selects a field from a struct.
    fn mk_field_select(&self, field_env: &FieldEnv<'_>, targs: &[Type], exp: RcExp) -> RcExp {
        let ty = field_env.get_type().instantiate(targs);
        let node_id = self.new_node(ty, None);
        Rc::new(Exp::Call(
            node_id,
            Operation::Select(
                field_env.struct_env.module_env.get_id(),
                field_env.struct_env.get_id(),
                field_env.get_id(),
            ),
            vec![exp],
        ))
    }

    /// Makes an expression for a temporary.
    fn mk_temporary(&self, temp: TempIndex) -> RcExp {
        let ty = self.get_local_type(temp);
        let node_id = self.new_node(ty, None);
        Rc::new(Exp::Temporary(node_id, temp))
    }

    /// Makes an expression for a named local.
    fn mk_local(&self, name: &str, ty: Type) -> RcExp {
        let node_id = self.new_node(ty, None);
        let sym = self.mk_symbol(name);
        Rc::new(Exp::LocalVar(node_id, sym))
    }

    /// Get's the memory associated with a Call(Global,..) or Call(Exists, ..) node. Crashes
    /// if the the node is not typed as expected.
    fn get_memory_of_node(&self, node_id: NodeId) -> QualifiedInstId<StructId> {
        // We do have a call `f<R<..>>` so extract the type from the function instantiation.
        let rty = &self.global_env().get_node_instantiation(node_id)[0];
        let (mid, sid, inst) = rty.require_struct();
        mid.qualified_inst(sid, inst.to_owned())
    }
}
