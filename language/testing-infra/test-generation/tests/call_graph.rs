// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate test_generation;
use move_binary_format::file_format::FunctionHandleIndex;
use test_generation::abstract_state::CallGraph;

#[test]
fn call_graph_no_self_call_recursive() {
    let call_graph = CallGraph::new(10);
    let can_call = call_graph.can_call(FunctionHandleIndex(0));
    assert!(can_call.len() == 9);
    assert!(!can_call.iter().any(|fh| *fh == FunctionHandleIndex(0)));
}

#[test]
fn call_graph_simple_recursive_call() {
    let mut call_graph = CallGraph::new(10);
    call_graph.add_call(FunctionHandleIndex(0), FunctionHandleIndex(1));
    let can_call0 = call_graph.can_call(FunctionHandleIndex(0));
    let can_call1 = call_graph.can_call(FunctionHandleIndex(1));
    assert!(can_call0.len() == 9);
    assert!(can_call1.len() == 8);
    assert!(!can_call0.iter().any(|fh| *fh == FunctionHandleIndex(0)));
    assert!(!can_call1
        .iter()
        .any(|fh| *fh == FunctionHandleIndex(0) || *fh == FunctionHandleIndex(1)));
}

#[test]
fn call_graph_transitive_recursive_call() {
    let mut call_graph = CallGraph::new(10);
    call_graph.add_call(FunctionHandleIndex(1), FunctionHandleIndex(2));
    call_graph.add_call(FunctionHandleIndex(2), FunctionHandleIndex(0));
    let can_call0 = call_graph.can_call(FunctionHandleIndex(0));
    assert!(can_call0.len() == 7);
}

#[test]
fn call_graph_call_graph_depth() {
    let mut call_graph = CallGraph::new(10);
    call_graph.add_call(FunctionHandleIndex(0), FunctionHandleIndex(1));
    call_graph.add_call(FunctionHandleIndex(1), FunctionHandleIndex(2));
    call_graph.add_call(FunctionHandleIndex(1), FunctionHandleIndex(3));
    call_graph.add_call(FunctionHandleIndex(3), FunctionHandleIndex(2));
    assert!(
        call_graph
            .call_depth(FunctionHandleIndex(0), FunctionHandleIndex(1))
            .unwrap()
            == 3
    );
}

#[test]
fn call_graph_call_call_into_graph_depth() {
    let mut call_graph = CallGraph::new(10);
    call_graph.add_call(FunctionHandleIndex(0), FunctionHandleIndex(1));
    call_graph.add_call(FunctionHandleIndex(1), FunctionHandleIndex(2));
    call_graph.add_call(FunctionHandleIndex(1), FunctionHandleIndex(3));
    call_graph.add_call(FunctionHandleIndex(3), FunctionHandleIndex(2));
    assert!(call_graph.max_calling_depth(FunctionHandleIndex(2)) == 3);
}
