// Copyright 2018 The Grin Developers

/// Types for a Cuck(at)oo proof of work and its encapsulation as a fully usable
/// proof of work within a block header.
/// Generic trait for a solver/verifier providing common interface into Cuckoo-family PoW
/// Mostly used for verification, but also for test mining if necessary
use crate::common::*;
use crate::error::{Error, ErrorKind};
use crate::siphash::siphash24;

use num::{PrimInt, ToPrimitive};
use rand::{thread_rng, Rng};
use serde::Serialize;
use std::hash::Hash;
use std::ops::{BitOrAssign, Mul};
use std::{fmt, iter};
pub const PROOF_SIZE: usize = 42;
pub const EDGE_BITS: u8 = 19;

pub trait PoWContext<T>
where
    T: EdgeType,
{
    /// Sets the header along with an optional nonce at the end
    /// solve: whether to set up structures for a solve (true) or just validate (false)
    fn set_header_nonce(
        &mut self,
        header: Vec<u8>,
        nonce: Option<u32>,
        solve: bool,
    ) -> Result<(), Error>;
    /// find solutions using the stored parameters and header
    fn find_cycles(&mut self) -> Result<Vec<Proof>, Error>;
    /// Verify a solution with the stored parameters
    fn verify(&self, proof: &Proof) -> Result<(), Error>;
}

/// A Cuck(at)oo Cycle proof of work, consisting of the edge_bits to get the graph
/// size (i.e. the 2-log of the number of edges) and the nonces
/// of the graph solution. While being expressed as u64 for simplicity,
/// nonces a.k.a. edge indices range from 0 to (1 << edge_bits) - 1
///
/// The hash of the `Proof` is the hash of its packed nonces when serializing
/// them at their exact bit size. The resulting bit sequence is padded to be
/// byte-aligned.
///
#[derive(Clone, PartialOrd, PartialEq, Serialize)]
pub struct Proof {
    /// Power of 2 used for the size of the cuckoo graph
    pub edge_bits: u8,
    /// The nonces
    pub nonces: Vec<u64>,
}

impl fmt::Debug for Proof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cuckoo{}(", self.edge_bits)?;
        for (i, val) in self.nonces[..].iter().enumerate() {
            write!(f, "{:x}", val)?;
            if i < self.nonces.len() - 1 {
                write!(f, " ")?;
            }
        }
        write!(f, ")")
    }
}

impl Eq for Proof {}

impl Proof {
    /// Builds a proof with provided nonces at default edge_bits
    pub fn new(mut in_nonces: Vec<u64>) -> Proof {
        in_nonces.sort_unstable();
        Proof {
            edge_bits: EDGE_BITS,
            nonces: in_nonces,
        }
    }

    /// Builds a proof with all bytes zeroed out
    pub fn zero(proof_size: usize) -> Proof {
        Proof {
            edge_bits: EDGE_BITS,
            nonces: vec![0; proof_size],
        }
    }

    /// Builds a proof with random POW data,
    /// needed so that tests that ignore POW
    /// don't fail due to duplicate hashes
    pub fn random(proof_size: usize) -> Proof {
        let edge_bits = EDGE_BITS;
        let nonce_mask = (1 << edge_bits) - 1;
        let mut rng = thread_rng();
        // force the random num to be within edge_bits bits
        let mut v: Vec<u64> = iter::repeat(())
            .map(|()| (rng.gen::<u32>() & nonce_mask) as u64)
            .take(proof_size)
            .collect();
        v.sort_unstable();
        Proof {
            edge_bits: EDGE_BITS,
            nonces: v,
        }
    }

    /// Returns the proof size
    pub fn proof_size(&self) -> usize {
        self.nonces.len()
    }
}

/// Operations needed for edge type (going to be u32 or u64)
pub trait EdgeType: PrimInt + ToPrimitive + Mul + BitOrAssign + Hash {}
impl EdgeType for u32 {}
impl EdgeType for u64 {}

/// An edge in the Cuckoo graph, simply references two u64 nodes.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Edge<T>
where
    T: EdgeType,
{
    pub u: T,
    pub v: T,
}

impl<T> fmt::Display for Edge<T>
where
    T: EdgeType,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(u: {}, v: {})",
            self.u.to_u64().unwrap_or(0),
            self.v.to_u64().unwrap_or(0)
        )
    }
}

/// An element of an adjencency list
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Link<T>
where
    T: EdgeType,
{
    pub next: T,
    pub to: T,
}

impl<T> fmt::Display for Link<T>
where
    T: EdgeType,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(next: {}, to: {})",
            self.next.to_u64().unwrap_or(0),
            self.to.to_u64().unwrap_or(0)
        )
    }
}

/// Utility struct to calculate commonly used Cuckoo parameters calculated
/// from header, nonce, edge_bits, etc.
pub struct CuckooParams<T>
where
    T: EdgeType,
{
    pub edge_bits: u8,
    pub proof_size: usize,
    pub num_edges: u64,
    pub siphash_keys: [u64; 4],
    pub edge_mask: T,
}

impl<T> CuckooParams<T>
where
    T: EdgeType,
{
    /// Instantiates new params and calculate edge mask, etc
    pub fn new(edge_bits: u8, proof_size: usize) -> Result<CuckooParams<T>, Error> {
        let num_edges = (1 as u64) << edge_bits;
        let edge_mask = to_edge!(num_edges - 1);
        Ok(CuckooParams {
            edge_bits,
            proof_size,
            num_edges,
            siphash_keys: [0; 4],
            edge_mask,
        })
    }

    /// Reset the main keys used for siphash from the header and nonce
    pub fn reset_header_nonce(&mut self, header: Vec<u8>, nonce: Option<u32>) -> Result<(), Error> {
        self.siphash_keys = set_header_nonce(&header, nonce)?;
        Ok(())
    }

    /// Return siphash masked for type
    #[allow(dead_code)]
    pub fn sipnode(&self, edge: T, uorv: u64, shift: bool) -> Result<T, Error> {
        let hash_u64 = siphash24(
            &self.siphash_keys,
            2 * edge.to_u64().ok_or(ErrorKind::IntegerCast)? + uorv,
        );
        let mut masked = hash_u64 & self.edge_mask.to_u64().ok_or(ErrorKind::IntegerCast)?;
        if shift {
            masked <<= 1;
            masked |= uorv;
        }
        Ok(T::from(masked).ok_or(ErrorKind::IntegerCast)?)
    }
}
