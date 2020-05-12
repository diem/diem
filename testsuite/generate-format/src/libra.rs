// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    hash::{CryptoHasher, TestOnlyHasher},
    multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature},
    traits::{SigningKey, Uniform},
};
use libra_types::{contract_event, event, transaction, write_set};
use move_core_types::language_storage;
use rand::{rngs::StdRng, SeedableRng};
use serde_reflection::{Registry, Result, Samples, Tracer, TracerConfig};

/// Default output file.
pub fn output_file() -> Option<&'static str> {
    Some("tests/staged/libra.yaml")
}

/// Record sample values for crypto types used by transactions.
fn trace_crypto_values(tracer: &mut Tracer, samples: &mut Samples) -> Result<()> {
    let mut hasher = TestOnlyHasher::default();
    hasher.update(b"Test message");
    let hashed_message = hasher.finish();

    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let private_key = Ed25519PrivateKey::generate(&mut rng);
    let public_key: Ed25519PublicKey = (&private_key).into();
    let signature = private_key.sign_message(&hashed_message);

    tracer.trace_value(samples, &hashed_message)?;
    tracer.trace_value(samples, &public_key)?;
    tracer.trace_value::<MultiEd25519PublicKey>(samples, &public_key.into())?;
    tracer.trace_value(samples, &signature)?;
    tracer.trace_value::<MultiEd25519Signature>(samples, &signature.into())?;
    Ok(())
}

pub fn get_registry() -> Result<Registry> {
    let mut tracer =
        Tracer::new(TracerConfig::default().is_human_readable(lcs::is_human_readable()));
    let mut samples = Samples::new();
    // 1. Record samples for types with custom deserializers.
    trace_crypto_values(&mut tracer, &mut samples)?;
    tracer.trace_value(&mut samples, &event::EventKey::random())?;

    // 2. Trace the main entry point(s) + every enum separately.
    tracer.trace_type::<contract_event::ContractEvent>(&samples)?;
    tracer.trace_type::<language_storage::TypeTag>(&samples)?;
    tracer.trace_type::<transaction::Transaction>(&samples)?;
    tracer.trace_type::<transaction::TransactionArgument>(&samples)?;
    tracer.trace_type::<transaction::TransactionPayload>(&samples)?;
    tracer.trace_type::<transaction::authenticator::TransactionAuthenticator>(&samples)?;
    tracer.trace_type::<write_set::WriteOp>(&samples)?;
    tracer.registry()
}
