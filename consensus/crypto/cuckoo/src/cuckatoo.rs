//! Implementation of Cuckatoo Cycle designed by John Tromp.
use std::mem;

use crate::error::{Error, ErrorKind};
use crate::types::{CuckooParams, EdgeType, Link, PoWContext, Proof};
use crate::PROOF_SIZE;
use byteorder::{BigEndian, WriteBytesExt};
use croaring::Bitmap;
use std::fmt::Write;

struct Graph<T>
where
    T: EdgeType,
{
    /// Maximum number of edges
    max_edges: T,
    /// Maximum nodes
    max_nodes: u64,
    /// Adjacency links
    links: Vec<Link<T>>,
    /// Index into links array
    adj_list: Vec<T>,
    ///
    visited: Bitmap,
    /// Maximum solutions
    max_sols: u32,
    ///
    pub solutions: Vec<Proof>,
    /// proof size
    proof_size: usize,
    /// define NIL type
    nil: T,
}

impl<T> Graph<T>
where
    T: EdgeType,
{
    /// Create a new graph with given parameters
    pub fn new(max_edges: T, max_sols: u32, proof_size: usize) -> Result<Graph<T>, Error> {
        let max_nodes = 2 * to_u64!(max_edges);
        Ok(Graph {
            max_edges,
            max_nodes,
            max_sols,
            proof_size,
            links: vec![],
            adj_list: vec![],
            visited: Bitmap::create(),
            solutions: vec![],
            nil: T::max_value(),
        })
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        //TODO: Can be optimised
        println!("reset");
        self.links = Vec::with_capacity(2 * self.max_nodes as usize);
        self.adj_list = vec![T::max_value(); 2 * self.max_nodes as usize];
        self.solutions = vec![Proof::zero(self.proof_size); 1];
        self.visited = Bitmap::create();
        Ok(())
    }

    pub fn byte_count(&self) -> Result<u64, Error> {
        Ok(
            2 * to_u64!(self.max_edges) * mem::size_of::<Link<T>>() as u64
                + mem::size_of::<T>() as u64 * 2 * self.max_nodes,
        )
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, u: T, mut v: T) -> Result<(), Error> {
        let max_nodes_t = to_edge!(self.max_nodes);
        if u >= max_nodes_t || v >= max_nodes_t {
            return Err(ErrorKind::EdgeAddition)?;
        }
        v = v + to_edge!(self.max_nodes);
        let adj_u = self.adj_list[to_usize!(u ^ T::one())];
        let adj_v = self.adj_list[to_usize!(v ^ T::one())];
        if adj_u != self.nil && adj_v != self.nil {
            let sol_index = self.solutions.len() - 1;
            self.solutions[sol_index].nonces[0] = self.links.len() as u64 / 2;
            self.cycles_with_link(1, u, v)?;
        }
        let ulink = self.links.len();
        let vlink = self.links.len() + 1;
        if to_edge!(vlink) == self.nil {
            return Err(ErrorKind::EdgeAddition)?;
        }
        self.links.push(Link {
            next: self.adj_list[to_usize!(u)],
            to: u,
        });
        self.links.push(Link {
            next: self.adj_list[to_usize!(v)],
            to: v,
        });
        self.adj_list[to_usize!(u)] = T::from(ulink).ok_or(ErrorKind::IntegerCast)?;
        self.adj_list[to_usize!(v)] = T::from(vlink).ok_or(ErrorKind::IntegerCast)?;
        Ok(())
    }

    fn test_bit(&mut self, u: u64) -> bool {
        self.visited.contains(u as u32)
    }

    fn cycles_with_link(&mut self, len: u32, u: T, dest: T) -> Result<(), Error> {
        if self.test_bit(to_u64!(u >> 1)) {
            return Ok(());
        }
        if (u ^ T::one()) == dest {
            if len == self.proof_size as u32 {
                if self.solutions.len() < self.max_sols as usize {
                    // create next solution
                    self.solutions.push(Proof::zero(self.proof_size));
                }
                return Ok(());
            }
        } else if len == self.proof_size as u32 {
            return Ok(());
        }
        let mut au1 = self.adj_list[to_usize!(u ^ T::one())];
        if au1 != self.nil {
            self.visited.add(to_u32!(u >> 1));
            while au1 != self.nil {
                let i = self.solutions.len() - 1;
                self.solutions[i].nonces[len as usize] = to_u64!(au1) / 2;
                let link_index = to_usize!(au1 ^ T::one());
                let link = self.links[link_index].to;
                if link != self.nil {
                    self.cycles_with_link(len + 1, link, dest)?;
                }
                au1 = self.links[to_usize!(au1)].next;
            }
            self.visited.remove(to_u32!(u >> 1));
        }
        Ok(())
    }
}

/// Instantiate a new CuckatooContext as a PowContext.
pub fn new_cuckatoo_ctx<T>(
    edge_bits: u8,
    proof_size: usize,
    max_sols: u32,
) -> Result<Box<dyn PoWContext<T>>, Error>
where
    T: EdgeType + 'static,
{
    Ok(Box::new(CuckatooContext::<T>::new_impl(
        edge_bits, proof_size, max_sols,
    )?))
}

/// Cuckatoo solver context
pub struct CuckatooContext<T>
where
    T: EdgeType,
{
    params: CuckooParams<T>,
    graph: Graph<T>,
}

impl<T> PoWContext<T> for CuckatooContext<T>
where
    T: EdgeType,
{
    fn set_header_nonce(
        &mut self,
        header: Vec<u8>,
        nonce: Option<u32>,
        solve: bool,
    ) -> Result<(), Error> {
        self.set_header_nonce_impl(header, nonce, solve)
    }

    fn find_cycles(&mut self) -> Result<Vec<Proof>, Error> {
        let num_edges = self.params.num_edges;
        self.find_cycles_iter(0..num_edges)
    }

    fn verify(&self, proof: &Proof) -> Result<(), Error> {
        self.verify_impl(proof)
    }
}

impl<T> CuckatooContext<T>
where
    T: EdgeType,
{
    /// New Solver context
    pub fn new_impl(
        edge_bits: u8,
        proof_size: usize,
        max_sols: u32,
    ) -> Result<CuckatooContext<T>, Error> {
        let params = CuckooParams::new(edge_bits, proof_size)?;
        let num_edges = to_edge!(params.num_edges);
        Ok(CuckatooContext {
            params,
            graph: Graph::new(num_edges, max_sols, proof_size)?,
        })
    }

    /// Get a siphash key as a hex string (for display convenience)
    pub fn sipkey_hex(&self, index: usize) -> Result<String, Error> {
        let mut rdr = vec![];
        rdr.write_u64::<BigEndian>(self.params.siphash_keys[index])?;
        let mut s = String::new();
        for byte in rdr {
            write!(&mut s, "{:02x}", byte).expect("Unable to write");
        }
        Ok(s)
    }

    /// Return number of bytes used by the graph
    pub fn byte_count(&self) -> Result<u64, Error> {
        self.graph.byte_count()
    }

    /// Set the header and optional nonce in the last part of the header
    pub fn set_header_nonce_impl(
        &mut self,
        header: Vec<u8>,
        nonce: Option<u32>,
        solve: bool,
    ) -> Result<(), Error> {
        self.params.reset_header_nonce(header, nonce)?;
        if solve {
            self.graph.reset()?;
        }
        Ok(())
    }

    /// Return siphash masked for type
    pub fn sipnode(&self, edge: T, uorv: u64) -> Result<T, Error> {
        self.params.sipnode(edge, uorv, false)
    }

    /// Simple implementation of algorithm

    pub fn find_cycles_iter<I>(&mut self, iter: I) -> Result<Vec<Proof>, Error>
    where
        I: Iterator<Item = u64>,
    {
        let mut val = vec![];
        for n in iter {
            val.push(n);
            let u = self.sipnode(to_edge!(n), 0)?;
            let v = self.sipnode(to_edge!(n), 1)?;
            self.graph.add_edge(to_edge!(u), to_edge!(v))?;
        }
        self.graph.solutions.pop();
        for s in &mut self.graph.solutions {
            s.nonces = s.nonces.iter().map(|n| val[*n as usize]).collect();
            s.nonces.sort_unstable();
        }
        for s in &self.graph.solutions {
            self.verify_impl(&s)?;
        }
        if self.graph.solutions.is_empty() {
            Err(ErrorKind::NoSolution)?
        } else {
            Ok(self.graph.solutions.clone())
        }
    }

    /// Verify that given edges are ascending and form a cycle in a header-generated
    /// graph
    pub fn verify_impl(&self, proof: &Proof) -> Result<(), Error> {
        if proof.proof_size() != PROOF_SIZE {
            return Err(ErrorKind::Verification("wrong cycle length".to_owned()))?;
        }
        let nonces = &proof.nonces;
        let mut uvs = vec![0u64; 2 * proof.proof_size()];
        let mut xor0: u64 = (self.params.proof_size as u64 / 2) & 1;
        let mut xor1: u64 = xor0;
        for n in 0..proof.proof_size() {
            if nonces[n] > to_u64!(self.params.edge_mask) {
                return Err(ErrorKind::Verification("edge too big".to_owned()))?;
            }
            if n > 0 && nonces[n] <= nonces[n - 1] {
                return Err(ErrorKind::Verification("edges not ascending".to_owned()))?;
            }
            uvs[2 * n] = to_u64!(self.sipnode(to_edge!(nonces[n]), 0)?);
            uvs[2 * n + 1] = to_u64!(self.sipnode(to_edge!(nonces[n]), 1)?);
            xor0 ^= uvs[2 * n];
            xor1 ^= uvs[2 * n + 1];
        }
        if xor0 | xor1 != 0 {
            return Err(ErrorKind::Verification(
                "endpoints don't match up".to_owned(),
            ))?;
        }
        let mut n = 0;
        let mut i = 0;
        let mut j;
        loop {
            // follow cycle
            j = i;
            let mut k = j;
            loop {
                k = (k + 2) % (2 * self.params.proof_size);
                if k == i {
                    break;
                }
                if uvs[k] >> 1 == uvs[i] >> 1 {
                    // find other edge endpoint matching one at i
                    if j != i {
                        return Err(ErrorKind::Verification("branch in cycle".to_owned()))?;
                    }
                    j = k;
                }
            }
            if j == i || uvs[j] == uvs[i] {
                return Err(ErrorKind::Verification("cycle dead ends".to_owned()))?;
            }
            i = j ^ 1;
            n += 1;
            if i == 0 {
                break;
            }
        }
        if n == self.params.proof_size {
            Ok(())
        } else {
            Err(ErrorKind::Verification("cycle too short".to_owned()))?
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Cuckatoo 29 Solution for Header [0u8;80] - nonce 20
    static V1_29: [u64; 42] = [
        0x48a9e2, 0x9cf043, 0x155ca30, 0x18f4783, 0x248f86c, 0x2629a64, 0x5bad752, 0x72e3569,
        0x93db760, 0x97d3b37, 0x9e05670, 0xa315d5a, 0xa3571a1, 0xa48db46, 0xa7796b6, 0xac43611,
        0xb64912f, 0xbb6c71e, 0xbcc8be1, 0xc38a43a, 0xd4faa99, 0xe018a66, 0xe37e49c, 0xfa975fa,
        0x11786035, 0x1243b60a, 0x12892da0, 0x141b5453, 0x1483c3a0, 0x1505525e, 0x1607352c,
        0x16181fe3, 0x17e3a1da, 0x180b651e, 0x1899d678, 0x1931b0bb, 0x19606448, 0x1b041655,
        0x1b2c20ad, 0x1bd7a83c, 0x1c05d5b0, 0x1c0b9caa,
    ];

    // Cuckatoo 31 Solution for Header [0u8;80] - nonce 99
    static V1_31: [u64; 42] = [
        0x1128e07, 0xc181131, 0x110fad36, 0x1135ddee, 0x1669c7d3, 0x1931e6ea, 0x1c0005f3,
        0x1dd6ecca, 0x1e29ce7e, 0x209736fc, 0x2692bf1a, 0x27b85aa9, 0x29bb7693, 0x2dc2a047,
        0x2e28650a, 0x2f381195, 0x350eb3f9, 0x3beed728, 0x3e861cbc, 0x41448cc1, 0x41f08f6d,
        0x42fbc48a, 0x4383ab31, 0x4389c61f, 0x4540a5ce, 0x49a17405, 0x50372ded, 0x512f0db0,
        0x588b6288, 0x5a36aa46, 0x5c29e1fe, 0x6118ab16, 0x634705b5, 0x6633d190, 0x6683782f,
        0x6728b6e1, 0x67adfb45, 0x68ae2306, 0x6d60f5e1, 0x78af3c4f, 0x7dde51ab, 0x7faced21,
    ];

    #[test]
    fn cuckatoo() {
        let ret = basic_solve::<u32>();
        if let Err(r) = ret {
            panic!("basic_solve u32: Error: {}", r);
        }
        let ret = basic_solve::<u64>();
        if let Err(r) = ret {
            panic!("basic_solve u64: Error: {}", r);
        }
        let ret = validate29_vectors::<u32>();
        if let Err(r) = ret {
            panic!("validate_29_vectors u32: Error: {}", r);
        }
        let ret = validate29_vectors::<u64>();
        if let Err(r) = ret {
            panic!("validate_29_vectors u64: Error: {}", r);
        }
        let ret = validate31_vectors::<u32>();
        if let Err(r) = ret {
            panic!("validate_31_vectors u32: Error: {}", r);
        }
        let ret = validate31_vectors::<u64>();
        if let Err(r) = ret {
            panic!("validate_31_vectors u64: Error: {}", r);
        }
        let ret = validate_fail::<u32>();
        if let Err(r) = ret {
            panic!("validate_fail u32: Error: {}", r);
        }
        let ret = validate_fail::<u64>();
        if let Err(r) = ret {
            panic!("validate_fail u64: Error: {}", r);
        }
    }

    fn validate29_vectors<T>() -> Result<(), Error>
    where
        T: EdgeType,
    {
        let mut ctx = CuckatooContext::<u32>::new_impl(29, 42, 10).unwrap();
        ctx.set_header_nonce([0u8; 80].to_vec(), Some(20), false)?;
        assert!(ctx.verify(&Proof::new(V1_29.to_vec().clone())).is_ok());
        Ok(())
    }

    fn validate31_vectors<T>() -> Result<(), Error>
    where
        T: EdgeType,
    {
        let mut ctx = CuckatooContext::<u32>::new_impl(31, 42, 10).unwrap();
        ctx.set_header_nonce([0u8; 80].to_vec(), Some(99), false)?;
        assert!(ctx.verify(&Proof::new(V1_31.to_vec().clone())).is_ok());
        Ok(())
    }

    fn validate_fail<T>() -> Result<(), Error>
    where
        T: EdgeType,
    {
        let mut ctx = CuckatooContext::<u32>::new_impl(29, 42, 10).unwrap();
        let mut header = [0u8; 80];
        header[0] = 1u8;
        ctx.set_header_nonce(header.to_vec(), Some(20), false)?;
        assert!(!ctx.verify(&Proof::new(V1_29.to_vec().clone())).is_ok());
        header[0] = 0u8;
        ctx.set_header_nonce(header.to_vec(), Some(20), false)?;
        assert!(ctx.verify(&Proof::new(V1_29.to_vec().clone())).is_ok());
        let mut bad_proof = V1_29.clone();
        bad_proof[0] = 0x48a9e1;
        assert!(!ctx.verify(&Proof::new(bad_proof.to_vec())).is_ok());
        Ok(())
    }

    fn basic_solve<T>() -> Result<(), Error>
    where
        T: EdgeType,
    {
        let nonce = 1546569;
        let _range = 1;
        let header = [0u8; 80].to_vec();
        let proof_size = 42;
        let edge_bits = 15;
        let max_sols = 4;

        println!(
            "Looking for {}-cycle on cuckatoo{}(\"{}\",{})",
            proof_size,
            edge_bits,
            String::from_utf8(header.clone()).unwrap(),
            nonce
        );
        let mut ctx_u32 = CuckatooContext::<u32>::new_impl(edge_bits, proof_size, max_sols)?;
        let mut bytes = ctx_u32.byte_count()?;
        let mut unit = 0;
        while bytes >= 10240 {
            bytes >>= 10;
            unit += 1;
        }
        println!("Using {}{}B memory", bytes, [' ', 'K', 'M', 'G', 'T'][unit]);
        ctx_u32.set_header_nonce(header, Some(nonce), true)?;
        println!(
            "Nonce {} k0 k1 k2 k3 {} {} {} {}",
            nonce,
            ctx_u32.sipkey_hex(0)?,
            ctx_u32.sipkey_hex(1)?,
            ctx_u32.sipkey_hex(2)?,
            ctx_u32.sipkey_hex(3)?
        );
        let sols = ctx_u32.find_cycles()?;
        // We know this nonce has 2 solutions
        assert_eq!(sols.len(), 2);
        for s in sols {
            println!("{:?}", s);
        }
        Ok(())
    }
}
