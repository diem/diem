// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    expansion::ast as E,
    parser::ast::ModuleIdent,
    shared::{unique_map::UniqueMap, *},
};
use move_ir_types::location::*;
use petgraph::{algo::toposort as petgraph_toposort, graphmap::DiGraphMap};
use std::collections::BTreeMap;

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn verify(errors: &mut Errors, modules: &mut UniqueMap<ModuleIdent, E::ModuleDefinition>) {
    let imm_modules = &modules;
    let mut context = Context::new(imm_modules);
    module_defs(&mut context, modules);

    let Context { neighbors, .. } = context;
    let graph = dependency_graph(&neighbors);
    match petgraph_toposort(&graph, None) {
        Err(cycle_node) => {
            let cycle_ident = cycle_node.node_id().clone();
            let error = cycle_error(&neighbors, cycle_ident);
            errors.push(error);
        }
        Ok(ordered_ids) => {
            let ordered_ids = ordered_ids.into_iter().cloned().collect::<Vec<_>>();
            for (order, mident) in ordered_ids.into_iter().rev().enumerate() {
                modules.get_mut(&mident).unwrap().dependency_order = order;
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum DepType {
    Use,
    Friend,
}

struct Context<'a> {
    modules: &'a UniqueMap<ModuleIdent, E::ModuleDefinition>,
    // A union of uses and friends:
    // - if A uses B,    add edge A -> B
    // - if A friends B, add edge B -> A
    neighbors: BTreeMap<ModuleIdent, BTreeMap<ModuleIdent, BTreeMap<DepType, Loc>>>,
    current_module: Option<ModuleIdent>,
}

impl<'a> Context<'a> {
    fn new(modules: &'a UniqueMap<ModuleIdent, E::ModuleDefinition>) -> Self {
        Context {
            modules,
            neighbors: BTreeMap::new(),
            current_module: None,
        }
    }

    fn add_neighbor(&mut self, mident: ModuleIdent, dep_type: DepType, loc: Loc) {
        let current_mident = self.current_module.clone().unwrap();
        if current_mident == mident || !self.modules.contains_key(&mident) {
            return;
        }
        let (node, new_neighbor) = match dep_type {
            DepType::Use => (current_mident, mident),
            DepType::Friend => (mident, current_mident),
        };

        let m = self
            .neighbors
            .entry(node)
            .or_insert_with(BTreeMap::new)
            .entry(new_neighbor)
            .or_insert_with(BTreeMap::new);
        if m.contains_key(&dep_type) {
            return;
        }
        m.insert(dep_type, loc);
    }

    fn add_usage(&mut self, mident: ModuleIdent, loc: Loc) {
        self.add_neighbor(mident, DepType::Use, loc);
    }

    fn add_friend(&mut self, mident: ModuleIdent, loc: Loc) {
        self.add_neighbor(mident, DepType::Friend, loc);
    }
}

fn dependency_graph(
    deps: &BTreeMap<ModuleIdent, BTreeMap<ModuleIdent, BTreeMap<DepType, Loc>>>,
) -> DiGraphMap<&ModuleIdent, ()> {
    let edges = deps
        .iter()
        .flat_map(|(parent, children)| children.iter().map(move |(child, _)| (parent, child)));
    DiGraphMap::from_edges(edges)
}

fn cycle_error(
    deps: &BTreeMap<ModuleIdent, BTreeMap<ModuleIdent, BTreeMap<DepType, Loc>>>,
    cycle_ident: ModuleIdent,
) -> Error {
    let graph = dependency_graph(deps);
    let cycle = shortest_cycle(&graph, &cycle_ident);

    let mut cycle_strings: String = cycle
        .windows(2)
        .map(|pair| {
            let node = pair[0];
            let neighbor = pair[1];
            let relations = deps.get(node).unwrap().get(neighbor).unwrap();
            let verb = if relations.contains_key(&DepType::Use) {
                "uses"
            } else {
                assert!(relations.contains_key(&DepType::Friend));
                "is a friend of"
            };
            format!("'{}' {} ", node, verb)
        })
        .collect();
    cycle_strings.push_str(&format!("'{}'", cycle.last().unwrap()));

    // For printing uses, sort the cycle by location (earliest first)
    let (dep_type, cycle_loc, node, neighbor) = best_cycle_loc(deps, cycle);

    let (use_msg, cycle_msg) = match dep_type {
        DepType::Use => (
            format!("Invalid use of module '{}' in module '{}'.", neighbor, node),
            format!(
                "Using this module creates a dependency cycle: {}",
                cycle_strings
            ),
        ),
        DepType::Friend => (
            format!("Invalid friend '{}' in module '{}'", node, neighbor),
            format!(
                "This friend relationship creates a dependency cycle: {}",
                cycle_strings
            ),
        ),
    };
    vec![(cycle_loc, use_msg), (cycle_loc, cycle_msg)]
}

fn best_cycle_loc<'a>(
    deps: &'a BTreeMap<ModuleIdent, BTreeMap<ModuleIdent, BTreeMap<DepType, Loc>>>,
    cycle: Vec<&'a ModuleIdent>,
) -> (DepType, Loc, &'a ModuleIdent, &'a ModuleIdent) {
    let len = cycle.len();
    assert!(len >= 3);
    let first = cycle[0];
    let node = cycle[len - 2];
    let neighbor = cycle[len - 1];
    assert_eq!(first, neighbor);
    let cycle_locs = deps.get(node).unwrap().get(neighbor).unwrap();
    let (dep_type, loc) = cycle_locs.iter().next().unwrap();
    (*dep_type, *loc, node, neighbor)
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

fn module_defs(context: &mut Context, modules: &UniqueMap<ModuleIdent, E::ModuleDefinition>) {
    modules
        .key_cloned_iter()
        .for_each(|(mident, mdef)| module(context, mident, mdef))
}

fn module(context: &mut Context, mident: ModuleIdent, mdef: &E::ModuleDefinition) {
    context.current_module = Some(mident);
    mdef.friends
        .key_cloned_iter()
        .for_each(|(mident, loc)| context.add_friend(mident, *loc));
    mdef.structs
        .iter()
        .for_each(|(_, _, sdef)| struct_def(context, sdef));
    mdef.functions
        .iter()
        .for_each(|(_, _, fdef)| function(context, fdef));
}

fn struct_def(context: &mut Context, sdef: &E::StructDefinition) {
    if let E::StructFields::Defined(fields) = &sdef.fields {
        fields.iter().for_each(|(_, _, (_, bt))| type_(context, bt));
    }
}

fn function(context: &mut Context, fdef: &E::Function) {
    function_signature(context, &fdef.signature);
    function_acquires(context, &fdef.acquires);
    if let E::FunctionBody_::Defined(seq) = &fdef.body.value {
        sequence(context, seq)
    }
}

fn function_signature(context: &mut Context, sig: &E::FunctionSignature) {
    types(context, sig.parameters.iter().map(|(_, st)| st));
    type_(context, &sig.return_type)
}

fn function_acquires(context: &mut Context, acqs: &[E::ModuleAccess]) {
    for acq in acqs {
        module_access(context, acq);
    }
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn module_access(context: &mut Context, sp!(loc, ma_): &E::ModuleAccess) {
    if let E::ModuleAccess_::ModuleAccess(m, _) = ma_ {
        context.add_usage(m.clone(), *loc)
    }
}

fn types<'a>(context: &mut Context, tys: impl IntoIterator<Item = &'a E::Type>) {
    tys.into_iter().for_each(|ty| type_(context, ty))
}

fn types_opt(context: &mut Context, tys_opt: &Option<Vec<E::Type>>) {
    tys_opt.iter().for_each(|tys| types(context, tys))
}

fn type_(context: &mut Context, sp!(_, ty_): &E::Type) {
    use E::Type_ as T;
    match ty_ {
        T::Apply(tn, tys) => {
            module_access(context, tn);
            types(context, tys);
        }
        T::Multiple(tys) => types(context, tys),
        T::Fun(tys, ret_ty) => {
            types(context, tys);
            type_(context, ret_ty)
        }
        T::Ref(_, t) => type_(context, t),
        T::Unit | T::UnresolvedError => (),
    }
}

fn type_opt(context: &mut Context, t_opt: &Option<E::Type>) {
    t_opt.iter().for_each(|t| type_(context, t))
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

fn sequence(context: &mut Context, sequence: &E::Sequence) {
    use E::SequenceItem_ as SI;
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

fn lvalues<'a>(context: &mut Context, al: impl IntoIterator<Item = &'a E::LValue>) {
    al.into_iter().for_each(|a| lvalue(context, a))
}

fn lvalues_with_range(context: &mut Context, sp!(_, ll): &E::LValueWithRangeList) {
    ll.iter().for_each(|lrange| {
        let sp!(_, (l, e)) = lrange;
        lvalue(context, l);
        exp(context, e);
    })
}

fn lvalue(context: &mut Context, sp!(_loc, a_): &E::LValue) {
    use E::LValue_ as L;
    if let L::Unpack(m, bs_opt, f) = a_ {
        module_access(context, m);
        types_opt(context, bs_opt);
        lvalues(context, f.iter().map(|(_, _, (_, b))| b));
    }
}

fn exp(context: &mut Context, sp!(_loc, e_): &E::Exp) {
    use E::Exp_ as E;
    match e_ {
        E::Unit { .. }
        | E::UnresolvedError
        | E::Break
        | E::Continue
        | E::Spec(_, _)
        | E::InferredNum(_)
        | E::Value(_)
        | E::Move(_)
        | E::Copy(_) => (),

        E::Name(ma, tys_opt) => {
            module_access(context, ma);
            types_opt(context, tys_opt)
        }
        E::Call(ma, tys_opt, args) => {
            module_access(context, ma);
            types_opt(context, tys_opt);
            args.value.iter().for_each(|e| exp(context, e))
        }
        E::Pack(ma, tys_opt, fields) => {
            module_access(context, ma);
            types_opt(context, tys_opt);
            fields.iter().for_each(|(_, _, (_, e))| exp(context, e))
        }

        E::IfElse(ec, et, ef) => {
            exp(context, ec);
            exp(context, et);
            exp(context, ef)
        }

        E::BinopExp(e1, _, e2) | E::Mutate(e1, e2) | E::While(e1, e2) | E::Index(e1, e2) => {
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

        E::Loop(e)
        | E::Return(e)
        | E::Abort(e)
        | E::Dereference(e)
        | E::UnaryExp(_, e)
        | E::Borrow(_, e) => exp(context, e),

        E::ExpList(es) => es.iter().for_each(|e| exp(context, e)),

        E::ExpDotted(edotted) => exp_dotted(context, edotted),

        E::Cast(e, ty) | E::Annotate(e, ty) => {
            exp(context, e);
            type_(context, ty)
        }

        E::Lambda(ll, e) => {
            lvalues(context, &ll.value);
            exp(context, e)
        }
        E::Quant(_, binds, es_vec, eopt, e) => {
            lvalues_with_range(context, binds);
            es_vec
                .iter()
                .for_each(|es| es.iter().for_each(|e| exp(context, e)));
            eopt.iter().for_each(|e| exp(context, e));
            exp(context, e)
        }
    }
}

fn exp_dotted(context: &mut Context, sp!(_, ed_): &E::ExpDotted) {
    use E::ExpDotted_ as D;
    match ed_ {
        D::Exp(e) => exp(context, e),
        D::Dot(edotted, _) => exp_dotted(context, edotted),
    }
}
