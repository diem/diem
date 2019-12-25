use cuckoo_miner::{self, CuckooMinerError, PluginLibrary};

use blake2_rfc::blake2b::blake2b;
use byteorder::{LittleEndian, WriteBytesExt};
use cuckoo::Cuckoo;
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

pub fn mine(header: Vec<u8>, nonce: u32) {
    let plugin = CuckooPlugin::Cuckaroo19Cpu;
    let pl = load_plugin_lib(plugin.into()).unwrap();
    let mut params = pl.get_default_params();
    let mut solutions = cuckoo_miner::SolverSolutions::default();
    let mut stats = cuckoo_miner::SolverStats::default();
    params.nthreads = 4;
    params.mutate_nonce = false;
    let ctx = pl.create_solver_ctx(&mut params);
    let header = set_header_nonce(&header, nonce);
    let _ = pl.run_solver(ctx, header.clone(), 0, 1, &mut solutions, &mut stats);
    println!("{:?}", solutions.num_sols);
    let proof = solutions.sols[0].proof;
    let verifier = Cuckoo::new();
    assert!(verifier.verify(&header, 143, proof.to_vec()));
}

pub fn set_header_nonce(header: &[u8], nonce: u32) -> Vec<u8> {
    let len = header.len();
    let mut header = header.to_owned();
    header.truncate(len - 4); // drop last 4 bytes (u32) off the end
    header.write_u32::<LittleEndian>(nonce);
    //let h = blake2b(32, &[], &header);
    //h.as_bytes().to_vec()
    header
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mine() {
        let header = vec![0u8; 80];
        let nonce = 143;
        mine(header, nonce);
    }
}
