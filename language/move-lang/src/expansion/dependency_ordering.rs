// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    diagnostics::{codes::*, Diagnostic},
    expansion::ast::{self as E, Address, ModuleIdent},
    shared::{unique_map::UniqueMap, *},
};
use move_ir_types::location::*;
use move_symbol_pool::Symbol;
use petgraph::{algo::toposort as petgraph_toposort, graphmap::DiGraphMap};
use std::collections::{BTreeMap, BTreeSet};

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn verify(
    compilation_env: &mut CompilationEnv,
    modules: &mut UniqueMap<ModuleIdent, E::ModuleDefinition>,
    scripts: &mut BTreeMap<Symbol, E::Script>,
) {
    let imm_modules = &modules;
    let mut context = Context::new(imm_modules);
    module_defs(&mut context, modules);
    script_defs(&mut context, scripts);

    let Context {
        module_neighbors,
        neighbors_by_node,
        addresses_by_node,
        ..
    } = context;
    let graph = dependency_graph(&module_neighbors);
    match petgraph_toposort(&graph, None) {
        Err(cycle_node) => {
            let cycle_ident = *cycle_node.node_id();
            let error = cycle_error(&module_neighbors, cycle_ident);
            compilation_env.add_diag(error);
        }
        Ok(ordered_ids) => {
            for (order, mident) in ordered_ids.iter().rev().enumerate() {
                modules.get_mut(mident).unwrap().dependency_order = order;
            }
        }
    }
    for (node, neighbors) in neighbors_by_node {
        match node {
            NodeIdent::Module(mident) => {
                let module = modules.get_mut(&mident).unwrap();
                module.immediate_neighbors = neighbors;
            }
            NodeIdent::Script(sname) => {
                let script = scripts.get_mut(&sname).unwrap();
                script.immediate_neighbors = neighbors;
            }
        }
    }
    for (node, used_addresses) in addresses_by_node {
        match node {
            NodeIdent::Module(mident) => {
                let module = modules.get_mut(&mident).unwrap();
                module.used_addresses = used_addresses;
            }
            NodeIdent::Script(sname) => {
                let script = scripts.get_mut(&sname).unwrap();
                script.used_addresses = used_addresses;
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum DepType {
    Use,
    Friend,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
enum NodeIdent {
    Module(ModuleIdent),
    Script(Symbol),
}

struct Context<'a> {
    modules: &'a UniqueMap<ModuleIdent, E::ModuleDefinition>,
    // A union of uses and friends for modules (used for cyclyc dependency checking)
    // - if A uses B,    add edge A -> B
    // - if A friends B, add edge B -> A
    // NOTE: neighbors of scripts are not tracked by this field, as nothing can depend on a script
    // and a script cannot declare friends. Hence, is no way to form a cyclic dependency via scripts
    module_neighbors: BTreeMap<ModuleIdent, BTreeMap<ModuleIdent, BTreeMap<DepType, Loc>>>,
    // A summary of neighbors keyed by module or script
    neighbors_by_node: BTreeMap<NodeIdent, UniqueMap<ModuleIdent, E::Neighbor>>,
    // All addresses used by a node
    addresses_by_node: BTreeMap<NodeIdent, BTreeSet<Address>>,
    // The module or script we are currently exploring
    current_node: Option<NodeIdent>,
}

impl<'a> Context<'a> {
    fn new(modules: &'a UniqueMap<ModuleIdent, E::ModuleDefinition>) -> Self {
        Context {
            modules,
            module_neighbors: BTreeMap::new(),
            neighbors_by_node: BTreeMap::new(),
            addresses_by_node: BTreeMap::new(),
            current_node: None,
        }
    }

    fn add_neighbor(&mut self, mident: ModuleIdent, dep_type: DepType, loc: Loc) {
        if !self.modules.contains_key(&mident) {
            // as the dependency checking happens before the naming phase, it is possible to refer
            // to a module with a ModuleIdent outside of the compilation context. Do not add such
            // modules as neighbors.
            return;
        }

        let current = self.current_node.clone().unwrap();
        if matches!(&current, NodeIdent::Module(current_mident) if &mident == current_mident) {
            // do not add the module itself as a neighbor
            return;
        }

        let neighbor = match dep_type {
            DepType::Use => E::Neighbor::Dependency,
            DepType::Friend => E::Neighbor::Friend,
        };
        let current_neighbors = self
            .neighbors_by_node
            .entry(current.clone())
            .or_insert_with(UniqueMap::new);
        let current_used_addresses = self
            .addresses_by_node
            .entry(current.clone())
            .or_insert_with(BTreeSet::new);
        current_neighbors.remove(&mident);
        current_neighbors.add(mident, neighbor).unwrap();
        current_used_addresses.insert(mident.value.address);

        match current {
            NodeIdent::Module(current_mident) => {
                let (node, new_neighbor) = match dep_type {
                    DepType::Use => (current_mident, mident),
                    DepType::Friend => (mident, current_mident),
                };
                let m = self
                    .module_neighbors
                    .entry(node)
                    .or_insert_with(BTreeMap::new)
                    .entry(new_neighbor)
                    .or_insert_with(BTreeMap::new);
                if m.contains_key(&dep_type) {
                    return;
                }
                m.insert(dep_type, loc);
            }
            NodeIdent::Script(_) => (),
        }
    }

    fn add_usage(&mut self, mident: ModuleIdent, loc: Loc) {
        self.add_neighbor(mident, DepType::Use, loc);
    }

    fn add_friend(&mut self, mident: ModuleIdent, loc: Loc) {
        self.add_neighbor(mident, DepType::Friend, loc);
    }

    fn add_address_usage(&mut self, address: Address) {
        self.addresses_by_node
            .entry(self.current_node.clone().unwrap())
            .or_insert_with(BTreeSet::new)
            .insert(address);
    }
}

fn dependency_graph(
    deps: &BTreeMap<ModuleIdent, BTreeMap<ModuleIdent, BTreeMap<DepType, Loc>>>,
) -> DiGraphMap<&ModuleIdent, ()> {
    let mut graph = DiGraphMap::new();
    for (parent, children) in deps {
        if children.is_empty() {
            graph.add_node(parent);
        } else {
            for child in children.keys() {
                graph.add_edge(parent, child, ());
            }
        }
    }
    graph
}

fn cycle_error(
    deps: &BTreeMap<ModuleIdent, BTreeMap<ModuleIdent, BTreeMap<DepType, Loc>>>,
    cycle_ident: ModuleIdent,
) -> Diagnostic {
    let graph = dependency_graph(deps);
    // For printing uses, sort the cycle by location (earliest first)
    let cycle = shortest_cycle(&graph, &cycle_ident);

    let mut cycle_info = cycle
        .windows(2)
        .map(|pair| {
            let node = pair[0];
            let neighbor = pair[1];
            let relations = deps.get(node).unwrap().get(neighbor).unwrap();
            match (
                relations.get(&DepType::Use),
                relations.get(&DepType::Friend),
            ) {
                (Some(loc), _) => (
                    *loc,
                    DepType::Use,
                    format!("'{}' uses '{}'", neighbor, node),
                    node,
                    neighbor,
                ),
                (_, Some(loc)) => (
                    *loc,
                    DepType::Friend,
                    format!("'{}' is a friend of '{}'", node, neighbor),
                    node,
                    neighbor,
                ),
                (None, None) => unreachable!(),
            }
        })
        .collect::<Vec<_>>();
    debug_assert!({
        let first_node = cycle_info.first().unwrap().3;
        let last_neighbor = cycle_info.last().unwrap().4;
        first_node == last_neighbor
    });
    let cycle_last = cycle_info.pop().unwrap();

    let (cycle_loc, use_msg) = {
        let (loc, dep_type, case_msg, _node, _neighbor) = cycle_last;
        let case = match dep_type {
            DepType::Use => "use",
            DepType::Friend => "friend",
        };
        let msg = format!(
            "{}. This '{}' relationship creates a dependency cycle.",
            case_msg, case
        );
        (loc, msg)
    };

    Diagnostic::new(
        Declarations::InvalidModule,
        (cycle_loc, use_msg),
        cycle_info
            .into_iter()
            .map(|(loc, _dep_type, msg, _node, _neighbor)| (loc, msg)),
    )
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
    context.current_node = Some(NodeIdent::Module(mident));
    mdef.friends
        .key_cloned_iter()
        .for_each(|(mident, friend)| context.add_friend(mident, friend.loc));
    mdef.structs
        .iter()
        .for_each(|(_, _, sdef)| struct_def(context, sdef));
    mdef.functions
        .iter()
        .for_each(|(_, _, fdef)| function(context, fdef));
    mdef.specs
        .iter()
        .for_each(|sblock| spec_block(context, sblock));
}

//**************************************************************************************************
// Scripts
//**************************************************************************************************

// Scripts cannot affect the dependency graph because 1) a script cannot friend anything and 2)
// nothing can depends on a script. Therefore, we iterate over the scripts just to collect their
// immediate dependencies.
fn script_defs(context: &mut Context, scripts: &BTreeMap<Symbol, E::Script>) {
    scripts
        .iter()
        .for_each(|(sname, sdef)| script(context, *sname, sdef))
}

fn script(context: &mut Context, sname: Symbol, sdef: &E::Script) {
    context.current_node = Some(NodeIdent::Script(sname));
    function(context, &sdef.function);
    sdef.specs
        .iter()
        .for_each(|sblock| spec_block(context, sblock));
}

//**************************************************************************************************
// Function
//**************************************************************************************************

fn function(context: &mut Context, fdef: &E::Function) {
    function_signature(context, &fdef.signature);
    function_acquires(context, &fdef.acquires);
    if let E::FunctionBody_::Defined(seq) = &fdef.body.value {
        sequence(context, seq)
    }
    fdef.specs
        .values()
        .for_each(|sblock| spec_block(context, sblock));
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
// Struct
//**************************************************************************************************

fn struct_def(context: &mut Context, sdef: &E::StructDefinition) {
    if let E::StructFields::Defined(fields) = &sdef.fields {
        fields.iter().for_each(|(_, _, (_, bt))| type_(context, bt));
    }
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn module_access(context: &mut Context, sp!(loc, ma_): &E::ModuleAccess) {
    if let E::ModuleAccess_::ModuleAccess(m, _) = ma_ {
        context.add_usage(*m, *loc)
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
    use crate::expansion::ast::{Exp_ as E, Value_ as V};
    match e_ {
        E::Value(sp!(_, V::Address(a))) => context.add_address_usage(*a),

        E::Unit { .. }
        | E::UnresolvedError
        | E::Break
        | E::Continue
        | E::Spec(_, _)
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

//**************************************************************************************************
// Specs
//**************************************************************************************************

fn spec_block(context: &mut Context, sp!(_, sb_): &E::SpecBlock) {
    sb_.members
        .iter()
        .for_each(|sbm| spec_block_member(context, sbm))
}

fn spec_block_member(context: &mut Context, sp!(_, sbm_): &E::SpecBlockMember) {
    use E::SpecBlockMember_ as M;
    match sbm_ {
        M::Condition {
            exp: e,
            additional_exps: es,
            ..
        } => {
            exp(context, e);
            es.iter().for_each(|e| exp(context, e))
        }
        M::Function { body, .. } => {
            if let E::FunctionBody_::Defined(seq) = &body.value {
                sequence(context, seq)
            }
        }
        M::Let { def: e, .. } | M::Include { exp: e, .. } | M::Apply { exp: e, .. } => {
            exp(context, e)
        }
        // A special treatment to the `pragma friend` declarations.
        //
        // The `pragma friend = <address::module_name::function_name>` notion exists before the
        // `friend` feature is implemented as a language feature. And it may still have a use case,
        // that is, to friend a module that is compiled with other modules but not published.
        //
        // To illustrate, suppose we have module `A` and `B` compiled and proved together locally,
        // but for some reason, module `A` is not published on-chain. In this case, we cannot
        // declare `friend A;` in module `B` because that will lead to a linking error (the loader
        // is unable to find module `A`). But the prover side still needs to know that `A` is a
        // friend of `B` (e.g., to verify global invariants). So, the `pragma friend = ...` syntax
        // might need to stay for this purpose. And for that, we need to add the module that is
        // declared as a friend in the `immediate_neighbors`.
        M::Pragma { properties } => {
            for prop in properties {
                let pragma = &prop.value;
                if pragma.name.value.as_str() == "friend" {
                    match &pragma.value {
                        None => (),
                        Some(E::PragmaValue::Literal(_)) => (),
                        Some(E::PragmaValue::Ident(maccess)) => match &maccess.value {
                            E::ModuleAccess_::Name(_) => (),
                            E::ModuleAccess_::ModuleAccess(mident, _) => {
                                context.add_friend(*mident, maccess.loc);
                            }
                        },
                    }
                }
            }
        }
        M::Variable { .. } => (),
    }
}
