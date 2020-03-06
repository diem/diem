// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate test_generation;
use test_generation::{abstract_state::AbstractState, transitions::Subst};
use vm::file_format::SignatureToken;

//---------------------------------------------------------------------------
// Substitution tests
//---------------------------------------------------------------------------

#[test]
fn unify_no_subst() {
    use SignatureToken::*;
    let state = AbstractState::new();
    let tys = vec![Bool, U64, Vector(Box::new(U8)), Address];
    for tok1 in tys.iter() {
        for tok2 in tys.iter() {
            let should_unify = tok1.clone() == tok2.clone();
            let mut s = Subst::new();
            assert!(s.check_and_add(&state, tok1.clone(), tok2.clone()) == should_unify);
            assert!(s.instantiation().is_empty());
        }
    }
}

#[test]
fn unify_ty_param_empty_subst1() {
    use SignatureToken::*;
    let state = AbstractState::new();
    let mut subst = Subst::new();
    assert!(subst.check_and_add(&state, Bool, TypeParameter(0)));
    assert!(!subst.check_and_add(&state, U64, TypeParameter(0)));
    assert!(!subst.check_and_add(&state, TypeParameter(0), U64));
    assert!(!subst.check_and_add(&state, TypeParameter(1), U64));
    assert!(subst.check_and_add(&state, U64, TypeParameter(1)));
    // Even if a type parameter can map to an instantiantion (due to a ground type on the stack) a
    // non-grounded type on the stack cannot be unified with a particular instantiation.
    assert!(!subst.check_and_add(&state, TypeParameter(1), U64));
    assert!(subst.instantiation().len() == 2);
}

#[test]
fn unify_ty_param_empty_subst2() {
    use SignatureToken::*;
    let state = AbstractState::new();
    let mut subst = Subst::new();
    assert!(subst.check_and_add(&state, U64, TypeParameter(0)));
    assert!(subst.check_and_add(&state, U64, TypeParameter(1)));
    assert!(subst.check_and_add(&state, Bool, TypeParameter(2)));

    assert!(!subst.check_and_add(&state, TypeParameter(0), U64));
    assert!(!subst.check_and_add(&state, TypeParameter(1), U64));
    assert!(!subst.check_and_add(&state, TypeParameter(2), Bool));

    assert!(subst.check_and_add(&state, U64, TypeParameter(0)));
    assert!(subst.check_and_add(&state, U64, TypeParameter(1)));
    assert!(subst.check_and_add(&state, Bool, TypeParameter(2)));

    assert!(!subst.check_and_add(&state, TypeParameter(0), TypeParameter(1)));

    assert!(!subst.check_and_add(&state, TypeParameter(0), TypeParameter(2)));
    assert!(!subst.check_and_add(&state, TypeParameter(1), TypeParameter(2)));

    assert!(!subst.check_and_add(&state, TypeParameter(2), TypeParameter(0)));
    assert!(!subst.check_and_add(&state, TypeParameter(2), TypeParameter(1)));
    assert!(subst.instantiation().len() == 3);
}

#[test]
fn unify_ty_params_infinite() {
    use SignatureToken::*;
    let state = AbstractState::new();
    let mut subst = Subst::new();
    assert!(subst.check_and_add(&state, TypeParameter(0), TypeParameter(1)));
    assert!(subst.check_and_add(&state, TypeParameter(1), TypeParameter(0)));
    // These should both return false.
    assert!(!subst.check_and_add(&state, Bool, TypeParameter(0)));
    assert!(!subst.check_and_add(&state, TypeParameter(0), Bool));
}

#[test]
fn unify_ty_param_empty_subst3() {
    use SignatureToken::*;
    let state = AbstractState::new();
    let mut subst = Subst::new();
    assert!(subst.check_and_add(&state, TypeParameter(1), TypeParameter(0)));
    assert!(subst.instantiation().len() == 1);
}

#[test]
fn unify_ty_param_empty_subst4() {
    use SignatureToken::*;
    let state = AbstractState::new();
    let mut subst = Subst::new();
    assert!(subst.check_and_add(&state, Bool, TypeParameter(0)));
    assert!(!subst.check_and_add(&state, U64, TypeParameter(0)));
    assert!(subst.check_and_add(&state, U64, TypeParameter(1)));
    assert!(subst.check_and_add(&state, U64, TypeParameter(2)));
}
