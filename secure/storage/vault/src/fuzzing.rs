// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ListPoliciesResponse, ReadSecretData, ReadSecretMetadata, ReadSecretResponse,
    SealStatusResponse,
};
use libra_types::proptest_types::arb_json_value;
use proptest::prelude::*;
use serde_json::Value;
use ureq::Response;

// This generates an arbitrary generic response returned by vault for various API calls.
prop_compose! {
    pub fn arb_generic_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        value in arb_json_value(),
    ) -> Response {
        let value =
            serde_json::to_string::<Value>(&value).unwrap();
        Response::new(status, &status_text, &value)
    }
}

// This generates an arbitrary policy list response returned by vault.
prop_compose! {
    pub fn arb_policy_list_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        policies in prop::collection::vec(any::<String>(), 1..100)
    ) -> Response {
        let policy_list = ListPoliciesResponse {
            policies,
        };
        let policy_list =
            serde_json::to_string::<ListPoliciesResponse>(&policy_list).unwrap();
        Response::new(status, &status_text, &policy_list)
    }
}

// This generates an arbitrary secret read response returned by vault, as well as an arbitrary pair
// of input strings for the secret and key.
prop_compose! {
    pub fn arb_secret_read_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        data in prop::collection::btree_map(any::<String>(), any::<String>(), 1..100),
        created_time in any::<String>(),
        version in any::<u32>(),
        secret in any::<String>(),
        key in any::<String>(),
    ) -> (Response, String, String) {
        let metadata = ReadSecretMetadata {
            created_time,
            version,
        };
        let data = ReadSecretData {
            data,
            metadata,
        };
        let read_secret_response = ReadSecretResponse {
            data
        };

        let read_secret_response =
            serde_json::to_string::<ReadSecretResponse>(&read_secret_response).unwrap();
        let read_secret_response = Response::new(status, &status_text, &read_secret_response);

        (read_secret_response, secret, key)
    }
}

// This generates an arbitrary unsealed response returned by vault.
prop_compose! {
    pub fn arb_unsealed_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        sealed in any::<bool>(),
    ) -> Response {
        let sealed_status_response = SealStatusResponse {
            sealed,
        };
        let sealed_status_response =
            serde_json::to_string::<SealStatusResponse>(&sealed_status_response).unwrap();
        Response::new(status, &status_text, &sealed_status_response)
    }
}

// Note: these tests ensure that the various fuzzers are maintained (i.e., not broken
// at some time in the future and only discovered when a fuzz test fails).
#[cfg(test)]
mod tests {
    use crate::{
        fuzzing::{
            arb_generic_response, arb_policy_list_response, arb_secret_read_response,
            arb_unsealed_response,
        },
        process_generic_response, process_policy_list_response, process_secret_read_response,
        process_unsealed_response,
    };
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn process_generic_response_proptest(input in arb_generic_response()) {
            let _ = process_generic_response(input);
        }

        #[test]
        fn process_policy_list_response_proptest(input in arb_policy_list_response()) {
            let _ = process_policy_list_response(input);
        }

        #[test]
        fn process_secret_read_response_proptest((response, secret, key) in arb_secret_read_response()) {
            let _ = process_secret_read_response(&secret, &key, response);
        }

        #[test]
        fn process_unsealed_response_proptest(input in arb_unsealed_response()) {
            let _ = process_unsealed_response(input);
        }
    }
}
