// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::unit_tests::testutils::compile_script_string;
use move_binary_format::{
    access::ScriptAccess,
    control_flow_graph::{ControlFlowGraph, VMControlFlowGraph},
};
use move_bytecode_verifier::control_flow;

#[test]
fn cfg_compile_script_ret() {
    let code = String::from(
        "
        main() {
        label b0:
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let loop_bounds = control_flow::verify(None, compiled_script.code()).unwrap();
    let cfg = VMControlFlowGraph::new(loop_bounds, &compiled_script.code().code);
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
        label b0:
            x = 3;
            y = 5;
            z = move(x) + copy(y) * 5 - copy(y);
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let loop_bounds = control_flow::verify(None, compiled_script.code()).unwrap();
    let cfg = VMControlFlowGraph::new(loop_bounds, &compiled_script.code().code);
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
        label b0:
            x = 0;
            jump_if (42 > 0) b2;
        label b1:
            jump b3;
        label b2:
            x = 1;
            jump b3;
        label b3:
            return;
        }
        ",
    );

    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let loop_bounds = control_flow::verify(None, compiled_script.code()).unwrap();
    let cfg = VMControlFlowGraph::new(loop_bounds, &compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 4);
    assert_eq!(cfg.num_blocks(), 4);
    assert_eq!(cfg.reachable_from(0).len(), 4);
}

#[test]
fn cfg_compile_if_else() {
    let code = String::from(
        "
        main() {
            let x: u64;
            let y: u64;
        label b0:
            jump_if (42 > 0) b2;
        label b1:
            y = 2;
            x = 1;
            jump b3;
        label b2:
            x = 1;
            y = 2;
            jump b3;
        label b3:
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let loop_bounds = control_flow::verify(None, compiled_script.code()).unwrap();
    let cfg = VMControlFlowGraph::new(loop_bounds, &compiled_script.code().code);
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
        label b0:
            jump_if (42 > 0) b2;
        label b1:
            return;
        label b2:
            x = 1;
            jump b3;
        label b3:
            return;
        }
        ",
    );

    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let loop_bounds = control_flow::verify(None, compiled_script.code()).unwrap();
    let cfg = VMControlFlowGraph::new(loop_bounds, &compiled_script.code().code);
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
        label entry:
            jump_if (42 > 0) if_0_then;
        label if_0_else:
            jump_if (5 > 10) if_1_then;
        label if_1_else:
            x = 3;
            jump if_1_cont;
        label if_1_then:
            x = 2;
            jump if_1_cont;
        label if_1_cont:
            jump if_0_cont;
        label if_0_then:
            x = 1;
            jump if_0_cont;
        label if_0_cont:
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let loop_bounds = control_flow::verify(None, compiled_script.code()).unwrap();
    let cfg = VMControlFlowGraph::new(loop_bounds, &compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 7);
    assert_eq!(cfg.num_blocks(), 7);
    assert_eq!(cfg.reachable_from(8).len(), 3);
}

#[test]
fn cfg_compile_if_else_with_if_return() {
    let code = String::from(
        "
        main() {
            let x: u64;
        label b0:
            jump_if (42 > 0) b2;
        label b1:
            x = 1;
            jump b3;
        label b2:
            return;
        label b3:
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let loop_bounds = control_flow::verify(None, compiled_script.code()).unwrap();
    let cfg = VMControlFlowGraph::new(loop_bounds, &compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 4);
    assert_eq!(cfg.num_blocks(), 4);
    assert_eq!(cfg.reachable_from(0).len(), 4);
    assert_eq!(cfg.reachable_from(4).len(), 2);
    assert_eq!(cfg.reachable_from(8).len(), 1);
}

#[test]
fn cfg_compile_if_else_with_two_returns() {
    let code = String::from(
        "
        main() {
        label b0:
            jump_if (42 > 0) b2;
        label b1:
            return;
        label b2:
            return;
        label b3:
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let loop_bounds = control_flow::verify(None, compiled_script.code()).unwrap();
    let cfg = VMControlFlowGraph::new(loop_bounds, &compiled_script.code().code);
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
        label b0:
            jump_if (42 > 0) b2;
        label b1:
            abort 0;
        label b2:
            x = 1;
            jump b3;
        label b3:
            abort 0;
        }
        ",
    );

    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let loop_bounds = control_flow::verify(None, compiled_script.code()).unwrap();
    let cfg = VMControlFlowGraph::new(loop_bounds, &compiled_script.code().code);
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
        label b0:
            jump_if (42 > 0) b2;
        label b1:
            x = 1;
            jump b3;
        label b2:
            abort 0;
        label b3:
            abort 0;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let loop_bounds = control_flow::verify(None, compiled_script.code()).unwrap();
    let cfg = VMControlFlowGraph::new(loop_bounds, &compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 4);
    assert_eq!(cfg.num_blocks(), 4);
    assert_eq!(cfg.reachable_from(0).len(), 4);
    assert_eq!(cfg.reachable_from(4).len(), 2);
    assert_eq!(cfg.reachable_from(7).len(), 1);
}

#[test]
fn cfg_compile_if_else_with_two_aborts() {
    let code = String::from(
        "
        main() {
        label b0:
            jump_if (42 > 0) b2;
        label b1:
            abort 0;
        label b2:
            abort 0;
        label b3:
            abort 0;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    let loop_bounds = control_flow::verify(None, compiled_script.code()).unwrap();
    let cfg = VMControlFlowGraph::new(loop_bounds, &compiled_script.code().code);
    println!("SCRIPT:\n {:?}", compiled_script);
    cfg.display();
    assert_eq!(cfg.blocks().len(), 4);
    assert_eq!(cfg.num_blocks(), 4);
    assert_eq!(cfg.reachable_from(0).len(), 3);
    assert_eq!(cfg.reachable_from(4).len(), 1);
    assert_eq!(cfg.reachable_from(6).len(), 1);
    assert_eq!(cfg.reachable_from(8).len(), 1);
}
