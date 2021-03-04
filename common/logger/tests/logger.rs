// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_infallible::RwLock;
use diem_logger::{info, DiemLogger, Writer};
use std::sync::Arc;

#[derive(Default)]
struct VecWriter {
    logs: Arc<RwLock<Vec<String>>>,
}

impl Writer for VecWriter {
    fn write(&self, log: String) {
        self.logs.write().push(log)
    }
}

#[test]
fn verify_end_to_end() {
    let writer = VecWriter::default();
    let logs = writer.logs.clone();
    DiemLogger::builder()
        .is_async(false)
        .printer(Box::new(writer))
        .build();

    assert_eq!(logs.read().len(), 0);
    info!("Hello");
    assert_eq!(logs.read().len(), 1);
    let string = logs.write().remove(0);
    assert!(string.contains("INFO"));
    assert!(string.ends_with("Hello"));
    info!(foo = 5, bar = 10, foobar = 15);
    let string = logs.write().remove(0);
    let expect = r#"{"bar":10,"foo":5,"foobar":15}"#;
    assert!(string.ends_with(expect));
}
