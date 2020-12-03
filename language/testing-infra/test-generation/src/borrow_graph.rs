// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::abstract_state::Mutability;
use std::collections::HashMap;

/// Each partition is associated with a (unique) ID
type PartitionID = u16;

/// A nonce represents a runtime reference. It has a unique identifier and a mutability
type Nonce = (u16, Mutability);

/// A set of nonces
type NonceSet = Vec<Nonce>;

/// A path representing field accesses of a struct
type Path = Vec<u8>;

/// An edge in the graph. It consists of two partition IDs, a `Path` which
/// may be empty, and an `EdgeType`
type Edge = (PartitionID, PartitionID, Path, EdgeType);

/// The `EdgeType` is either weak or strong. A weak edge represents imprecise information
/// on the path along which the borrow takes place. A strong edge is precise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EdgeType {
    Weak,
    Strong,
}

/// The `BorrowGraph` stores information sufficient to determine whether the instruction
/// of a bytecode instruction that interacts with references is memory safe.
#[derive(Debug, Clone, PartialEq)]
pub struct BorrowGraph {
    /// All of the partitions that make up the graph
    partitions: Vec<PartitionID>,

    /// A mapping from partitions to the associated with them
    partition_map: HashMap<PartitionID, NonceSet>,

    /// The edges of the graph
    edges: Vec<Edge>,

    /// A counter for determining what the next partition ID should be
    partition_counter: u16,
}

impl BorrowGraph {
    /// Construct a new `BorrowGraph` given the number of locals it has
    pub fn new(num_locals: u8) -> BorrowGraph {
        BorrowGraph {
            partitions: Vec::with_capacity(num_locals as usize),
            partition_map: HashMap::new(),
            edges: Vec::new(),
            partition_counter: u16::from(num_locals),
        }
    }

    /// Add a new partition to the graph containing nonce `n`
    /// This operation may fail with an error a fresh partition ID
    /// cannot be chosen.
    pub fn fresh_partition(&mut self, n: Nonce) -> Result<(), String> {
        if self.partition_counter.checked_add(1).is_some() {
            if self.partition_map.get(&self.partition_counter).is_some() {
                return Err("Partition map already contains ID".to_string());
            }
            self.partition_map.insert(self.partition_counter, vec![n]);
            // Implication of `checked_add`
            assume!(self.partitions.len() < usize::max_value());
            self.partitions.push(self.partition_counter);
            Ok(())
        } else {
            Err("Partition map is full".to_string())
        }
    }

    /// Determine whether a partition is mutable, immutable, or either.
    /// This operation may fail with an error if the given partition does
    /// not exist in the graph.
    pub fn partition_mutability(&self, partition_id: PartitionID) -> Result<Mutability, String> {
        if let Some(nonce_set) = self.partition_map.get(&partition_id) {
            if nonce_set
                .iter()
                .all(|(_, mutability)| *mutability == Mutability::Mutable)
            {
                Ok(Mutability::Mutable)
            } else if nonce_set
                .iter()
                .all(|(_, mutability)| *mutability == Mutability::Immutable)
            {
                Ok(Mutability::Immutable)
            } else {
                Ok(Mutability::Either)
            }
        } else {
            Err("Partition map does not contain given partition ID".to_string())
        }
    }

    /// Determine whether the given partition is freezable. This operation may fail
    /// with an error if the given partition ID is not in the graph.
    pub fn partition_freezable(&self, partition_id: PartitionID) -> Result<bool, String> {
        let mut freezable = true;
        if self.partition_map.get(&partition_id).is_some() {
            for (p1, p2, _, _) in self.edges.iter() {
                if *p1 == partition_id && self.partition_mutability(*p2)? == Mutability::Mutable {
                    freezable = false;
                }
            }
            Ok(freezable)
        } else {
            Err("Partition map does not contain given partition ID".to_string())
        }
    }

    /// Determine whether the `path_1` is a prefix of `path_2`
    fn path_is_prefix(&self, path_1: Path, path_2: Path) -> bool {
        let mut prefix = true;
        for (i, field) in path_1.iter().enumerate() {
            if *field != path_2[i] {
                prefix = false;
            }
        }
        prefix
    }

    /// Determine whether two edges are consistent; i.e. whether the path of the
    /// first edge is a prefix of the second or vice versa.
    pub fn edges_consistent(&self, edge_1: Edge, edge_2: Edge) -> bool {
        let path_1 = edge_1.2;
        let path_2 = edge_2.2;
        self.path_is_prefix(path_1.clone(), path_2.clone()) || self.path_is_prefix(path_2, path_1)
    }
}
