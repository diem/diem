use std::io::{BufRead, BufReader, Write, Error, ErrorKind, Result};
use std::net::{TcpStream, TcpListener};
use libra_types::{validator_signer::ValidatorSigner, epoch_change::EpochChangeProof};
use consensus_types::{
    block_data::BlockData,
    vote::Vote,
    vote_data::VoteData,
    vote_proposal::{MaybeSignedVoteProposal, VoteProposal},
    timeout::Timeout,
};

use crate::{safety_rules::SafetyRules};
mod safety_rules;
mod consensus_state;

pub const LSR_SGX_ADDRESS: &str = "localhost:8888";


struct LSRCore {
    stream: TcpStream,
}

impl LSRCore {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream
        }
    }

}

/* this works for using ValidatorSigner */
fn test_validator_signer() {
    let a = ValidatorSigner::from_int(1);
    println!("signer = {:#?}", a);
}


fn test_data_types() {
    test_validator_signer();
}

fn process_safety_rules_reqs(lsr: &mut SafetyRules, stream: &TcpStream) -> Result<()> {
    eprintln!("LSR_CORE: received a new incoming req...");
    let peer_addr = stream.peer_addr()?;
    let local_addr = stream.local_addr()?;
    eprintln!(
        "LSR_CORE: accept meesage from local {:?}, peer {:?}, stream = {:?}",
        local_addr, peer_addr, stream
    );
    let mut reader = BufReader::new(stream);
    let mut request = String::new();
    eprintln!("About to read...");
    let read_bytes = reader.read_line(&mut request).unwrap();
    eprintln!("finish reading {} bytes...", read_bytes);
    // fill the read of buf
    //let buf = reader.fill_buf().unwrap();
    let buf = reader.buffer();
    eprintln!("buf = {:?}", buf);
    match request.as_str().trim() {
        "req:init" => {
            // fill the read of buf
            let input: EpochChangeProof = lcs::from_bytes(buf).unwrap();
            eprintln!("{} -- {:#?}", request, input);
        }
        "req:consensus_state" => {
            let response = lsr.consensus_state();
            let mut stream = BufReader::new(stream.try_clone().unwrap());
            stream.get_mut().write_all("done".as_bytes()).unwrap();
            eprintln!("{} -- {:?}", request, buf);
        }
        "req:construct_and_sign_vote" => {
            let input: MaybeSignedVoteProposal = lcs::from_bytes(buf).unwrap();
            eprintln!("{} -- {}", request, input.vote_proposal);
        }
        "req:sign_proposal" => {
            let input: BlockData = lcs::from_bytes(buf).unwrap();
            eprintln!("{} -- {:#?}", request, input);
        }
        "req:sign_timeout" => {
            let input: Timeout = lcs::from_bytes(buf).unwrap();
            eprintln!("{} -- {:#?}", request, input);
        }
        _ => {
            eprintln!("invalid req...{}", request);
        }
    }
  Ok(())
}

fn main() -> Result<()> {
    test_data_types();
    let mut safety_rules = SafetyRules::new();
    let listener = TcpListener::bind(LSR_SGX_ADDRESS)?;
    eprintln!("Ready to accept...");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                process_safety_rules_reqs(&mut safety_rules, &stream)?;
            }
            Err(_) => {
                eprintln!("unable to connect...");
            }
        }
    }
    eprintln!(
        "Wohoo! LSR_CORE about to terminate",
    );
    Ok(())
}
