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

#[derive(Debug, Deserialize, Serialize)]
pub struct RawJsonRpcRequest {
    #[serde(default)]
    pub jsonrpc: serde_json::Value,
    #[serde(default)]
    pub method: serde_json::Value,
    #[serde(default)]
    pub params: serde_json::Value,
    pub id: Id,
}

impl JsonRpcRequest {
    pub fn from_value(
        value: serde_json::Value,
    ) -> Result<Self, (JsonRpcError, Option<Method>, Option<Id>)> {
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
                JsonRpcError::invalid_params(method.as_str()),
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
#[allow(clippy::large_enum_variant)]
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
    GetAccumulatorConsistencyProof(GetAccumulatorConsistencyProofParams),
    GetAccountStateWithProof(GetAccountStateWithProofParams),
    GetTransactionsWithProofs(GetTransactionsWithProofsParams),
    GetAccountTransactionsWithProofs(GetAccountTransactionsWithProofsParams),
    GetEventsWithProofs(GetEventsWithProofsParams),
    GetEventByVersionWithProof(GetEventByVersionWithProof),
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
            Method::GetAccumulatorConsistencyProof => {
                MethodRequest::GetAccumulatorConsistencyProof(serde_json::from_value(value)?)
            }
            Method::GetAccountStateWithProof => {
                MethodRequest::GetAccountStateWithProof(serde_json::from_value(value)?)
            }
            Method::GetTransactionsWithProofs => {
                MethodRequest::GetTransactionsWithProofs(serde_json::from_value(value)?)
            }
            Method::GetAccountTransactionsWithProofs => {
                MethodRequest::GetAccountTransactionsWithProofs(serde_json::from_value(value)?)
            }
            Method::GetEventsWithProofs => {
                MethodRequest::GetEventsWithProofs(serde_json::from_value(value)?)
            }
            Method::GetEventByVersionWithProof => {
                MethodRequest::GetEventByVersionWithProof(serde_json::from_value(value)?)
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
            MethodRequest::GetAccumulatorConsistencyProof(_) => {
                Method::GetAccumulatorConsistencyProof
            }
            MethodRequest::GetAccountStateWithProof(_) => Method::GetAccountStateWithProof,
            MethodRequest::GetTransactionsWithProofs(_) => Method::GetTransactionsWithProofs,
            MethodRequest::GetAccountTransactionsWithProofs(_) => {
                Method::GetAccountTransactionsWithProofs
            }
            MethodRequest::GetEventsWithProofs(_) => Method::GetEventsWithProofs,
            MethodRequest::GetEventByVersionWithProof(_) => Method::GetEventByVersionWithProof,
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
pub struct GetAccumulatorConsistencyProofParams {
    #[serde(default)]
    pub client_known_version: Option<u64>,
    #[serde(default)]
    pub ledger_version: Option<u64>,
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
    pub include_events: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetAccountTransactionsWithProofsParams {
    pub account: AccountAddress,
    pub start: u64,
    pub limit: u64,
    pub include_events: bool,
    #[serde(default)]
    pub ledger_version: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEventsWithProofsParams {
    pub key: EventKey,
    pub start: u64,
    pub limit: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEventByVersionWithProof {
    pub key: EventKey,
    #[serde(default)]
    pub version: Option<u64>,
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn metadata() {
        let parse_ok = |value| serde_json::from_value::<GetMetadataParams>(value).unwrap();
        let parse_err = |value| serde_json::from_value::<GetMetadataParams>(value).unwrap_err();

        // Too many array params
        parse_err(json!([10, 11]));

        // Correct number of array params
        parse_ok(json!([10]));

        // Correct number of array params but version is Null
        parse_ok(json!([serde_json::Value::Null]));

        // Empty array still correctly deserializes since the only param is optional
        parse_ok(json!([]));

        // Empty object still correctly deserializes since all params are optional
        parse_ok(json!({}));

        // Even if there's no object it still deserializes correctly since all params are optional
        parse_ok(serde_json::Value::Null);

        // Named params
        parse_ok(json!({"version": 10}));

        // JsonRpcRequest with no params
        parse_ok(json!({
            "jsonrpc": "2.0",
            "method": Method::GetMetadata,
            "id": 1,
        }));
    }

    #[test]
    fn get_account() {
        let parse_ok = |value| serde_json::from_value::<GetAccountParams>(value).unwrap();
        let parse_err = |value| serde_json::from_value::<GetAccountParams>(value).unwrap_err();
        let account = "1668f6be25668c1a17cd8caf6b8d2f25";

        // Array without optional param
        parse_ok(json!([account]));

        // Array with optional param
        parse_ok(json!([account, 10]));

        // Array with wrong optional param
        parse_err(json!([account, "foo"]));

        // Array with too many params
        parse_err(json!([account, 10, 1]));

        // Empty array without required params should fail
        parse_err(json!([]));

        // Object without required params should fail
        parse_err(json!({}));

        // Object without optional param
        parse_ok(json!({
            "account": account,
        }));

        // Object with optional param
        parse_ok(json!({
            "account": account,
            "version": 10,
        }));

        // Object with more params
        parse_ok(json!({
            "account": account,
            "version": 10,
            "foo": 11,
        }));
    }

    #[test]
    fn get_transactions() {
        let parse_ok = |value| serde_json::from_value::<GetTransactionsParams>(value).unwrap();
        let parse_err = |value| serde_json::from_value::<GetTransactionsParams>(value).unwrap_err();

        // Array with all params
        parse_ok(json!([10, 11, false]));

        // Array with too many params
        parse_err(json!([10, 11, false, "foo"]));

        // Array with wrong param
        parse_err(json!(["foo", 11, false]));

        // Array with too few params
        parse_err(json!([10, 11]));

        // Empty array without required params should fail
        parse_err(json!([]));

        // Object without required params should fail
        parse_err(json!({}));

        // Object params
        parse_ok(json!({
            "start_version": 10,
            "limit": 10,
            "include_events": true,
        }));

        // Object without all params
        parse_err(json!({
            "limit": 10,
            "include_events": true,
        }));

        // Object with more params
        parse_ok(json!({
            "start_version": 10,
            "limit": 10,
            "include_events": true,
            "foo": 11,
        }));
    }

    #[test]
    fn get_account_transaction() {
        let parse_ok =
            |value| serde_json::from_value::<GetAccountTransactionParams>(value).unwrap();
        let parse_err =
            |value| serde_json::from_value::<GetAccountTransactionParams>(value).unwrap_err();

        let account = "1668f6be25668c1a17cd8caf6b8d2f25";

        // Array with all params
        parse_ok(json!([account, 11, false]));

        // Array with too many params
        parse_err(json!([account, 11, false, "foo"]));

        // Array with wrong param
        parse_err(json!(["foo", 11, false]));

        // Array with too few params
        parse_err(json!([10, 11]));

        // Empty array without required params should fail
        parse_err(json!([]));

        // Object without required params should fail
        parse_err(json!({}));

        // Object params
        parse_ok(json!({
            "account": account,
            "sequence_number": 10,
            "include_events": true,
        }));

        // Object without all params
        parse_err(json!({
            "sequence_number": 10,
            "include_events": true,
        }));

        // Object with more params
        parse_ok(json!({
            "account": account,
            "sequence_number": 10,
            "include_events": true,
            "foo": 11,
        }));
    }

    #[test]
    fn get_account_transactions() {
        let parse_ok =
            |value| serde_json::from_value::<GetAccountTransactionsParams>(value).unwrap();
        let parse_err =
            |value| serde_json::from_value::<GetAccountTransactionsParams>(value).unwrap_err();

        let account = "1668f6be25668c1a17cd8caf6b8d2f25";

        // Array with all params
        parse_ok(json!([account, 10, 11, false]));

        // Array with too many params
        parse_err(json!([account, 10, 11, false, "foo"]));

        // Array with wrong param
        parse_err(json!(["foo", 10, 11, false]));

        // Array with too few params
        parse_err(json!([10, 11]));

        // Empty array without required params should fail
        parse_err(json!([]));

        // Object without required params should fail
        parse_err(json!({}));

        // Object params
        parse_ok(json!({
            "account": account,
            "start": 10,
            "limit": 11,
            "include_events": true,
        }));

        // Object without all params
        parse_err(json!({
            "include_events": true,
        }));

        // Object with more params
        parse_ok(json!({
            "account": account,
            "start": 10,
            "limit": 11,
            "include_events": true,
            "foo": 11,
        }));
    }

    #[test]
    fn get_account_transactions_with_proofs() {
        let parse = serde_json::from_value::<GetAccountTransactionsWithProofsParams>;
        let parse_ok = |value| parse(value).unwrap();
        let parse_err = |value| parse(value).unwrap_err();
        let account = "1668f6be25668c1a17cd8caf6b8d2f25";

        // Array with all params
        parse_ok(json!([account, 10, 11, false]));
        parse_ok(json!([account, 10, 11, false, 123]));

        // Array with too many params
        parse_err(json!([account, 10, 11, false, 123, "foo"]));

        // Array with wrong param
        parse_err(json!(["foo", 10, 11, false]));

        // Array with too few params
        parse_err(json!([account, 10]));
        parse_err(json!([]));
        parse_err(json!({}));

        // Object params
        parse_ok(json!({
            "account": account,
            "start": 10,
            "limit": 11,
            "include_events": true,
        }));
        parse_ok(json!({
            "account": account,
            "start": 10,
            "limit": 11,
            "include_events": true,
            "ledger_version": 999,
        }));

        // Object without all params
        parse_err(json!({
            "include_events": true,
        }));

        // Object with more params
        parse_ok(json!({
            "account": account,
            "start": 10,
            "limit": 11,
            "include_events": true,
            "foo": 11,
        }));
    }

    #[test]
    fn get_events() {
        let parse_ok = |value| serde_json::from_value::<GetEventsParams>(value).unwrap();
        let parse_err = |value| serde_json::from_value::<GetEventsParams>(value).unwrap_err();

        let key = "13000000000000000000000000000000000000000a550c18";

        // Array with all params
        parse_ok(json!([key, 10, 11]));

        // Array with too many params
        parse_err(json!([key, 10, 11, false]));

        // Array with wrong param
        parse_err(json!(["foo", 10, 11]));

        // Array with too few params
        parse_err(json!([10, 11]));

        // Empty array without required params should fail
        parse_err(json!([]));

        // Object without required params should fail
        parse_err(json!({}));

        // Object params
        parse_ok(json!({
            "key": key,
            "start": 10,
            "limit": 11,
        }));

        // Object without all params
        parse_err(json!({
            "start": 10,
            "limit": 11,
        }));

        // Object with more params
        parse_ok(json!({
            "key": key,
            "start": 10,
            "limit": 11,
            "foo": 11,
        }));
    }

    #[test]
    fn get_currencies() {
        let parse_ok = |value| serde_json::from_value::<GetCurrenciesParams>(value).unwrap();
        let parse_err = |value| serde_json::from_value::<GetCurrenciesParams>(value).unwrap_err();

        parse_err(json!([10]));
        parse_ok(json!([]));
        parse_ok(json!({}));
        parse_ok(serde_json::Value::Null);

        let request = json!({
            "jsonrpc": "2.0",
            "method": Method::GetCurrencies,
            "id": 1,
        });
        serde_json::from_value::<JsonRpcRequest>(request).unwrap();
    }

    #[test]
    fn get_network_status() {
        let parse_ok = |value| serde_json::from_value::<GetNetworkStatusParams>(value).unwrap();
        let parse_err =
            |value| serde_json::from_value::<GetNetworkStatusParams>(value).unwrap_err();

        parse_err(json!([10]));
        parse_ok(json!([]));
        parse_ok(json!({}));
        parse_ok(serde_json::Value::Null);

        let request = json!({
            "jsonrpc": "2.0",
            "method": Method::GetNetworkStatus,
            "id": 1,
        });
        serde_json::from_value::<JsonRpcRequest>(request).unwrap();
    }

    #[test]
    fn get_state_proof() {
        let parse_ok = |value| serde_json::from_value::<GetStateProofParams>(value).unwrap();
        let parse_err = |value| serde_json::from_value::<GetStateProofParams>(value).unwrap_err();

        // Array with all params
        parse_ok(json!([11]));

        // Array with too many params
        parse_err(json!([11, false]));

        // Array with wrong param
        parse_err(json!(["foo"]));

        // Empty array without required params should fail
        parse_err(json!([]));

        // Object without required params should fail
        parse_err(json!({}));

        // Object params
        parse_ok(json!({
            "version": 10,
        }));

        // Object with more params
        parse_ok(json!({
            "version": 10,
            "foo": 11,
        }));
    }

    #[test]
    fn get_accumulator_consistency_proof() {
        let parse = serde_json::from_value::<GetAccumulatorConsistencyProofParams>;
        let parse_ok = |value| parse(value).unwrap();
        let parse_err = |value| parse(value).unwrap_err();

        // Array with all params
        parse_ok(json!([11, 42]));

        // Array with too many params
        parse_err(json!([11, 42, 7]));

        // Array with no ledger version
        parse_ok(json!([11]));

        // Array with no ledger version or client known version
        parse_ok(json!([]));

        // Array with wrong first param
        parse_err(json!(["foo"]));

        // Array with wrong second param
        parse_err(json!([123, "bar"]));

        // Object with no ledger version or client known version
        parse_ok(json!({}));

        // Object with no ledger version
        parse_ok(json!({
            "client_known_version": 123,
        }));

        // Object with no client known version
        parse_ok(json!({
            "ledger_version": 123,
        }));

        // Object with all params
        parse_ok(json!({
            "client_known_version": 42,
            "ledger_version": 123,
        }));

        // Object with extra params
        parse_ok(json!({
            "client_known_version": 42,
            "ledger_version": 123,
            "foo": "bar",
        }));
    }

    #[test]
    fn get_account_state_with_proof() {
        let parse = serde_json::from_value::<GetAccountStateWithProofParams>;
        let parse_ok = |value| parse(value).unwrap();
        let parse_err = |value| parse(value).unwrap_err();

        let account = "1668f6be25668c1a17cd8caf6b8d2f25";

        // Array with all params
        parse_ok(json!([account, 11, 12]));

        // Array without optional params
        parse_ok(json!([account]));
        parse_ok(json!([account, 12]));

        // Array with too many params
        parse_err(json!([account, 11, 12, "foo"]));

        // Array with wrong param
        parse_err(json!(["foo", 11, 12]));

        // Array with too few params
        parse_err(json!([10, 11]));

        // Empty array without required params should fail
        parse_err(json!([]));

        // Object without required params should fail
        parse_err(json!({}));

        // Object params
        parse_ok(json!({
            "account": account,
            "version": 10,
            "ledger_version": 10,
        }));

        // Object without all params
        parse_ok(json!({
            "account": account,
            "ledger_version": 10,
        }));

        // Object with more params
        parse_ok(json!({
            "account": account,
            "version": 10,
            "ledger_version": 10,
            "foo": 11,
        }));
    }

    #[test]
    fn get_transactions_with_proofs() {
        let parse = serde_json::from_value::<GetTransactionsWithProofsParams>;
        let parse_ok = |value| parse(value).unwrap();
        let parse_err = |value| parse(value).unwrap_err();

        // Array with all params
        parse_ok(json!([10, 11, true]));

        // Array with too many params
        parse_err(json!([10, 11, 42, false]));

        // Array with wrong param
        parse_err(json!(["foo", 11, false]));

        // Array with too few params
        parse_err(json!([10, 11]));

        // Empty array without required params should fail
        parse_err(json!([]));

        // Object without required params should fail
        parse_err(json!({}));

        // Object params
        parse_ok(json!({
            "start_version": 10,
            "limit": 11,
            "include_events": true,
        }));

        // Object with more params
        parse_ok(json!({
            "start_version": 10,
            "limit": 11,
            "include_events": true,
            "foo": "bar",
        }));
    }

    #[test]
    fn get_events_with_proofs() {
        let parse = serde_json::from_value::<GetEventsWithProofsParams>;
        let parse_ok = |value| parse(value).unwrap();
        let parse_err = |value| parse(value).unwrap_err();

        let key = "13000000000000000000000000000000000000000a550c18";

        // Array with all params
        parse_ok(json!([key, 10, 11]));

        // Array with too many params
        parse_err(json!([key, 10, 11, false]));

        // Array with wrong param
        parse_err(json!(["foo", 10, 11]));

        // Array with too few params
        parse_err(json!([10, 11]));

        // Empty array without required params should fail
        parse_err(json!([]));

        // Object without required params should fail
        parse_err(json!({}));

        // Object params
        parse_ok(json!({
            "key": key,
            "start": 10,
            "limit": 11,
        }));

        // Object without all params
        parse_err(json!({
            "start": 10,
            "limit": 11,
        }));

        // Object with more params
        parse_ok(json!({
            "key": key,
            "start": 10,
            "limit": 11,
            "foo": 11,
        }));
    }

    #[test]
    fn get_event_by_version_with_proof() {
        let parse = serde_json::from_value::<GetEventByVersionWithProof>;
        let parse_ok = |value| parse(value).unwrap();
        let parse_err = |value| parse(value).unwrap_err();

        let key = "13000000000000000000000000000000000000000a550c18";

        // Correct arguments
        parse_ok(json!([key, 10]));
        parse_ok(json!([key]));

        // Incorrect arguments
        parse_err(json!([key, 10, false]));
        parse_err(json!(["foo", 10]));
        parse_err(json!([]));
        parse_err(json!({}));

        // Object params
        parse_ok(json!({ "key": key, "version": 10 }));
        parse_ok(json!({ "key": key }));

        // Object without all required params
        parse_err(json!({ "version": 10 }));

        // Object with more params
        parse_ok(json!({ "key": key, "version": 10, "foo": 99 }));
    }
}
