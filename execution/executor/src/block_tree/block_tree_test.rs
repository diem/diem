// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]

use super::{AddBlockError, Block, CommitBlockError};
use crypto::HashValue;
use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq)]
struct TestBlock {
    committed: bool,
    id: HashValue,
    parent_id: HashValue,
    children: HashSet<HashValue>,
    output: Option<()>,
    signature: Option<()>,
}

impl Block for TestBlock {
    type Output = ();
    type Signature = ();

    fn is_committed(&self) -> bool {
        self.committed
    }

    fn set_committed(&mut self) {
        assert!(!self.committed);
        self.committed = true;
    }

    fn is_executed(&self) -> bool {
        self.output.is_some()
    }

    fn set_output(&mut self, output: Self::Output) {
        assert!(self.output.is_none());
        self.output = Some(output);
    }

    fn set_signature(&mut self, signature: Self::Signature) {
        assert!(self.signature.is_none());
        self.signature = Some(signature);
    }

    fn id(&self) -> HashValue {
        self.id
    }

    fn parent_id(&self) -> HashValue {
        self.parent_id
    }

    fn add_child(&mut self, child_id: HashValue) {
        assert!(self.children.insert(child_id));
    }

    fn children(&self) -> &HashSet<HashValue> {
        &self.children
    }
}

impl TestBlock {
    fn new(parent_id: HashValue, id: HashValue) -> Self {
        TestBlock {
            committed: false,
            id,
            parent_id,
            children: HashSet::new(),
            output: None,
            signature: None,
        }
    }
}

type BlockTree = super::BlockTree<TestBlock>;

fn id(i: u8) -> HashValue {
    HashValue::new([i; HashValue::LENGTH])
}

fn create_block_tree() -> BlockTree {
    //    0 ---> 1 ---> 2
    //    |      |
    //    |      └----> 3 ---> 4
    //    |             |
    //    |             └----> 5
    //    |
    //    └----> 6 ---> 7 ---> 8
    //           |
    //           └----> 9 ---> 10
    //                  |
    //                  └----> 11
    let mut block_tree = BlockTree::new(id(0));

    block_tree.add_block(TestBlock::new(id(0), id(1))).unwrap();
    block_tree.add_block(TestBlock::new(id(1), id(2))).unwrap();
    block_tree.add_block(TestBlock::new(id(1), id(3))).unwrap();
    block_tree.add_block(TestBlock::new(id(3), id(4))).unwrap();
    block_tree.add_block(TestBlock::new(id(3), id(5))).unwrap();
    block_tree.add_block(TestBlock::new(id(0), id(6))).unwrap();
    block_tree.add_block(TestBlock::new(id(6), id(7))).unwrap();
    block_tree.add_block(TestBlock::new(id(7), id(8))).unwrap();
    block_tree.add_block(TestBlock::new(id(6), id(9))).unwrap();
    block_tree.add_block(TestBlock::new(id(9), id(10))).unwrap();
    block_tree.add_block(TestBlock::new(id(9), id(11))).unwrap();

    block_tree
}

#[test]
fn test_add_duplicate_block() {
    let mut block_tree = create_block_tree();
    let block = TestBlock::new(id(1), id(7));
    let res = block_tree.add_block(block.clone());
    assert_eq!(
        res.err().unwrap(),
        AddBlockError::BlockAlreadyExists { block }
    );
}

#[test]
fn test_add_block_missing_parent() {
    let mut block_tree = create_block_tree();
    let block = TestBlock::new(id(99), id(200));
    let res = block_tree.add_block(block.clone());
    assert_eq!(res.err().unwrap(), AddBlockError::ParentNotFound { block });
}

fn assert_parent_and_children(block: &TestBlock, expected_parent: u8, expected_children: Vec<u8>) {
    assert_eq!(block.parent_id, id(expected_parent));
    assert_eq!(
        block.children,
        expected_children
            .into_iter()
            .map(id)
            .collect::<HashSet<_>>(),
    );
}

fn assert_heads(block_tree: &BlockTree, expected_heads: Vec<u8>) {
    assert_eq!(block_tree.heads.len(), expected_heads.len());
    assert_eq!(
        block_tree.heads,
        expected_heads.into_iter().map(id).collect::<HashSet<_>>(),
    );
}

#[test]
fn test_add_block() {
    let block_tree = create_block_tree();

    assert_heads(&block_tree, vec![1, 6]);
    assert_eq!(block_tree.last_committed_id, id(0));

    for i in 1..=11 {
        let block = block_tree.get_block(id(i)).unwrap();
        assert_eq!(block.id(), id(i));
        assert!(!block.is_committed());
        assert!(!block.is_executed());

        match i {
            1 => assert_parent_and_children(block, 0, vec![2, 3]),
            2 => assert_parent_and_children(block, 1, vec![]),
            3 => assert_parent_and_children(block, 1, vec![4, 5]),
            4 => assert_parent_and_children(block, 3, vec![]),
            5 => assert_parent_and_children(block, 3, vec![]),
            6 => assert_parent_and_children(block, 0, vec![7, 9]),
            7 => assert_parent_and_children(block, 6, vec![8]),
            8 => assert_parent_and_children(block, 7, vec![]),
            9 => assert_parent_and_children(block, 6, vec![10, 11]),
            10 => assert_parent_and_children(block, 9, vec![]),
            11 => assert_parent_and_children(block, 9, vec![]),
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_mark_as_committed_missing_block() {
    let mut block_tree = create_block_tree();
    let res = block_tree.mark_as_committed(id(99), ());
    assert_eq!(
        res.err().unwrap(),
        CommitBlockError::BlockNotFound { id: id(99) }
    );
}

#[test]
fn test_mark_as_committed_twice() {
    let mut block_tree = create_block_tree();
    block_tree.mark_as_committed(id(9), ()).unwrap();
    let res = block_tree.mark_as_committed(id(9), ());
    assert_eq!(
        res.err().unwrap(),
        CommitBlockError::BlockAlreadyMarkedAsCommitted { id: id(9) }
    );
}

#[test]
fn test_mark_as_committed_1_2() {
    let mut block_tree = create_block_tree();

    block_tree.mark_as_committed(id(1), ()).unwrap();
    for i in 1..=11 {
        let block = block_tree.get_block(id(i)).unwrap();
        match i {
            1 => assert!(block.is_committed()),
            _ => assert!(!block.is_committed()),
        }
    }

    block_tree.mark_as_committed(id(2), ()).unwrap();
    for i in 1..=11 {
        let block = block_tree.get_block(id(i)).unwrap();
        match i {
            1 | 2 => assert!(block.is_committed()),
            _ => assert!(!block.is_committed()),
        }
    }
}

#[test]
fn test_mark_as_committed_2() {
    let mut block_tree = create_block_tree();
    block_tree.mark_as_committed(id(2), ()).unwrap();
    for i in 1..=11 {
        let block = block_tree.get_block(id(i)).unwrap();
        match i {
            1 | 2 => assert!(block.is_committed()),
            _ => assert!(!block.is_committed()),
        }
    }
}

#[test]
fn test_mark_as_committed_4() {
    let mut block_tree = create_block_tree();
    block_tree.mark_as_committed(id(4), ()).unwrap();
    for i in 1..=11 {
        let block = block_tree.get_block(id(i)).unwrap();
        match i {
            1 | 3 | 4 => assert!(block.is_committed()),
            _ => assert!(!block.is_committed()),
        }
    }
}

#[test]
fn test_mark_as_committed_3_5() {
    let mut block_tree = create_block_tree();

    block_tree.mark_as_committed(id(3), ()).unwrap();
    for i in 1..=11 {
        let block = block_tree.get_block(id(i)).unwrap();
        match i {
            1 | 3 => assert!(block.is_committed()),
            _ => assert!(!block.is_committed()),
        }
    }

    block_tree.mark_as_committed(id(5), ()).unwrap();
    for i in 1..=11 {
        let block = block_tree.get_block(id(i)).unwrap();
        match i {
            1 | 3 | 5 => assert!(block.is_committed()),
            _ => assert!(!block.is_committed()),
        }
    }
}

#[test]
fn test_mark_as_committed_8() {
    let mut block_tree = create_block_tree();
    block_tree.mark_as_committed(id(8), ()).unwrap();
    for i in 1..=11 {
        let block = block_tree.get_block(id(i)).unwrap();
        match i {
            6 | 7 | 8 => assert!(block.is_committed()),
            _ => assert!(!block.is_committed()),
        }
    }
}

#[test]
fn test_mark_as_committed_11() {
    let mut block_tree = create_block_tree();
    block_tree.mark_as_committed(id(11), ()).unwrap();
    for i in 1..=11 {
        let block = block_tree.get_block(id(i)).unwrap();
        match i {
            6 | 9 | 11 => assert!(block.is_committed()),
            _ => assert!(!block.is_committed()),
        }
    }
}

#[test]
fn test_get_committed_head_one_committed() {
    let mut block_tree = create_block_tree();
    block_tree.mark_as_committed(id(7), ()).unwrap();
    assert_eq!(
        block_tree.get_committed_head(&block_tree.heads),
        Some(id(6)),
    );
}

#[test]
fn test_get_committed_head_all_pending() {
    let block_tree = create_block_tree();
    assert_eq!(block_tree.get_committed_head(&block_tree.heads), None);
}

#[test]
#[should_panic(expected = "Conflicting blocks are both committed.")]
fn test_get_committed_head_two_committed() {
    let mut block_tree = create_block_tree();
    block_tree.mark_as_committed(id(2), ()).unwrap();
    block_tree.mark_as_committed(id(7), ()).unwrap();
    let _committed_head = block_tree.get_committed_head(&block_tree.heads);
}

#[test]
#[should_panic(expected = "Trying to remove a committed block")]
fn test_remove_branch_committed_block() {
    let mut block_tree = create_block_tree();
    block_tree.mark_as_committed(id(2), ()).unwrap();
    block_tree.remove_branch(id(1));
}

#[test]
fn test_remove_branch_1_7_11() {
    let mut block_tree = create_block_tree();

    block_tree.remove_branch(id(1));
    for i in 1..=11 {
        let block = block_tree.get_block(id(i));
        match i {
            1 | 2 | 3 | 4 | 5 => assert!(block.is_none()),
            _ => assert!(block.is_some()),
        }
    }

    block_tree.remove_branch(id(7));
    for i in 1..=11 {
        let block = block_tree.get_block(id(i));
        match i {
            1 | 2 | 3 | 4 | 5 | 7 | 8 => assert!(block.is_none()),
            _ => assert!(block.is_some()),
        }
    }

    block_tree.remove_branch(id(11));
    for i in 1..=11 {
        let block = block_tree.get_block(id(i));
        match i {
            1 | 2 | 3 | 4 | 5 | 7 | 8 | 11 => assert!(block.is_none()),
            _ => assert!(block.is_some()),
        }
    }
}

fn set_executed(block_tree: &mut BlockTree, ids: &[HashValue]) {
    for id in ids {
        let block = block_tree.get_block_mut(*id).unwrap();
        block.set_output(());
    }
}

fn assert_block_id_in_set(block_id: HashValue, candidates: Vec<u8>) {
    let ids: Vec<_> = candidates.into_iter().map(id).collect();
    assert!(ids.contains(&block_id));
}

#[test]
fn test_get_block_to_execute() {
    let mut block_tree = create_block_tree();
    let to_execute = block_tree.get_block_to_execute().unwrap();
    assert_block_id_in_set(to_execute, vec![1, 6]);

    set_executed(&mut block_tree, &[id(6)]);
    let to_execute = block_tree.get_block_to_execute().unwrap();
    assert_block_id_in_set(to_execute, vec![1, 7, 9]);

    set_executed(
        &mut block_tree,
        &[
            id(1),
            id(2),
            id(3),
            id(4),
            id(5),
            id(7),
            id(8),
            id(9),
            id(10),
        ],
    );
    let to_execute = block_tree.get_block_to_execute().unwrap();
    assert_block_id_in_set(to_execute, vec![11]);

    set_executed(&mut block_tree, &[id(11)]);
    assert!(block_tree.get_block_to_execute().is_none());
}

fn assert_to_store(to_store: &[TestBlock], ids: &[HashValue]) {
    let committed_ids: Vec<_> = to_store.iter().map(Block::id).collect();
    assert_eq!(committed_ids, ids);
}

#[test]
fn test_prune_1_executed() {
    let mut block_tree = create_block_tree();
    set_executed(&mut block_tree, &[id(1)]);
    block_tree.mark_as_committed(id(1), ()).unwrap();
    let to_store = block_tree.prune();

    assert_to_store(&to_store, &[id(1)]);
    assert_heads(&block_tree, vec![2, 3]);
    assert_eq!(block_tree.last_committed_id, id(1));
    for i in 1..=11 {
        let block = block_tree.get_block(id(i));
        match i {
            2 | 3 | 4 | 5 => assert!(block.is_some()),
            _ => assert!(block.is_none()),
        }
    }
}

#[test]
fn test_prune_1_not_executed() {
    let mut block_tree = create_block_tree();
    block_tree.mark_as_committed(id(1), ()).unwrap();
    let to_store = block_tree.prune();

    assert!(to_store.is_empty());
    assert_heads(&block_tree, vec![1]);
    assert_eq!(block_tree.last_committed_id, id(0));
    for i in 1..=11 {
        let block = block_tree.get_block(id(i));
        match i {
            1 | 2 | 3 | 4 | 5 => assert!(block.is_some()),
            _ => assert!(block.is_none()),
        }
    }
}

#[test]
fn test_prune_2_executed() {
    let mut block_tree = create_block_tree();
    set_executed(&mut block_tree, &[id(1), id(2)]);
    block_tree.mark_as_committed(id(2), ()).unwrap();
    let to_store = block_tree.prune();

    assert_to_store(&to_store, &[id(1), id(2)]);
    assert_eq!(block_tree.last_committed_id, id(2));
    assert!(block_tree.heads.is_empty());
    assert!(block_tree.id_to_block.is_empty());
}

#[test]
fn test_prune_2_not_all_executed() {
    let mut block_tree = create_block_tree();
    set_executed(&mut block_tree, &[id(1)]);
    block_tree.mark_as_committed(id(2), ()).unwrap();
    let to_store = block_tree.prune();

    assert_to_store(&to_store, &[id(1)]);
    assert_heads(&block_tree, vec![2]);
    assert_eq!(block_tree.last_committed_id, id(1));
    for i in 1..=11 {
        let block = block_tree.get_block(id(i));
        match i {
            2 => assert!(block.is_some()),
            _ => assert!(block.is_none()),
        }
    }

    set_executed(&mut block_tree, &[id(2)]);
    let to_store = block_tree.prune();
    assert_to_store(&to_store, &[id(2)]);
    assert!(block_tree.heads.is_empty());
    assert!(block_tree.id_to_block.is_empty());
}

#[test]
fn test_prune_7_executed() {
    let mut block_tree = create_block_tree();
    set_executed(&mut block_tree, &[id(6), id(7)]);
    block_tree.mark_as_committed(id(7), ()).unwrap();
    let to_store = block_tree.prune();

    assert_to_store(&to_store, &[id(6), id(7)]);
    assert_heads(&block_tree, vec![8]);
    assert_eq!(block_tree.last_committed_id, id(7));
    for i in 1..=11 {
        let block = block_tree.get_block(id(i));
        match i {
            8 => assert!(block.is_some()),
            _ => assert!(block.is_none()),
        }
    }
}

#[test]
fn test_prune_9_not_executed() {
    let mut block_tree = create_block_tree();
    block_tree.mark_as_committed(id(9), ()).unwrap();
    let to_store = block_tree.prune();

    assert_to_store(&to_store, &[]);
    assert_heads(&block_tree, vec![6]);
    assert_eq!(block_tree.last_committed_id, id(0));
    for i in 1..=11 {
        let block = block_tree.get_block(id(i));
        match i {
            6 | 9 | 10 | 11 => assert!(block.is_some()),
            _ => assert!(block.is_none()),
        }
    }
}

#[test]
fn test_prune_10_not_executed() {
    let mut block_tree = create_block_tree();
    block_tree.mark_as_committed(id(10), ()).unwrap();
    let to_store = block_tree.prune();

    assert_to_store(&to_store, &[]);
    assert_heads(&block_tree, vec![6]);
    assert_eq!(block_tree.last_committed_id, id(0));
    for i in 1..=11 {
        let block = block_tree.get_block(id(i));
        match i {
            6 | 9 | 10 => assert!(block.is_some()),
            _ => assert!(block.is_none()),
        }
    }
}

#[test]
fn test_remove_subtree_1() {
    let mut block_tree = create_block_tree();
    block_tree.remove_subtree(id(1));

    assert_heads(&block_tree, vec![6]);
    assert_eq!(block_tree.last_committed_id, id(0));
    for i in 1..=11 {
        let block = block_tree.get_block(id(i));
        match i {
            1 | 2 | 3 | 4 | 5 => assert!(block.is_none()),
            _ => assert!(block.is_some()),
        }
    }
}

#[test]
fn test_remove_subtree_3() {
    let mut block_tree = create_block_tree();
    block_tree.remove_subtree(id(3));

    assert_heads(&block_tree, vec![1, 6]);
    assert_eq!(block_tree.last_committed_id, id(0));
    for i in 1..=11 {
        let block = block_tree.get_block(id(i));
        match i {
            3 | 4 | 5 => assert!(block.is_none()),
            _ => assert!(block.is_some()),
        }
    }
}

#[test]
fn test_reset() {
    let mut block_tree = create_block_tree();
    block_tree.reset(id(100));
    assert!(block_tree.id_to_block.is_empty());
    assert!(block_tree.heads.is_empty());
    assert_eq!(block_tree.last_committed_id, id(100));
}
