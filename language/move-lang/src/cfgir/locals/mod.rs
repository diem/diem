// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod state;

use super::absint::*;
use crate::{
    diag,
    diagnostics::{Diagnostic, Diagnostics},
    expansion::ast::{AbilitySet, ModuleIdent},
    hlir::{
        ast::*,
        translate::{display_var, DisplayVar},
    },
    naming::ast::{self as N, TParam},
    parser::ast::{Ability_, StructName, Var},
    shared::{unique_map::UniqueMap, *},
};
use move_ir_types::location::*;
use state::*;
use std::collections::BTreeMap;

//**************************************************************************************************
// Entry and trait bindings
//**************************************************************************************************

struct LocalsSafety<'a> {
    struct_declared_abilities: &'a UniqueMap<ModuleIdent, UniqueMap<StructName, AbilitySet>>,
    local_types: &'a UniqueMap<Var, SingleType>,
    signature: &'a FunctionSignature,
}

impl<'a> LocalsSafety<'a> {
    fn new(
        struct_declared_abilities: &'a UniqueMap<ModuleIdent, UniqueMap<StructName, AbilitySet>>,
        local_types: &'a UniqueMap<Var, SingleType>,
        signature: &'a FunctionSignature,
    ) -> Self {
        Self {
            struct_declared_abilities,
            local_types,
            signature,
        }
    }
}

struct Context<'a, 'b> {
    struct_declared_abilities: &'a UniqueMap<ModuleIdent, UniqueMap<StructName, AbilitySet>>,
    local_types: &'a UniqueMap<Var, SingleType>,
    local_states: &'b mut LocalStates,
    signature: &'a FunctionSignature,
    diags: Diagnostics,
}

impl<'a, 'b> Context<'a, 'b> {
    fn new(locals_safety: &'a LocalsSafety, local_states: &'b mut LocalStates) -> Self {
        let struct_declared_abilities = &locals_safety.struct_declared_abilities;
        let local_types = &locals_safety.local_types;
        let signature = &locals_safety.signature;
        Self {
            struct_declared_abilities,
            local_types,
            local_states,
            signature,
            diags: Diagnostics::new(),
        }
    }

    fn add_diag(&mut self, d: Diagnostic) {
        self.diags.add(d)
    }

    fn extend_diags(&mut self, diags: Diagnostics) {
        self.diags.extend(diags)
    }

    fn get_diags(self) -> Diagnostics {
        self.diags
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
    ) -> Diagnostics {
        let mut context = Context::new(self, pre);
        command(&mut context, cmd);
        context.get_diags()
    }
}

impl<'a> AbstractInterpreter for LocalsSafety<'a> {}

pub fn verify(
    compilation_env: &mut CompilationEnv,
    struct_declared_abilities: &UniqueMap<ModuleIdent, UniqueMap<StructName, AbilitySet>>,
    signature: &FunctionSignature,
    _acquires: &BTreeMap<StructName, Loc>,
    locals: &UniqueMap<Var, SingleType>,
    cfg: &super::cfg::BlockCFG,
) -> BTreeMap<Label, LocalStates> {
    let initial_state = LocalStates::initial(&signature.parameters, locals);
    let mut locals_safety = LocalsSafety::new(struct_declared_abilities, locals, signature);
    let (final_state, ds) = locals_safety.analyze_function(cfg, initial_state);
    compilation_env.add_diags(ds);
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

        C::Return { exp: e, .. } => {
            exp(context, e);
            let mut diags = Diagnostics::new();
            for (local, state) in context.local_states.iter() {
                match state {
                    LocalState::Unavailable(_) => (),
                    LocalState::Available(available)
                    | LocalState::MaybeUnavailable { available, .. } => {
                        let ty = context.local_type(&local);
                        let abilities = ty.value.abilities(ty.loc);
                        if !abilities.has_ability_(Ability_::Drop) {
                            let verb = match state {
                                LocalState::Unavailable(_) => unreachable!(),
                                LocalState::Available(_) => "still contains",
                                LocalState::MaybeUnavailable { .. } => "might still contain",
                            };
                            let available = *available;
                            let stmt = match display_var(local.value()) {
                                DisplayVar::Tmp => "The value is created but not used".to_owned(),
                                DisplayVar::Orig(l) => {
                                    if context.signature.is_parameter(&local) {
                                        format!("The parameter '{}' {} a value", l, verb,)
                                    } else {
                                        format!("The local variable '{}' {} a value", l, verb,)
                                    }
                                }
                            };
                            let msg = format!(
                                "{}. The value does not have the '{}' ability and must be \
                                 consumed before the function returns",
                                stmt,
                                Ability_::Drop,
                            );
                            let mut diag = diag!(
                                MoveSafety::UnusedUndroppable,
                                (*loc, "Invalid return"),
                                (available, msg)
                            );
                            add_drop_ability_tip(context, &mut diag, ty.clone());
                            diags.add(diag);
                        }
                    }
                }
            }
            context.extend_diags(diags)
        }
        C::Jump { .. } => (),
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
            let abilities = ty.value.abilities(ty.loc);
            if !abilities.has_ability_(Ability_::Drop) {
                let old_state = context.get_state(v);
                match old_state {
                    LocalState::Unavailable(_) => (),
                    LocalState::Available(available)
                    | LocalState::MaybeUnavailable { available, .. } => {
                        let verb = match old_state {
                            LocalState::Unavailable(_) => unreachable!(),
                            LocalState::Available(_) => "contains",
                            LocalState::MaybeUnavailable { .. } => "might contain",
                        };
                        let available = *available;
                        let vstr = match display_var(v.value()) {
                            DisplayVar::Tmp => panic!("ICE invalid assign tmp local"),
                            DisplayVar::Orig(s) => s,
                        };
                        let msg = format!(
                            "The variable {} a value due to this assignment. The value does not \
                             have the '{}' ability and must be used before you assign to this \
                             variable again",
                            verb,
                            Ability_::Drop,
                        );
                        let mut diag = diag!(
                            MoveSafety::UnusedUndroppable,
                            (*loc, format!("Invalid assignment to variable '{}'", vstr)),
                            (available, msg),
                        );
                        add_drop_ability_tip(context, &mut diag, ty.clone());
                        context.add_diag(diag)
                    }
                }
            }
            context.set_state(*v, LocalState::Available(*loc))
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
            context.set_state(*var, LocalState::Unavailable(*eloc))
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
                "The variable {} not have a value due to this position. The variable must be \
                 assigned a value before being used",
                verb
            );
            context.add_diag(diag!(
                MoveSafety::UnassignedVariable,
                (*loc, format!("Invalid usage of variable '{}'", vstr)),
                (unavailable, msg),
            ))
        }
    }
}

//**************************************************************************************************
// Error helper
//**************************************************************************************************

fn add_drop_ability_tip(context: &Context, diag: &mut Diagnostic, st: SingleType) {
    use N::{TypeName_ as TN, Type_ as T};
    let ty = single_type_to_naming_type(st);
    let owned_abilities;
    let (declared_loc_opt, declared_abilities, ty_args) = match &ty.value {
        T::Param(TParam {
            user_specified_name,
            abilities,
            ..
        }) => (Some(user_specified_name.loc), abilities, vec![]),
        T::Apply(_, sp!(_, TN::Builtin(b)), ty_args) => {
            owned_abilities = b.value.declared_abilities(b.loc);
            (None, &owned_abilities, ty_args.clone())
        }
        T::Apply(_, sp!(_, TN::ModuleType(m, s)), ty_args) => {
            let decl_loc = *context
                .struct_declared_abilities
                .get(m)
                .unwrap()
                .get_loc(s)
                .unwrap();
            let declared_abilities = context
                .struct_declared_abilities
                .get(m)
                .unwrap()
                .get(s)
                .unwrap();
            (Some(decl_loc), declared_abilities, ty_args.clone())
        }
        t => panic!(
            "ICE either the type did not have 'drop' when it should have or it was converted \
             incorrectly {:?}",
            t
        ),
    };
    crate::typing::core::ability_not_satisified_tips(
        &crate::typing::core::Subst::empty(),
        diag,
        Ability_::Drop,
        &ty,
        declared_loc_opt,
        declared_abilities,
        ty_args.iter().map(|ty_arg| {
            let abilities = match &ty_arg.value {
                T::Unit => AbilitySet::collection(ty_arg.loc),
                T::Ref(_, _) => AbilitySet::references(ty_arg.loc),
                T::UnresolvedError | T::Anything => AbilitySet::all(ty_arg.loc),
                T::Param(TParam { abilities, .. }) | T::Apply(Some(abilities), _, _) => {
                    abilities.clone()
                }
                T::Var(_) | T::Apply(None, _, _) => panic!("ICE expansion failed"),
            };
            (ty_arg, abilities)
        }),
    )
}

fn single_type_to_naming_type(sp!(loc, st_): SingleType) -> N::Type {
    sp(loc, single_type_to_naming_type_(st_))
}

fn single_type_to_naming_type_(st_: SingleType_) -> N::Type_ {
    use SingleType_ as S;
    use N::Type_ as T;
    match st_ {
        S::Ref(mut_, b) => T::Ref(mut_, Box::new(base_type_to_naming_type(b))),
        S::Base(sp!(_, b_)) => base_type_to_naming_type_(b_),
    }
}

fn base_type_to_naming_type(sp!(loc, bt_): BaseType) -> N::Type {
    sp(loc, base_type_to_naming_type_(bt_))
}

fn base_type_to_naming_type_(bt_: BaseType_) -> N::Type_ {
    use BaseType_ as B;
    use N::Type_ as T;
    match bt_ {
        B::Unreachable => T::Anything,
        B::UnresolvedError => T::UnresolvedError,
        B::Param(tp) => T::Param(tp),
        B::Apply(abilities, tn, ty_args) => T::Apply(
            Some(abilities),
            type_name_to_naming_type_name(tn),
            ty_args.into_iter().map(base_type_to_naming_type).collect(),
        ),
    }
}

fn type_name_to_naming_type_name(sp!(loc, tn_): TypeName) -> N::TypeName {
    sp(loc, type_name_to_naming_type_name_(tn_))
}

fn type_name_to_naming_type_name_(tn_: TypeName_) -> N::TypeName_ {
    use TypeName_ as TN;
    use N::TypeName_ as NTN;
    match tn_ {
        TN::Builtin(b) => NTN::Builtin(b),
        TN::ModuleType(m, n) => NTN::ModuleType(m, n),
    }
}
