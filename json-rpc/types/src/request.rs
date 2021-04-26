// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{Id, JsonRpcVersion, Method};
use crate::{errors::JsonRpcError, views::BytesView};
use diem_types::{
    account_address::AccountAddress, event::EventKey, transaction::SignedTransaction,
};
use serde::{de, Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    jsonrpc: JsonRpcVersion,
    #[serde(flatten)]
    pub method_request: MethodRequest,
    pub id: Id,
}

impl JsonRpcRequest {
    pub fn from_value(
        value: serde_json::Value,
    ) -> Result<Self, (JsonRpcError, Option<Method>, Option<Id>)> {
        #[derive(Debug, Deserialize, Serialize)]
        struct RawJsonRpcRequest {
            #[serde(default)]
            jsonrpc: serde_json::Value,
            #[serde(default)]
            method: serde_json::Value,
            #[serde(default)]
            params: serde_json::Value,
            id: Id,
        }

        let RawJsonRpcRequest {
            jsonrpc,
            method,
            params,
            id,
        } = serde_json::from_value(value)
            .map_err(|_| (JsonRpcError::invalid_format(), None, None))?;
        let jsonrpc: JsonRpcVersion = serde_json::from_value(jsonrpc)
            .map_err(|_| (JsonRpcError::invalid_request(), None, Some(id.clone())))?;
        let method: Method = serde_json::from_value(method)
            .map_err(|_| (JsonRpcError::method_not_found(), None, Some(id.clone())))?;
        let method_request = MethodRequest::from_value(method, params).map_err(|_| {
            (
                JsonRpcError::invalid_params_from_method(method),
                Some(method),
                Some(id.clone()),
            )
        })?;

        Ok(JsonRpcRequest {
            jsonrpc,
            method_request,
            id,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
pub enum MethodRequest {
    Submit(SubmitParams),
    GetMetadata(GetMetadataParams),
    GetAccount(GetAccountParams),
    GetTransactions(GetTransactionsParams),
    GetAccountTransaction(GetAccountTransactionParams),
    GetAccountTransactions(GetAccountTransactionsParams),
    GetEvents(GetEventsParams),
    GetCurrencies(GetCurrenciesParams),
    GetNetworkStatus(GetNetworkStatusParams),

    //
    // Experimental APIs
    //
    GetStateProof(GetStateProofParams),
    GetAccountStateWithProof(GetAccountStateWithProofParams),
    GetTransactionsWithProofs(GetTransactionsWithProofsParams),
    GetEventsWithProofs(GetEventsWithProofsParams),
}

impl MethodRequest {
    pub fn from_value(method: Method, value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let method_request = match method {
            Method::Submit => MethodRequest::Submit(serde_json::from_value(value)?),
            Method::GetMetadata => MethodRequest::GetMetadata(serde_json::from_value(value)?),
            Method::GetAccount => MethodRequest::GetAccount(serde_json::from_value(value)?),
            Method::GetTransactions => {
                MethodRequest::GetTransactions(serde_json::from_value(value)?)
            }
            Method::GetAccountTransaction => {
                MethodRequest::GetAccountTransaction(serde_json::from_value(value)?)
            }
            Method::GetAccountTransactions => {
                MethodRequest::GetAccountTransactions(serde_json::from_value(value)?)
            }
            Method::GetEvents => MethodRequest::GetEvents(serde_json::from_value(value)?),
            Method::GetCurrencies => MethodRequest::GetCurrencies(serde_json::from_value(value)?),
            Method::GetNetworkStatus => {
                MethodRequest::GetNetworkStatus(serde_json::from_value(value)?)
            }
            Method::GetStateProof => MethodRequest::GetStateProof(serde_json::from_value(value)?),
            Method::GetAccountStateWithProof => {
                MethodRequest::GetAccountStateWithProof(serde_json::from_value(value)?)
            }
            Method::GetTransactionsWithProofs => {
                MethodRequest::GetTransactionsWithProofs(serde_json::from_value(value)?)
            }
            Method::GetEventsWithProofs => {
                MethodRequest::GetEventsWithProofs(serde_json::from_value(value)?)
            }
        };

        Ok(method_request)
    }

    pub fn method(&self) -> Method {
        match self {
            MethodRequest::Submit(_) => Method::Submit,
            MethodRequest::GetMetadata(_) => Method::GetMetadata,
            MethodRequest::GetAccount(_) => Method::GetAccount,
            MethodRequest::GetTransactions(_) => Method::GetTransactions,
            MethodRequest::GetAccountTransaction(_) => Method::GetAccountTransaction,
            MethodRequest::GetAccountTransactions(_) => Method::GetAccountTransactions,
            MethodRequest::GetEvents(_) => Method::GetEvents,
            MethodRequest::GetCurrencies(_) => Method::GetCurrencies,
            MethodRequest::GetNetworkStatus(_) => Method::GetNetworkStatus,
            MethodRequest::GetStateProof(_) => Method::GetStateProof,
            MethodRequest::GetAccountStateWithProof(_) => Method::GetAccountStateWithProof,
            MethodRequest::GetTransactionsWithProofs(_) => Method::GetTransactionsWithProofs,
            MethodRequest::GetEventsWithProofs(_) => Method::GetEventsWithProofs,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubmitParams {
    #[serde(serialize_with = "serialize_signed_transaction")]
    #[serde(deserialize_with = "deserialize_signed_transaction")]
    pub data: SignedTransaction,
}

fn serialize_signed_transaction<S>(
    txn: &SignedTransaction,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::Error;
    BytesView::new(bcs::to_bytes(txn).map_err(S::Error::custom)?).serialize(serializer)
}

fn deserialize_signed_transaction<'de, D>(deserializer: D) -> Result<SignedTransaction, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let bytes = BytesView::deserialize(deserializer)
        .map_err(|_| D::Error::custom("expected hex-encoded SignedTransaction"))?;
    bcs::from_bytes(bytes.inner())
        .map_err(|_| D::Error::custom("expected hex-encoded SignedTransaction"))
}

#[derive(Clone, Debug, Serialize)]
pub struct GetMetadataParams {
    #[serde(default)]
    pub version: Option<u64>,
}

impl<'de> Deserialize<'de> for GetMetadataParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Params {
            #[serde(default)]
            pub version: Option<u64>,
        }

        let params = match Option::<Params>::deserialize(deserializer)? {
            Some(params) => GetMetadataParams {
                version: params.version,
            },
            None => GetMetadataParams { version: None },
        };
        Ok(params)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetAccountParams {
    pub account: AccountAddress,
    #[serde(default)]
    pub version: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTransactionsParams {
    pub start_version: u64,
    pub limit: u64,
    pub include_events: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetAccountTransactionParams {
    pub account: AccountAddress,
    pub sequence_number: u64,
    pub include_events: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetAccountTransactionsParams {
    pub account: AccountAddress,
    pub start: u64,
    pub limit: u64,
    pub include_events: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEventsParams {
    pub key: EventKey,
    pub start: u64,
    pub limit: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct GetCurrenciesParams;

impl<'de> Deserialize<'de> for GetCurrenciesParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_option(NoParamsVisitor("get_currencies params"))
            .map(|_| GetCurrenciesParams)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct GetNetworkStatusParams;

impl<'de> Deserialize<'de> for GetNetworkStatusParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_option(NoParamsVisitor("get_network_status params"))
            .map(|_| GetNetworkStatusParams)
    }
}

/// A de::Visitor implementation for jsonrpc param structs without any parameters
struct NoParamsVisitor(&'static str);
impl<'de> de::Visitor<'de> for NoParamsVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }

    fn visit_seq<A>(self, _seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        Ok(())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(())
    }

    fn visit_map<A>(self, _map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetStateProofParams {
    pub version: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetAccountStateWithProofParams {
    pub account: AccountAddress,
    #[serde(default)]
    pub version: Option<u64>,
    #[serde(default)]
    pub ledger_version: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTransactionsWithProofsParams {
    pub start_version: u64,
    pub limit: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEventsWithProofsParams {
    pub key: EventKey,
    pub start: u64,
    pub limit: u64,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn metadata() {
        // Too many array params
        let value = serde_json::json!([10, 11]);
        serde_json::from_value::<GetMetadataParams>(value).unwrap_err();

        // Correct number of array params
        let value = serde_json::json!([10]);
        serde_json::from_value::<GetMetadataParams>(value).unwrap();

        // Correct number of array params but version is Null
        let value = serde_json::json!([serde_json::Value::Null]);
        serde_json::from_value::<GetMetadataParams>(value).unwrap();

        // Empty array still correctly deserializes since the only param is optional
        let value = serde_json::json!([]);
        serde_json::from_value::<GetMetadataParams>(value).unwrap();

        // Empty object still correctly deserializes since all params are optional
        let value = serde_json::json!({});
        serde_json::from_value::<GetMetadataParams>(value).unwrap();

        // Even if there's no object it still deserializes correctly since all params are optional
        let value = serde_json::Value::Null;
        serde_json::from_value::<GetMetadataParams>(value).unwrap();

        // Named params
        let value = serde_json::json!({"version": 10});
        serde_json::from_value::<GetMetadataParams>(value).unwrap();

        // JsonRpcRequest with no params
        let value = serde_json::json! {{
            "jsonrpc": "2.0",
            "method": Method::GetMetadata,
            "id": 1,
        }};
        serde_json::from_value::<JsonRpcRequest>(value).unwrap();
    }

    #[test]
    fn get_account() {
        let account = "1668f6be25668c1a17cd8caf6b8d2f25";

        // Array without optional param
        let value = serde_json::json!([account]);
        serde_json::from_value::<GetAccountParams>(value).unwrap();

        // Array with optional param
        let value = serde_json::json!([account, 10]);
        serde_json::from_value::<GetAccountParams>(value).unwrap();

        // Array with wrong optional param
        let value = serde_json::json!([account, "foo"]);
        serde_json::from_value::<GetAccountParams>(value).unwrap_err();

        // Array with too many params
        let value = serde_json::json!([account, 10, 1]);
        serde_json::from_value::<GetAccountParams>(value).unwrap_err();

        // Empty array without required params should fail
        let value = serde_json::json!([]);
        serde_json::from_value::<GetAccountParams>(value).unwrap_err();

        // Object without required params should fail
        let value = serde_json::json!({});
        serde_json::from_value::<GetAccountParams>(value).unwrap_err();

        // Object without optional param
        let value = serde_json::json!({
            "account": account,
        });
        serde_json::from_value::<GetAccountParams>(value).unwrap();

        // Object with optional param
        let value = serde_json::json!({
            "account": account,
            "version": 10,
        });
        serde_json::from_value::<GetAccountParams>(value).unwrap();

        // Object with more params
        let value = serde_json::json!({
            "account": account,
            "version": 10,
            "foo": 11,
        });
        serde_json::from_value::<GetAccountParams>(value).unwrap();
    }

    #[test]
    fn get_transactions() {
        // Array with all params
        let value = serde_json::json!([10, 11, false]);
        serde_json::from_value::<GetTransactionsParams>(value).unwrap();

        // Array with too many params
        let value = serde_json::json!([10, 11, false, "foo"]);
        serde_json::from_value::<GetTransactionsParams>(value).unwrap_err();

        // Array with wrong param
        let value = serde_json::json!(["foo", 11, false]);
        serde_json::from_value::<GetTransactionsParams>(value).unwrap_err();

        // Array with too few params
        let value = serde_json::json!([10, 11]);
        serde_json::from_value::<GetTransactionsParams>(value).unwrap_err();

        // Empty array without required params should fail
        let value = serde_json::json!([]);
        serde_json::from_value::<GetTransactionsParams>(value).unwrap_err();

        // Object without required params should fail
        let value = serde_json::json!({});
        serde_json::from_value::<GetTransactionsParams>(value).unwrap_err();

        // Object params
        let value = serde_json::json!({
            "start_version": 10,
            "limit": 10,
            "include_events": true,
        });
        serde_json::from_value::<GetTransactionsParams>(value).unwrap();

        // Object without all params
        let value = serde_json::json!({
            "limit": 10,
            "include_events": true,
        });
        serde_json::from_value::<GetTransactionsParams>(value).unwrap_err();

        // Object with more params
        let value = serde_json::json!({
            "start_version": 10,
            "limit": 10,
            "include_events": true,
            "foo": 11,
        });
        serde_json::from_value::<GetTransactionsParams>(value).unwrap();
    }

    #[test]
    fn get_account_transaction() {
        let account = "1668f6be25668c1a17cd8caf6b8d2f25";

        // Array with all params
        let value = serde_json::json!([account, 11, false]);
        serde_json::from_value::<GetAccountTransactionParams>(value).unwrap();

        // Array with too many params
        let value = serde_json::json!([account, 11, false, "foo"]);
        serde_json::from_value::<GetAccountTransactionParams>(value).unwrap_err();

        // Array with wrong param
        let value = serde_json::json!(["foo", 11, false]);
        serde_json::from_value::<GetAccountTransactionParams>(value).unwrap_err();

        // Array with too few params
        let value = serde_json::json!([10, 11]);
        serde_json::from_value::<GetAccountTransactionParams>(value).unwrap_err();

        // Empty array without required params should fail
        let value = serde_json::json!([]);
        serde_json::from_value::<GetAccountTransactionParams>(value).unwrap_err();

        // Object without required params should fail
        let value = serde_json::json!({});
        serde_json::from_value::<GetAccountTransactionParams>(value).unwrap_err();

        // Object params
        let value = serde_json::json!({
            "account": account,
            "sequence_number": 10,
            "include_events": true,
        });
        serde_json::from_value::<GetAccountTransactionParams>(value).unwrap();

        // Object without all params
        let value = serde_json::json!({
            "sequence_number": 10,
            "include_events": true,
        });
        serde_json::from_value::<GetAccountTransactionParams>(value).unwrap_err();

        // Object with more params
        let value = serde_json::json!({
            "account": account,
            "sequence_number": 10,
            "include_events": true,
            "foo": 11,
        });
        serde_json::from_value::<GetAccountTransactionParams>(value).unwrap();
    }

    #[test]
    fn get_account_transactions() {
        let account = "1668f6be25668c1a17cd8caf6b8d2f25";

        // Array with all params
        let value = serde_json::json!([account, 10, 11, false]);
        serde_json::from_value::<GetAccountTransactionsParams>(value).unwrap();

        // Array with too many params
        let value = serde_json::json!([account, 10, 11, false, "foo"]);
        serde_json::from_value::<GetAccountTransactionsParams>(value).unwrap_err();

        // Array with wrong param
        let value = serde_json::json!(["foo", 10, 11, false]);
        serde_json::from_value::<GetAccountTransactionsParams>(value).unwrap_err();

        // Array with too few params
        let value = serde_json::json!([10, 11]);
        serde_json::from_value::<GetAccountTransactionsParams>(value).unwrap_err();

        // Empty array without required params should fail
        let value = serde_json::json!([]);
        serde_json::from_value::<GetAccountTransactionsParams>(value).unwrap_err();

        // Object without required params should fail
        let value = serde_json::json!({});
        serde_json::from_value::<GetAccountTransactionsParams>(value).unwrap_err();

        // Object params
        let value = serde_json::json!({
            "account": account,
            "start": 10,
            "limit": 11,
            "include_events": true,
        });
        serde_json::from_value::<GetAccountTransactionsParams>(value).unwrap();

        // Object without all params
        let value = serde_json::json!({
            "include_events": true,
        });
        serde_json::from_value::<GetAccountTransactionsParams>(value).unwrap_err();

        // Object with more params
        let value = serde_json::json!({
            "account": account,
            "start": 10,
            "limit": 11,
            "include_events": true,
            "foo": 11,
        });
        serde_json::from_value::<GetAccountTransactionsParams>(value).unwrap();
    }

    #[test]
    fn get_events() {
        let key = "13000000000000000000000000000000000000000a550c18";

        // Array with all params
        let value = serde_json::json!([key, 10, 11]);
        serde_json::from_value::<GetEventsParams>(value).unwrap();

        // Array with too many params
        let value = serde_json::json!([key, 10, 11, false]);
        serde_json::from_value::<GetEventsParams>(value).unwrap_err();

        // Array with wrong param
        let value = serde_json::json!(["foo", 10, 11]);
        serde_json::from_value::<GetEventsParams>(value).unwrap_err();

        // Array with too few params
        let value = serde_json::json!([10, 11]);
        serde_json::from_value::<GetEventsParams>(value).unwrap_err();

        // Empty array without required params should fail
        let value = serde_json::json!([]);
        serde_json::from_value::<GetEventsParams>(value).unwrap_err();

        // Object without required params should fail
        let value = serde_json::json!({});
        serde_json::from_value::<GetEventsParams>(value).unwrap_err();

        // Object params
        let value = serde_json::json!({
            "key": key,
            "start": 10,
            "limit": 11,
        });
        serde_json::from_value::<GetEventsParams>(value).unwrap();

        // Object without all params
        let value = serde_json::json!({
            "start": 10,
            "limit": 11,
        });
        serde_json::from_value::<GetEventsParams>(value).unwrap_err();

        // Object with more params
        let value = serde_json::json!({
            "key": key,
            "start": 10,
            "limit": 11,
            "foo": 11,
        });
        serde_json::from_value::<GetEventsParams>(value).unwrap();
    }

    #[test]
    fn get_currencies() {
        let value = serde_json::json!([10]);
        serde_json::from_value::<GetCurrenciesParams>(value).unwrap_err();

        let value = serde_json::json!([]);
        serde_json::from_value::<GetCurrenciesParams>(value).unwrap();

        let value = serde_json::json!({});
        serde_json::from_value::<GetCurrenciesParams>(value).unwrap();

        let value = serde_json::Value::Null;
        serde_json::from_value::<GetCurrenciesParams>(value).unwrap();

        let value = serde_json::json! {{
            "jsonrpc": "2.0",
            "method": Method::GetCurrencies,
            "id": 1,
        }};
        serde_json::from_value::<JsonRpcRequest>(value).unwrap();
    }

    #[test]
    fn get_network_status() {
        let value = serde_json::json!([10]);
        serde_json::from_value::<GetNetworkStatusParams>(value).unwrap_err();

        let value = serde_json::json!([]);
        serde_json::from_value::<GetNetworkStatusParams>(value).unwrap();

        let value = serde_json::json!({});
        serde_json::from_value::<GetNetworkStatusParams>(value).unwrap();

        let value = serde_json::Value::Null;
        serde_json::from_value::<GetNetworkStatusParams>(value).unwrap();

        let value = serde_json::json! {{
            "jsonrpc": "2.0",
            "method": Method::GetNetworkStatus,
            "id": 1,
        }};
        serde_json::from_value::<JsonRpcRequest>(value).unwrap();
    }

    #[test]
    fn get_state_proof() {
        // Array with all params
        let value = serde_json::json!([11]);
        serde_json::from_value::<GetStateProofParams>(value).unwrap();

        // Array with too many params
        let value = serde_json::json!([11, false]);
        serde_json::from_value::<GetStateProofParams>(value).unwrap_err();

        // Array with wrong param
        let value = serde_json::json!(["foo"]);
        serde_json::from_value::<GetStateProofParams>(value).unwrap_err();

        // Empty array without required params should fail
        let value = serde_json::json!([]);
        serde_json::from_value::<GetStateProofParams>(value).unwrap_err();

        // Object without required params should fail
        let value = serde_json::json!({});
        serde_json::from_value::<GetStateProofParams>(value).unwrap_err();

        // Object params
        let value = serde_json::json!({
            "version": 10,
        });
        serde_json::from_value::<GetStateProofParams>(value).unwrap();

        // Object with more params
        let value = serde_json::json!({
            "version": 10,
            "foo": 11,
        });
        serde_json::from_value::<GetStateProofParams>(value).unwrap();
    }

    #[test]
    fn get_account_state_with_proof() {
        let account = "1668f6be25668c1a17cd8caf6b8d2f25";

        // Array with all params
        let value = serde_json::json!([account, 11, 12]);
        serde_json::from_value::<GetAccountStateWithProofParams>(value).unwrap();

        // Array without optional params
        let value = serde_json::json!([account]);
        serde_json::from_value::<GetAccountStateWithProofParams>(value).unwrap();

        let value = serde_json::json!([account, 12]);
        serde_json::from_value::<GetAccountStateWithProofParams>(value).unwrap();

        // Array with too many params
        let value = serde_json::json!([account, 11, 12, "foo"]);
        serde_json::from_value::<GetAccountStateWithProofParams>(value).unwrap_err();

        // Array with wrong param
        let value = serde_json::json!(["foo", 11, 12]);
        serde_json::from_value::<GetAccountStateWithProofParams>(value).unwrap_err();

        // Array with too few params
        let value = serde_json::json!([10, 11]);
        serde_json::from_value::<GetAccountStateWithProofParams>(value).unwrap_err();

        // Empty array without required params should fail
        let value = serde_json::json!([]);
        serde_json::from_value::<GetAccountStateWithProofParams>(value).unwrap_err();

        // Object without required params should fail
        let value = serde_json::json!({});
        serde_json::from_value::<GetAccountStateWithProofParams>(value).unwrap_err();

        // Object params
        let value = serde_json::json!({
            "account": account,
            "version": 10,
            "ledger_version": 10,
        });
        serde_json::from_value::<GetAccountStateWithProofParams>(value).unwrap();

        // Object without all params
        let value = serde_json::json!({
            "account": account,
            "ledger_version": 10,
        });
        serde_json::from_value::<GetAccountStateWithProofParams>(value).unwrap();

        // Object with more params
        let value = serde_json::json!({
            "account": account,
            "version": 10,
            "ledger_version": 10,
            "foo": 11,
        });
        serde_json::from_value::<GetAccountStateWithProofParams>(value).unwrap();
    }

    #[test]
    fn get_transactions_with_proofs() {
        // Array with all params
        let value = serde_json::json!([10, 11]);
        serde_json::from_value::<GetTransactionsWithProofsParams>(value).unwrap();

        // Array with too many params
        let value = serde_json::json!([10, 11, false]);
        serde_json::from_value::<GetTransactionsWithProofsParams>(value).unwrap_err();

        // Array with wrong param
        let value = serde_json::json!(["foo", 11]);
        serde_json::from_value::<GetTransactionsWithProofsParams>(value).unwrap_err();

        // Empty array without required params should fail
        let value = serde_json::json!([]);
        serde_json::from_value::<GetTransactionsWithProofsParams>(value).unwrap_err();

        // Object without required params should fail
        let value = serde_json::json!({});
        serde_json::from_value::<GetTransactionsWithProofsParams>(value).unwrap_err();

        // Object params
        let value = serde_json::json!({
            "start_version": 10,
            "limit": 11,
        });
        serde_json::from_value::<GetTransactionsWithProofsParams>(value).unwrap();

        // Object with more params
        let value = serde_json::json!({
            "start_version": 10,
            "limit": 11,
            "foo": 11,
        });
        serde_json::from_value::<GetTransactionsWithProofsParams>(value).unwrap();
    }

    #[test]
    fn get_events_with_proofs() {
        let key = "13000000000000000000000000000000000000000a550c18";

        // Array with all params
        let value = serde_json::json!([key, 10, 11]);
        serde_json::from_value::<GetEventsWithProofsParams>(value).unwrap();

        // Array with too many params
        let value = serde_json::json!([key, 10, 11, false]);
        serde_json::from_value::<GetEventsWithProofsParams>(value).unwrap_err();

        // Array with wrong param
        let value = serde_json::json!(["foo", 10, 11]);
        serde_json::from_value::<GetEventsWithProofsParams>(value).unwrap_err();

        // Array with too few params
        let value = serde_json::json!([10, 11]);
        serde_json::from_value::<GetEventsWithProofsParams>(value).unwrap_err();

        // Empty array without required params should fail
        let value = serde_json::json!([]);
        serde_json::from_value::<GetEventsWithProofsParams>(value).unwrap_err();

        // Object without required params should fail
        let value = serde_json::json!({});
        serde_json::from_value::<GetEventsWithProofsParams>(value).unwrap_err();

        // Object params
        let value = serde_json::json!({
            "key": key,
            "start": 10,
            "limit": 11,
        });
        serde_json::from_value::<GetEventsWithProofsParams>(value).unwrap();

        // Object without all params
        let value = serde_json::json!({
            "start": 10,
            "limit": 11,
        });
        serde_json::from_value::<GetEventsWithProofsParams>(value).unwrap_err();

        // Object with more params
        let value = serde_json::json!({
            "key": key,
            "start": 10,
            "limit": 11,
            "foo": 11,
        });
        serde_json::from_value::<GetEventsWithProofsParams>(value).unwrap();
    }
}
