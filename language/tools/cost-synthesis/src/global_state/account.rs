// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::global_state::inhabitor::RandomInhabitor;
use bytecode_verifier::VerifiedModule;
use libra_crypto::ed25519::{compat, Ed25519PrivateKey, Ed25519PublicKey};
use libra_types::{
    access_path::AccessPath, account_address::AccountAddress, account_config, byte_array::ByteArray,
};
use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};
use std::iter::Iterator;
use vm::{
    access::*,
    file_format::{SignatureToken, StructDefinitionIndex, TableIndex},
};
use vm_runtime::identifier::{create_access_path, resource_storage_key};
use vm_runtime_types::value::{Struct, Value};

/// Details about an account.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Account {
    /// The account address
    pub addr: AccountAddress,
    /// The account private key
    pub privkey: Ed25519PrivateKey,
    /// The account public key
    pub pubkey: Ed25519PublicKey,
    /// The set of modules that are published under that account.
    pub modules: Vec<VerifiedModule>,
}

impl Account {
    /// Create a new Account. The account is a logical entity at this point
    pub fn new() -> Self {
        let mut seed_rng = OsRng::new().expect("can't access OsRng");
        let seed_buf: [u8; 32] = seed_rng.gen();
        let mut rng = StdRng::from_seed(seed_buf);
        let (privkey, pubkey) = compat::generate_keypair(&mut rng);
        let addr = AccountAddress::from_public_key(&pubkey);
        Account {
            addr,
            privkey,
            pubkey,
            modules: Vec::new(),
        }
    }

    /// Publish all available resources under the compiled modules that we have added to this
    /// account.
    pub fn generate_resources(
        &self,
        inhabitor: &mut RandomInhabitor,
    ) -> Vec<(AccessPath, Vec<u8>)> {
        let mut ret_vec = Vec::new();
        for mod_ref in self.modules.iter() {
            ret_vec.extend(mod_ref.struct_defs().iter().enumerate().filter_map(
                |(struct_idx, struct_def)| {
                    // Determine if the struct definition is a resource
                    let is_nominal_resource = mod_ref
                        .struct_handle_at(struct_def.struct_handle)
                        .is_nominal_resource;
                    if is_nominal_resource {
                        // Generate the type for the struct
                        let typ = SignatureToken::Struct(struct_def.struct_handle, vec![]);
                        // Generate a value of that type
                        let struct_val = inhabitor.inhabit(&typ);
                        // Now serialize that value into the correct binary blob.
                        let val_blob = struct_val.simple_serialize().unwrap();
                        // Generate the struct tag for the resource so that we can create the
                        // correct access path for it.
                        let struct_tag = resource_storage_key(
                            mod_ref,
                            StructDefinitionIndex::new(struct_idx as TableIndex),
                        );
                        // Create the access path for the resource and associate the binary blob
                        // with that access path.
                        let access_path = create_access_path(&self.addr, struct_tag);
                        Some((access_path, val_blob))
                    } else {
                        None
                    }
                },
            ))
        }
        // Generate default account state.
        let account_access_path =
            create_access_path(&self.addr, account_config::account_struct_tag());
        let account = {
            let coin = Value::struct_(Struct::new(vec![Value::u64(10_000_000)]));
            let account = Value::struct_(Struct::new(vec![
                Value::byte_array(ByteArray::new(
                    AccountAddress::from_public_key(&self.pubkey).to_vec(),
                )),
                coin,
                Value::u64(0),
                Value::u64(0),
                Value::u64(1),
            ]));
            account
                .simple_serialize()
                .expect("Can't create Account resource data")
        };

        ret_vec.push((account_access_path, account));

        ret_vec
    }
}

// This is needed since `PrivateKey` doesn't implement default.
impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}
