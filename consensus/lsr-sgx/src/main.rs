use std::io::{BufRead, BufReader, Write, Error, ErrorKind};
use std::net::{TcpStream, TcpListener};
use libra_types::{validator_signer::ValidatorSigner};

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

fn init_lsr_core(addr: String) -> Result<LSRCore, Error> {
    let a = ValidatorSigner::from_int(1);
    println!("signer = {:#?}", a);
    if let Ok(tcp_stream) = TcpStream::connect(addr) {
        Ok(LSRCore::new(tcp_stream))
    } else {
        return Err(Error::new(ErrorKind::Other, "oh no"));
    }
}


fn main() -> std::io::Result<()> {
    //let lsr_core = init_lsr_core("lsr".into()).unwrap();

    let listener = TcpListener::bind("127.0.0.1:8888")?;
    let (stream, peer_addr) = listener.accept()?;
    let peer_addr = peer_addr.to_string();
    let local_addr = stream.local_addr()?;
    eprintln!(
        "LSR_CORE:accept meesage from local {}, peer {}",
        local_addr, peer_addr
        );

    let mut reader = BufReader::new(stream);
    let mut message = String::new();
    loop {
        let read_bytes = reader.read_line(&mut message)?;
        if read_bytes == 0 {
            break;
        }
        println!("{}", message);
    }
    Ok(())
}
