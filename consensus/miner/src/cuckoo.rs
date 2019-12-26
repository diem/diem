use crate::types::{set_header_nonce, Proof, Solution};
use byteorder::{LittleEndian, WriteBytesExt};
use cuckaroo::CuckarooProof;
use cuckaroo::{new_cuckaroo_ctx, PoWContext as CuckooCtx};
use cuckoo_miner::{self, CuckooMinerError, PluginLibrary};
use std::convert::Into;
use std::env;

enum CuckooPlugin {
    Cuckaroo29Cpu,
    Cuckaroo19Cpu,
}

impl Into<&str> for CuckooPlugin {
    fn into(self) -> &'static str {
        match self {
            CuckooPlugin::Cuckaroo29Cpu => "cuckaroo_cpu_compat_29.cuckooplugin",
            CuckooPlugin::Cuckaroo19Cpu => "cuckaroo_cpu_compat_19.cuckooplugin",
        }
    }
}

fn load_plugin_lib(plugin: &str) -> Result<PluginLibrary, CuckooMinerError> {
    let mut p_path = env::current_exe().unwrap();
    p_path.pop();
    p_path.pop();
    p_path.push("plugins");
    p_path.push(plugin);
    PluginLibrary::new(p_path.to_str().unwrap())
}

pub fn mine(header: &[u8], nonce: u32) -> Solution {
    let plugin = CuckooPlugin::Cuckaroo19Cpu;
    let pl = load_plugin_lib(plugin.into()).unwrap();
    let mut params = pl.get_default_params();
    let mut solutions = cuckoo_miner::SolverSolutions::default();
    let mut stats = cuckoo_miner::SolverStats::default();
    params.nthreads = 4;
    params.mutate_nonce = false;
    let ctx = pl.create_solver_ctx(&mut params);
    let header = set_header_nonce(header, nonce);
    let _ = pl.run_solver(ctx, header.to_owned(), 0, 1, &mut solutions, &mut stats);
    if solutions.num_sols <= 0 {
        return Solution::default();
    }
    let solution: Solution = solutions.sols[0].proof.into();
    solution
}

pub fn verify(header: &[u8], nonce: u32, solution: Solution) -> bool {
    let mut ctx = new_cuckaroo_ctx::<u64>(19, 42).unwrap();

    let _ = ctx.set_header_nonce(header.to_owned(), Some(nonce), true);
    let proof = CuckarooProof::new(solution.into());
    ctx.verify(&proof).is_ok()
}

#[cfg(test)]
mod test {
    use super::*;
    use cuckaroo::PROOF_SIZE;

    #[test]
    fn test_mine() {
        let header = vec![0u8; 80];
        let nonce = 143;
        let solution = mine(&header, nonce);
        let s64: [u64; PROOF_SIZE] = solution.clone().into();
        assert!(verify(&header, nonce, solution.clone()));
        println!("solution:{:?},{:?}", s64.to_vec(), solution.hash());
    }
}
