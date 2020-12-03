// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    CreateTokenAuth, CreateTokenResponse, ExportKey, ExportKeyResponse, ListKeys, ListKeysResponse,
    ListPoliciesResponse, ReadKey, ReadKeyResponse, ReadKeys, ReadSecretData, ReadSecretListData,
    ReadSecretListResponse, ReadSecretMetadata, ReadSecretResponse, RenewTokenAuth,
    RenewTokenResponse, SealStatusResponse, Signature, SignatureResponse,
};
use diem_types::proptest_types::arb_json_value;
use proptest::prelude::*;
use serde_json::Value;
use ureq::Response;

const MAX_COLLECTION_SIZE: usize = 100;

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
        policies in prop::collection::vec(any::<String>(), 0..MAX_COLLECTION_SIZE)
    ) -> Response {
        let policy_list = ListPoliciesResponse {
            policies,
        };

        let policy_list =
            serde_json::to_string::<ListPoliciesResponse>(&policy_list).unwrap();
        Response::new(status, &status_text, &policy_list)
    }
}

// This generates an arbitrary secret list response returned by vault.
prop_compose! {
    pub fn arb_secret_list_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        keys in prop::collection::vec(any::<String>(), 0..MAX_COLLECTION_SIZE),
    ) -> Response {
        let data = ReadSecretListData {
            keys,
        };
        let read_secret_list_response = ReadSecretListResponse {
            data
        };

        let read_secret_list_response =
            serde_json::to_string::<ReadSecretListResponse>(&read_secret_list_response).unwrap();
        Response::new(status, &status_text, &read_secret_list_response)
    }
}

// This generates an arbitrary secret read response returned by vault, as well as an arbitrary pair
// of input strings for the secret and key.
prop_compose! {
    pub fn arb_secret_read_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        data in prop::collection::btree_map(any::<String>(), arb_json_value(), 0..MAX_COLLECTION_SIZE),
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

// This generates an arbitrary token create response returned by vault.
prop_compose! {
    pub fn arb_token_create_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        client_token in any::<String>(),
    ) -> Response {
    let auth = CreateTokenAuth {
        client_token,
    };
    let create_token_response = CreateTokenResponse {
        auth,
     };

     let create_token_response =
            serde_json::to_string::<CreateTokenResponse>(&create_token_response).unwrap();
     Response::new(status, &status_text, &create_token_response)
    }
}

// This generates an arbitrary token renew response returned by vault.
prop_compose! {
    pub fn arb_token_renew_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        lease_duration in any::<u32>(),
    ) -> Response {
    let auth = RenewTokenAuth {
        lease_duration,
    };
    let renew_token_response = RenewTokenResponse {
        auth,
     };

     let renew_token_response =
            serde_json::to_string::<RenewTokenResponse>(&renew_token_response).unwrap();
     Response::new(status, &status_text, &renew_token_response)
    }
}

// This generates an arbitrary transit create response returned by vault, as well as an arbitrary
// string name.
prop_compose! {
    pub fn arb_transit_create_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        value in arb_json_value(),
        name in any::<String>(),
    ) -> (Response, String) {
        let value =
            serde_json::to_string::<Value>(&value).unwrap();
        let create_key_response = Response::new(status, &status_text, &value);

        (create_key_response, name)
    }
}

// This generates an arbitrary transit export response returned by vault, as well as an arbitrary
// string name and version.
prop_compose! {
    pub fn arb_transit_export_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        keys in prop::collection::btree_map(any::<u32>(), any::<String>(), 0..MAX_COLLECTION_SIZE),
        name in any::<String>(),
        key_name in any::<String>(),
        version in any::<Option<u32>>(),
    ) -> (Response, String, Option<u32>) {
        let data = ExportKey {
            name,
            keys,
        };
        let export_key_response = ExportKeyResponse {
            data,
        };

        let export_key_response =
            serde_json::to_string::<ExportKeyResponse>(&export_key_response).unwrap();
        let export_key_response = Response::new(status, &status_text, &export_key_response);

        (export_key_response, key_name, version)
    }
}

// This generates an arbitrary transit list response returned by vault.
prop_compose! {
    pub fn arb_transit_list_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        keys in prop::collection::vec(any::<String>(), 0..MAX_COLLECTION_SIZE),
    ) -> Response {
        let data = ListKeys {
            keys,
        };
        let list_keys_response = ListKeysResponse {
            data,
        };

        let list_keys_response =
            serde_json::to_string::<ListKeysResponse>(&list_keys_response).unwrap();
        Response::new(status, &status_text, &list_keys_response)
    }
}

// This generates an arbitrary read key struct.
prop_compose! {
    pub fn arb_transit_read_key(
    )(
        creation_time in any::<String>(),
        public_key in any::<String>(),
    ) -> ReadKey {
        ReadKey {
            creation_time,
            public_key
        }
    }
}

// This generates an arbitrary transit read response returned by vault, as well as an arbitrary
// string name.
prop_compose! {
    pub fn arb_transit_read_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        keys in prop::collection::btree_map(any::<u32>(), arb_transit_read_key(), 0..MAX_COLLECTION_SIZE),
        name in any::<String>(),
        key_type in any::<String>(),
        key_name in any::<String>(),
    ) -> (Response, String) {
        let data = ReadKeys {
            keys,
            name,
            key_type,
        };
        let read_key_response = ReadKeyResponse {
            data,
        };

        let read_key_response =
            serde_json::to_string::<ReadKeyResponse>(&read_key_response).unwrap();
        let read_key_response = Response::new(status, &status_text, &read_key_response);

        (read_key_response, key_name)
    }
}

// This generates an arbitrary transit sign response returned by vault.
prop_compose! {
    pub fn arb_transit_sign_response(
    )(
        status in any::<u16>(),
        status_text in any::<String>(),
        signature in any::<String>(),
    ) -> Response {
        let data = Signature {
            signature,
        };
        let signature_response = SignatureResponse {
            data,
        };

        let signature_response =
            serde_json::to_string::<SignatureResponse>(&signature_response).unwrap();
        Response::new(status, &status_text, &signature_response)
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
            arb_generic_response, arb_policy_list_response, arb_secret_list_response,
            arb_secret_read_response, arb_token_create_response, arb_token_renew_response,
            arb_transit_create_response, arb_transit_export_response, arb_transit_list_response,
            arb_transit_read_response, arb_transit_sign_response, arb_unsealed_response,
        },
        process_generic_response, process_policy_list_response, process_policy_read_response,
        process_secret_list_response, process_secret_read_response, process_token_create_response,
        process_token_renew_response, process_transit_create_response,
        process_transit_export_response, process_transit_list_response,
        process_transit_read_response, process_transit_restore_response,
        process_transit_sign_response, process_unsealed_response,
    };
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn process_generic_response_proptest(response in arb_generic_response()) {
            let _ = process_generic_response(response);
        }

        #[test]
        fn process_policy_read_response_proptest(response in arb_generic_response()) {
            let _ = process_policy_read_response(response);
        }

        #[test]
        fn process_policy_list_response_proptest(response in arb_policy_list_response()) {
            let _ = process_policy_list_response(response);
        }

        #[test]
        fn process_secret_list_response_proptest(response in arb_secret_list_response()) {
            let _ = process_secret_list_response(response);
        }

        #[test]
        fn process_secret_read_response_proptest((response, secret, key) in arb_secret_read_response()) {
            let _ = process_secret_read_response(&secret, &key, response);
        }

        #[test]
        fn process_token_create_response_proptest(response in arb_token_create_response()) {
            let _ = process_token_create_response(response);
        }

        #[test]
        fn process_token_renew_response_proptest(response in arb_token_renew_response()) {
            let _ = process_token_renew_response(response);
        }

        #[test]
        fn process_transit_create_response_proptest((response, name) in arb_transit_create_response()) {
            let _ = process_transit_create_response(&name, response);
        }

        #[test]
        fn process_transit_export_response_proptest((response, name, version) in arb_transit_export_response()) {
            let _ = process_transit_export_response(&name, version, response);
        }

        #[test]
        fn process_transit_list_response_proptest(response in arb_transit_list_response()) {
            let _ = process_transit_list_response(response);
        }

        #[test]
        fn process_transit_read_response_proptest((response, name) in arb_transit_read_response()) {
            let _ = process_transit_read_response(&name, response);
        }

        #[test]
        fn process_transit_restore_response_proptest(response in arb_generic_response()) {
            let _ = process_transit_restore_response(response);
        }

        #[test]
        fn process_transit_sign_response_proptest(response in arb_transit_sign_response()) {
            let _ = process_transit_sign_response(response);
        }

        #[test]
        fn process_unsealed_response_proptest(response in arb_unsealed_response()) {
            let _ = process_unsealed_response(response);
        }
    }
}
