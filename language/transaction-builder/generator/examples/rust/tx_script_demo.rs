// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_framework::{encode_peer_to_peer_with_metadata_script, ScriptCall};
use diem_types::{AccountAddress, Identifier, StructTag, TypeTag};
use serde_bytes::ByteBuf as Bytes;

fn demo_p2p_script() {
    let token = TypeTag::Struct(StructTag {
        address: AccountAddress([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        module: Identifier("XDX".into()),
        name: Identifier("XDX".into()),
        type_params: Vec::new(),
    });
    let payee = AccountAddress([
        0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
        0x22,
    ]);
    let amount = 1234567;

    // encode and decode a peer to peer transaction script
    let script = encode_peer_to_peer_with_metadata_script(
        token,
        payee.clone(),
        amount,
        Bytes::from(Vec::new()),
        Bytes::from(Vec::new()),
    );

    let call = ScriptCall::decode(&script);
    match call {
        Some(ScriptCall::PeerToPeerWithMetadata {
            amount: a,
            payee: p,
            ..
        }) => {
            assert_eq!(a, amount);
            assert_eq!(p, payee);
        }
        _ => panic!("unexpected type of script"),
    }

    let output = bcs::to_bytes(&script).unwrap();
    for o in output {
        print!("{} ", o);
    }
    println!();
}

fn main() {
    demo_p2p_script();
}
