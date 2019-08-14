// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::proof::position::{treebits::pos_counting_from_left, *};

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
fn test_position_get_parent() {
    let position = Position::from_inorder_index(5);
    let target = position.get_parent();
    assert_eq!(target, Position::from_inorder_index(3));
}

#[test]
fn test_position_get_sibling_right() {
    let position = Position::from_inorder_index(5);
    let target = position.get_sibling();
    assert_eq!(target, Position::from_inorder_index(1));
}

#[test]
fn test_position_get_sibling_left() {
    let position = Position::from_inorder_index(4);
    let target = position.get_sibling();
    assert_eq!(target, Position::from_inorder_index(6));
}

#[test]
fn test_position_get_left_child() {
    let position = Position::from_inorder_index(5);
    let target = position.get_left_child();
    assert_eq!(target, Position::from_inorder_index(4));
}

#[test]
fn test_position_get_right_child() {
    let position = Position::from_inorder_index(5);
    let target = position.get_right_child();
    assert_eq!(target, Position::from_inorder_index(6));
}

#[test]
#[should_panic]
fn test_position_get_left_child_from_leaf() {
    let position = Position::from_inorder_index(0);
    let _target = position.get_left_child();
}
#[test]
#[should_panic]
fn test_position_get_right_child_from_leaf() {
    let position = Position::from_inorder_index(0);
    let _target = position.get_right_child();
}

#[test]
fn test_position_get_level() {
    let mut position = Position::from_inorder_index(5);
    let level = position.get_level();
    assert_eq!(level, 1);

    position = Position::from_inorder_index(0);
    let level = position.get_level();
    assert_eq!(level, 0);
}

#[test]
fn test_position_get_next_sibling() {
    for i in 0..1000 {
        let left_position = Position::from_inorder_index(i);
        let position = left_position.get_next_sibling();
        assert_eq!(left_position.get_level(), position.get_level());
        assert_eq!(
            pos_counting_from_left(left_position.to_inorder_index()) + 1,
            pos_counting_from_left(position.to_inorder_index())
        );
    }
}

#[test]
fn test_position_is_left_child() {
    assert!(!Position::from_inorder_index(5).is_left_child());
    assert!(!Position::from_inorder_index(6).is_left_child());
    assert!(!Position::from_inorder_index(2).is_left_child());
    assert!(!Position::from_inorder_index(11).is_left_child());
    assert!(!Position::from_inorder_index(13).is_left_child());
    assert!(!Position::from_inorder_index(14).is_left_child());
    assert!(!Position::from_inorder_index(10).is_left_child());

    assert!(Position::from_inorder_index(1).is_left_child());
    assert!(Position::from_inorder_index(0).is_left_child());
    assert!(Position::from_inorder_index(3).is_left_child());
    assert!(Position::from_inorder_index(8).is_left_child());
    assert!(Position::from_inorder_index(12).is_left_child());
}

#[test]
fn test_position_is_left_child_from_root() {
    assert!(Position::from_inorder_index(7).is_left_child());
}

#[test]
fn test_position_get_root_position() {
    let target = Position::get_root_position(6);
    assert_eq!(target, Position::from_inorder_index(7));

    let target = Position::get_root_position(0);
    assert_eq!(target, Position::from_inorder_index(0));

    let target = Position::get_root_position(3);
    assert_eq!(target, Position::from_inorder_index(3));
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
        let mut roots = slow_get_frozen_subtree_roots_impl(root.get_left_child(), max_leaf_index);
        roots.extend(slow_get_frozen_subtree_roots_impl(
            root.get_right_child(),
            max_leaf_index,
        ));
        roots
    }
}

fn slow_get_frozen_subtree_roots(num_leaves: u64) -> Vec<Position> {
    if num_leaves == 0 {
        Vec::new()
    } else {
        let max_leaf_index = num_leaves - 1;
        let root = Position::get_root_position(max_leaf_index);
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

fn collect_first_n_positions(num_leaves: u64, n: usize) -> Vec<u64> {
    FrozenSubtreeSiblingIterator::new(num_leaves)
        .take(n)
        .map(Position::to_inorder_index)
        .collect()
}

fn collect_all_positions(num_leaves: u64) -> Vec<u64> {
    FrozenSubtreeSiblingIterator::new(num_leaves)
        .map(Position::to_inorder_index)
        .collect()
}

#[test]
fn test_frozen_subtree_sibling_iterator() {
    assert_eq!(collect_first_n_positions(1, 5), vec![2, 5, 11, 23, 47]);
    assert_eq!(collect_first_n_positions(2, 5), vec![5, 11, 23, 47, 95]);
    assert_eq!(collect_first_n_positions(3, 5), vec![6, 11, 23, 47, 95]);
    assert_eq!(collect_first_n_positions(4, 4), vec![11, 23, 47, 95]);
    assert_eq!(collect_first_n_positions(5, 5), vec![10, 13, 23, 47, 95]);
    assert_eq!(collect_first_n_positions(6, 4), vec![13, 23, 47, 95]);
    assert_eq!(collect_first_n_positions(7, 4), vec![14, 23, 47, 95]);
    assert_eq!(collect_first_n_positions(8, 3), vec![23, 47, 95]);

    assert_eq!(collect_all_positions(1).len(), 63);
    assert_eq!(
        collect_all_positions(1 << 62),
        vec![(1 << 63) + (1 << 62) - 1]
    );
    assert_eq!(
        collect_all_positions((1 << 63) - 1),
        vec![std::u64::MAX - 1]
    );
    assert_eq!(collect_all_positions(1 << 63).len(), 0);
}
