use crate::util::blake2b_256;
use byteorder::{ByteOrder, LittleEndian};
use std::collections::HashMap;

extern "C" {
    pub fn c_solve(output: *mut u32, input: *const u8, edge_bits: u64, cycle_length: u32) -> u32;
}

#[derive(Clone)]
pub struct Cuckoo {
    pub max_edge: u64,
    pub edge_mask: u64,
    pub cycle_length: usize,
}

impl Cuckoo {
    pub fn new(edge_bits: u8, cycle_length: usize) -> Self {
        assert!(cycle_length > 0, "cycle_length must be larger than 0");
        Self {
            max_edge: 1 << edge_bits,
            edge_mask: (1 << edge_bits) - 1,
            cycle_length,
        }
    }
    pub fn verify(&self, input: &[u8], proof: &[u32]) -> bool {
        if proof.len() != self.cycle_length {
            return false;
        }

        if u64::from(proof[self.cycle_length - 1]) > self.edge_mask {
            return false;
        }

        let is_monotonous = proof.windows(2).all(|w| w[0] < w[1]);
        if !is_monotonous {
            return false;
        }

        let keys = CuckooSip::input_to_keys(input);
        let hasher = CuckooSip::new(keys[0], keys[1], keys[2], keys[3]);

        let mut from_upper: HashMap<_, Vec<_>> = HashMap::with_capacity(proof.len());
        let mut from_lower: HashMap<_, Vec<_>> = HashMap::with_capacity(proof.len());
        for (u, v) in proof.iter().map(|i| hasher.edge(*i, self.edge_mask)) {
            from_upper
                .entry(u)
                .and_modify(|upper| upper.push(v))
                .or_insert_with(|| vec![v]);
            from_lower
                .entry(v)
                .and_modify(|lower| lower.push(u))
                .or_insert_with(|| vec![u]);
        }
        if from_upper.values().any(|list| list.len() != 2) {
            return false;
        }
        if from_lower.values().any(|list| list.len() != 2) {
            return false;
        }

        let mut cycle_length = 0;
        let mut cur_edge = hasher.edge(proof[0], self.edge_mask);
        let start = cur_edge.0;
        loop {
            let next_lower = *from_upper[&cur_edge.0]
                .iter()
                .find(|v| **v != cur_edge.1)
                .expect("next_lower should be found");
            let next_upper = *from_lower[&next_lower]
                .iter()
                .find(|u| **u != cur_edge.0)
                .expect("next_upper should be found");
            cur_edge = (next_upper, next_lower);
            cycle_length += 2;

            if start == cur_edge.0 {
                break;
            }
        }
        cycle_length == self.cycle_length
    }

    pub fn find_solve(&self, input: &[u8]) -> Option<Vec<u32>> {
        let input = blake2b_256(input.as_ref());
        unsafe {
            let mut output = vec![0u32; self.cycle_length];
            if c_solve(
                output.as_mut_ptr(),
                input.as_ptr(),
                self.max_edge,
                self.cycle_length as u32,
            ) > 0
            {
                return Some(output);
            }
            return None;
        }
    }
}

pub struct CuckooSip {
    keys: [u64; 4],
}

impl CuckooSip {
    pub fn new(key0: u64, key1: u64, key2: u64, key3: u64) -> Self {
        Self {
            keys: [key0, key1, key2, key3],
        }
    }

    fn hash(&self, val: u64) -> u64 {
        let mut v0 = self.keys[0];
        let mut v1 = self.keys[1];
        let mut v2 = self.keys[2];
        let mut v3 = self.keys[3] ^ val;
        CuckooSip::sipround(&mut v0, &mut v1, &mut v2, &mut v3);
        CuckooSip::sipround(&mut v0, &mut v1, &mut v2, &mut v3);
        v0 ^= val;
        v2 ^= 0xff;
        CuckooSip::sipround(&mut v0, &mut v1, &mut v2, &mut v3);
        CuckooSip::sipround(&mut v0, &mut v1, &mut v2, &mut v3);
        CuckooSip::sipround(&mut v0, &mut v1, &mut v2, &mut v3);
        CuckooSip::sipround(&mut v0, &mut v1, &mut v2, &mut v3);

        v0 ^ v1 ^ v2 ^ v3
    }

    fn sipround(v0: &mut u64, v1: &mut u64, v2: &mut u64, v3: &mut u64) {
        *v0 = v0.wrapping_add(*v1);
        *v2 = v2.wrapping_add(*v3);
        *v1 = v1.rotate_left(13);

        *v3 = v3.rotate_left(16);
        *v1 ^= *v0;
        *v3 ^= *v2;

        *v0 = v0.rotate_left(32);
        *v2 = v2.wrapping_add(*v1);
        *v0 = v0.wrapping_add(*v3);

        *v1 = v1.rotate_left(17);
        *v3 = v3.rotate_left(21);

        *v1 ^= *v2;
        *v3 ^= *v0;
        *v2 = v2.rotate_left(32);
    }

    pub fn edge(&self, val: u32, edge_mask: u64) -> (u64, u64) {
        let upper = self.hash(u64::from(val) << 1) & edge_mask;
        let lower = self.hash((u64::from(val) << 1) + 1) & edge_mask;

        (upper, lower)
    }

    pub fn input_to_keys(input: &[u8]) -> [u64; 4] {
        let result = blake2b_256(input.as_ref());
        [
            LittleEndian::read_u64(&result[0..8]).to_le(),
            LittleEndian::read_u64(&result[8..16]).to_le(),
            LittleEndian::read_u64(&result[16..24]).to_le(),
            LittleEndian::read_u64(&result[24..32]).to_le(),
        ]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::blake2b_256;

    #[test]
    fn test_pow() {
        let header_hash = [
            24, 75, 179, 121, 98, 241, 250, 124, 100, 197, 125, 237, 29, 128, 222, 12, 134, 5, 241,
            148, 87, 86, 159, 53, 217, 6, 202, 87, 71, 169, 8, 6, 202, 47, 50, 214, 18, 68, 84,
            248, 105, 201, 162, 182, 95, 189, 145, 108, 234, 173, 81, 191, 109, 56, 192, 59, 176,
            113, 85, 75, 254, 237, 161, 177, 189, 22, 219, 131, 24, 67, 96, 12, 22, 192, 108, 1,
            189, 243, 22, 31,
        ];
        let pow = &Cuckoo::new(6, 8);
        let proof = pow.find_solve(&header_hash).expect("not find solve");
        println!("find solve");
        assert!(pow.verify(&header_hash, &proof));
    }
}
