// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_to_boogie::stackless_control_flow_graph::StacklessControlFlowGraph;
use bytecode_verifier::control_flow_graph::ControlFlowGraph;
use stackless_bytecode_generator::stackless_bytecode::StacklessBytecode::*;

#[test]
fn test_straight_line() {
    let instrs = [CopyLoc(3, 0), MoveLoc(4, 1), Add(5, 3, 4), Ret(vec![])];

    let cfg = StacklessControlFlowGraph::new(&instrs);

    assert_eq!(cfg.blocks(), vec![0]);
    assert!(cfg.successors(0).is_empty());
    assert_eq!(cfg.block_start(0), 0);
    assert_eq!(cfg.block_end(0), 3);
}

#[test]
fn test_branch() {
    let instrs = [
        LdTrue(0),
        BrFalse(4, 0),
        Branch(6),
        Branch(5),
        Branch(0),
        Branch(0),
        LdFalse(1),
        Not(2, 1),
        Not(3, 2),
        BrFalse(12, 3),
        LdU64(4, 42),
        Abort(4),
        Ret(vec![]),
    ];

    let cfg = StacklessControlFlowGraph::new(&instrs);

    assert_eq!(cfg.entry_block_id(), 0);
    assert_eq!(cfg.num_blocks(), 8);
    assert_eq!(cfg.blocks(), vec![0, 2, 3, 4, 5, 6, 10, 12]);

    assert_eq!(cfg.successors(0).to_vec(), vec![2, 4]);
    assert_eq!(cfg.block_start(0), 0);
    assert_eq!(cfg.block_end(0), 1);

    assert_eq!(cfg.successors(3).to_vec(), vec![5]);
    assert_eq!(cfg.block_start(3), 3);
    assert_eq!(cfg.block_end(3), 3);

    assert_eq!(cfg.successors(6).to_vec(), vec![10, 12]);
    assert_eq!(cfg.block_start(6), 6);
    assert_eq!(cfg.block_end(6), 9);

    assert!(cfg.successors(10).is_empty());
}
