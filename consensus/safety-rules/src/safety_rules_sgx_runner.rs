/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate aesm_client;
extern crate enclave_runner;
extern crate sgxs_loaders;

use aesm_client::AesmClient;
use enclave_runner::usercalls::{SyncListener, SyncStream, UsercallExtension};
//use enclave_runner::usercalls::{AsyncListener, UsercallExtension};
use std::io::{Result as IoResult};
use std::thread;
use enclave_runner::EnclaveBuilder;
use sgxs_loaders::isgx::Device as IsgxDevice;


/// This example demonstrates use of usercall extensions for bind call.
/// User call extension allow the enclave code to "bind" to an external service via a customized enclave runner.
/// Here we customize the runner to intercept calls to bind to an address and advance the stream before returning it to enclave
/// This can be useful to strip protocol encapsulations, say while servicing requests load balanced by HA Proxy.
/// This example demonstrates de-encapsulation for various HA proxy configurations before handing over the stream to the enclave.
/// To simulate HA proxy configurations, the runner spawns a thread that connects to the same address which enclave binds to and
/// writes encapsulated test data for various HA proxy configurations to the stream.

fn usage(name: String) {
    println!("Usage:\n{} <path_to_sgxs_file>", name);
}

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
	Ok("dummy.sgxs".into())
}

#[derive(Debug)]
struct LSRService;
impl UsercallExtension for LSRService {
	fn bind_stream(
	&self,
	addr: &str,
	local_addr: Option<&mut String>,
	) -> IoResult<Option<Box<dyn SyncListener>>> {
		Ok(None)
	}
} 

fn run_server(file: String) -> Result<(), ()> {
    let mut device = IsgxDevice::new()
        .unwrap()
        .einittoken_provider(AesmClient::new())
        .build();

    println!("lwg:the sgx enclave file is {}", file);

    let mut enclave_builder = EnclaveBuilder::new(file.as_ref());
    enclave_builder.dummy_signature();
    enclave_builder.usercall_extension(LSRService);
    let enclave = enclave_builder.build(&mut device).unwrap();

    enclave.run().map_err(|e| {
        eprintln!("Error in running enclave {}", e);
    })
}

pub fn start_lsr_enclave() {
    let file = parse_args().unwrap();
    let server = thread::spawn(move || run_server(file));
    let _ = server.join().unwrap();
}
