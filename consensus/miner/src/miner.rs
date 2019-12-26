use crate::types::{Algo, H256, U256};
use cuckoo::util::{blake2b_256, pow_input};
use cuckoo::{Cuckoo, Solution};
use rand::Rng;
use scrypt::scrypt_1024_1_1_256;

fn calculate_pow_hash(
    header_hash: &H256,
    algo: &Algo,
    nonce: u64,
) -> (Option<H256>, Option<Solution>) {
    match algo {
        &Algo::CUCKOO => {
            let cuckoo = Cuckoo::new();
            let input = blake2b_256(pow_input(header_hash.as_bytes(), nonce).as_ref());
            let solution = cuckoo.solve(&input);
            match solution {
                Some(solution) => (
                    Some(blake2b_256(solution.0.as_ref()).into()),
                    Some(solution),
                ),
                None => (None, None),
            }
        }
        &Algo::SCRYPT => {
            let mut output = [0u8; 32];
            scrypt_1024_1_1_256(&pow_input(header_hash.as_bytes(), nonce), &mut output);
            let hash: H256 = output.into();
            (Some(hash), None)
        }
    }
}

fn generate_nonce() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen::<u64>();
    rng.gen_range(0, u64::max_value())
}

pub fn solve(header_hash: &H256, algo: &Algo, target: &U256) -> (u64, Option<Solution>) {
    let mut nonce = generate_nonce();
    loop {
        let (hash, solution) = calculate_pow_hash(header_hash, algo, nonce);
        if *algo == Algo::CUCKOO && solution.is_none() {
            nonce += 1;
            continue;
        }
        let hash_u256: U256 = hash.unwrap().into();
        if hash_u256 > *target {
            nonce += 1;
            continue;
        }
        return (nonce, solution);
    }
}

pub fn verify(
    header_hash: &H256,
    nonce: u64,
    solution: Option<Solution>,
    algo: &Algo,
    target: &U256,
) -> bool {
    let mut pow_hash = [0u8; 32];
    match *algo {
        Algo::CUCKOO => {
            let input_hash = blake2b_256(pow_input(header_hash.as_bytes(), nonce).as_ref());
            let cuckoo = Cuckoo::new();
            if solution.is_none() {
                return false;
            }
            if cuckoo.verify(&input_hash, &solution.clone().unwrap()) == false {
                return false;
            }
            pow_hash = blake2b_256(solution.unwrap().0.as_ref()).into();
        }
        Algo::SCRYPT => {
            scrypt_1024_1_1_256(&pow_input(header_hash.as_bytes(), nonce), &mut pow_hash);
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

    #[test]
    fn test_solve() {
        let difficult: U256 = (1 as u32).into();
        let target = U256::max_value() / difficult;
        let header = "header is me".as_bytes();
        let header_hash: H256 = blake2b_256(header.as_ref()).into();
        let (nonce, solution) = solve(&header_hash, &Algo::CUCKOO, &target);
        assert_eq!(
            true,
            verify(&header_hash, nonce, solution, &Algo::CUCKOO, &target)
        );
    }
}
