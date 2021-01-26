// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_logger::{error, DiemLogger};
use serde::Deserialize;
use std::{
    io::{BufRead, BufReader},
    net::TcpListener,
};

#[derive(Deserialize)]
struct ErrorLog {
    backtrace: Option<String>,
}

#[test]
fn remote_end_to_end() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    DiemLogger::builder().address(addr).is_async(true).build();

    let handle = std::thread::spawn(|| {
        error!("Hello");
        diem_logger::flush();
    });

    let (stream, _) = listener.accept().unwrap();

    let mut stream = BufReader::new(stream);
    let mut buf = Vec::new();
    stream.read_until(b'\n', &mut buf).unwrap();

    let log: ErrorLog = serde_json::from_slice(&buf).unwrap();
    assert!(log.backtrace.is_some());

    // Test that flush properly returns
    handle.join().unwrap();
}
