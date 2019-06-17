// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::proof::treebits::*;

fn slow_nodes_to_left_of(node: u64) -> u64 {
    let ret_add = if node == right_child(parent(node)) {
        children_from_level(level(node)) + 1
    } else {
        0
    };
    let parent_add = if pos_counting_from_left(node) == 0 {
        0
    } else {
        nodes_to_left_of(parent(node))
    };
    ret_add + parent_add
}

fn test_invariant(invariant_fn: fn(u64) -> bool) {
    for node in 0..300 {
        assert!(invariant_fn(node), "node = {}", node)
    }
}

fn test_invariant_non_leaf(invariant_fn: fn(u64) -> bool) {
    for node in 0..300 {
        assert!(level(node) == 0 || invariant_fn(node), "node = {}", node)
    }
}

#[test]
fn test_basic_invariants() {
    test_invariant_non_leaf(|node| node == parent(right_child(node)));
    test_invariant_non_leaf(|node| node == parent(left_child(node)));

    test_invariant(|node| level(node) == level(parent(node)) - 1);
    test_invariant(|node| {
        node_from_level_and_pos(level(node), pos_counting_from_left(node)) == node
    });

    test_invariant_non_leaf(|node| {
        pos_counting_from_left(right_child(node)) == pos_counting_from_left(left_child(node)) + 1
    });

    test_invariant_non_leaf(|node| left_child(node) < node);
    test_invariant_non_leaf(|node| node < right_child(node));
    test_invariant_non_leaf(|node| post_order_index(left_child(node)) < post_order_index(node));
    test_invariant_non_leaf(|node| post_order_index(right_child(node)) < post_order_index(node));

    test_invariant_non_leaf(|node| {
        post_order_index(right_child(node)) + 1 == post_order_index(node)
    });

    test_invariant_non_leaf(|node| right_child(node) == sibling(left_child(node)));
    test_invariant_non_leaf(|node| sibling(right_child(node)) == left_child(node));

    test_invariant_non_leaf(|node| right_child(node) == child(node, NodeDirection::Right));
    test_invariant_non_leaf(|node| left_child(node) == child(node, NodeDirection::Left));
    test_invariant(|node| node == child(parent(node), direction_from_parent(node)));
}

#[test]
fn test_children_from_level_nary() {
    assert_eq!(children_from_level_nary(0, 2), 0);
    assert_eq!(children_from_level_nary(1, 2), 4);
    assert_eq!(children_from_level_nary(2, 2), 20);
    assert_eq!(children_from_level_nary(3, 2), 84);
}

#[test]
fn test_disk_write_order() {
    assert_eq!(disk_write_order(0, 2), 0);
    assert_eq!(disk_write_order(1, 2), 0);
    assert_eq!(disk_write_order(2, 2), 0);
    assert_eq!(disk_write_order(4, 2), 1);
    assert_eq!(disk_write_order(3, 2), 4);
    assert_eq!(disk_write_order(35, 2), 14);

    assert_eq!(disk_write_order(57, 2), 17);
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn test_treebits() {
    for x in 0..300 {
        assert_eq!(slow_nodes_to_left_of(x), nodes_to_left_of(x));
        assert_eq!(
            node_from_level_and_pos(level(x), pos_counting_from_left(x)),
            x
        );
    }

    for x in &[1u64 << 33, 1u64 << 63] {
        assert_eq!(slow_nodes_to_left_of(*x), nodes_to_left_of(*x));
        assert_eq!(
            node_from_level_and_pos(level(*x), pos_counting_from_left(*x)),
            *x
        );
    }

    assert_eq!(children_from_level(0), 0);
    assert_eq!(children_from_level(1), 2);
    assert_eq!(children_from_level(2), 6);
    assert_eq!(children_from_level(3), 14);
    assert_eq!(children_from_level(4), 30);
    assert_eq!(children_from_level(5), 62);
    assert_eq!(children_from_level(6), 126);
    assert_eq!(children_from_level(7), 254);
    assert_eq!(children_from_level(8), 510);
    assert_eq!(children_from_level(9), 1022);
    // Test for level > 32 to discover overflow bugs
    assert_eq!(children_from_level(50), 2_251_799_813_685_246);
    assert_eq!(level(0), 0);
    assert_eq!(pos_counting_from_left(0), 0);
    assert_eq!(post_order_index(0), 0);
    assert_eq!(parent(0), 1);

    assert_eq!(level(1), 1);
    assert_eq!(pos_counting_from_left(1), 0);
    assert_eq!(post_order_index(1), 2);
    assert_eq!(parent(1), 3);
    assert_eq!(left_child(1), 0);
    assert_eq!(right_child(1), 2);

    assert_eq!(level(2), 0);
    assert_eq!(pos_counting_from_left(2), 1);
    assert_eq!(post_order_index(2), 1);
    assert_eq!(parent(2), 1);

    assert_eq!(level(3), 2);
    assert_eq!(pos_counting_from_left(3), 0);
    assert_eq!(post_order_index(3), 6);
    assert_eq!(parent(3), 7);
    assert_eq!(left_child(3), 1);
    assert_eq!(right_child(3), 5);

    assert_eq!(level(4), 0);
    assert_eq!(pos_counting_from_left(4), 2);
    assert_eq!(post_order_index(4), 3);
    assert_eq!(parent(4), 5);

    assert_eq!(level(5), 1);
    assert_eq!(pos_counting_from_left(5), 1);
    assert_eq!(post_order_index(5), 5);
    assert_eq!(parent(5), 3);
    assert_eq!(left_child(5), 4);
    assert_eq!(right_child(5), 6);

    assert_eq!(level(6), 0);
    assert_eq!(pos_counting_from_left(6), 3);
    assert_eq!(post_order_index(6), 4);
    assert_eq!(parent(6), 5);

    assert_eq!(level(7), 3);
    assert_eq!(pos_counting_from_left(7), 0);
    assert_eq!(post_order_index(7), 14);
    assert_eq!(parent(7), 15);
    assert_eq!(left_child(7), 3);
    assert_eq!(right_child(7), 11);

    assert_eq!(level(8), 0);
    assert_eq!(pos_counting_from_left(8), 4);
    assert_eq!(post_order_index(8), 7);
    assert_eq!(parent(8), 9);

    assert_eq!(level(9), 1);
    assert_eq!(pos_counting_from_left(9), 2);
    assert_eq!(post_order_index(9), 9);
    assert_eq!(parent(9), 11);
    assert_eq!(left_child(9), 8);
    assert_eq!(right_child(9), 10);

    assert_eq!(level(10), 0);
    assert_eq!(pos_counting_from_left(10), 5);
    assert_eq!(post_order_index(10), 8);
    assert_eq!(parent(10), 9);

    assert_eq!(level(11), 2);
    assert_eq!(pos_counting_from_left(11), 1);
    assert_eq!(post_order_index(11), 13);
    assert_eq!(parent(11), 7);
    assert_eq!(left_child(11), 9);
    assert_eq!(right_child(11), 13);

    assert_eq!(level(12), 0);
    assert_eq!(pos_counting_from_left(12), 6);
    assert_eq!(post_order_index(12), 10);
    assert_eq!(parent(12), 13);

    assert_eq!(level(13), 1);
    assert_eq!(pos_counting_from_left(13), 3);
    assert_eq!(post_order_index(13), 12);
    assert_eq!(parent(13), 11);
    assert_eq!(left_child(13), 12);
    assert_eq!(right_child(13), 14);

    assert_eq!(level(14), 0);
    assert_eq!(pos_counting_from_left(14), 7);
    assert_eq!(post_order_index(14), 11);
    assert_eq!(parent(14), 13);

    assert_eq!(level(15), 4);
    assert_eq!(pos_counting_from_left(15), 0);
    assert_eq!(post_order_index(15), 30);
    assert_eq!(parent(15), 31);
    assert_eq!(left_child(15), 7);
    assert_eq!(right_child(15), 23);

    assert_eq!(level(16), 0);
    assert_eq!(pos_counting_from_left(16), 8);
    assert_eq!(post_order_index(16), 15);
    assert_eq!(parent(16), 17);

    assert_eq!(level(17), 1);
    assert_eq!(pos_counting_from_left(17), 4);
    assert_eq!(post_order_index(17), 17);
    assert_eq!(parent(17), 19);
    assert_eq!(left_child(17), 16);
    assert_eq!(right_child(17), 18);

    assert_eq!(level(18), 0);
    assert_eq!(pos_counting_from_left(18), 9);
    assert_eq!(post_order_index(18), 16);
    assert_eq!(parent(18), 17);

    assert_eq!(level(19), 2);
    assert_eq!(pos_counting_from_left(19), 2);
    assert_eq!(post_order_index(19), 21);
    assert_eq!(parent(19), 23);
    assert_eq!(left_child(19), 17);
    assert_eq!(right_child(19), 21);
}

#[test]
fn test_right_most_child() {
    assert_eq!(right_most_child(0), 0);
    assert_eq!(right_most_child(1), 2);
    assert_eq!(right_most_child(5), 6);
    assert_eq!(right_most_child(7), 14);
    assert_eq!(right_most_child(3), 6);
    assert_eq!(right_most_child(11), 14);
    assert_eq!(right_most_child(12), 12);
    assert_eq!(right_most_child(14), 14);
}

#[test]
fn test_left_most_child() {
    assert_eq!(left_most_child(0), 0);
    assert_eq!(left_most_child(1), 0);
    assert_eq!(left_most_child(5), 4);
    assert_eq!(left_most_child(7), 0);
    assert_eq!(left_most_child(3), 0);
    assert_eq!(left_most_child(11), 8);
    assert_eq!(left_most_child(12), 12);
    assert_eq!(left_most_child(14), 14);
}
