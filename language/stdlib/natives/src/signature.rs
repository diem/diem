// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::dispatch::{CostedReturnType, NativeReturnType, Result, StackAccessor};
use nextgen_crypto::{ed25519, traits::*};
use std::convert::TryFrom;

// TODO: Talk to Crypto to determine these costs
const ED25519_COST: u64 = 35;

pub fn native_ed25519_signature_verification<T: StackAccessor>(
    mut accessor: T,
) -> Result<CostedReturnType> {
    let signature = accessor.get_byte_array()?;
    let pubkey = accessor.get_byte_array()?;
    let msg = accessor.get_byte_array()?;

    let native_cost = ED25519_COST * msg.len() as u64;

    let sig = ed25519::Ed25519Signature::try_from(signature.as_bytes())?;
    let pk = ed25519::Ed25519PublicKey::try_from(pubkey.as_bytes())?;

    match sig.verify_arbitrary_msg(msg.as_bytes(), &pk) {
        Ok(()) => Ok(CostedReturnType::new(
            native_cost,
            NativeReturnType::Bool(true),
        )),
        Err(_) => Ok(CostedReturnType::new(
            native_cost,
            NativeReturnType::Bool(false),
        )),
    }
}
