use crate::cuckoo::Cuckoo;
use byteorder::{ByteOrder, LittleEndian};
use crate::util::blake2b_256;

pub trait PowContext: Send + Sync {
    fn verify(&self, header_hash: &[u8], nonce: u64, proof: Proof) -> bool;
    fn solve(&self, header_hash: &[u8], nonce: u64) -> Option<Proof>;
}

pub struct Proof {
    solve: Vec<u32>
}

pub struct Pow {
    cuckoo: Cuckoo,
}

impl Pow {
    fn new(edge_bits: u8, cycle_length: usize) -> Self {
        Self {
            cuckoo: Cuckoo::new(edge_bits, cycle_length),
        }
    }
}

impl PowContext for Pow {
    fn verify(&self, header_hash: &[u8], nonce: u64, proof: Proof) -> bool {
        let input = pow_input(header_hash, nonce);
        self.cuckoo.verify(&input, &proof.solve)
    }

    fn solve(&self, header_hash: &[u8], nonce: u64) -> Option<Proof> {
        let input = pow_input(header_hash, nonce);
        self.cuckoo.find_solve(&input).and_then(|solve| Some(Proof { solve }))
    }
}

fn pow_input(header_hash: &[u8], nonce: u64) -> [u8; 40] {
    let mut input = [0; 40];
    input[8..40].copy_from_slice(&header_hash[..32]);
    LittleEndian::write_u64(&mut input, nonce);
    input
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::util::blake2b_256;

    fn test_pow() {
        let header_hash = [197, 125, 237, 29, 128, 222, 12, 134, 5,
            241, 148, 87, 86, 159, 53, 217, 6, 202, 87, 71, 169, 8, 6, 202, 47, 50, 214, 18,
            68, 84, 248, 105, 201, 162, 182, 95, 189, 145, 108, 234, 173, 81, 191, 109, 56,
            192, 59, 176, 113, 85, 75, 254, 237, 161, 177, 189, 22, 219, 131, 24, 67, 96, 12,
            22, 192, 108, 1, 189, 243, 22, 31];
        let pow = Pow::new(6, 8);
        let nonce = 21000;
        let proof = pow.solve(&header_hash, nonce).expect("no solution found");
        println!("find solve");
        assert!(pow.verify(&header_hash, nonce, proof));
    }
}