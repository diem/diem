// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod handlers;

use crate::handlers::get_routes;
use libradb::LibraDB;
use std::{net::SocketAddr, sync::Arc};
use tokio::runtime::{Builder, Runtime};

pub fn start_backup_service(address: SocketAddr, db: Arc<LibraDB>) -> Runtime {
    let backup_handler = db.get_backup_handler();
    let routes = get_routes(backup_handler);

    let runtime = Builder::new()
        .thread_name("backup-")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("[backup] failed to create runtime");

    // Ensure that we actually bind to the socket first before spawning the
    // server tasks. This helps in tests to prevent races where a client attempts
    // to make a request before the server task is actually listening on the
    // socket.
    //
    // Note: we need to enter the runtime context first to actually bind, since
    //       tokio TcpListener can only be bound inside a tokio context.
    let server = runtime.enter(move || warp::serve(routes).bind(address));
    runtime.handle().spawn(server);
    runtime
}

#[cfg(test)]
mod tests {
    use super::*;
    use libra_config::utils::get_available_port;
    use libra_crypto::hash::HashValue;
    use libra_temppath::TempPath;
    use reqwest::blocking::get;
    use std::net::{IpAddr, Ipv4Addr};

    /// 404 - endpoint not found
    /// 400 - params not provided or failed parsing
    /// 500 - endpoint handler raised error
    ///
    /// And failure on one endpoint doesn't result in warp::Rejection which makes it fallback to other matches.
    #[test]
    fn routing_and_error_codes() {
        let tmpdir = TempPath::new();
        let db = Arc::new(LibraDB::new_for_test(&tmpdir));
        let port = get_available_port();
        let _rt = start_backup_service(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port), db);

        // Endpoint doesn't exist.
        let resp = get(&format!("http://127.0.0.1:{}/", port)).unwrap();
        assert_eq!(resp.status(), 404);
        let resp = get(&format!("http://127.0.0.1:{}/x", port)).unwrap();
        assert_eq!(resp.status(), 404);

        // Params not provided.
        let resp = get(&format!("http://127.0.0.1:{}/state_range_proof", port)).unwrap();
        assert_eq!(resp.status(), 400);
        let resp = get(&format!(
            "http://127.0.0.1:{}/state_range_proof/{}",
            port, 123
        ))
        .unwrap();
        assert_eq!(resp.status(), 400);
        let resp = get(&format!("http://127.0.0.1:{}/state_snapshot", port)).unwrap();
        assert_eq!(resp.status(), 400);

        // Params fail to parse (HashValue)
        let resp = get(&format!("http://127.0.0.1:{}/state_range_proof/1/ff", port)).unwrap();
        assert_eq!(resp.status(), 400);

        // Request handler raised Error (non-bootstrapped DB)
        let resp = get(&format!(
            "http://127.0.0.1:{}/state_range_proof/1/{}",
            port,
            HashValue::zero().to_hex()
        ))
        .unwrap();
        assert_eq!(resp.status(), 500);
        let resp = get(&format!("http://127.0.0.1:{}/state_root_proof/0", port,)).unwrap();
        assert_eq!(resp.status(), 500);

        // an endpoint handled by `reply_with_async_channel_writer' always returns 200,
        // connection terminates prematurely when the channel writer errors.
        let resp = get(&format!("http://127.0.0.1:{}/state_snapshot/1", port,)).unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.content_length(), None);
        assert!(resp.bytes().is_err());
    }
}
