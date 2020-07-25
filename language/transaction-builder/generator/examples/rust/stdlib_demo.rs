// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_canonical_serialization as lcs;
use libra_stdlib::encode_peer_to_peer_with_metadata_script;
use libra_types::{AccountAddress, Identifier, TypeTag, StructTag};

fn main() {
    let token = TypeTag::Struct(StructTag {
        address: AccountAddress([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        module: Identifier("LBR".into()),
        name: Identifier("LBR".into()),
        type_params: Vec::new(),
    });
    let payee = AccountAddress([0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
                                0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22]);
    let amount = 1234567;
    let script =
        encode_peer_to_peer_with_metadata_script(token, payee, amount, Vec::new(), Vec::new());

    let output = lcs::to_bytes(&script).unwrap();
    for o in output {
        print!("{} ", o);
    };
    println!();
}
