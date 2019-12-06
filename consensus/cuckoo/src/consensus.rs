use crate::util::pow_input;
use crate::Cuckoo;
use byteorder::{ByteOrder, LittleEndian};

pub trait PowService: Send + Sync {
    fn verify(&self, header_hash: &[u8], nonce: u64, proof: Proof) -> bool;
}

pub struct Proof {
    pub solve: Vec<u8>,
}

pub struct PowCuckoo {
    cuckoo: Cuckoo,
}

impl PowCuckoo {
    pub fn new(edge_bits: u8, cycle_length: usize) -> Self {
        Self {
            cuckoo: Cuckoo::new(edge_bits, cycle_length),
        }
    }
}

impl PowService for PowCuckoo {
    fn verify(&self, header_hash: &[u8], nonce: u64, proof: Proof) -> bool {
        let input = pow_input(header_hash, nonce);
        let mut proof_u32 = vec![0u32; self.cuckoo.cycle_length];
        LittleEndian::read_u32_into(&proof.solve, &mut proof_u32);
        self.cuckoo.verify(&input, &proof_u32)
    }
}
