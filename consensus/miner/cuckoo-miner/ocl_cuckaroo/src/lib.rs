extern crate blake2_rfc;
extern crate byteorder;
extern crate cuckoo_plugin as plugin;
extern crate hashbrown;
extern crate libc;
extern crate ocl;

use blake2_rfc::blake2b::blake2b;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use libc::*;
use plugin::*;
use std::io::Cursor;
use std::io::Error;
use std::mem;
use std::ptr;
use std::time::{Duration, SystemTime};

pub use self::finder::Graph;
pub use self::trimmer::Trimmer;

mod finder;
mod trimmer;

#[repr(C)]
struct Solver {
    trimmer: Trimmer,
    graph: Option<Graph>,
    mutate_nonce: bool,
}

#[no_mangle]
pub unsafe extern "C" fn create_solver_ctx(params: *mut SolverParams) -> *mut SolverCtx {
    let platform = match (*params).platform {
        1 => Some("AMD"),
        2 => Some("NVIDIA"),
        _ => None,
    };
    let device_id = Some((*params).device as usize);

    let trimmer = Trimmer::build(platform, device_id).expect("can't build trimmer");
    let solver = Solver {
        trimmer,
        graph: None,
        mutate_nonce: (*params).mutate_nonce,
    };
    let solver_box = Box::new(solver);
    let solver_ref = Box::leak(solver_box);
    mem::transmute::<&mut Solver, *mut SolverCtx>(solver_ref)
}

#[no_mangle]
pub unsafe extern "C" fn destroy_solver_ctx(solver_ctx_ptr: *mut SolverCtx) {
    // create box to clear memory
    let solver_ptr = mem::transmute::<*mut SolverCtx, *mut Solver>(solver_ctx_ptr);
    let _solver_box = Box::from_raw(solver_ptr);
}

#[no_mangle]
pub unsafe extern "C" fn stop_solver(_solver_ctx_ptr: *mut SolverCtx) {}

#[no_mangle]
pub unsafe extern "C" fn fill_default_params(params: *mut SolverParams) {
    (*params).device = 0;
    (*params).platform = 0;
    (*params).edge_bits = 29;
}

#[no_mangle]
pub unsafe extern "C" fn run_solver(
    ctx: *mut SolverCtx,
    header_ptr: *const c_uchar,
    header_length: u32,
    nonce: u64,
    _range: u32,
    solutions: *mut SolverSolutions,
    stats: *mut SolverStats,
) -> u32 {
    let start = SystemTime::now();
    let solver_ptr = mem::transmute::<*mut SolverCtx, *mut Solver>(ctx);
    let solver = &*solver_ptr;
    let mut header = Vec::with_capacity(header_length as usize + 32);
    let r_ptr = header.as_mut_ptr();
    ptr::copy_nonoverlapping(header_ptr, r_ptr, header_length as usize);
    header.set_len(header_length as usize);
    let n = nonce as u64;
    let k = match set_header_nonce(&header, Some(n), solver.mutate_nonce) {
        Err(_e) => {
            return 2;
        }
        Ok(v) => v,
    };
    let res = solver.trimmer.run(&k).unwrap();

    let sols = Graph::search(&res).unwrap();
    let mut i = 0;
    (*solutions).edge_bits = 29;
    for sol in sols {
        let (nonces_cand, valid) = solver.trimmer.recover(sol.nodes, &k).unwrap();
        if valid {
            let nonces = nonces_cand
                .into_iter()
                .map(|v| v as u64)
                .collect::<Vec<u64>>();
            (*solutions).sols[i].nonce = nonce;
            (*solutions).sols[i].proof.copy_from_slice(&nonces[..]);
            i += 1;
        }
    }
    (*solutions).num_sols = i as u32;
    let end = SystemTime::now();
    let elapsed = end.duration_since(start).unwrap();
    (*stats).edge_bits = 29;
    (*stats).device_id = solver.trimmer.device_id as u32;
    let name_bytes = solver.trimmer.device_name.as_bytes();
    let n = std::cmp::min((*stats).device_name.len(), name_bytes.len());
    (*stats).device_name[..n].copy_from_slice(&solver.trimmer.device_name.as_bytes()[..n]);
    (*stats).last_solution_time = duration_to_u64(elapsed);
    (*stats).last_start_time =
        duration_to_u64(start.duration_since(SystemTime::UNIX_EPOCH).unwrap());
    (*stats).last_end_time = duration_to_u64(end.duration_since(SystemTime::UNIX_EPOCH).unwrap());
    0
}

fn duration_to_u64(elapsed: Duration) -> u64 {
    elapsed.as_secs() * 1_000_000_000 + elapsed.subsec_nanos() as u64
}

pub fn set_header_nonce(
    header: &[u8],
    nonce: Option<u64>,
    mutate_nonce: bool,
) -> Result<[u64; 4], Error> {
    if let Some(n) = nonce {
        let len = header.len();
        let mut header = header.to_owned();
        if mutate_nonce {
            header.truncate(len - 4);
            header.write_u32::<LittleEndian>(n as u32)?;
        }
        create_siphash_keys(&header)
    } else {
        create_siphash_keys(&header)
    }
}

pub fn create_siphash_keys(header: &[u8]) -> Result<[u64; 4], Error> {
    let h = blake2b(32, &[], &header);
    let hb = h.as_bytes();
    let mut rdr = Cursor::new(hb);
    Ok([
        rdr.read_u64::<LittleEndian>()?,
        rdr.read_u64::<LittleEndian>()?,
        rdr.read_u64::<LittleEndian>()?,
        rdr.read_u64::<LittleEndian>()?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[ignore]
    // results in Error executing function: clEnqueueNDRangeKernel("LeanRound")
    //            Status error code: CL_INVALID_WORK_GROUP_SIZE (-54)
    // on MacOSX
    #[test]
    fn test_solve() {
        let trimmer = Trimmer::build(None, None).expect("can't build trimmer");
        let k = [
            0x27580576fe290177,
            0xf9ea9b2031f4e76e,
            0x1663308c8607868f,
            0xb88839b0fa180d0e,
        ];

        unsafe {
            let res = trimmer.run(&k).unwrap();
            println!("Trimmed to {}", res.len());

            let sols = Graph::search(&res).unwrap();
            assert_eq!(1, sols.len());
        }
    }
}
