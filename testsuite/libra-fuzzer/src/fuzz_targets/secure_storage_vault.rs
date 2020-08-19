// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use libra_proptest_helpers::ValueGenerator;
use libra_vault_client::{fuzzing::arb_generic_response, process_generic_response};

#[derive(Clone, Debug, Default)]
pub struct VaultGenericResponse;

/// This implementation will fuzz the process_generic_response(): the method used by the vault
/// client to process generic responses from the vault backend.
impl FuzzTargetImpl for VaultGenericResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_generic_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_generic_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_generic_response());
        let _ = process_generic_response(input);
    }
}
