// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for Protocol negotiation

use crate::negotiate::{
    inbound::negotiate_inbound,
    outbound::{negotiate_outbound_interactive, negotiate_outbound_select},
};
use futures::{executor::block_on, future::join};
use memsocket::MemorySocket;
use std::io::Result;

#[test]
fn interactive_negotiation() -> Result<()> {
    let (a, b) = MemorySocket::new_pair();
    let test_protocol = b"/hello/1.0.0";

    let outbound = async move {
        let (_stream, proto) = negotiate_outbound_interactive(a, [test_protocol]).await?;
        assert_eq!(proto, test_protocol);
        // Force return type of the async block
        let result: Result<()> = Ok(());
        result
    };
    let inbound = async move {
        let vec: Vec<&'static [u8]> = vec![b"some", b"stuff", test_protocol];
        let (_stream, proto) = negotiate_inbound(b, &vec).await?;
        assert_eq!(proto, test_protocol);
        // Force return type of the async block
        let result: Result<()> = Ok(());
        result
    };

    let (result_outbound, result_inbound) = block_on(join(outbound, inbound));
    assert!(result_outbound.is_ok());
    assert!(result_inbound.is_ok());

    Ok(())
}

#[test]
fn optimistic_negotiation() -> Result<()> {
    let (a, b) = MemorySocket::new_pair();
    let test_protocol = b"/hello/1.0.0";

    let outbound = negotiate_outbound_select(a, test_protocol);
    let inbound = async move {
        let vec: Vec<&'static [u8]> = vec![b"some", b"stuff", test_protocol];
        let (_stream, proto) = negotiate_inbound(b, &vec).await?;
        assert_eq!(proto, test_protocol);
        // Force return type of the async block
        let result: Result<()> = Ok(());
        result
    };

    let (result_outbound, result_inbound) = block_on(join(outbound, inbound));
    assert!(result_outbound.is_ok());
    assert!(result_inbound.is_ok());

    Ok(())
}
