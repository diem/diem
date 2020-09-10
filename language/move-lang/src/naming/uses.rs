// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    naming::ast as N,
    parser::ast::{ModuleIdent, StructName},
    shared::{unique_map::UniqueMap, *},
};
use move_ir_types::location::*;
use petgraph::{algo::toposort as petgraph_toposort, graphmap::DiGraphMap};
use std::collections::BTreeMap;

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn verify(errors: &mut Errors, modules: &mut UniqueMap<ModuleIdent, N::ModuleDefinition>) {
    let imm_modules = &modules;
    let context = &mut Context::new(imm_modules);
    module_defs(context, modules);
    let graph = &context.dependency_graph();
    match petgraph_toposort(graph, None) {
        Err(cycle_node) => {
            let cycle_ident = cycle_node.node_id().clone();
            let error = cycle_error(context, cycle_ident);
            errors.push(error)
        }
        Ok(ordered_ids) => {
            let ordered_ids = ordered_ids.into_iter().cloned().collect::<Vec<_>>();
            for (order, mident) in ordered_ids.into_iter().rev().enumerate() {
                modules.get_mut(&mident).unwrap().dependency_order = order
            }
        }
    }
}

struct Context<'a> {
    modules: &'a UniqueMap<ModuleIdent, N::ModuleDefinition>,
    neighbors: BTreeMap<ModuleIdent, BTreeMap<ModuleIdent, Loc>>,
    current_module: Option<ModuleIdent>,
}

impl<'a> Context<'a> {
    fn new(modules: &'a UniqueMap<ModuleIdent, N::ModuleDefinition>) -> Self {
        Context {
            modules,
            neighbors: BTreeMap::new(),
            current_module: None,
        }
    }

    fn add_usage(&mut self, uses: &ModuleIdent, loc: Loc) {
        if self.current_module.as_ref().unwrap() == uses || !self.modules.contains_key(uses) {
            return;
        }

        let m = self
            .neighbors
            .entry(self.current_module.clone().unwrap())
            .or_insert_with(BTreeMap::new);
        if m.contains_key(uses) {
            return;
        }

        m.insert(uses.clone(), loc);
    }

    fn dependency_graph(&self) -> DiGraphMap<&ModuleIdent, ()> {
        let edges = self
            .neighbors
            .iter()
            .flat_map(|(parent, children)| children.iter().map(move |(child, _)| (parent, child)));
        DiGraphMap::from_edges(edges)
    }
}

fn cycle_error(context: &Context, cycle_ident: ModuleIdent) -> Error {
    let cycle = shortest_cycle(&context.dependency_graph(), &cycle_ident);

    // For printing uses, sort the cycle by location (earliest first)
    let cycle_strings = cycle
        .iter()
        .map(|m| format!("'{}'", m))
        .collect::<Vec<_>>()
        .join(" uses ");

    let (used_loc, user, used) = best_cycle_loc(context, cycle);

    let use_msg = format!("Invalid use of module '{}' in module '{}'.", used, user);
    let cycle_msg = format!(
        "Using this module creates a dependency cycle: {}",
        cycle_strings
    );
    vec![(used_loc, use_msg), (used_loc, cycle_msg)]
}

fn best_cycle_loc<'a>(
    context: &'a Context,
    cycle: Vec<&'a ModuleIdent>,
) -> (Loc, &'a ModuleIdent, &'a ModuleIdent) {
    let len = cycle.len();
    assert!(len >= 3);
    let first = cycle[0];
    let user = cycle[len - 2];
    let used = cycle[len - 1];
    assert!(first == used);
    let used_loc = context.neighbors.get(user).unwrap().get(used).unwrap();
    (*used_loc, user, used)
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

fn module_defs(context: &mut Context, modules: &UniqueMap<ModuleIdent, N::ModuleDefinition>) {
    modules
        .iter()
        .for_each(|(mident, mdef)| module(context, mident, mdef))
}

fn module(context: &mut Context, mident: ModuleIdent, mdef: &N::ModuleDefinition) {
    context.current_module = Some(mident);
    mdef.structs
        .iter()
        .for_each(|(_, sdef)| struct_def(context, sdef));
    mdef.functions
        .iter()
        .for_each(|(_, fdef)| function(context, fdef));
}

fn struct_def(context: &mut Context, sdef: &N::StructDefinition) {
    if let N::StructFields::Defined(fields) = &sdef.fields {
        fields.iter().for_each(|(_, (_, bt))| type_(context, bt));
    }
}

fn function(context: &mut Context, fdef: &N::Function) {
    function_signature(context, &fdef.signature);
    function_acquires(context, &fdef.acquires);
    if let N::FunctionBody_::Defined(seq) = &fdef.body.value {
        sequence(context, seq)
    }
}

fn function_signature(context: &mut Context, sig: &N::FunctionSignature) {
    types(context, sig.parameters.iter().map(|(_, st)| st));
    type_(context, &sig.return_type)
}

fn function_acquires(_context: &mut Context, _acqs: &BTreeMap<StructName, Loc>) {}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn type_name(context: &mut Context, sp!(loc, tn_): &N::TypeName) {
    use N::TypeName_ as TN;
    if let TN::ModuleType(m, _) = tn_ {
        context.add_usage(m, *loc)
    }
}

fn types<'a>(context: &mut Context, tys: impl IntoIterator<Item = &'a N::Type>) {
    tys.into_iter().for_each(|ty| type_(context, ty))
}

fn types_opt(context: &mut Context, tys_opt: &Option<Vec<N::Type>>) {
    tys_opt.iter().for_each(|tys| types(context, tys))
}

fn type_(context: &mut Context, sp!(_, ty_): &N::Type) {
    use N::Type_ as T;
    match ty_ {
        T::Apply(_, tn, tys) => {
            type_name(context, tn);
            types(context, tys);
        }
        T::Ref(_, t) => type_(context, t),
        T::Param(_) | T::Unit | T::Anything | T::UnresolvedError | T::Var(_) => (),
    }
}

fn type_opt(context: &mut Context, t_opt: &Option<N::Type>) {
    t_opt.iter().for_each(|t| type_(context, t))
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

fn sequence(context: &mut Context, sequence: &N::Sequence) {
    use N::SequenceItem_ as SI;
    for sp!(_, item_) in sequence {
        match item_ {
            SI::Seq(e) => exp(context, e),
            SI::Declare(bl, ty_opt) => {
                lvalues(context, &bl.value);
                type_opt(context, ty_opt);
            }
            SI::Bind(bl, e) => {
                lvalues(context, &bl.value);
                exp(context, e)
            }
        }
    }
}

fn lvalues<'a>(context: &mut Context, al: impl IntoIterator<Item = &'a N::LValue>) {
    al.into_iter().for_each(|a| lvalue(context, a))
}

fn lvalue(context: &mut Context, sp!(loc, a_): &N::LValue) {
    use N::LValue_ as L;
    if let L::Unpack(m, _, bs_opt, f) = a_ {
        context.add_usage(m, *loc);
        types_opt(context, bs_opt);
        lvalues(context, f.iter().map(|(_, (_, b))| b));
    }
}

fn exp(context: &mut Context, sp!(loc, e_): &N::Exp) {
    use N::Exp_ as E;
    match e_ {
        E::Unit { .. }
        | E::UnresolvedError
        | E::Break
        | E::Continue
        | E::Spec(_, _)
        | E::InferredNum(_)
        | E::Value(_)
        | E::Constant(None, _)
        | E::Move(_)
        | E::Copy(_)
        | E::Use(_) => (),

        E::Constant(Some(m), _c) => context.add_usage(m, *loc),
        E::ModuleCall(m, _, bs_opt, sp!(_, es_)) => {
            context.add_usage(m, *loc);
            types_opt(context, bs_opt);
            es_.iter().for_each(|e| exp(context, e))
        }

        E::Builtin(bf, sp!(_, es_)) => {
            builtin_function(context, bf);
            es_.iter().for_each(|e| exp(context, e))
        }

        E::IfElse(ec, et, ef) => {
            exp(context, ec);
            exp(context, et);
            exp(context, ef)
        }

        E::BinopExp(e1, _, e2) | E::Mutate(e1, e2) | E::While(e1, e2) => {
            exp(context, e1);
            exp(context, e2)
        }
        E::Block(seq) => sequence(context, seq),
        E::Assign(al, e) => {
            lvalues(context, &al.value);
            exp(context, e)
        }
        E::FieldMutate(edotted, e) => {
            exp_dotted(context, edotted);
            exp(context, e);
        }

        E::Loop(e) | E::Return(e) | E::Abort(e) | E::Dereference(e) | E::UnaryExp(_, e) => {
            exp(context, e)
        }

        E::Pack(m, _, bs_opt, fes) => {
            context.add_usage(m, *loc);
            types_opt(context, bs_opt);
            fes.iter().for_each(|(_, (_, e))| exp(context, e))
        }

        E::ExpList(es) => es.iter().for_each(|e| exp(context, e)),

        E::DerefBorrow(edotted) | E::Borrow(_, edotted) => exp_dotted(context, edotted),

        E::Cast(e, ty) | E::Annotate(e, ty) => {
            exp(context, e);
            type_(context, ty)
        }
    }
}

fn exp_dotted(context: &mut Context, sp!(_, ed_): &N::ExpDotted) {
    use N::ExpDotted_ as D;
    match ed_ {
        D::Exp(e) => exp(context, e),
        D::Dot(edotted, _) => exp_dotted(context, edotted),
    }
}

fn builtin_function(context: &mut Context, sp!(_, bf_): &N::BuiltinFunction) {
    use N::BuiltinFunction_ as B;
    match bf_ {
        B::MoveTo(bt_opt)
        | B::MoveFrom(bt_opt)
        | B::BorrowGlobal(_, bt_opt)
        | B::Exists(bt_opt)
        | B::Freeze(bt_opt) => type_opt(context, bt_opt),
        B::Assert => (),
    }
}
