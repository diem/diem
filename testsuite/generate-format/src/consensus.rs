// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    hash::{CryptoHasher, TestOnlyHasher},
    traits::{SigningKey, Uniform},
};
use rand::{rngs::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use serde_reflection::{Registry, Result, Samples, Tracer, TracerConfig};

/// Return a relative path to start tracking changes in commits.
pub fn output_file() -> Option<&'static str> {
    Some("tests/staged/consensus.yaml")
}

/// Record sample values for crypto types used by consensus.
fn trace_crypto_values(tracer: &mut Tracer, samples: &mut Samples) -> Result<()> {
    let mut hasher = TestOnlyHasher::default();
    hasher.write(b"Test message");
    let hashed_message = hasher.finish();

    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let private_key = Ed25519PrivateKey::generate(&mut rng);
    let public_key: Ed25519PublicKey = (&private_key).into();
    let signature = private_key.sign_message(&hashed_message);

    tracer.trace_value(samples, &public_key)?;
    tracer.trace_value(samples, &signature)?;
    Ok(())
}

/// Placeholder type.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Default, Clone)]
struct Payload;

/// Create a registry for consensus types.
pub fn get_registry() -> Result<Registry> {
    let mut tracer =
        Tracer::new(TracerConfig::default().is_human_readable(lcs::is_human_readable()));
    let mut samples = Samples::new();
    // 1. Record samples for types with custom deserializers.
    trace_crypto_values(&mut tracer, &mut samples)?;
    tracer.trace_value(
        &mut samples,
        &consensus_types::block::Block::<Payload>::make_genesis_block(),
    )?;

    // 2. Trace the main entry point(s) + every enum separately.
    tracer.trace_type::<consensus::network_interface::ConsensusMsg<Payload>>(&samples)?;
    tracer.trace_type::<consensus_types::block_data::BlockType<Payload>>(&samples)?;
    tracer.trace_type::<consensus_types::block_retrieval::BlockRetrievalStatus>(&samples)?;

    tracer.registry()
}
