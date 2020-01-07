use crate::config::MinerConfig;
use crate::cuckoo;
use crate::types::{set_header_nonce, Algo, Solution, H256, U256};
use rand::Rng;
use scrypt::scrypt_1024_1_1_256;

fn calculate_pow_hash(header: &[u8], nonce: u32, cfg: &MinerConfig) -> (H256, Solution) {
    match cfg.algorithm {
        Algo::CUCKOO => {
            let solution = cuckoo::mine(&header, nonce, cfg.nthread, cfg.device);
            return (solution.hash(), solution);
        }
        _ => {
            let mut output = [0u8; 32];
            scrypt_1024_1_1_256(&set_header_nonce(header, nonce), &mut output);
            let hash: H256 = output.into();
            return (hash, Solution::default());
        }
    }
}

fn generate_nonce() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen::<u32>();
    rng.gen_range(0, u32::max_value())
}

pub fn solve(header: &[u8], target: &U256, cfg: &MinerConfig) -> (u32, Solution) {
    let mut nonce = generate_nonce();
    loop {
        let (hash, solution) = calculate_pow_hash(header, nonce, cfg);
        if cfg.algorithm == Algo::CUCKOO && solution == Solution::default() {
            nonce += 1;
            continue;
        }
        let hash_u256: U256 = hash.into();
        if hash_u256 > *target {
            nonce += 1;
            continue;
        }
        return (nonce, solution);
    }
}

pub fn verify(header: &[u8], nonce: u32, solution: Solution, algo: &Algo, target: &U256) -> bool {
    let mut pow_hash = [0u8; 32];
    match *algo {
        Algo::CUCKOO => {
            if !cuckoo::verify(&header, nonce, solution.clone()) {
                return false;
            }
            pow_hash = solution.hash().into();
        }
        Algo::SCRYPT => {
            scrypt_1024_1_1_256(&set_header_nonce(&header, nonce), &mut pow_hash);
        }
    }
    let hash_h256: H256 = pow_hash.into();
    let hash_u256: U256 = hash_h256.into();
    if hash_u256 <= *target {
        return true;
    }
    return false;
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore] // Too slow, ignore it
    #[test]
    fn test_solve() {
        let difficult: U256 = (1 as u32).into();
        let target = U256::max_value() / difficult;
        let header = "header is me".as_bytes();
        let mut cfg = MinerConfig::default();
        cfg.algorithm = Algo::CUCKOO;
        cfg.device = 2;
        let (nonce, solution) = solve(&header, &target, &cfg);
        assert_eq!(
            true,
            verify(&header, nonce, solution, &Algo::CUCKOO, &target)
        );
    }
}
