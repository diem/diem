use crate::{
    interface::{Position, Store},
    metrics::{HISTOGRAM, TIMER},
    types::DummyValue,
};
use anyhow::{anyhow, Result};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diem_crypto::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, HashValue};
use diem_types::{
    proof::SparseMerkleProof,
    transaction::{StateTreeNode, StateTreeNodeType, Version},
    vec_bool::VecBool,
};
use rayon::prelude::*;
use rocksdb::{Options, WriteOptions};
use std::{mem::size_of, path::Path};

pub struct RocksDbStore {
    db: rocksdb::DB,
}

impl RocksDbStore {
    #[allow(dead_code)]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db = rocksdb::DB::open(&opts, path)?;

        Ok(Self { db })
    }

    fn encode_key(position: &Position, version: Version) -> Vec<u8> {
        let mut data =
            Vec::with_capacity(position.bytes().len() + size_of::<u16>() + size_of::<Version>());

        Self::encode_position_impl(position, &mut data);
        data.write_u64::<BigEndian>(!version).unwrap();

        data
    }

    fn encode_value(node: &StateTreeNode) -> Vec<u8> {
        bcs::to_bytes(node).unwrap()
    }

    fn encode_position_impl(position: &Position, data: &mut Vec<u8>) {
        data.write_u16::<LittleEndian>(position.len() as u16)
            .unwrap();
        data.extend_from_slice(position.bytes());
    }

    fn decode_position(data: &[u8]) -> Position {
        let len = (&data[0..2]).read_u16::<LittleEndian>().unwrap() as usize;
        let nbytes = len / 8 + if len % 8 == 0 { 0 } else { 1 };
        let bytes = (&data[2..2 + nbytes]).to_vec();

        Position::new_hack(len, bytes)
    }

    fn get_node_before_opt(
        &self,
        position: &Position,
        version: Version,
    ) -> Result<Option<StateTreeNode>> {
        let mut iter = self.db.raw_iterator();
        iter.seek(&Self::encode_key(position, version));

        if !iter.valid() {
            iter.status()?;
            return Ok(None);
        }

        let pos = Self::decode_position(iter.key().unwrap());
        if pos == *position {
            Ok(Some(bcs::from_bytes(iter.value().unwrap())?))
        } else {
            Ok(None)
        }
    }

    fn get_node_before(&self, position: &Position, version: Version) -> Result<StateTreeNode> {
        self.get_node_before_opt(position, version)?.ok_or_else(|| {
            anyhow!(
                "node at {:?} not found before version {}",
                position,
                version
            )
        })
    }
}

impl Store for RocksDbStore {
    fn get_proof(
        &self,
        target_key: HashValue,
        version: Version,
        starting_depth: usize,
    ) -> Result<SparseMerkleProof<DummyValue>> {
        let _timer = TIMER.with_label_values(&["get_proof"]).start_timer();
        HISTOGRAM
            .with_label_values(&["proof_starting_depth"])
            .observe(starting_depth as f64);

        let mut pos = VecBool::new();
        let mut bit_iter = target_key.iter_bits();
        for _ in 0..starting_depth {
            let bit = bit_iter.next().unwrap();
            pos.push(bit)
        }

        let start = self.get_node_before(&pos, version)?;
        let (mut left_empty, mut right_empty) = match start.node_type {
            StateTreeNodeType::Leaf { leaf } => {
                return Ok(SparseMerkleProof::new_partial(
                    Some(leaf),
                    Vec::new(),
                    starting_depth,
                ));
            }
            StateTreeNodeType::Internal {
                left_empty,
                right_empty,
            } => (left_empty, right_empty),
        };

        let mut depth = starting_depth;
        let mut siblings = Vec::new();
        for bit in bit_iter {
            depth += 1;
            let (me_empty, sibling_empty) = swap_if(left_empty, right_empty, bit);
            siblings.push(if sibling_empty {
                *SPARSE_MERKLE_PLACEHOLDER_HASH
            } else {
                pos.push(!bit);
                let sibling = self.get_node_before(&pos, version)?;
                pos.pop();
                sibling.hash
            });

            if me_empty {
                HISTOGRAM
                    .with_label_values(&["proof_bottom_depth"])
                    .observe(depth as f64);
                return Ok(SparseMerkleProof::new_partial(
                    None,
                    {
                        siblings.reverse();
                        siblings
                    },
                    starting_depth,
                ));
            }

            pos.push(bit);
            let node = self.get_node_before(&pos, version)?;
            match node.node_type {
                StateTreeNodeType::Leaf { leaf } => {
                    HISTOGRAM
                        .with_label_values(&["proof_bottom_depth"])
                        .observe(depth as f64);
                    return Ok(SparseMerkleProof::new_partial(
                        Some(leaf),
                        {
                            siblings.reverse();
                            siblings
                        },
                        starting_depth,
                    ));
                }
                StateTreeNodeType::Internal {
                    left_empty: l,
                    right_empty: r,
                } => {
                    left_empty = l;
                    right_empty = r;
                }
            }
        }

        unreachable!("Should've seen a leaf and returned already.")
    }

    fn put_nodes(&self, nodes: &[(Position, StateTreeNode)], version: Version) -> Result<()> {
        let _timer = TIMER
            .with_label_values(&["rocksdb_put_nodes"])
            .start_timer();

        let encoded_nodes: Vec<_> = {
            let _timer = TIMER
                .with_label_values(&["encode_state_tree_nodes"])
                .start_timer();
            nodes
                .par_iter()
                .with_min_len(100)
                .map(|(pos, node)| (Self::encode_key(pos, version), Self::encode_value(node)))
                .collect()
        };

        let mut batch = rocksdb::WriteBatch::default();
        for (k, v) in encoded_nodes {
            batch.put(&k, &v);
        }

        {
            let _timer = TIMER
                .with_label_values(&["rocksdb_write_opt"])
                .start_timer();
            self.db.write_opt(batch, &WriteOptions::default())?;
        }
        Ok(())
    }
}

fn swap_if<T>(first: T, second: T, cond: bool) -> (T, T) {
    if cond {
        (second, first)
    } else {
        (first, second)
    }
}
