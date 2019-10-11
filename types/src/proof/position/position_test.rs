// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::proof::position::*;

/// Position is marked with in-order-traversal sequence.
///
/// For example
/// ```text
///     0
/// ```
///
/// Another example
/// ```text
///      3
///     /  \
///    /    \
///   1      5
///  / \    / \
/// 0   2  4   6
/// ```
#[test]
fn test_position_parent() {
    let position = Position::from_inorder_index(5);
    let target = position.parent();
    assert_eq!(target, Position::from_inorder_index(3));
}

#[test]
fn test_position_sibling_right() {
    let position = Position::from_inorder_index(5);
    let target = position.sibling();
    assert_eq!(target, Position::from_inorder_index(1));
}

#[test]
fn test_position_sibling_left() {
    let position = Position::from_inorder_index(4);
    let target = position.sibling();
    assert_eq!(target, Position::from_inorder_index(6));
}

#[test]
fn test_position_left_child() {
    let position = Position::from_inorder_index(5);
    let target = position.left_child();
    assert_eq!(target, Position::from_inorder_index(4));
}

#[test]
fn test_position_right_child() {
    let position = Position::from_inorder_index(5);
    let target = position.right_child();
    assert_eq!(target, Position::from_inorder_index(6));
}

#[test]
#[should_panic]
fn test_position_left_child_from_leaf() {
    let position = Position::from_inorder_index(0);
    let _target = position.left_child();
}
#[test]
#[should_panic]
fn test_position_right_child_from_leaf() {
    let position = Position::from_inorder_index(0);
    let _target = position.right_child();
}

#[test]
fn test_position_level() {
    let mut position = Position::from_inorder_index(5);
    let level = position.level();
    assert_eq!(level, 1);

    position = Position::from_inorder_index(0);
    let level = position.level();
    assert_eq!(level, 0);
}

#[test]
fn test_position_is_left_child() {
    assert!(Position::from_inorder_index(1).is_left_child());
    assert!(Position::from_inorder_index(0).is_left_child());
    assert!(Position::from_inorder_index(3).is_left_child());
    assert!(Position::from_inorder_index(7).is_left_child());
    assert!(Position::from_inorder_index(8).is_left_child());
    assert!(Position::from_inorder_index(12).is_left_child());
}

#[test]
fn test_position_is_right_child() {
    assert!(Position::from_inorder_index(5).is_right_child());
    assert!(Position::from_inorder_index(6).is_right_child());
    assert!(Position::from_inorder_index(2).is_right_child());
    assert!(Position::from_inorder_index(11).is_right_child());
    assert!(Position::from_inorder_index(13).is_right_child());
    assert!(Position::from_inorder_index(14).is_right_child());
    assert!(Position::from_inorder_index(10).is_right_child());
}

#[test]
fn test_position_root_from_leaf_index() {
    let target = Position::root_from_leaf_index(6);
    assert_eq!(target, Position::from_inorder_index(7));

    let target = Position::root_from_leaf_index(0);
    assert_eq!(target, Position::from_inorder_index(0));

    let target = Position::root_from_leaf_index(3);
    assert_eq!(target, Position::from_inorder_index(3));
}

#[test]
fn test_root_level_from_leaf_count() {
    assert_eq!(Position::root_level_from_leaf_count(1), 0);
    assert_eq!(Position::root_level_from_leaf_count(2), 1);
    assert_eq!(Position::root_level_from_leaf_count(3), 2);
    assert_eq!(Position::root_level_from_leaf_count(4), 2);
    for i in 1..100 {
        assert_eq!(
            Position::root_level_from_leaf_count(i),
            Position::root_from_leaf_count(i).level()
        );
    }
}

#[test]
fn test_is_freezable() {
    let mut position = Position::from_inorder_index(5);
    assert_eq!(position.is_freezable(2), false);
    assert_eq!(position.is_freezable(3), true);
    assert_eq!(position.is_freezable(4), true);

    position = Position::from_inorder_index(0);
    assert_eq!(position.is_freezable(0), true);
    assert_eq!(position.is_freezable(3), true);
    assert_eq!(position.is_freezable(4), true);

    // Testing a root
    position = Position::from_inorder_index(7);
    assert_eq!(position.is_freezable(6), false);
    assert_eq!(position.is_freezable(7), true);
    assert_eq!(position.is_freezable(8), true);

    // Testing a leaf
    position = Position::from_inorder_index(10);
    assert_eq!(position.is_freezable(5), true);
}

#[test]
fn test_is_freezable_out_of_boundary() {
    // Testing out of boundary
    let position = Position::from_inorder_index(10);
    assert_eq!(position.is_freezable(2), false);
}

#[test]
fn test_is_placeholder() {
    assert_eq!(Position::from_inorder_index(5).is_placeholder(0), true);
    assert_eq!(Position::from_inorder_index(5).is_placeholder(1), true);
    assert_eq!(Position::from_inorder_index(5).is_placeholder(2), false);
    assert_eq!(Position::from_inorder_index(5).is_placeholder(3), false);
    assert_eq!(Position::from_inorder_index(13).is_placeholder(5), true);
    assert_eq!(Position::from_inorder_index(13).is_placeholder(6), false);
}

#[test]
fn test_is_placeholder_out_of_boundary() {
    // Testing out of boundary
    assert_eq!(Position::from_inorder_index(7).is_placeholder(2), false);
    assert_eq!(Position::from_inorder_index(11).is_placeholder(2), true);
    assert_eq!(Position::from_inorder_index(14).is_placeholder(2), true);
}

#[test]
pub fn test_sibling_sequence() {
    let sibling_sequence1 = Position::from_inorder_index(0)
        .iter_ancestor_sibling()
        .take(20)
        .map(Position::to_inorder_index)
        .collect::<Vec<u64>>();
    assert_eq!(
        sibling_sequence1,
        vec![
            2, 5, 11, 23, 47, 95, 191, 383, 767, 1535, 3071, 6143, 12287, 24575, 49151, 98303,
            196_607, 393_215, 786_431, 1_572_863
        ]
    );

    let sibling_sequence2 = Position::from_inorder_index(6)
        .iter_ancestor_sibling()
        .take(20)
        .map(Position::to_inorder_index)
        .collect::<Vec<u64>>();
    assert_eq!(
        sibling_sequence2,
        vec![
            4, 1, 11, 23, 47, 95, 191, 383, 767, 1535, 3071, 6143, 12287, 24575, 49151, 98303,
            196_607, 393_215, 786_431, 1_572_863
        ]
    );

    let sibling_sequence3 = Position::from_inorder_index(7)
        .iter_ancestor_sibling()
        .take(20)
        .map(Position::to_inorder_index)
        .collect::<Vec<u64>>();
    assert_eq!(
        sibling_sequence3,
        vec![
            23, 47, 95, 191, 383, 767, 1535, 3071, 6143, 12287, 24575, 49151, 98303, 196_607,
            393_215, 786_431, 1_572_863, 3_145_727, 6_291_455, 12_582_911
        ]
    );
}

#[test]
pub fn test_parent_sequence() {
    let parent_sequence1 = Position::from_inorder_index(0)
        .iter_ancestor()
        .take(20)
        .map(Position::to_inorder_index)
        .collect::<Vec<u64>>();
    assert_eq!(
        parent_sequence1,
        vec![
            0, 1, 3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383, 32767, 65535,
            131_071, 262_143, 524_287
        ]
    );

    let parent_sequence2 = Position::from_inorder_index(12)
        .iter_ancestor()
        .take(20)
        .map(Position::to_inorder_index)
        .collect::<Vec<u64>>();
    assert_eq!(
        parent_sequence2,
        vec![
            12, 13, 11, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383, 32767, 65535,
            131_071, 262_143, 524_287
        ]
    );
}

fn slow_get_frozen_subtree_roots_impl(root: Position, max_leaf_index: u64) -> Vec<Position> {
    if root.is_freezable(max_leaf_index) {
        vec![root]
    } else if root.is_placeholder(max_leaf_index) {
        Vec::new()
    } else {
        let mut roots = slow_get_frozen_subtree_roots_impl(root.left_child(), max_leaf_index);
        roots.extend(slow_get_frozen_subtree_roots_impl(
            root.right_child(),
            max_leaf_index,
        ));
        roots
    }
}

fn slow_get_frozen_subtree_roots(num_leaves: LeafCount) -> Vec<Position> {
    if num_leaves == 0 {
        Vec::new()
    } else {
        let max_leaf_index = num_leaves - 1;
        let root = Position::root_from_leaf_count(num_leaves);
        slow_get_frozen_subtree_roots_impl(root, max_leaf_index)
    }
}

#[test]
fn test_frozen_subtree_iterator() {
    for n in 0..10000 {
        assert_eq!(
            FrozenSubTreeIterator::new(n).collect::<Vec<_>>(),
            slow_get_frozen_subtree_roots(n),
        );
    }
}

fn collect_all_positions(num_leaves: LeafCount, num_new_leaves: LeafCount) -> Vec<u64> {
    FrozenSubtreeSiblingIterator::new(num_leaves, num_new_leaves)
        .map(Position::to_inorder_index)
        .collect()
}

#[test]
fn test_frozen_subtree_sibling_iterator() {
    assert!(collect_all_positions(0, 0).is_empty());
    assert_eq!(collect_all_positions(0, 1), vec![0]);
    assert_eq!(collect_all_positions(0, 2), vec![1]);
    assert_eq!(collect_all_positions(0, 7), vec![3, 9, 12]);
    assert_eq!(collect_all_positions(0, 1 << 63), vec![(1 << 63) - 1]);

    assert!(collect_all_positions(1, 1).is_empty());
    assert_eq!(collect_all_positions(1, 2), vec![2]);
    assert_eq!(collect_all_positions(1, 3), vec![2, 4]);
    assert_eq!(collect_all_positions(1, 4), vec![2, 5]);
    assert_eq!(collect_all_positions(1, 5), vec![2, 5, 8]);
    assert_eq!(collect_all_positions(1, 1 << 63).len(), 63);

    assert!(collect_all_positions(2, 2).is_empty());
    assert_eq!(collect_all_positions(2, 3), vec![4]);
    assert_eq!(collect_all_positions(2, 4), vec![5]);
    assert_eq!(collect_all_positions(2, 5), vec![5, 8]);
    assert_eq!(collect_all_positions(2, 6), vec![5, 9]);
    assert_eq!(collect_all_positions(2, 7), vec![5, 9, 12]);
    assert_eq!(collect_all_positions(2, 1 << 63).len(), 62);

    assert!(collect_all_positions(3, 3).is_empty());
    assert_eq!(collect_all_positions(3, 4), vec![6]);
    assert_eq!(collect_all_positions(3, 5), vec![6, 8]);
    assert_eq!(collect_all_positions(3, 8), vec![6, 11]);
    assert_eq!(collect_all_positions(3, 1 << 63).len(), 62);

    assert!(collect_all_positions(6, 6).is_empty());
    assert_eq!(collect_all_positions(6, 7), vec![12]);
    assert_eq!(collect_all_positions(6, 8), vec![13]);
    assert_eq!(collect_all_positions(6, 16), vec![13, 23]);
    assert_eq!(collect_all_positions(6, 1 << 63).len(), 61);
}

/// Returns the number of children a node `level` nodes high in a perfect
/// binary tree has.
///
/// Recursively,
///
/// children_from_level(0) = 0
/// children_from_level(n) = 2 * (1 + children(n-1))
///
/// But expanding the series this can be computed non-recursively
/// sum 2^n, n=1 to x = 2^(x+1) - 2
fn children_from_level(level: u32) -> u64 {
    (1u64 << (level + 1)) - 2
}

fn slow_nodes_to_left_of(pos: Position) -> u64 {
    let ret_add = if pos == pos.parent().right_child() {
        children_from_level(pos.level()) + 1
    } else {
        0
    };
    let parent_add = if pos.pos_counting_from_left() == 0 {
        0
    } else {
        nodes_to_left_of(pos.parent().to_inorder_index())
    };
    ret_add + parent_add
}

fn test_invariant(invariant_fn: fn(Position) -> bool) {
    for x in 0..300 {
        let position = Position::from_inorder_index(x);
        assert!(
            invariant_fn(position),
            "position = {}",
            position.to_inorder_index()
        )
    }
}

fn test_invariant_non_leaf(invariant_fn: fn(Position) -> bool) {
    for x in 0..300 {
        let position = Position::from_inorder_index(x);
        assert!(
            position.level() == 0 || invariant_fn(position),
            "position = {}",
            position.to_inorder_index()
        )
    }
}

#[test]
fn test_basic_invariants() {
    test_invariant_non_leaf(|pos| pos == pos.right_child().parent());
    test_invariant_non_leaf(|pos| pos == pos.left_child().parent());

    test_invariant(|pos| pos.level() == pos.parent().level() - 1);
    test_invariant(|pos| {
        Position::from_level_and_pos(pos.level(), pos.pos_counting_from_left()) == pos
    });
    test_invariant(|pos| {
        Position::from_inorder_index(postorder_to_inorder(inorder_to_postorder(
            pos.to_inorder_index(),
        ))) == pos
    });

    test_invariant_non_leaf(|pos| {
        pos.right_child().pos_counting_from_left() == pos.left_child().pos_counting_from_left() + 1
    });

    test_invariant_non_leaf(|pos| pos.left_child().to_inorder_index() < pos.to_inorder_index());
    test_invariant_non_leaf(|pos| pos.to_inorder_index() < pos.right_child().to_inorder_index());
    test_invariant_non_leaf(|pos| {
        inorder_to_postorder(pos.left_child().to_inorder_index())
            < inorder_to_postorder(pos.to_inorder_index())
    });
    test_invariant_non_leaf(|pos| {
        inorder_to_postorder(pos.right_child().to_inorder_index())
            < inorder_to_postorder(pos.to_inorder_index())
    });

    test_invariant_non_leaf(|pos| {
        inorder_to_postorder(pos.right_child().to_inorder_index()) + 1
            == inorder_to_postorder(pos.to_inorder_index())
    });

    test_invariant_non_leaf(|pos| pos.right_child() == pos.left_child().sibling());
    test_invariant_non_leaf(|pos| pos.right_child().sibling() == pos.left_child());

    test_invariant_non_leaf(|pos| pos.right_child() == pos.child(NodeDirection::Right));
    test_invariant_non_leaf(|pos| pos.left_child() == pos.child(NodeDirection::Left));
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn test_position_extended() {
    for x in 0..300 {
        let pos = Position::from_inorder_index(x);
        assert_eq!(slow_nodes_to_left_of(pos), nodes_to_left_of(x));
        let pos = Position::from_inorder_index(x);
        assert_eq!(
            Position::from_level_and_pos(pos.level(), pos.pos_counting_from_left()),
            pos
        );
    }

    for x in &[1u64 << 33, 1u64 << 63] {
        let pos = Position::from_inorder_index(*x);
        assert_eq!(slow_nodes_to_left_of(pos), nodes_to_left_of(*x));
        let pos = Position::from_inorder_index(*x);
        assert_eq!(
            Position::from_level_and_pos(pos.level(), pos.pos_counting_from_left()),
            pos
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
    assert_eq!(Position::from_inorder_index(0).level(), 0);
    assert_eq!(Position::from_inorder_index(0).pos_counting_from_left(), 0);
    assert_eq!(inorder_to_postorder(0), 0);
    assert_eq!(postorder_to_inorder(0), 0);
    assert_eq!(
        Position::from_inorder_index(0).parent(),
        Position::from_inorder_index(1)
    );

    assert_eq!(Position::from_inorder_index(1).level(), 1);
    assert_eq!(Position::from_inorder_index(1).pos_counting_from_left(), 0);
    assert_eq!(inorder_to_postorder(1), 2);
    assert_eq!(postorder_to_inorder(2), 1);
    assert_eq!(
        Position::from_inorder_index(1).parent(),
        Position::from_inorder_index(3)
    );
    assert_eq!(
        Position::from_inorder_index(1).left_child(),
        Position::from_inorder_index(0)
    );
    assert_eq!(
        Position::from_inorder_index(1).right_child(),
        Position::from_inorder_index(2)
    );

    assert_eq!(Position::from_inorder_index(2).level(), 0);
    assert_eq!(Position::from_inorder_index(2).pos_counting_from_left(), 1);
    assert_eq!(inorder_to_postorder(2), 1);
    assert_eq!(postorder_to_inorder(1), 2);
    assert_eq!(
        Position::from_inorder_index(2).parent(),
        Position::from_inorder_index(1)
    );

    assert_eq!(Position::from_inorder_index(3).level(), 2);
    assert_eq!(Position::from_inorder_index(3).pos_counting_from_left(), 0);
    assert_eq!(inorder_to_postorder(3), 6);
    assert_eq!(postorder_to_inorder(6), 3);
    assert_eq!(
        Position::from_inorder_index(3).parent(),
        Position::from_inorder_index(7)
    );
    assert_eq!(
        Position::from_inorder_index(3).left_child(),
        Position::from_inorder_index(1)
    );
    assert_eq!(
        Position::from_inorder_index(3).right_child(),
        Position::from_inorder_index(5)
    );

    assert_eq!(Position::from_inorder_index(4).level(), 0);
    assert_eq!(Position::from_inorder_index(4).pos_counting_from_left(), 2);
    assert_eq!(inorder_to_postorder(4), 3);
    assert_eq!(postorder_to_inorder(3), 4);
    assert_eq!(
        Position::from_inorder_index(4).parent(),
        Position::from_inorder_index(5)
    );

    assert_eq!(Position::from_inorder_index(5).level(), 1);
    assert_eq!(Position::from_inorder_index(5).pos_counting_from_left(), 1);
    assert_eq!(inorder_to_postorder(5), 5);
    assert_eq!(postorder_to_inorder(5), 5);
    assert_eq!(
        Position::from_inorder_index(5).parent(),
        Position::from_inorder_index(3)
    );
    assert_eq!(
        Position::from_inorder_index(5).left_child(),
        Position::from_inorder_index(4)
    );
    assert_eq!(
        Position::from_inorder_index(5).right_child(),
        Position::from_inorder_index(6)
    );

    assert_eq!(Position::from_inorder_index(6).level(), 0);
    assert_eq!(Position::from_inorder_index(6).pos_counting_from_left(), 3);
    assert_eq!(inorder_to_postorder(6), 4);
    assert_eq!(postorder_to_inorder(4), 6);
    assert_eq!(
        Position::from_inorder_index(6).parent(),
        Position::from_inorder_index(5)
    );

    assert_eq!(Position::from_inorder_index(7).level(), 3);
    assert_eq!(Position::from_inorder_index(7).pos_counting_from_left(), 0);
    assert_eq!(inorder_to_postorder(7), 14);
    assert_eq!(postorder_to_inorder(14), 7);
    assert_eq!(
        Position::from_inorder_index(7).parent(),
        Position::from_inorder_index(15)
    );
    assert_eq!(
        Position::from_inorder_index(7).left_child(),
        Position::from_inorder_index(3)
    );
    assert_eq!(
        Position::from_inorder_index(7).right_child(),
        Position::from_inorder_index(11)
    );

    assert_eq!(Position::from_inorder_index(8).level(), 0);
    assert_eq!(Position::from_inorder_index(8).pos_counting_from_left(), 4);
    assert_eq!(inorder_to_postorder(8), 7);
    assert_eq!(postorder_to_inorder(7), 8);
    assert_eq!(
        Position::from_inorder_index(8).parent(),
        Position::from_inorder_index(9)
    );

    assert_eq!(Position::from_inorder_index(9).level(), 1);
    assert_eq!(Position::from_inorder_index(9).pos_counting_from_left(), 2);
    assert_eq!(inorder_to_postorder(9), 9);
    assert_eq!(postorder_to_inorder(9), 9);
    assert_eq!(
        Position::from_inorder_index(9).parent(),
        Position::from_inorder_index(11)
    );
    assert_eq!(
        Position::from_inorder_index(9).left_child(),
        Position::from_inorder_index(8)
    );
    assert_eq!(
        Position::from_inorder_index(9).right_child(),
        Position::from_inorder_index(10)
    );

    assert_eq!(Position::from_inorder_index(10).level(), 0);
    assert_eq!(Position::from_inorder_index(10).pos_counting_from_left(), 5);
    assert_eq!(inorder_to_postorder(10), 8);
    assert_eq!(postorder_to_inorder(8), 10);
    assert_eq!(
        Position::from_inorder_index(10).parent(),
        Position::from_inorder_index(9)
    );

    assert_eq!(Position::from_inorder_index(11).level(), 2);
    assert_eq!(Position::from_inorder_index(11).pos_counting_from_left(), 1);
    assert_eq!(inorder_to_postorder(11), 13);
    assert_eq!(postorder_to_inorder(13), 11);
    assert_eq!(
        Position::from_inorder_index(11).parent(),
        Position::from_inorder_index(7)
    );
    assert_eq!(
        Position::from_inorder_index(11).left_child(),
        Position::from_inorder_index(9)
    );
    assert_eq!(
        Position::from_inorder_index(11).right_child(),
        Position::from_inorder_index(13)
    );

    assert_eq!(Position::from_inorder_index(12).level(), 0);
    assert_eq!(Position::from_inorder_index(12).pos_counting_from_left(), 6);
    assert_eq!(inorder_to_postorder(12), 10);
    assert_eq!(postorder_to_inorder(10), 12);
    assert_eq!(
        Position::from_inorder_index(12).parent(),
        Position::from_inorder_index(13)
    );

    assert_eq!(Position::from_inorder_index(13).level(), 1);
    assert_eq!(Position::from_inorder_index(13).pos_counting_from_left(), 3);
    assert_eq!(inorder_to_postorder(13), 12);
    assert_eq!(postorder_to_inorder(12), 13);
    assert_eq!(
        Position::from_inorder_index(13).parent(),
        Position::from_inorder_index(11)
    );
    assert_eq!(
        Position::from_inorder_index(13).left_child(),
        Position::from_inorder_index(12)
    );
    assert_eq!(
        Position::from_inorder_index(13).right_child(),
        Position::from_inorder_index(14)
    );

    assert_eq!(Position::from_inorder_index(14).level(), 0);
    assert_eq!(Position::from_inorder_index(14).pos_counting_from_left(), 7);
    assert_eq!(inorder_to_postorder(14), 11);
    assert_eq!(postorder_to_inorder(11), 14);
    assert_eq!(
        Position::from_inorder_index(14).parent(),
        Position::from_inorder_index(13)
    );

    assert_eq!(Position::from_inorder_index(15).level(), 4);
    assert_eq!(Position::from_inorder_index(15).pos_counting_from_left(), 0);
    assert_eq!(inorder_to_postorder(15), 30);
    assert_eq!(postorder_to_inorder(30), 15);
    assert_eq!(
        Position::from_inorder_index(15).parent(),
        Position::from_inorder_index(31)
    );
    assert_eq!(
        Position::from_inorder_index(15).left_child(),
        Position::from_inorder_index(7)
    );
    assert_eq!(
        Position::from_inorder_index(15).right_child(),
        Position::from_inorder_index(23)
    );

    assert_eq!(Position::from_inorder_index(16).level(), 0);
    assert_eq!(Position::from_inorder_index(16).pos_counting_from_left(), 8);
    assert_eq!(inorder_to_postorder(16), 15);
    assert_eq!(postorder_to_inorder(15), 16);
    assert_eq!(
        Position::from_inorder_index(16).parent(),
        Position::from_inorder_index(17)
    );

    assert_eq!(Position::from_inorder_index(17).level(), 1);
    assert_eq!(Position::from_inorder_index(17).pos_counting_from_left(), 4);
    assert_eq!(inorder_to_postorder(17), 17);
    assert_eq!(postorder_to_inorder(17), 17);
    assert_eq!(
        Position::from_inorder_index(17).parent(),
        Position::from_inorder_index(19)
    );
    assert_eq!(
        Position::from_inorder_index(17).left_child(),
        Position::from_inorder_index(16)
    );
    assert_eq!(
        Position::from_inorder_index(17).right_child(),
        Position::from_inorder_index(18)
    );

    assert_eq!(Position::from_inorder_index(18).level(), 0);
    assert_eq!(Position::from_inorder_index(18).pos_counting_from_left(), 9);
    assert_eq!(inorder_to_postorder(18), 16);
    assert_eq!(postorder_to_inorder(16), 18);
    assert_eq!(
        Position::from_inorder_index(18).parent(),
        Position::from_inorder_index(17)
    );

    assert_eq!(Position::from_inorder_index(19).level(), 2);
    assert_eq!(Position::from_inorder_index(19).pos_counting_from_left(), 2);
    assert_eq!(inorder_to_postorder(19), 21);
    assert_eq!(postorder_to_inorder(21), 19);
    assert_eq!(
        Position::from_inorder_index(19).parent(),
        Position::from_inorder_index(23)
    );
    assert_eq!(
        Position::from_inorder_index(19).left_child(),
        Position::from_inorder_index(17)
    );
    assert_eq!(
        Position::from_inorder_index(19).right_child(),
        Position::from_inorder_index(21)
    );
}

#[test]
fn test_right_most_child() {
    assert_eq!(
        Position::from_inorder_index(0).right_most_child(),
        Position::from_inorder_index(0)
    );
    assert_eq!(
        Position::from_inorder_index(1).right_most_child(),
        Position::from_inorder_index(2)
    );
    assert_eq!(
        Position::from_inorder_index(5).right_most_child(),
        Position::from_inorder_index(6)
    );
    assert_eq!(
        Position::from_inorder_index(7).right_most_child(),
        Position::from_inorder_index(14)
    );
    assert_eq!(
        Position::from_inorder_index(3).right_most_child(),
        Position::from_inorder_index(6)
    );
    assert_eq!(
        Position::from_inorder_index(11).right_most_child(),
        Position::from_inorder_index(14)
    );
    assert_eq!(
        Position::from_inorder_index(12).right_most_child(),
        Position::from_inorder_index(12)
    );
    assert_eq!(
        Position::from_inorder_index(14).right_most_child(),
        Position::from_inorder_index(14)
    );
}

#[test]
fn test_left_most_child() {
    assert_eq!(
        Position::from_inorder_index(0).left_most_child(),
        Position::from_inorder_index(0)
    );
    assert_eq!(
        Position::from_inorder_index(1).left_most_child(),
        Position::from_inorder_index(0)
    );
    assert_eq!(
        Position::from_inorder_index(5).left_most_child(),
        Position::from_inorder_index(4)
    );
    assert_eq!(
        Position::from_inorder_index(7).left_most_child(),
        Position::from_inorder_index(0)
    );
    assert_eq!(
        Position::from_inorder_index(3).left_most_child(),
        Position::from_inorder_index(0)
    );
    assert_eq!(
        Position::from_inorder_index(11).left_most_child(),
        Position::from_inorder_index(8)
    );
    assert_eq!(
        Position::from_inorder_index(12).left_most_child(),
        Position::from_inorder_index(12)
    );
    assert_eq!(
        Position::from_inorder_index(14).left_most_child(),
        Position::from_inorder_index(14)
    );
}
