// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use diem_proptest_helpers::ValueGenerator;
use diem_vault_client::{
    fuzzing::{
        arb_generic_response, arb_policy_list_response, arb_secret_list_response,
        arb_secret_read_response, arb_token_create_response, arb_token_renew_response,
        arb_transit_create_response, arb_transit_export_response, arb_transit_list_response,
        arb_transit_read_response, arb_transit_sign_response, arb_unsealed_response,
    },
    process_generic_response, process_policy_list_response, process_policy_read_response,
    process_secret_list_response, process_secret_read_response, process_token_create_response,
    process_token_renew_response, process_transit_create_response, process_transit_export_response,
    process_transit_list_response, process_transit_read_response, process_transit_restore_response,
    process_transit_sign_response, process_unsealed_response,
};

#[derive(Clone, Debug, Default)]
pub struct VaultGenericResponse;

/// This implementation will fuzz process_generic_response(): the method used by the vault
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

#[derive(Clone, Debug, Default)]
pub struct VaultPolicyReadResponse;

/// This implementation will fuzz process_policy_read_response(): the method used by the vault
/// client to process policies read from the vault backend.
impl FuzzTargetImpl for VaultPolicyReadResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_policy_read_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_generic_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_generic_response());
        let _ = process_policy_read_response(input);
    }
}

#[derive(Clone, Debug, Default)]
pub struct VaultPolicyListResponse;

/// This implementation will fuzz process_policy_list_response(): the method used by the vault
/// client to process policy lists from the vault backend.
impl FuzzTargetImpl for VaultPolicyListResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_policy_list_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_policy_list_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_policy_list_response());
        let _ = process_policy_list_response(input);
    }
}

#[derive(Clone, Debug, Default)]
pub struct VaultSecretListResponse;

/// This implementation will fuzz process_secret_list_response(): the method used by the vault
/// client to process secrets listed from the vault backend.
impl FuzzTargetImpl for VaultSecretListResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_secret_list_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_secret_list_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let response = fuzz_data_to_value(data, arb_secret_list_response());
        let _ = process_secret_list_response(response);
    }
}

#[derive(Clone, Debug, Default)]
pub struct VaultSecretReadResponse;

/// This implementation will fuzz process_secret_read_response(): the method used by the vault
/// client to process secrets read from the vault backend.
impl FuzzTargetImpl for VaultSecretReadResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_secret_read_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_secret_read_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let (response, key, secret) = fuzz_data_to_value(data, arb_secret_read_response());
        let _ = process_secret_read_response(&secret, &key, response);
    }
}

#[derive(Clone, Debug, Default)]
pub struct VaultTokenCreateResponse;

/// This implementation will fuzz process_token_create_response(): the method used by the vault
/// client to process a token create request from the vault backend.
impl FuzzTargetImpl for VaultTokenCreateResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_token_create_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_token_create_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let response = fuzz_data_to_value(data, arb_token_create_response());
        let _ = process_token_create_response(response);
    }
}

#[derive(Clone, Debug, Default)]
pub struct VaultTokenRenewResponse;

/// This implementation will fuzz process_token_renew_response(): the method used by the vault
/// client to process a token renew request from the vault backend.
impl FuzzTargetImpl for VaultTokenRenewResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_token_renew_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_token_renew_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let response = fuzz_data_to_value(data, arb_token_renew_response());
        let _ = process_token_renew_response(response);
    }
}

#[derive(Clone, Debug, Default)]
pub struct VaultTransitCreateResponse;

/// This implementation will fuzz process_transit_create_response(): the method used by the vault
/// client to process a key create request from the vault backend.
impl FuzzTargetImpl for VaultTransitCreateResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_transit_create_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_transit_create_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let (response, name) = fuzz_data_to_value(data, arb_transit_create_response());
        let _ = process_transit_create_response(&name, response);
    }
}

#[derive(Clone, Debug, Default)]
pub struct VaultTransitExportResponse;

/// This implementation will fuzz process_transit_export_response(): the method used by the vault
/// client to process a key export request from the vault backend.
impl FuzzTargetImpl for VaultTransitExportResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_transit_export_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_transit_export_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let (response, name, version) = fuzz_data_to_value(data, arb_transit_export_response());
        let _ = process_transit_export_response(&name, version, response);
    }
}

#[derive(Clone, Debug, Default)]
pub struct VaultTransitListResponse;

/// This implementation will fuzz process_transit_list_response(): the method used by the vault
/// client to process a key list request from the vault backend.
impl FuzzTargetImpl for VaultTransitListResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_transit_list_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_transit_list_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_transit_list_response());
        let _ = process_transit_list_response(input);
    }
}

#[derive(Clone, Debug, Default)]
pub struct VaultTransitReadResponse;

/// This implementation will fuzz process_transit_read_response(): the method used by the vault
/// client to process a key read request from the vault backend.
impl FuzzTargetImpl for VaultTransitReadResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_transit_read_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_transit_read_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let (response, name) = fuzz_data_to_value(data, arb_transit_read_response());
        let _ = process_transit_read_response(&name, response);
    }
}

#[derive(Clone, Debug, Default)]
pub struct VaultTransitRestoreResponse;

/// This implementation will fuzz process_transit_restore_response(): the method used by the vault
/// client to process a key restore request from the vault backend.
impl FuzzTargetImpl for VaultTransitRestoreResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_transit_restore_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_generic_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_generic_response());
        let _ = process_transit_restore_response(input);
    }
}

#[derive(Clone, Debug, Default)]
pub struct VaultTransitSignResponse;

/// This implementation will fuzz process_transit_sign_response(): the method used by the vault
/// client to process a signature request from the vault backend.
impl FuzzTargetImpl for VaultTransitSignResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_transit_sign_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_transit_sign_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_transit_sign_response());
        let _ = process_transit_sign_response(input);
    }
}

#[derive(Clone, Debug, Default)]
pub struct VaultUnsealedResponse;

/// This implementation will fuzz process_unsealed_response(): the method used by the vault
/// client to process an unsealed request from the vault backend.
impl FuzzTargetImpl for VaultUnsealedResponse {
    fn description(&self) -> &'static str {
        "Secure storage vault: process_unsealed_response()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_unsealed_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_unsealed_response());
        let _ = process_unsealed_response(input);
    }
}
