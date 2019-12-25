// Copyright 2018 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Difficulty calculation as from Grin
use blake2::blake2b::Blake2b;
use byteorder::{BigEndian, ByteOrder};
use std::cmp::{max, min};
use std::fmt;

// constants from grin
const DEFAULT_MIN_EDGE_BITS: u8 = 31;
const SECOND_POW_EDGE_BITS: u8 = 29;
const PROOF_SIZE: usize = 42;
const BLOCK_TIME_SEC: u64 = 60;

const HOUR_HEIGHT: u64 = 3600 / BLOCK_TIME_SEC;
const DAY_HEIGHT: u64 = 24 * HOUR_HEIGHT;
const WEEK_HEIGHT: u64 = 7 * DAY_HEIGHT;
const YEAR_HEIGHT: u64 = 52 * WEEK_HEIGHT;

const BASE_EDGE_BITS: u8 = 24;

/// Compute weight of a graph as number of siphash bits defining the graph
/// Must be made dependent on height to phase out smaller size over the years
/// This can wait until end of 2019 at latest
pub fn graph_weight(height: u64, edge_bits: u8) -> u64 {
	let mut xpr_edge_bits = edge_bits as u64;

	let bits_over_min = edge_bits.saturating_sub(DEFAULT_MIN_EDGE_BITS);
	let expiry_height = (1 << bits_over_min) * YEAR_HEIGHT;
	if height >= expiry_height {
		xpr_edge_bits = xpr_edge_bits.saturating_sub(1 + (height - expiry_height) / WEEK_HEIGHT);
	}

	(2 << (edge_bits - BASE_EDGE_BITS) as u64) * xpr_edge_bits
}

/// The difficulty is defined as the maximum target divided by the block hash.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Difficulty {
	num: u64,
}

impl Difficulty {
	/// Convert a `u32` into a `Difficulty`
	pub fn from_num(num: u64) -> Difficulty {
		// can't have difficulty lower than 1
		Difficulty { num: max(num, 1) }
	}

	/// Computes the difficulty from a hash. Divides the maximum target by the
	/// provided hash and applies the Cuck(at)oo size adjustment factor (see
	/// https://lists.launchpad.net/mimblewimble/msg00494.html).
	fn from_proof_adjusted(height: u64, proof: &Proof) -> Difficulty {
		// scale with natural scaling factor
		Difficulty::from_num(proof.scaled_difficulty(graph_weight(height, proof.edge_bits)))
	}

	/// unscaled proof
	fn from_proof_unscaled(proof: &Proof) -> Difficulty {
		Difficulty::from_num(proof.scaled_difficulty(1u64))
	}

	/// Same as `from_proof_adjusted` but instead of an adjustment based on
	/// cycle size, scales based on a provided factor. Used by dual PoW system
	/// to scale one PoW against the other.
	fn from_proof_scaled(proof: &Proof, scaling: u32) -> Difficulty {
		// Scaling between 2 proof of work algos
		Difficulty::from_num(proof.scaled_difficulty(scaling as u64))
	}

	/// Converts the difficulty into a u64
	pub fn to_num(&self) -> u64 {
		self.num
	}
}

impl fmt::Display for Difficulty {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.num)
	}
}

/// A Cuck(at)oo Cycle proof of work, consisting of the edge_bits to get the graph
/// size (i.e. the 2-log of the number of edges) and the nonces
/// of the graph solution. While being expressed as u64 for simplicity,
/// nonces a.k.a. edge indices range from 0 to (1 << edge_bits) - 1
///
/// The hash of the `Proof` is the hash of its packed nonces when serializing
/// them at their exact bit size. The resulting bit sequence is padded to be
/// byte-aligned.

#[derive(Clone, PartialOrd, PartialEq)]
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
	pub fn new(mut in_nonces: Vec<u64>, edge_bits: u8) -> Proof {
		in_nonces.sort();
		Proof {
			edge_bits: edge_bits,
			nonces: in_nonces,
		}
	}

	/// Difficulty achieved by this proof with given scaling factor
	fn scaled_difficulty(&self, scale: u64) -> u64 {
		let diff = ((scale as u128) << 64) / (max(1, self.hash().to_u64()) as u128);
		min(diff, <u64>::max_value() as u128) as u64
	}

	/// Hash, as in Grin
	fn hash(&self) -> Hash {
		let nonce_bits = self.edge_bits as usize;
		let mut bitvec = BitVec::new(nonce_bits * PROOF_SIZE);
		for (n, nonce) in self.nonces.iter().enumerate() {
			for bit in 0..nonce_bits {
				if nonce & (1 << bit) != 0 {
					bitvec.set_bit_at(n * nonce_bits + (bit as usize))
				}
			}
		}
		let mut blake2b = Blake2b::new(32);
		blake2b.update(&bitvec.bits);
		let mut ret = [0; 32];
		ret.copy_from_slice(blake2b.finalize().as_bytes());
		Hash(ret)
	}

	/// Maximum difficulty this proof of work can achieve
	pub fn to_difficulty(&self, height: u64, sec_scaling: u32) -> Difficulty {
		// 2 proof of works, Cuckoo29 (for now) and Cuckoo30+, which are scaled
		// differently (scaling not controlled for now)
		if self.edge_bits == SECOND_POW_EDGE_BITS {
			Difficulty::from_proof_scaled(&self, sec_scaling)
		} else {
			Difficulty::from_proof_adjusted(height, &self)
		}
	}

	/// unscaled difficulty
	pub fn to_difficulty_unscaled(&self) -> Difficulty {
		Difficulty::from_proof_unscaled(&self)
	}
}

struct BitVec {
	bits: Vec<u8>,
}

impl BitVec {
	/// Number of bytes required to store the provided number of bits
	fn bytes_len(bits_len: usize) -> usize {
		(bits_len + 7) / 8
	}

	fn new(bits_len: usize) -> BitVec {
		BitVec {
			bits: vec![0; BitVec::bytes_len(bits_len)],
		}
	}

	fn set_bit_at(&mut self, pos: usize) {
		self.bits[pos / 8] |= 1 << (pos % 8) as u8;
	}
}

impl Hash {
	/// to u64
	pub fn to_u64(&self) -> u64 {
		BigEndian::read_u64(&self.0)
	}

	/// to hex
	pub fn to_hex(&self) -> String {
		util::to_hex(self.0.to_vec())
	}
}

/// A hash to uniquely (or close enough) identify one of the main blockchain
/// constructs. Used pervasively for blocks, transactions and outputs.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl fmt::Debug for Hash {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let hash_hex = self.to_hex();
		const NUM_SHOW: usize = 12;

		write!(f, "{}", &hash_hex[..NUM_SHOW])
	}
}

impl fmt::Display for Hash {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn proof_hash() {
		let mut in_nonces: Vec<u64> = [0u64; 42].to_vec();
		let proof = Proof::new(in_nonces.clone(), DEFAULT_MIN_EDGE_BITS);
		let hash_str = format!("{}", proof.hash());
		println!("Hash is: {}", hash_str);
		assert_eq!(&hash_str, "5fa5af8a4c86");

		in_nonces[41] = 23402320128419283;
		in_nonces[11] = 81239481234781924;
		let proof = Proof::new(in_nonces.clone(), DEFAULT_MIN_EDGE_BITS);
		let hash_str = format!("{}", proof.hash());
		println!("Hash is: {}", hash_str);
		assert_eq!(&hash_str, "378594bac9a4");

		for i in in_nonces.iter_mut() {
			*i = std::u64::MAX;
		}
		let proof = Proof::new(in_nonces.clone(), DEFAULT_MIN_EDGE_BITS);
		let hash_str = format!("{}", proof.hash());
		println!("Hash is: {}", hash_str);
		assert_eq!(&hash_str, "99f04aafcbc1");
	}

	#[test]
	fn proof_difficulty() {
		let mut in_nonces: Vec<u64> = [0u64; 42].to_vec();
		let proof = Proof::new(in_nonces.clone(), DEFAULT_MIN_EDGE_BITS);
		let difficulty = proof.to_difficulty(20, 1);
		println!("Diff is: {}", difficulty);
		assert_eq!(difficulty, Difficulty::from_num(21240));

		in_nonces[41] = 23402320128419283;
		in_nonces[11] = 81239481234781924;
		let proof = Proof::new(in_nonces.clone(), 31);
		let difficulty = proof.to_difficulty(120000, 32348);
		println!("Diff is: {}", difficulty);
		assert_eq!(difficulty, Difficulty::from_num(36591));

		for i in in_nonces.iter_mut() {
			*i = std::u64::MAX;
		}
		let proof = Proof::new(in_nonces.clone(), 35);
		let difficulty = proof.to_difficulty(1300000, 92348);
		println!("Diff is: {}", difficulty);
		assert_eq!(difficulty, Difficulty::from_num(296303));
	}
}
