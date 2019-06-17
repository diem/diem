// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{fs::File, io::prelude::*};
use vm_genesis::{default_config, encode_genesis_transaction, GENESIS_KEYPAIR};

const CONFIG_LOCATION: &str = "genesis/vm_config.toml";
const GENESIS_LOCATION: &str = "genesis/genesis.blob";

use proto_conv::IntoProtoBytes;

fn main() {
    println!(
        "Creating genesis binary blob at {} from configuration file {}",
        GENESIS_LOCATION, CONFIG_LOCATION
    );
    let config = default_config();
    config.save_config(CONFIG_LOCATION);

    // Generate a genesis blob used for vm tests.
    let genesis_txn = encode_genesis_transaction(&GENESIS_KEYPAIR.0, GENESIS_KEYPAIR.1);
    let mut file = File::create(GENESIS_LOCATION).unwrap();
    file.write_all(&genesis_txn.into_proto_bytes().unwrap())
        .unwrap();
}
