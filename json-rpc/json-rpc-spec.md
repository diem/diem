# JSON-RPC SPEC

## Overview

The Libra client API is based on the JSON-RPC protocol. This specification defines the client API endpoints and types, and provides usage examples.

List of released stable methods (unless specifically mentioned, all parameters are required for the method.):

* [submit](docs/method_submit.md)(data: string) -> void
* [get_transactions](docs/method_get_transactions.md)(start_version: unsigned_int64, limit: unsigned_int64, include_events: boolean) -> List<[Transaction](docs/type_transaction.md)>
* [get_account](docs/method_get_account.md)(account: string) -> [Account](docs/type_account.md)
* [get_account_transaction](docs/method_get_account_transaction.md)(account: string, sequence_number: unsigned_int64, include_events: boolean) -> List<[Transaction](docs/type_transaction.md)>
* [get_account_transactions](docs/method_get_account_transactions.md)(account: string, start: unsigned_int64, limit: unsigned_int64, include_events: boolean) -> [Transaction](docs/type_transaction.md)
* [get_metadata](docs/method_get_metadata.md)(version: unsigned_int64) -> [Metadata](docs/type_metadata.md)
* [get_events](docs/method_get_events.md)(key: string, start: unsigned_int64, limit: unsigned_int64) -> List<[Event](docs/type_event.md)>
* [get_currencies](docs/method_get_currencies.md)() -> List<[CurrencyInfo](docs/type_currency_info.md)>


> To implement a client, please checkout our [Client Implementation Guide](docs/client_implementation_guide.md).

## Official Client SDKs

* [Go] (https://github.com/libra/libra-client-sdk-go)
* [Java] (https://github.com/libra/libra-client-sdk-java)


## JSON-RPC specification

JSON-RPC is a stateless, light-weight remote procedure call (RPC) protocol. Refer to the [JSON-RPC Specification](https://www.jsonrpc.org/specification) for further details.

### Libra extensions

The JSON-RPC response object is extended with the following fields:

| Field                      | Type           | Meaning                                      |
|----------------------------|----------------|----------------------------------------------|
| libra_chain_id             | unsigned int8  | network chain id, e.g. testnet chain id is 2 |
| libra_ledger_version       | unsigned int64 | server-side latest ledger version number     |
| libra_ledger_timestampusec | unsigned int64 | server-side latest ledger timestamp microseconds |

You can use this information to verify liveness / status of nodes in the network: if the timestamp or version is old (from the past), it means that the request hit a full node that is not up-to-date.


#### Example:

``` json

{
  "id": 1,
  "jsonrpc": "2.0",
  "libra_chain_id": 2,
  "libra_ledger_timestampusec": 1596680521771648,
  "libra_ledger_version": 3253133,
  "result": {
    "timestamp": 1596680521771648,
    "version": 3253133
  }
}

```

### Batched requests

The JSON-RPC protocol allows requests to be batched. An arbitrary number of requests (maximum 20 by default) can be combined into a single batch and submitted to the server. These requests will be processed together under a single request context.


### Errors

If errors occur during a request, they are returned in an error object, as defined in: [https://www.jsonrpc.org/specification#error_object](https://www.jsonrpc.org/specification#error_object).
For any invalid request or parameters request, a standard error code and message with human readable information will be returned.

| Code   | Meaning                                 |
|--------|-----------------------------------------|
| -32600 | standard invalid request error          |
| -32601 | method not found or not specified       |
| -32602 | invalid params                          |
| -32604 | invalid format                          |

Unless specifically mentioned below, Libra JSON-RPC will return the default error code - 32000 for generic server-side errors. More information may be returned in the ‘message’ and the ‘data’ fields, but this is not guaranteed.

## Versioning

We use URI versioning to version our API, current version is v1.
For example, to hit testnet, the server url is: https://testnet.libra.org/v1.
You may check [API-CHANGELOG.md](API-CHANGELOG.md) and learn more about our API changes.
