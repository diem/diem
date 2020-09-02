// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::{info, LibraLogger, Writer};
use std::sync::{Arc, RwLock};

#[derive(Default)]
struct VecWriter {
    logs: Arc<RwLock<Vec<String>>>,
}

impl Writer for VecWriter {
    fn write(&self, log: String) {
        self.logs.write().unwrap().push(log)
    }
}

#[test]
fn verify_end_to_end() {
    let writer = VecWriter::default();
    let logs = writer.logs.clone();
    LibraLogger::builder()
        .is_async(false)
        .printer(Box::new(writer))
        .build();

    assert_eq!(logs.read().unwrap().len(), 0);
    info!("Hello");
    assert_eq!(logs.read().unwrap().len(), 1);
    let string = logs.write().unwrap().remove(0);
    assert!(string.contains("INFO"));
    assert!(string.ends_with("Hello"));
}
