// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_to_boogie::{
    lifetime_analysis::LifetimeAnalysis, stackless_control_flow_graph::StacklessControlFlowGraph,
};
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::{compiler::compile_module, parser::parse_module};
use libra_types::account_address::AccountAddress;
use stackless_bytecode_generator::{
    stackless_bytecode::StacklessBytecode, stackless_bytecode_generator::StacklessModuleGenerator,
};
use std::collections::BTreeSet;
use vm::file_format::{LocalIndex, SignatureToken};

#[test]
fn test_branching() {
    let code = String::from(
        "module TestIfRef {
            resource S {
                value: u64,
            }

            resource T {
                value: u64,
            }

            public branch_ref(cond: bool) acquires S, T {
                let value_ref: &mut u64;
                let s_ref: &mut Self.S;
                let t_ref: &mut Self.T;

                if (move(cond)) {
                    s_ref = borrow_global_mut<S>(get_txn_sender());
                    value_ref = &mut move(s_ref).value;
                } else {
                    t_ref = borrow_global_mut<T>(get_txn_sender());
                    value_ref = &mut move(t_ref).value;
                }

                *move(value_ref) = 42;
                return;
            }
        }
        ",
    );

    let (instrs, local_types) = gen_stackless_from_mvir(code);
    let cfg = StacklessControlFlowGraph::new(&instrs);

    // result is a map from CodeOffset to a set of references that go out of scope at that line
    let res = LifetimeAnalysis::analyze(&cfg, &instrs, local_types);

    for (code_offset, dead_refs) in res {
        // check that all three mutable references go out of scope only at the second last line
        if code_offset == 17 {
            let expected: BTreeSet<LocalIndex> = vec![7, 11, 14].iter().cloned().collect();
            assert_eq!(dead_refs, expected);
        } else {
            assert!(dead_refs.is_empty());
        }
    }
}

#[test]
fn test_loop() {
    let code = String::from(
        "module TestLoop {
            resource S {
                value: u64,
            }

            resource T {
                s: Self.S,
            }

            public test_loop(cond: bool) acquires T {
                let value_ref: &mut u64;
                let s_ref: &mut Self.S;
                let t_ref: &mut Self.T;
                let i: u64;

                i = 1;
                while (copy(cond)) {
                    t_ref = borrow_global_mut<T>(get_txn_sender());
                    s_ref = &mut move(t_ref).s;
                    value_ref = &mut move(s_ref).value;
                    *move(value_ref) = copy(i);
                    i = move(i)+1;
                }
                return;
            }
        }",
    );
    let (instrs, local_types) = gen_stackless_from_mvir(code);
    let cfg = StacklessControlFlowGraph::new(&instrs);
    let res = LifetimeAnalysis::analyze(&cfg, &instrs, local_types);
    for (code_offset, dead_refs) in res {
        // check that all three mutable references go out of scope only at WriteRef
        if code_offset == 15 {
            let expected: BTreeSet<LocalIndex> = vec![9, 11, 14].iter().cloned().collect();
            assert_eq!(dead_refs, expected);
        } else {
            assert!(dead_refs.is_empty());
        }
    }
}

#[test]
fn test_borrowloc_call() {
    let code = String::from(
        "module Test {
            resource S {
                value: u64,
            }

            public test_borrow_loc() : Self.S {
                let s: Self.S;
                let s_ref: &mut Self.S;
                let value_ref: &mut u64;

                s = S { value: 4 };
                s_ref = &mut s;
                value_ref = Self.mut_st(move(s_ref));
                *move(value_ref) = 42;
                return move(s);
            }

            public mut_st(s_ref: &mut Self.S) : &mut u64 {
                let value_ref: &mut u64;
                value_ref = &mut move(s_ref).value;
                *copy(value_ref) = 1;
                return move(value_ref);
            }
        }",
    );

    let (instrs, local_types) = gen_stackless_from_mvir(code);
    let cfg = StacklessControlFlowGraph::new(&instrs);
    let res = LifetimeAnalysis::analyze(&cfg, &instrs, local_types);
    for (code_offset, dead_refs) in res {
        // check that all three mutable references go out of scope only at WriteRef
        if code_offset == 10 {
            let expected: BTreeSet<LocalIndex> = vec![6, 9].iter().cloned().collect();
            assert_eq!(dead_refs, expected);
        } else {
            assert!(dead_refs.is_empty());
        }
    }
}
fn gen_stackless_from_mvir(code: String) -> (Vec<StacklessBytecode>, Vec<SignatureToken>) {
    let parsed_module = parse_module("file_name", &code).unwrap();
    let address = AccountAddress::default();
    let (compiled_module, _) =
        compile_module(address, parsed_module, &Vec::<VerifiedModule>::new()).unwrap();
    let stackless_module = StacklessModuleGenerator::new(&compiled_module).generate_module();
    let stackless_bytecode = stackless_module[0].code.clone();
    let local_types = stackless_module[0].local_types.clone();
    (stackless_bytecode, local_types)
}
