// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::{
    traits::Uniform,
    x25519::{PrivateKey, PublicKey},
};
use libra_network_address as address;
use network::protocols::wire::{handshake, messaging};
use rand::{rngs::StdRng, SeedableRng};
use serde_reflection::{Registry, Result, Samples, Tracer, TracerConfig};
use std::str::FromStr;

/// Return a relative path to start tracking changes in commits.
pub fn output_file() -> Option<&'static str> {
    Some("tests/staged/network.yaml")
}

/// Record sample values for crypto types used by network.
fn trace_crypto_values(tracer: &mut Tracer, samples: &mut Samples) -> Result<()> {
    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let private_key = PrivateKey::generate(&mut rng);
    let public_key: PublicKey = (&private_key).into();

    tracer.trace_value(samples, &public_key)?;
    Ok(())
}

/// Create a registry of network data formats.
pub fn get_registry() -> Result<Registry> {
    let mut tracer =
        Tracer::new(TracerConfig::default().is_human_readable(lcs::is_human_readable()));
    let mut samples = Samples::new();
    // 1. Record samples for types with custom deserializers.
    trace_crypto_values(&mut tracer, &mut samples)?;
    tracer.trace_value(
        &mut samples,
        &address::DnsName::from_str("example.com").unwrap(),
    )?;
    tracer.trace_value(&mut samples, &address::NetworkAddress::mock())?;

    // 2. Trace the main entry point(s) + every enum separately.
    tracer.trace_type::<messaging::v1::NetworkMessage>(&samples)?;
    tracer.trace_type::<handshake::v1::HandshakeMsg>(&samples)?;
    tracer.trace_type::<address::NetworkAddress>(&samples)?;
    tracer.trace_type::<address::RawNetworkAddress>(&samples)?;

    tracer.trace_type::<messaging::v1::ErrorCode>(&samples)?;
    tracer.trace_type::<handshake::v1::ProtocolId>(&samples)?;
    tracer.trace_type::<address::Protocol>(&samples)?;

    tracer.registry()
}
