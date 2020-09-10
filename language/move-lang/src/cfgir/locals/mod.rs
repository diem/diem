// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod state;

use super::absint::*;
use crate::{
    errors::*,
    hlir::{
        ast::*,
        translate::{display_var, DisplayVar},
    },
    parser::ast::{Kind_, StructName, Var},
    shared::{unique_map::UniqueMap, *},
};
use move_ir_types::location::*;
use state::*;
use std::collections::BTreeMap;

//**************************************************************************************************
// Entry and trait bindings
//**************************************************************************************************

struct LocalsSafety<'a> {
    local_types: &'a UniqueMap<Var, SingleType>,
    signature: &'a FunctionSignature,
}

impl<'a> LocalsSafety<'a> {
    fn new(local_types: &'a UniqueMap<Var, SingleType>, signature: &'a FunctionSignature) -> Self {
        Self {
            local_types,
            signature,
        }
    }
}

struct Context<'a, 'b> {
    local_types: &'a UniqueMap<Var, SingleType>,
    local_states: &'b mut LocalStates,
    signature: &'a FunctionSignature,
    errors: Errors,
}

impl<'a, 'b> Context<'a, 'b> {
    fn new(locals_safety: &'a LocalsSafety, local_states: &'b mut LocalStates) -> Self {
        let local_types = &locals_safety.local_types;
        let signature = &locals_safety.signature;
        Self {
            local_types,
            local_states,
            signature,
            errors: vec![],
        }
    }

    fn error(&mut self, e: Vec<(Loc, impl Into<String>)>) {
        self.errors
            .push(e.into_iter().map(|(loc, msg)| (loc, msg.into())).collect())
    }

    fn get_errors(self) -> Errors {
        self.errors
    }

    fn get_state(&self, local: &Var) -> &LocalState {
        self.local_states.get_state(local)
    }

    fn set_state(&mut self, local: Var, state: LocalState) {
        self.local_states.set_state(local, state)
    }

    fn local_type(&self, local: &Var) -> &SingleType {
        self.local_types.get(local).unwrap()
    }
}

impl<'a> TransferFunctions for LocalsSafety<'a> {
    type State = LocalStates;

    fn execute(
        &mut self,
        pre: &mut Self::State,
        _lbl: Label,
        _idx: usize,
        cmd: &Command,
    ) -> Errors {
        let mut context = Context::new(self, pre);
        command(&mut context, cmd);
        context.get_errors()
    }
}

impl<'a> AbstractInterpreter for LocalsSafety<'a> {}

pub fn verify(
    errors: &mut Errors,
    signature: &FunctionSignature,
    _acquires: &BTreeMap<StructName, Loc>,
    locals: &UniqueMap<Var, SingleType>,
    cfg: &super::cfg::BlockCFG,
) -> BTreeMap<Label, LocalStates> {
    let initial_state = LocalStates::initial(&signature.parameters, locals);
    let mut locals_safety = LocalsSafety::new(locals, signature);
    let (final_state, mut es) = locals_safety.analyze_function(cfg, initial_state);
    errors.append(&mut es);
    final_state
}

//**************************************************************************************************
// Command
//**************************************************************************************************

fn command(context: &mut Context, sp!(loc, cmd_): &Command) {
    use Command_ as C;
    match cmd_ {
        C::Assign(ls, e) => {
            exp(context, e);
            lvalues(context, ls);
        }
        C::Mutate(el, er) => {
            exp(context, er);
            exp(context, el)
        }
        C::Abort(e) | C::IgnoreAndPop { exp: e, .. } | C::JumpIf { cond: e, .. } => exp(context, e),

        C::Return(e) => {
            exp(context, e);
            let mut errors = Errors::new();
            for (local, state) in context.local_states.iter() {
                match state {
                    LocalState::Unavailable(_) => (),
                    LocalState::Available(available)
                    | LocalState::MaybeUnavailable { available, .. } => {
                        let ty = context.local_type(&local);
                        let kind = ty.value.kind(ty.loc);
                        if kind.value.is_resourceful() {
                            let verb = match (state, &kind.value) {
                                (LocalState::Unavailable(_), _) => unreachable!(),
                                (LocalState::Available(_), Kind_::Resource) => "still contains",
                                _ => "might still contain",
                            };
                            let available = *available;
                            let stmt = match display_var(local.value()) {
                                DisplayVar::Tmp => {
                                    "The resource is created but not used".to_owned()
                                }
                                DisplayVar::Orig(l) => {
                                    if context.signature.is_parameter(&local) {
                                        format!("The parameter '{}' {} a resource value", l, verb)
                                    } else {
                                        format!(
                                            "The local '{}' {} a resource value due to this \
                                             assignment",
                                            l, verb
                                        )
                                    }
                                }
                            };
                            let msg = format!(
                                "{}. The resource must be consumed before the function returns",
                                stmt
                            );
                            errors.push(vec![(*loc, "Invalid return".into()), (available, msg)])
                        }
                    }
                }
            }
            errors.into_iter().for_each(|error| context.error(error))
        }
        C::Jump(_) => (),
        C::Break | C::Continue => panic!("ICE break/continue not translated to jumps"),
    }
}

fn lvalues(context: &mut Context, ls: &[LValue]) {
    ls.iter().for_each(|l| lvalue(context, l))
}

fn lvalue(context: &mut Context, sp!(loc, l_): &LValue) {
    use LValue_ as L;
    match l_ {
        L::Ignore => (),
        L::Var(v, _) => {
            let ty = context.local_type(v);
            let kind = ty.value.kind(ty.loc);
            if kind.value.is_resourceful() {
                let old_state = context.get_state(v);
                match old_state {
                    LocalState::Unavailable(_) => (),
                    LocalState::Available(available)
                    | LocalState::MaybeUnavailable { available, .. } => {
                        let verb = match (old_state, &kind.value) {
                            (LocalState::Unavailable(_), _) => unreachable!(),
                            (LocalState::Available(_), Kind_::Resource) => "contains",
                            _ => "might contain",
                        };
                        let available = *available;
                        let vstr = match display_var(v.value()) {
                            DisplayVar::Tmp => panic!("ICE invalid assign tmp local"),
                            DisplayVar::Orig(s) => s,
                        };
                        let msg = format!(
                            "The local {} a resource value due to this assignment. The resource \
                             must be used before you assign to this local again",
                            verb
                        );
                        context.error(vec![
                            (*loc, format!("Invalid assignment to local '{}'", vstr)),
                            (available, msg),
                        ])
                    }
                }
            }
            context.set_state(v.clone(), LocalState::Available(*loc))
        }
        L::Unpack(_, _, fields) => fields.iter().for_each(|(_, l)| lvalue(context, l)),
    }
}

fn exp(context: &mut Context, parent_e: &Exp) {
    use UnannotatedExp_ as E;
    let eloc = &parent_e.exp.loc;
    match &parent_e.exp.value {
        E::Unit { .. } | E::Value(_) | E::Constant(_) | E::Spec(_, _) | E::UnresolvedError => (),

        E::BorrowLocal(_, var) | E::Copy { var, .. } => use_local(context, eloc, var),

        E::Move { var, .. } => {
            use_local(context, eloc, var);
            context.set_state(var.clone(), LocalState::Unavailable(*eloc))
        }

        E::ModuleCall(mcall) => exp(context, &mcall.arguments),
        E::Builtin(_, e)
        | E::Freeze(e)
        | E::Dereference(e)
        | E::UnaryExp(_, e)
        | E::Borrow(_, e, _)
        | E::Cast(e, _) => exp(context, e),

        E::BinopExp(e1, _, e2) => {
            exp(context, e1);
            exp(context, e2)
        }

        E::Pack(_, _, fields) => fields.iter().for_each(|(_, _, e)| exp(context, e)),

        E::ExpList(es) => es.iter().for_each(|item| exp_list_item(context, item)),

        E::Unreachable => panic!("ICE should not analyze dead code"),
    }
}

fn exp_list_item(context: &mut Context, item: &ExpListItem) {
    match item {
        ExpListItem::Single(e, _) | ExpListItem::Splat(_, e, _) => exp(context, e),
    }
}

fn use_local(context: &mut Context, loc: &Loc, local: &Var) {
    use LocalState as L;
    let state = context.get_state(local);
    match state {
        L::Available(_) => (),
        L::Unavailable(unavailable) | L::MaybeUnavailable { unavailable, .. } => {
            let verb = match state {
                LocalState::Available(_) => unreachable!(),
                LocalState::Unavailable(_) => "does",
                LocalState::MaybeUnavailable { .. } => "might",
            };
            let unavailable = *unavailable;
            let vstr = match display_var(local.value()) {
                DisplayVar::Tmp => panic!("ICE invalid use tmp local {}", local.value()),
                DisplayVar::Orig(s) => s,
            };
            let msg = format!(
                "The local {} not have a value due to this position. The local must be assigned a \
                 value before being used",
                verb
            );
            context.error(vec![
                (*loc, format!("Invalid usage of local '{}'", vstr)),
                (unavailable, msg),
            ])
        }
    }
}
