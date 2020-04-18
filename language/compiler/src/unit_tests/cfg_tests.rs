// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::unit_tests::testutils::compile_script_string;
use bytecode_verifier::control_flow_graph::{ControlFlowGraph, VMControlFlowGraph};
use vm::access::ScriptAccess;

#[test]
fn cfg_compile_script_ret() {
    let code = String::from(
        "
        main() {
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let cfg: VMControlFlowGraph = VMControlFlowGraph::new(&compiled_script.code().code);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 1);
    assert_eq!(cfg.num_blocks(), 1);
    assert_eq!(cfg.reachable_from(0).len(), 1);
}

#[test]
fn cfg_compile_script_let() {
    let code = String::from(
        "
        main() {
            let x: u64;
            let y: u64;
            let z: u64;
            x = 3;
            y = 5;
            z = move(x) + copy(y) * 5 - copy(y);
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let cfg: VMControlFlowGraph = VMControlFlowGraph::new(&compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 1);
    assert_eq!(cfg.num_blocks(), 1);
    assert_eq!(cfg.reachable_from(0).len(), 1);
}

#[test]
fn cfg_compile_if() {
    let code = String::from(
        "
        main() {
            let x: u64;
            x = 0;
            if (42 > 0) {
                x = 1;
            }
            return;
        }
        ",
    );

    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let cfg: VMControlFlowGraph = VMControlFlowGraph::new(&compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 3);
    assert_eq!(cfg.num_blocks(), 3);
    assert_eq!(cfg.reachable_from(0).len(), 3);
}

#[test]
fn cfg_compile_if_else() {
    let code = String::from(
        "
        main() {
            let x: u64;
            let y: u64;
            if (42 > 0) {
                x = 1;
                y = 2;
            } else {
                y = 2;
                x = 1;
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let cfg: VMControlFlowGraph = VMControlFlowGraph::new(&compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 4);
    assert_eq!(cfg.num_blocks(), 4);
    assert_eq!(cfg.reachable_from(0).len(), 4);
}

#[test]
fn cfg_compile_if_else_with_else_return() {
    let code = String::from(
        "
        main() {
            let x: u64;
            if (42 > 0) {
                x = 1;
            } else {
                return;
            }
            return;
        }
        ",
    );

    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let cfg: VMControlFlowGraph = VMControlFlowGraph::new(&compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 4);
    assert_eq!(cfg.num_blocks(), 4);
    assert_eq!(cfg.reachable_from(0).len(), 4);
}

#[test]
fn cfg_compile_nested_if() {
    let code = String::from(
        "
        main() {
            let x: u64;
            if (42 > 0) {
                x = 1;
            } else {
                if (5 > 10) {
                    x = 2;
                } else {
                    x = 3;
                }
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let cfg: VMControlFlowGraph = VMControlFlowGraph::new(&compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 6);
    assert_eq!(cfg.num_blocks(), 6);
    assert_eq!(cfg.reachable_from(7).len(), 4);
}

#[test]
fn cfg_compile_if_else_with_if_return() {
    let code = String::from(
        "
        main() {
            let x: u64;
            if (42 > 0) {
                return;
            } else {
                x = 1;
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let cfg: VMControlFlowGraph = VMControlFlowGraph::new(&compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 3);
    assert_eq!(cfg.num_blocks(), 3);
    assert_eq!(cfg.reachable_from(0).len(), 3);
    assert_eq!(cfg.reachable_from(4).len(), 1);
    assert_eq!(cfg.reachable_from(5).len(), 1);
}

#[test]
fn cfg_compile_if_else_with_two_returns() {
    let code = String::from(
        "
        main() {
            if (42 > 0) {
                return;
            } else {
                return;
            }
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let cfg: VMControlFlowGraph = VMControlFlowGraph::new(&compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 4);
    assert_eq!(cfg.num_blocks(), 4);
    assert_eq!(cfg.reachable_from(0).len(), 3);
    assert_eq!(cfg.reachable_from(4).len(), 1);
    assert_eq!(cfg.reachable_from(5).len(), 1);
    assert_eq!(cfg.reachable_from(6).len(), 1);
}

#[test]
fn cfg_compile_if_else_with_else_abort() {
    let code = String::from(
        "
        main() {
            let x: u64;
            if (42 > 0) {
                x = 1;
            } else {
                abort 0;
            }
            abort 0;
        }
        ",
    );

    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let cfg: VMControlFlowGraph = VMControlFlowGraph::new(&compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 4);
    assert_eq!(cfg.num_blocks(), 4);
    assert_eq!(cfg.reachable_from(0).len(), 4);
}

#[test]
fn cfg_compile_if_else_with_if_abort() {
    let code = String::from(
        "
        main() {
            let x: u64;
            if (42 > 0) {
                abort 0;
            } else {
                x = 1;
            }
            abort 0;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let cfg: VMControlFlowGraph = VMControlFlowGraph::new(&compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 3);
    assert_eq!(cfg.num_blocks(), 3);
    assert_eq!(cfg.reachable_from(0).len(), 3);
    assert_eq!(cfg.reachable_from(4).len(), 1);
    assert_eq!(cfg.reachable_from(6).len(), 1);
}

#[test]
fn cfg_compile_if_else_with_two_aborts() {
    let code = String::from(
        "
        main() {
            if (42 > 0) {
                abort 0;
            } else {
                abort 0;
            }
            abort 0;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let cfg: VMControlFlowGraph = VMControlFlowGraph::new(&compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 4);
    assert_eq!(cfg.num_blocks(), 4);
    assert_eq!(cfg.reachable_from(0).len(), 3);
    assert_eq!(cfg.reachable_from(4).len(), 1);
    assert_eq!(cfg.reachable_from(6).len(), 1);
    assert_eq!(cfg.reachable_from(8).len(), 1);
}
