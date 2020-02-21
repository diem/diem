// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::unique_map::UniqueMap;
use crate::{
    errors::*,
    naming::ast as N,
    parser::ast::{ModuleIdent, StructName},
    shared::*,
};
use petgraph::algo::astar as petgraph_astar;
use petgraph::algo::toposort as petgraph_toposort;
use petgraph::algo::Cycle;
use petgraph::graphmap::DiGraphMap;
use std::collections::{BTreeMap, BTreeSet};

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn verify(errors: &mut Errors, modules: &mut UniqueMap<ModuleIdent, N::ModuleDefinition>) {
    let imm_modules = &modules;
    let context = &mut Context::new(errors, imm_modules);
    module_defs(context, modules);
    match build_ordering(context) {
        Err(cycle_node) => {
            let cycle_ident = cycle_node.node_id().clone();
            report_cycle(context, cycle_ident)
        }
        Ok(ordered_ids) => {
            let ordering = ordered_ids
                .into_iter()
                .filter(|m| imm_modules.get(m).unwrap().is_source_module.is_some())
                .cloned()
                .collect::<Vec<_>>();
            for (order, mident) in ordering.into_iter().rev().enumerate() {
                modules.get_mut(&mident).unwrap().is_source_module = Some(order)
            }
        }
    }
}

struct Context<'a> {
    errors: &'a mut Errors,
    modules: &'a UniqueMap<ModuleIdent, N::ModuleDefinition>,
    neighbors: BTreeMap<ModuleIdent, BTreeMap<ModuleIdent, Loc>>,
    current_module: Option<ModuleIdent>,
}

impl<'a> Context<'a> {
    fn new(
        errors: &'a mut Errors,
        modules: &'a UniqueMap<ModuleIdent, N::ModuleDefinition>,
    ) -> Self {
        Context {
            errors,
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

fn build_ordering<'a>(
    context: &'a Context,
) -> Result<Vec<&'a ModuleIdent>, Cycle<&'a ModuleIdent>> {
    petgraph_toposort(&context.dependency_graph(), None)
}

fn report_cycle(context: &mut Context, cycle_ident: ModuleIdent) {
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
    context
        .errors
        .push(vec![(used_loc, use_msg), (used_loc, cycle_msg)])
}

fn shortest_cycle<'a>(
    dependency_graph: &DiGraphMap<&'a ModuleIdent, ()>,
    start: &'a ModuleIdent,
) -> Vec<&'a ModuleIdent> {
    let shortest_path = dependency_graph
        .neighbors(start)
        .fold(None, |shortest_path, neighbor| {
            let path_opt = petgraph_astar(
                dependency_graph,
                neighbor,
                |finish| finish == start,
                |_e| 1,
                |_| 0,
            );
            match (shortest_path, path_opt) {
                (p, None) | (None, p) => p,
                (Some((acc_len, acc_path)), Some((cur_len, cur_path))) => {
                    Some(if cur_len < acc_len {
                        (cur_len, cur_path)
                    } else {
                        (acc_len, acc_path)
                    })
                }
            }
        });
    let (_, mut path) = shortest_path.unwrap();
    path.insert(0, start);
    path
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
    mdef.uses
        .iter()
        .for_each(|(mident, loc)| context.add_usage(mident, *loc));
    mdef.structs
        .iter()
        .for_each(|(_, sdef)| struct_def(context, sdef));
    mdef.functions
        .iter()
        .for_each(|(_, fdef)| function(context, fdef));
}

fn struct_def(context: &mut Context, sdef: &N::StructDefinition) {
    if let N::StructFields::Defined(fields) = &sdef.fields {
        fields
            .iter()
            .for_each(|(_, (_, bt))| base_type(context, bt));
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
    single_types(context, sig.parameters.iter().map(|(_, st)| st));
    type_(context, &sig.return_type)
}

fn function_acquires(_context: &mut Context, _acqs: &BTreeSet<StructName>) {}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn type_name(context: &mut Context, sp!(loc, tn_): &N::TypeName) {
    use N::TypeName_ as TN;
    if let TN::ModuleType(m, _) = tn_ {
        context.add_usage(m, *loc)
    }
}

fn base_types_opt(context: &mut Context, bs_opt: &Option<Vec<N::BaseType>>) {
    bs_opt.iter().for_each(|bs| base_types(context, bs))
}

fn base_types<'a>(context: &mut Context, bs: impl IntoIterator<Item = &'a N::BaseType>) {
    bs.into_iter().for_each(|bt| base_type(context, bt))
}

fn base_type(context: &mut Context, sp!(_, bt_): &N::BaseType) {
    use N::BaseType_ as BT;
    if let BT::Apply(_, tn, bs) = bt_ {
        type_name(context, tn);
        base_types(context, bs);
    }
}

fn single_types<'a>(context: &mut Context, ss: impl IntoIterator<Item = &'a N::SingleType>) {
    ss.into_iter().for_each(|st| single_type(context, st))
}

fn single_type(context: &mut Context, sp!(_, st_): &N::SingleType) {
    use N::SingleType_ as ST;
    match st_ {
        ST::Base(bt) | ST::Ref(_, bt) => base_type(context, bt),
    }
}

fn type_opt(context: &mut Context, t_opt: &Option<N::Type>) {
    t_opt.iter().for_each(|t| type_(context, t))
}

fn type_(context: &mut Context, sp!(_, t_): &N::Type) {
    use N::Type_ as T;
    match t_ {
        T::Unit => (),
        T::Single(s) => single_type(context, s),
        T::Multiple(ss) => single_types(context, ss),
    }
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
                binds(context, &bl.value);
                type_opt(context, ty_opt);
            }
            SI::Bind(bl, e) => {
                binds(context, &bl.value);
                exp(context, e)
            }
        }
    }
}

fn binds<'a>(context: &mut Context, bl: impl IntoIterator<Item = &'a N::Bind>) {
    bl.into_iter().for_each(|b| bind(context, b))
}

fn bind(context: &mut Context, sp!(loc, b_): &N::Bind) {
    use N::Bind_ as B;
    if let B::Unpack(m, _, bs_opt, f) = b_ {
        context.add_usage(m, *loc);
        base_types_opt(context, bs_opt);
        binds(context, f.iter().map(|(_, (_, b))| b));
    }
}

fn assigns<'a>(context: &mut Context, al: impl IntoIterator<Item = &'a N::Assign>) {
    al.into_iter().for_each(|a| assign(context, a))
}

fn assign(context: &mut Context, sp!(loc, a_): &N::Assign) {
    use N::Assign_ as A;
    if let A::Unpack(m, _, bs_opt, f) = a_ {
        context.add_usage(m, *loc);
        base_types_opt(context, bs_opt);
        assigns(context, f.iter().map(|(_, (_, b))| b));
    }
}

fn exp(context: &mut Context, sp!(loc, e_): &N::Exp) {
    use N::Exp_ as E;
    match e_ {
        E::Unit
        | E::UnresolvedError
        | E::Break
        | E::Continue
        | E::InferredNum(_)
        | E::Value(_)
        | E::Move(_)
        | E::Copy(_)
        | E::Use(_) => (),

        E::ModuleCall(m, _, bs_opt, er) => {
            context.add_usage(m, *loc);
            base_types_opt(context, bs_opt);
            exp(context, er);
        }

        E::Builtin(bf, er) => {
            builtin_function(context, bf);
            exp(context, er);
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
            assigns(context, &al.value);
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
            base_types_opt(context, bs_opt);
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
        B::MoveToSender(bt_opt)
        | B::MoveFrom(bt_opt)
        | B::BorrowGlobal(_, bt_opt)
        | B::Exists(bt_opt)
        | B::Freeze(bt_opt) => bt_opt.iter().for_each(|bt| base_type(context, bt)),
    }
}
