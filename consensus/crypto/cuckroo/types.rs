// Copyright 2018 The Grin Developers
/// Types for a Cuck(at)oo proof of work and its encapsulation as a fully usable
/// proof of work within a block header.
use std::cmp::{max, min};
use std::ops::{Add, Div, Mul, Sub};
use std::{fmt, iter};

use rand::{thread_rng, Rng};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::consensus::{graph_weight, MIN_DIFFICULTY, SECOND_POW_EDGE_BITS};
use crate::core::hash::{DefaultHashable, Hashed};
use crate::global;
use crate::ser::{self, FixedLength, Readable, Reader, Writeable, Writer};

use crate::pow::common::EdgeType;
use crate::pow::error::Error;

/// Generic trait for a solver/verifier providing common interface into Cuckoo-family PoW
/// Mostly used for verification, but also for test mining if necessary
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

/// The difficulty is defined as the maximum target divided by the block hash.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Difficulty {
    num: u64,
}

impl Difficulty {
    /// Difficulty of zero, which is invalid (no target can be
    /// calculated from it) but very useful as a start for additions.
    pub fn zero() -> Difficulty {
        Difficulty { num: 0 }
    }

    /// Difficulty of MIN_DIFFICULTY
    pub fn min() -> Difficulty {
        Difficulty {
            num: MIN_DIFFICULTY,
        }
    }

    /// Difficulty unit, which is the graph weight of minimal graph
    pub fn unit() -> Difficulty {
        Difficulty {
            num: global::initial_graph_weight() as u64,
        }
    }

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

impl Add<Difficulty> for Difficulty {
    type Output = Difficulty;
    fn add(self, other: Difficulty) -> Difficulty {
        Difficulty {
            num: self.num + other.num,
        }
    }
}

impl Sub<Difficulty> for Difficulty {
    type Output = Difficulty;
    fn sub(self, other: Difficulty) -> Difficulty {
        Difficulty {
            num: self.num - other.num,
        }
    }
}

impl Mul<Difficulty> for Difficulty {
    type Output = Difficulty;
    fn mul(self, other: Difficulty) -> Difficulty {
        Difficulty {
            num: self.num * other.num,
        }
    }
}

impl Div<Difficulty> for Difficulty {
    type Output = Difficulty;
    fn div(self, other: Difficulty) -> Difficulty {
        Difficulty {
            num: self.num / other.num,
        }
    }
}

impl Writeable for Difficulty {
    fn write<W: Writer>(&self, writer: &mut W) -> Result<(), ser::Error> {
        writer.write_u64(self.num)
    }
}

impl Readable for Difficulty {
    fn read(reader: &mut dyn Reader) -> Result<Difficulty, ser::Error> {
        let data = reader.read_u64()?;
        Ok(Difficulty { num: data })
    }
}

impl FixedLength for Difficulty {
    const LEN: usize = 8;
}

impl Serialize for Difficulty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.num)
    }
}

impl<'de> Deserialize<'de> for Difficulty {
    fn deserialize<D>(deserializer: D) -> Result<Difficulty, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(DiffVisitor)
    }
}

struct DiffVisitor;

impl<'de> de::Visitor<'de> for DiffVisitor {
    type Value = Difficulty;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a difficulty")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let num_in = s.parse::<u64>();
        if num_in.is_err() {
            return Err(de::Error::invalid_value(
                de::Unexpected::Str(s),
                &"a value number",
            ));
        };
        Ok(Difficulty {
            num: num_in.unwrap(),
        })
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Difficulty { num: value })
    }
}

/// Block header information pertaining to the proof of work
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProofOfWork {
    /// Total accumulated difficulty since genesis block
    pub total_difficulty: Difficulty,
    /// Variable difficulty scaling factor fo secondary proof of work
    pub secondary_scaling: u32,
    /// Nonce increment used to mine this block.
    pub nonce: u64,
    /// Proof of work data.
    pub proof: Proof,
}

impl Default for ProofOfWork {
    fn default() -> ProofOfWork {
        let proof_size = global::proofsize();
        ProofOfWork {
            total_difficulty: Difficulty::min(),
            secondary_scaling: 1,
            nonce: 0,
            proof: Proof::zero(proof_size),
        }
    }
}

impl Writeable for ProofOfWork {
    fn write<W: Writer>(&self, writer: &mut W) -> Result<(), ser::Error> {
        if writer.serialization_mode() != ser::SerializationMode::Hash {
            self.write_pre_pow(writer)?;
            writer.write_u64(self.nonce)?;
        }
        self.proof.write(writer)?;
        Ok(())
    }
}

impl Readable for ProofOfWork {
    fn read(reader: &mut dyn Reader) -> Result<ProofOfWork, ser::Error> {
        let total_difficulty = Difficulty::read(reader)?;
        let secondary_scaling = reader.read_u32()?;
        let nonce = reader.read_u64()?;
        let proof = Proof::read(reader)?;
        Ok(ProofOfWork {
            total_difficulty,
            secondary_scaling,
            nonce,
            proof,
        })
    }
}

impl ProofOfWork {
    /// Write the pre-hash portion of the header
    pub fn write_pre_pow<W: Writer>(&self, writer: &mut W) -> Result<(), ser::Error> {
        ser_multiwrite!(
            writer,
            [write_u64, self.total_difficulty.to_num()],
            [write_u32, self.secondary_scaling]
        );
        Ok(())
    }

    /// Maximum difficulty this proof of work can achieve
    pub fn to_difficulty(&self, height: u64) -> Difficulty {
        // 2 proof of works, Cuckoo29 (for now) and Cuckoo30+, which are scaled
        // differently (scaling not controlled for now)
        if self.proof.edge_bits == SECOND_POW_EDGE_BITS {
            Difficulty::from_proof_scaled(&self.proof, self.secondary_scaling)
        } else {
            Difficulty::from_proof_adjusted(height, &self.proof)
        }
    }

    /// The edge_bits used for the cuckoo cycle size on this proof
    pub fn edge_bits(&self) -> u8 {
        self.proof.edge_bits
    }

    /// Whether this proof of work is for the primary algorithm (as opposed
    /// to secondary). Only depends on the edge_bits at this time.
    pub fn is_primary(&self) -> bool {
        // 2 conditions are redundant right now but not necessarily in
        // the future
        self.proof.edge_bits != SECOND_POW_EDGE_BITS
            && self.proof.edge_bits >= global::min_edge_bits()
    }

    /// Whether this proof of work is for the secondary algorithm (as opposed
    /// to primary). Only depends on the edge_bits at this time.
    pub fn is_secondary(&self) -> bool {
        self.proof.edge_bits == SECOND_POW_EDGE_BITS
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
///
#[derive(Clone, PartialOrd, PartialEq, Serialize)]
pub struct Proof {
    /// Power of 2 used for the size of the cuckoo graph
    pub edge_bits: u8,
    /// The nonces
    pub nonces: Vec<u64>,
}

impl DefaultHashable for Proof {}

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
            edge_bits: global::min_edge_bits(),
            nonces: in_nonces,
        }
    }

    /// Builds a proof with all bytes zeroed out
    pub fn zero(proof_size: usize) -> Proof {
        Proof {
            edge_bits: global::min_edge_bits(),
            nonces: vec![0; proof_size],
        }
    }

    /// Builds a proof with random POW data,
    /// needed so that tests that ignore POW
    /// don't fail due to duplicate hashes
    pub fn random(proof_size: usize) -> Proof {
        let edge_bits = global::min_edge_bits();
        let nonce_mask = (1 << edge_bits) - 1;
        let mut rng = thread_rng();
        // force the random num to be within edge_bits bits
        let mut v: Vec<u64> = iter::repeat(())
            .map(|()| (rng.gen::<u32>() & nonce_mask) as u64)
            .take(proof_size)
            .collect();
        v.sort_unstable();
        Proof {
            edge_bits: global::min_edge_bits(),
            nonces: v,
        }
    }

    /// Returns the proof size
    pub fn proof_size(&self) -> usize {
        self.nonces.len()
    }

    /// Difficulty achieved by this proof with given scaling factor
    fn scaled_difficulty(&self, scale: u64) -> u64 {
        let diff = ((scale as u128) << 64) / (max(1, self.hash().to_u64()) as u128);
        min(diff, <u64>::max_value() as u128) as u64
    }
}

impl Readable for Proof {
    fn read(reader: &mut dyn Reader) -> Result<Proof, ser::Error> {
        let edge_bits = reader.read_u8()?;
        if edge_bits == 0 || edge_bits > 64 {
            return Err(ser::Error::CorruptedData);
        }

        // prepare nonces and read the right number of bytes
        let mut nonces = Vec::with_capacity(global::proofsize());
        let nonce_bits = edge_bits as usize;
        let bits_len = nonce_bits * global::proofsize();
        let bytes_len = BitVec::bytes_len(bits_len);
        let bits = reader.read_fixed_bytes(bytes_len)?;

        // set our nonces from what we read in the bitvec
        let bitvec = BitVec { bits };
        for n in 0..global::proofsize() {
            let mut nonce = 0;
            for bit in 0..nonce_bits {
                if bitvec.bit_at(n * nonce_bits + (bit as usize)) {
                    nonce |= 1 << bit;
                }
            }
            nonces.push(nonce);
        }

        // check the last bits of the last byte are zeroed, we don't use them but
        // still better to enforce to avoid any malleability
        for n in bits_len..(bytes_len * 8) {
            if bitvec.bit_at(n) {
                return Err(ser::Error::CorruptedData);
            }
        }

        Ok(Proof { edge_bits, nonces })
    }
}

impl Writeable for Proof {
    fn write<W: Writer>(&self, writer: &mut W) -> Result<(), ser::Error> {
        if writer.serialization_mode() != ser::SerializationMode::Hash {
            writer.write_u8(self.edge_bits)?;
        }
        let nonce_bits = self.edge_bits as usize;
        let mut bitvec = BitVec::new(nonce_bits * global::proofsize());
        for (n, nonce) in self.nonces.iter().enumerate() {
            for bit in 0..nonce_bits {
                if nonce & (1 << bit) != 0 {
                    bitvec.set_bit_at(n * nonce_bits + (bit as usize))
                }
            }
        }
        writer.write_fixed_bytes(&bitvec.bits)?;
        Ok(())
    }
}

// TODO this could likely be optimized by writing whole bytes (or even words)
// in the `BitVec` at once, dealing with the truncation, instead of bits by bits
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

    fn bit_at(&self, pos: usize) -> bool {
        self.bits[pos / 8] & (1 << (pos % 8) as u8) != 0
    }
}
