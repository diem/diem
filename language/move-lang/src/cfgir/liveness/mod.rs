// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod state;

use super::{
    absint::*,
    cfg::{BlockCFG, ReverseBlockCFG, CFG},
    locals,
};
use crate::{
    diagnostics::Diagnostics,
    hlir::ast::{self as H, *},
    parser::ast::Var,
    shared::{unique_map::UniqueMap, CompilationEnv},
};
use move_ir_types::location::*;
use state::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

//**************************************************************************************************
// Entry and trait bindings
//**************************************************************************************************

type PerCommandStates = BTreeMap<Label, VecDeque<LivenessState>>;
type ForwardIntersections = BTreeMap<Label, BTreeSet<Var>>;
type FinalInvariants = BTreeMap<Label, LivenessState>;

struct Liveness {
    states: PerCommandStates,
}

impl Liveness {
    fn new(cfg: &super::cfg::ReverseBlockCFG) -> Self {
        let states = cfg
            .blocks()
            .iter()
            .map(|(lbl, block)| {
                let init = block.iter().map(|_| LivenessState::initial()).collect();
                (*lbl, init)
            })
            .collect();
        Liveness { states }
    }
}

impl TransferFunctions for Liveness {
    type State = LivenessState;

    fn execute(
        &mut self,
        state: &mut Self::State,
        label: Label,
        idx: usize,
        cmd: &Command,
    ) -> Diagnostics {
        command(state, cmd);
        // set current [label][command_idx] data with the new liveness data
        let cur_label_states = self.states.get_mut(&label).unwrap();
        cur_label_states[idx] = state.clone();
        Diagnostics::new()
    }
}

impl AbstractInterpreter for Liveness {}

//**************************************************************************************************
// Analysis
//**************************************************************************************************

fn analyze(
    cfg: &mut BlockCFG,
    infinite_loop_starts: &BTreeSet<Label>,
) -> (FinalInvariants, PerCommandStates) {
    let reverse = &mut ReverseBlockCFG::new(cfg, infinite_loop_starts);
    let initial_state = LivenessState::initial();
    let mut liveness = Liveness::new(reverse);
    let (final_invariants, errors) = liveness.analyze_function(reverse, initial_state);
    assert!(errors.is_empty());
    (final_invariants, liveness.states)
}

fn command(state: &mut LivenessState, sp!(_, cmd_): &Command) {
    use Command_ as C;
    match cmd_ {
        C::Assign(ls, e) => {
            lvalues(state, ls);
            exp(state, e);
        }
        C::Mutate(el, er) => {
            exp(state, er);
            exp(state, el)
        }
        C::Return { exp: e, .. }
        | C::Abort(e)
        | C::IgnoreAndPop { exp: e, .. }
        | C::JumpIf { cond: e, .. } => exp(state, e),

        C::Jump { .. } => (),
        C::Break | C::Continue => panic!("ICE break/continue not translated to jumps"),
    }
}

fn lvalues(state: &mut LivenessState, ls: &[LValue]) {
    ls.iter().for_each(|l| lvalue(state, l))
}

fn lvalue(state: &mut LivenessState, sp!(_, l_): &LValue) {
    use LValue_ as L;
    match l_ {
        L::Ignore => (),
        L::Var(v, _) => {
            state.0.remove(v);
        }
        L::Unpack(_, _, fields) => fields.iter().for_each(|(_, l)| lvalue(state, l)),
    }
}

fn exp(state: &mut LivenessState, parent_e: &Exp) {
    use UnannotatedExp_ as E;
    match &parent_e.exp.value {
        E::Unit { .. } | E::Value(_) | E::Constant(_) | E::UnresolvedError => (),

        E::BorrowLocal(_, var) | E::Copy { var, .. } | E::Move { var, .. } => {
            state.0.insert(*var);
        }

        E::Spec(_, used_locals) => used_locals.keys().for_each(|v| {
            state.0.insert(*v);
        }),

        E::ModuleCall(mcall) => exp(state, &mcall.arguments),
        E::Builtin(_, e)
        | E::Freeze(e)
        | E::Dereference(e)
        | E::UnaryExp(_, e)
        | E::Borrow(_, e, _)
        | E::Cast(e, _) => exp(state, e),

        E::BinopExp(e1, _, e2) => {
            exp(state, e1);
            exp(state, e2)
        }

        E::Pack(_, _, fields) => fields.iter().for_each(|(_, _, e)| exp(state, e)),

        E::ExpList(es) => es.iter().for_each(|item| exp_list_item(state, item)),

        E::Unreachable => panic!("ICE should not analyze dead code"),
    }
}

fn exp_list_item(state: &mut LivenessState, item: &ExpListItem) {
    match item {
        ExpListItem::Single(e, _) | ExpListItem::Splat(_, e, _) => exp(state, e),
    }
}

//**************************************************************************************************
// Copy Refinement
//**************************************************************************************************

/// This pass:
/// - Switches the last inferred `copy` to a `move`.
///   It will error if the `copy` was specified by the user
/// - Reports an error if an assignment/let was not used
///   Switches it to an `Ignore` if it has the drop ability (helps with error messages for borrows)

pub fn last_usage(
    compilation_env: &mut CompilationEnv,
    locals: &UniqueMap<Var, SingleType>,
    cfg: &mut BlockCFG,
    infinite_loop_starts: &BTreeSet<Label>,
) {
    let (final_invariants, per_command_states) = analyze(cfg, infinite_loop_starts);
    for (lbl, block) in cfg.blocks_mut() {
        let final_invariant = final_invariants.get(lbl).unwrap();
        let command_states = per_command_states.get(lbl).unwrap();
        last_usage::block(
            compilation_env,
            locals,
            final_invariant,
            command_states,
            block,
        )
    }
}

mod last_usage {
    use crate::{
        cfgir::liveness::state::LivenessState,
        diag,
        hlir::{
            ast::*,
            translate::{display_var, DisplayVar},
        },
        parser::ast::{Ability_, Var},
        shared::{unique_map::*, *},
    };
    use std::collections::{BTreeSet, VecDeque};

    struct Context<'a, 'b> {
        env: &'a mut CompilationEnv,
        locals: &'a UniqueMap<Var, SingleType>,
        next_live: &'b BTreeSet<Var>,
        dropped_live: BTreeSet<Var>,
    }

    impl<'a, 'b> Context<'a, 'b> {
        fn new(
            env: &'a mut CompilationEnv,
            locals: &'a UniqueMap<Var, SingleType>,
            next_live: &'b BTreeSet<Var>,
            dropped_live: BTreeSet<Var>,
        ) -> Self {
            Context {
                env,
                locals,
                next_live,
                dropped_live,
            }
        }

        fn has_drop(&self, local: &Var) -> bool {
            let ty = self.locals.get(local).unwrap();
            ty.value.abilities(ty.loc).has_ability_(Ability_::Drop)
        }
    }

    pub fn block(
        compilation_env: &mut CompilationEnv,
        locals: &UniqueMap<Var, SingleType>,
        final_invariant: &LivenessState,
        command_states: &VecDeque<LivenessState>,
        block: &mut BasicBlock,
    ) {
        let len = block.len();
        let last_cmd = block.get(len - 1).unwrap();
        assert!(
            last_cmd.value.is_terminal(),
            "ICE malformed block. missing jump"
        );
        for idx in 0..len {
            let cmd = block.get_mut(idx).unwrap();
            let cur_data = &command_states.get(idx).unwrap().0;
            let next_data = match command_states.get(idx + 1) {
                Some(s) => &s.0,
                None => &final_invariant.0,
            };

            let dropped_live = cur_data
                .difference(next_data)
                .cloned()
                .collect::<BTreeSet<_>>();
            command(
                &mut Context::new(compilation_env, locals, next_data, dropped_live),
                cmd,
            )
        }
    }

    fn command(context: &mut Context, sp!(_, cmd_): &mut Command) {
        use Command_ as C;
        match cmd_ {
            C::Assign(ls, e) => {
                lvalues(context, ls);
                exp(context, e);
            }
            C::Mutate(el, er) => {
                exp(context, el);
                exp(context, er)
            }
            C::Return { exp: e, .. }
            | C::Abort(e)
            | C::IgnoreAndPop { exp: e, .. }
            | C::JumpIf { cond: e, .. } => exp(context, e),

            C::Jump { .. } => (),
            C::Break | C::Continue => panic!("ICE break/continue not translated to jumps"),
        }
    }

    fn lvalues(context: &mut Context, ls: &mut [LValue]) {
        ls.iter_mut().for_each(|l| lvalue(context, l))
    }

    fn lvalue(context: &mut Context, l: &mut LValue) {
        use LValue_ as L;
        match &mut l.value {
            L::Ignore => (),
            L::Var(v, _) => {
                context.dropped_live.insert(*v);
                if !context.next_live.contains(v) {
                    match display_var(v.value()) {
                        DisplayVar::Tmp => (),
                        DisplayVar::Orig(v_str) => {
                            if !v.starts_with_underscore() {
                                let msg = format!(
                                    "Unused assignment or binding for local '{}'. Consider \
                                     removing, replacing with '_', or prefixing with '_' (e.g., \
                                     '_{}')",
                                    v_str, v_str
                                );
                                context
                                    .env
                                    .add_diag(diag!(UnusedItem::Assignment, (l.loc, msg)));
                            }
                            if context.has_drop(v) {
                                l.value = L::Ignore
                            }
                        }
                    }
                }
            }
            L::Unpack(_, _, fields) => fields.iter_mut().for_each(|(_, l)| lvalue(context, l)),
        }
    }

    fn exp(context: &mut Context, parent_e: &mut Exp) {
        use UnannotatedExp_ as E;
        match &mut parent_e.exp.value {
            E::Unit { .. } | E::Value(_) | E::Constant(_) | E::UnresolvedError => (),

            E::BorrowLocal(_, var) | E::Move { var, .. } => {
                // remove it from context to prevent accidental dropping in previous usages
                context.dropped_live.remove(var);
            }

            E::Spec(_, used_locals) => {
                // remove it from context to prevent accidental dropping in previous usages
                used_locals.keys().for_each(|var| {
                    context.dropped_live.remove(var);
                })
            }

            E::Copy { var, from_user } => {
                // Even if not switched to a move:
                // remove it from dropped_live to prevent accidental dropping in previous usages
                let var_is_dead = context.dropped_live.remove(var);
                // Non-references might still be borrowed
                // Switching such non-locals to a copy is an optimization and not
                // needed for this refinement
                let is_reference = matches!(
                    &parent_e.ty.value,
                    Type_::Single(sp!(_, SingleType_::Ref(_, _)))
                );
                if var_is_dead && is_reference && !*from_user {
                    parent_e.exp.value = E::Move {
                        var: *var,
                        from_user: *from_user,
                    };
                }
            }

            E::ModuleCall(mcall) => exp(context, &mut mcall.arguments),
            E::Builtin(_, e)
            | E::Freeze(e)
            | E::Dereference(e)
            | E::UnaryExp(_, e)
            | E::Borrow(_, e, _)
            | E::Cast(e, _) => exp(context, e),

            E::BinopExp(e1, _, e2) => {
                exp(context, e2);
                exp(context, e1)
            }

            E::Pack(_, _, fields) => fields
                .iter_mut()
                .rev()
                .for_each(|(_, _, e)| exp(context, e)),

            E::ExpList(es) => es
                .iter_mut()
                .rev()
                .for_each(|item| exp_list_item(context, item)),

            E::Unreachable => panic!("ICE should not analyze dead code"),
        }
    }

    fn exp_list_item(context: &mut Context, item: &mut ExpListItem) {
        match item {
            ExpListItem::Single(e, _) | ExpListItem::Splat(_, e, _) => exp(context, e),
        }
    }
}

//**************************************************************************************************
// Refs Refinement
//**************************************************************************************************

/// This refinement releases dead reference values by adding a move + pop. In other words, if a
/// reference `r` is dead, it will insert `_ = move r` after the last usage
///
/// However, due to the previous `last_usage` analysis. Any last usage of a reference is a move.
/// And any unused assignment to a reference holding local is switched to a `Ignore`.
/// Thus the only way a reference could still be dead is if it was live in a loop
/// Additionally, the borrow checker will consider any reference to be released if it was released
/// in any predecessor.
/// As such, the only references that need to be released by an added `_ = move r` are references
/// at the beginning of a block given that
/// (1) The reference is live in the predecessor and the predecessor is a loop
/// (2)  The reference is live in ALL predecessors (otherwise the borrow checker will release them)
///
/// Because of this, `build_forward_intersections` intersects all of the forward post states of
/// predecessors.
/// Then `release_dead_refs_block` adds a release at the beginning of the block if the reference
/// satisfies (1) and (2)

pub fn release_dead_refs(
    locals_pre_states: &BTreeMap<Label, locals::state::LocalStates>,
    locals: &UniqueMap<Var, SingleType>,
    cfg: &mut BlockCFG,
    infinite_loop_starts: &BTreeSet<Label>,
) {
    let (liveness_pre_states, _per_command_states) = analyze(cfg, infinite_loop_starts);
    let forward_intersections = build_forward_intersections(cfg, &liveness_pre_states);
    for (lbl, block) in cfg.blocks_mut() {
        let locals_pre_state = locals_pre_states.get(lbl).unwrap();
        let liveness_pre_state = liveness_pre_states.get(lbl).unwrap();
        let forward_intersection = forward_intersections.get(lbl).unwrap();
        release_dead_refs_block(
            locals,
            locals_pre_state,
            liveness_pre_state,
            forward_intersection,
            block,
        )
    }
}

fn build_forward_intersections(
    cfg: &BlockCFG,
    final_invariants: &FinalInvariants,
) -> ForwardIntersections {
    cfg.blocks()
        .keys()
        .map(|lbl| {
            let mut states = cfg
                .predecessors(*lbl)
                .iter()
                .map(|pred| &final_invariants.get(pred).unwrap().0);
            let intersection = states
                .next()
                .map(|init| states.fold(init.clone(), |acc, s| &acc & s))
                .unwrap_or_else(BTreeSet::new);
            (*lbl, intersection)
        })
        .collect()
}

fn release_dead_refs_block(
    locals: &UniqueMap<Var, SingleType>,
    locals_pre_state: &locals::state::LocalStates,
    liveness_pre_state: &LivenessState,
    forward_intersection: &BTreeSet<Var>,
    block: &mut BasicBlock,
) {
    if forward_intersection.is_empty() {
        return;
    }

    let cmd_loc = block.get(0).unwrap().loc;
    let cur_state = {
        let mut s = liveness_pre_state.clone();
        for cmd in block.iter().rev() {
            command(&mut s, cmd);
        }
        s
    };
    // Free references that were live in ALL predecessors and that have a value
    // (could not have a value due to errors)
    let dead_refs = forward_intersection
        .difference(&cur_state.0)
        .filter(|var| locals_pre_state.get_state(var).is_available())
        .map(|var| (var, locals.get(var).unwrap()))
        .filter(is_ref);
    for (dead_ref, ty) in dead_refs {
        block.push_front(pop_ref(cmd_loc, *dead_ref, ty.clone()));
    }
}

fn is_ref((_local, sp!(_, local_ty_)): &(&Var, &SingleType)) -> bool {
    match local_ty_ {
        SingleType_::Ref(_, _) => true,
        SingleType_::Base(_) => false,
    }
}

fn pop_ref(loc: Loc, var: Var, ty: SingleType) -> Command {
    use Command_ as C;
    use UnannotatedExp_ as E;
    let move_e_ = E::Move {
        from_user: false,
        var,
    };
    let move_e = H::exp(Type_::single(ty), sp(loc, move_e_));
    let pop_ = C::IgnoreAndPop {
        pop_num: 1,
        exp: move_e,
    };
    sp(loc, pop_)
}
