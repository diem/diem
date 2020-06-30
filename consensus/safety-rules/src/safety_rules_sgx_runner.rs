/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/* lwg:
 * The skeleton is from Fortanix example/usercall-extensions-bind
 * */

extern crate aesm_client;
extern crate enclave_runner;
extern crate sgxs_loaders;

use aesm_client::AesmClient;
use enclave_runner::usercalls::{SyncStream, UsercallExtension, SyncListener};
use std::io::{Read, Write, Result as IoResult, Error, ErrorKind};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::thread;
use enclave_runner::EnclaveBuilder;
use sgxs_loaders::isgx::Device as IsgxDevice;
use std::process::{Child, Command, Stdio};
use tokio::sync::lock::Lock;
use tokio::prelude::Async;



/// This example demonstrates use of usercall extensions for bind call.
/// User call extension allow the enclave code to "bind" to an external service via a customized enclave runner.
/// Here we customize the runner to intercept calls to bind to an address and advance the stream before returning it to enclave
/// This can be useful to strip protocol encapsulations, say while servicing requests load balanced by HA Proxy.
/// This example demonstrates de-encapsulation for various HA proxy configurations before handing over the stream to the enclave.
/// To simulate HA proxy configurations, the runner spawns a thread that connects to the same address which enclave binds to and
/// writes encapsulated test data for various HA proxy configurations to the stream.

fn parse_args() -> Result<String, ()> {
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

struct LSRService{
    listener: TcpListener,
}

impl LSRService {
    fn new(addr: &str) -> IoResult<(Self, String)> {
        println!("init service for addr {}...", addr);
        TcpListener::bind(addr).map(|listener| {
            let local_address = match listener.local_addr() {
                Ok(local_address) => local_address.to_string(),
                Err(_) => "xxx".to_string(),
            };
            (LSRService {listener}, local_address)
        })
    }
}

impl SyncListener for LSRService {
    fn accept(&self, local_addr: Option<&mut String>, peer_addr: Option<&mut String>) -> IoResult<Box<dyn SyncStream>> {

        eprintln!(
            "hello from {}", line!()
        );
        let (mut stream, peer_address_tcp) = self.listener.accept()?;
        eprintln!(
            "hello from {}", line!()
        );
        let local_addr_tcp = stream.local_addr()?;
        eprintln!(
            "runner:: bind -- local_address is {}, peer_address is {}",
            local_addr_tcp, peer_address_tcp
            );
        Ok(Box::new(stream))
    }

}

const LSR_CORE_ADDRESS: &str = "localhost:8888";

#[derive(Debug)]
struct LSRCoreService;
impl UsercallExtension for LSRCoreService {
    fn bind_stream(
        &self,
        addr: &str,
        local_addr: Option<&mut String>,
    ) -> IoResult<Option<Box<dyn SyncListener>>> {
        if addr == LSR_CORE_ADDRESS {
            println!("trying to bind {}...", addr);
            let (listener, local_address) = LSRService::new(addr)?;
            println!("local addr: {}", local_address);
            if let Some(local_addr) = local_addr {
                *local_addr = local_address;
            }
            Ok(Some(Box::new(listener)))
        } else {
            Ok(None)
        }
    }

}

fn test_connect() {
    /* lwg: this takes some time for enclave to setup port */
    thread::sleep(std::time::Duration::from_secs(5));
    println!("trying to connect ... {}", line!());
    let mut stream = TcpStream::connect(LSR_CORE_ADDRESS).unwrap();
    println!("trying to connect ... {}", line!());
    stream.write_all("shit".as_bytes()).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
}

fn run_server(file: String) -> Result<(), ()> {
    let mut device = IsgxDevice::new()
        .unwrap()
        .einittoken_provider(AesmClient::new())
        .build();

    println!("lwg:the sgx enclave file is {}", file);

    let mut enclave_builder = EnclaveBuilder::new(file.as_ref());
    enclave_builder.dummy_signature();
    println!("lwg:running enclave...{}", line!());
    enclave_builder.usercall_extension(LSRCoreService);
    println!("lwg:running enclave...{}", line!());
    let enclave = enclave_builder.build(&mut device).unwrap();
    println!("lwg:running enclave...{}", line!());
    enclave.run().map_err(|e| {
        eprintln!("Error in running enclave {}", e);
    })
}

pub fn start_lsr_enclave() {
    let file = parse_args().unwrap();
    let server = thread::spawn(move || run_server(file));
    let test = thread::spawn(move || test_connect());
    let _ = test.join().unwrap();
    let _ = server.join().unwrap();
}




