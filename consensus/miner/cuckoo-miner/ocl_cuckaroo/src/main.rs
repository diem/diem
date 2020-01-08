extern crate ocl_cuckaroo;

use ocl_cuckaroo::{Graph, Trimmer};
use std::time::SystemTime;

fn main() -> Result<(), String> {
    let trimmer = Trimmer::build(None, None).expect("can't build trimmer");
    let k = [
        0xf4956dc403730b01,
        0xe6d45de39c2a5a3e,
        0xcbf626a8afee35f6,
        0x4307b94b1a0c9980,
    ];

    unsafe {
        let mut start = SystemTime::now();
        let res = trimmer.run(&k).unwrap();
        let mut end = SystemTime::now();
        let elapsed = end.duration_since(start).unwrap();
        println!("Time: {:?}", elapsed);
        println!("Trimmed to {}", res.len());

        start = SystemTime::now();
        let sols = Graph::search(&res).unwrap();
        end = SystemTime::now();
        let elapsed = end.duration_since(start).unwrap();
        println!("Finder: {:?}", elapsed);
        for sol in sols {
            println!("Solution: {:x?}", sol.nodes);
            start = SystemTime::now();
            let (nonces_c, valid) = trimmer.recover(sol.nodes.clone(), &k).unwrap();
            if valid {
                let nonces = nonces_c.into_iter().map(|v| v as u64).collect::<Vec<u64>>();
                let end = SystemTime::now();
                let elapsed = end.duration_since(start).unwrap();
                println!("Recovering: {:?}", elapsed);
                println!("Nonces: {:?}", nonces);
            } else {
                println!("Not valid");
            }
        }
    }
    Ok(())
}
