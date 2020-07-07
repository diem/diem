/* lwg:
 * The skeleton is from Fortanix example/usercall-extensions-bind
 * This runner is purely a helper for loading up the lsr enclave, which binds to
 * a local LSR_SGX_ADDRESS for the byte stream communication
 * */

extern crate aesm_client;
extern crate enclave_runner;
extern crate sgxs_loaders;

use aesm_client::AesmClient;
use enclave_runner::usercalls::{SyncStream, UsercallExtension, SyncListener};
use std::io::{Result as IoResult};
use std::net::{TcpListener};
use std::thread;
use enclave_runner::EnclaveBuilder;
use sgxs_loaders::isgx::Device as IsgxDevice;

pub const LSR_SGX_ADDRESS: &str = "localhost:8888";

fn get_enclave_file() -> Result<String, ()> {
/*
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        2 => Ok(args[1].to_owned()),
        _ => {
            usage(args[0].to_owned());
            Err(())
        }
    }
*/
	Ok("/home/lwg/libra/target/x86_64-fortanix-unknown-sgx/debug/lsr-sgx.sgxs".into())
}

struct SafetyRulesSGXListener {
    listener: TcpListener,
}

impl SafetyRulesSGXListener {
     fn new(addr: &str) -> IoResult<(Self, String)> {
        println!("init service for addr {}...", addr);
        TcpListener::bind(addr).map(|listener| {
            let local_address = match listener.local_addr() {
                Ok(local_address) => local_address.to_string(),
                Err(_) => "xxx".to_string(),
            };
            ( SafetyRulesSGXListener {listener}, local_address)
        })
    }
}

impl SyncListener for SafetyRulesSGXListener {
    fn accept(&self, local_addr: Option<&mut String>, peer_addr: Option<&mut String>) -> IoResult<Box<dyn SyncStream>> {
        let (mut stream, peer_address_tcp) = self.listener.accept()?;
        let local_addr_tcp = stream.local_addr()?;
        eprintln!(
            "runner:: bind -- local_address is {}, peer_address is {}",
            local_addr_tcp, peer_address_tcp
            );

        if let Some(local_addr) = local_addr {
            *local_addr = local_addr_tcp.to_string();
        }
        if let Some(peer_addr) = peer_addr {
            *peer_addr  = peer_address_tcp.to_string();
        }
        Ok(Box::new(stream))
    }
}

/* lwg: the dummy struct is kind of ugly but has to exist for "initialize" the binding process */
#[derive(Debug)]
struct SGXService;
impl UsercallExtension for SGXService{
    /* lwg: this is more like a stub, agreed upon Fortanix ABI to communicate with enclave */
    fn bind_stream(
        &self,
        addr: &str,
        local_addr: Option<&mut String>,
    ) -> IoResult<Option<Box<dyn SyncListener>>> {
        if addr == LSR_SGX_ADDRESS {
            println!("Trying to bind {}...", addr);
            let (listener, local_address) = SafetyRulesSGXListener::new(addr)?;
            println!("Successfully bind to local addr: {}", local_address);
            if let Some(local_addr) = local_addr {
                *local_addr = local_address;
            }
            Ok(Some(Box::new(listener)))
        } else {
            Ok(None)
        }
    }
}

fn run_server(file: String) -> Result<(), ()> {
    let mut device = IsgxDevice::new()
        .unwrap()
        .einittoken_provider(AesmClient::new())
        .build();
    let mut enclave_builder = EnclaveBuilder::new(file.as_ref());
    enclave_builder.dummy_signature();
    enclave_builder.usercall_extension(SGXService);
    let enclave = enclave_builder.build(&mut device).unwrap();
    enclave.run().map_err(|e| {
        eprintln!("Error in running enclave {}", e);
    })
}

pub fn start_lsr_enclave() {
    let file = get_enclave_file().unwrap();
    let _server = thread::spawn(move || run_server(file));
    thread::sleep(std::time::Duration::from_secs(5));
}

