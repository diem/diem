// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod state;

use super::{
    absint::*,
    ast,
    ast::*,
    cfg::{BlockCFG, ReverseBlockCFG, CFG},
};
use crate::shared::unique_map::UniqueMap;
use crate::{errors::*, parser::ast::Var, shared::*};
use state::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

//**************************************************************************************************
// Entry and trait bindings
//**************************************************************************************************

type PerCommandStates = BTreeMap<Label, VecDeque<LivenessState>>;
type ForwardIntersections = BTreeMap<Label, LivenessState>;
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
    ) -> Errors {
        command(state, cmd);
        // set current [label][command_idx] data with the new liveness data
        let cur_label_states = self.states.get_mut(&label).unwrap();
        cur_label_states[idx] = state.clone();
        Errors::new()
    }
}

impl AbstractInterpreter for Liveness {}

pub fn refine_and_verify(
    errors: &mut Errors,
    _signature: &FunctionSignature,
    locals: &UniqueMap<Var, SingleType>,
    cfg: &mut BlockCFG,
    infinite_loop_starts: &BTreeSet<Label>,
) {
    let (_, per_command_states) = analyze(cfg, &infinite_loop_starts);
    last_usage(errors, locals, per_command_states, cfg.blocks_mut());
    let (final_invariants, per_command_states) = analyze(cfg, &infinite_loop_starts);
    let forward_intersections = build_forward_intersections(cfg, final_invariants);
    release_dead_refs(
        locals,
        forward_intersections,
        per_command_states,
        cfg.blocks_mut(),
    );
}

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
    let final_invariants = liveness.analyze_function(reverse, initial_state).unwrap();
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
        C::Return(e) | C::Abort(e) | C::IgnoreAndPop { exp: e, .. } | C::JumpIf { cond: e, .. } => {
            exp(state, e)
        }

        C::Jump(_) => (),
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
        E::Unit | E::Value(_) | E::UnresolvedError => (),

        E::BorrowLocal(_, var) | E::Copy { var, .. } | E::Move { var, .. } => {
            state.0.insert(var.clone());
        }

        E::ModuleCall(mcall) => exp(state, &mcall.arguments),
        E::Builtin(_, e)
        | E::Freeze(e)
        | E::Dereference(e)
        | E::UnaryExp(_, e)
        | E::Borrow(_, e, _) => exp(state, e),

        E::BinopExp(e1, _, e2) => {
            exp(state, e1);
            exp(state, e2)
        }

        E::Pack(_, _, fields) => fields.iter().for_each(|(_, _, e)| exp(state, e)),

        E::ExpList(es) => es.iter().for_each(|item| exp_list_item(state, item)),
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
///   Switches it to an `Ignore` if it is not a resource (helps with error messages for borrows)

fn last_usage(
    errors: &mut Errors,
    locals: &UniqueMap<Var, SingleType>,
    block_command_states: PerCommandStates,
    blocks: &mut Blocks,
) {
    for (lbl, block) in blocks {
        let command_states = block_command_states.get(lbl).unwrap();
        last_usage::block(errors, locals, command_states, block)
    }
}

mod last_usage {
    use crate::cfgir::ast::*;
    use crate::cfgir::liveness::state::LivenessState;
    use crate::hlir::translate::{display_var, DisplayVar};
    use crate::{errors::*, parser::ast::Var, shared::unique_map::*, shared::*};
    use std::collections::{BTreeSet, VecDeque};

    struct Context<'a, 'b> {
        errors: &'a mut Errors,
        locals: &'a UniqueMap<Var, SingleType>,
        next_live: &'b BTreeSet<Var>,
        dropped_live: BTreeSet<Var>,
    }

    impl<'a, 'b> Context<'a, 'b> {
        fn new(
            errors: &'a mut Errors,
            locals: &'a UniqueMap<Var, SingleType>,
            next_live: &'b BTreeSet<Var>,
            dropped_live: BTreeSet<Var>,
        ) -> Self {
            Context {
                errors,
                locals,
                next_live,
                dropped_live,
            }
        }

        fn is_resourceful(&self, local: &Var) -> bool {
            let ty = self.locals.get(local).unwrap();
            let k = ty.value.kind(ty.loc);
            k.value.is_resourceful()
        }

        fn error(&mut self, e: Vec<(Loc, impl Into<String>)>) {
            self.errors
                .push(e.into_iter().map(|(loc, msg)| (loc, msg.into())).collect())
        }
    }

    pub fn block(
        errors: &mut Errors,
        locals: &UniqueMap<Var, SingleType>,
        command_states: &VecDeque<LivenessState>,
        block: &mut BasicBlock,
    ) {
        let len = block.len();
        let last_cmd = block.get(len - 1).unwrap();
        assert!(
            last_cmd.value.is_terminal(),
            "ICE malformed block. missing jump"
        );
        for idx in 0..(len - 1) {
            let cmd = block.get_mut(idx).unwrap();
            let cur_data = command_states.get(idx).unwrap();
            let next_data = command_states.get(idx + 1).unwrap();

            let dropped_live = cur_data
                .0
                .difference(&next_data.0)
                .cloned()
                .collect::<BTreeSet<_>>();
            command(
                &mut Context::new(errors, locals, &next_data.0, dropped_live),
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
            C::Return(e)
            | C::Abort(e)
            | C::IgnoreAndPop { exp: e, .. }
            | C::JumpIf { cond: e, .. } => exp(context, e),

            C::Jump(_) => (),
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
                context.dropped_live.insert(v.clone());
                if !context.next_live.contains(v) {
                    match display_var(v.value()) {
                        DisplayVar::Tmp => (),
                        DisplayVar::Orig(v_str) => {
                            let msg = format!("Unused assignment or binding for local '{}'. Consider removing or replacing it with '_'", v_str);
                            context.error(vec![(l.loc, msg)]);
                            if !context.is_resourceful(v) {
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
            E::Unit | E::Value(_) | E::UnresolvedError => (),

            E::BorrowLocal(_, var) | E::Move { var, .. } => {
                // remove it from context to prevent accidental dropping in previous usages
                context.dropped_live.remove(var);
            }

            E::Copy { var, from_user } => {
                // Even if not switched to a move:
                // remove it from dropped_live to prevent accidental dropping in previous usages
                let var_is_dead = context.dropped_live.remove(var);
                if var_is_dead && *from_user {
                    match display_var(var.value()) {
                        DisplayVar::Tmp => (),
                        DisplayVar::Orig(v_str) => {
                            let msg = format!("Invalid 'copy'. The local '{}' is not live following this expression. Remove the 'copy' annotation or re-annotate as 'move'", v_str);
                            context.error(vec![(parent_e.exp.loc, msg)])
                        }
                    }
                }
                if var_is_dead {
                    parent_e.exp.value = E::Move {
                        var: var.clone(),
                        from_user: *from_user,
                    };
                }
            }

            E::ModuleCall(mcall) => exp(context, &mut mcall.arguments),
            E::Builtin(_, e)
            | E::Freeze(e)
            | E::Dereference(e)
            | E::UnaryExp(_, e)
            | E::Borrow(_, e, _) => exp(context, e),

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

fn release_dead_refs(
    locals: &UniqueMap<Var, SingleType>,
    forward_intersections: ForwardIntersections,
    block_command_states: PerCommandStates,
    blocks: &mut Blocks,
) {
    for (lbl, block) in &mut *blocks {
        let initial = forward_intersections.get(lbl).unwrap();
        let command_states = block_command_states.get(lbl).unwrap();
        release_dead_refs_block(locals, initial, command_states, block)
    }
}

fn build_forward_intersections(
    cfg: &BlockCFG,
    final_invariants: FinalInvariants,
) -> ForwardIntersections {
    cfg.blocks()
        .keys()
        .map(|lbl| {
            let mut states = cfg
                .predecessors(*lbl)
                .iter()
                .flat_map(|pred| final_invariants.get(pred))
                .map(|state| &state.0);
            let intersection = states
                .next()
                .map(|init| states.fold(init.clone(), |acc, s| &acc & s))
                .unwrap_or_else(BTreeSet::new);
            (*lbl, LivenessState(intersection))
        })
        .collect()
}

fn release_dead_refs_block(
    locals: &UniqueMap<Var, SingleType>,
    initial: &LivenessState,
    command_states: &VecDeque<LivenessState>,
    block: &mut BasicBlock,
) {
    let cmd_loc = block.get(0).unwrap().loc;
    let cur_data = command_states.get(0).unwrap();
    let dead_refs = initial
        .0
        .difference(&cur_data.0)
        .map(|var| (var, locals.get(var).unwrap()))
        .filter(is_ref);
    for (dead_ref, ty) in dead_refs {
        block.push_front(pop_ref(cmd_loc, dead_ref.clone(), ty.clone()));
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
    let move_e = ast::exp(Type_::single(ty), sp(loc, move_e_));
    let pop_ = C::IgnoreAndPop {
        pop_num: 1,
        exp: move_e,
    };
    sp(loc, pop_)
}
