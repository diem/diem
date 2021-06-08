# JSON-RPC

JSON-RPC service provides APIs for client applications to query the Diem blockchain.

There are two types of clients, and thus two types of APIs:

* verifying clients. These clients verify the cryptographic proofs accompanying responses from the server.
* non-verifying clients. Such clients completely trust the full node they query. If the server decides to lie about the data contained in a response, the client wouldn't know it. This mode is used when a client runs their own full node.

There are also two types of endpoints:
* Standard: JSON-RPC over HTTP `POST` requests. One response per request.
* Streaming: JSON-RPC over various transport options (currently: websockets). One or more responses per request,
             over a persistent connection.

## Usage and versioning

Refer to the [specification](https://github.com/diem/diem/blob/main/json-rpc/json-rpc-spec.md).

## Security Concerns

It currently is exposed to 0.0.0.0 by default, so be aware of that.

It also uses liberal CORS policies, so its API must not use any secrets (e.g. tokens, credentials, etc.)

## Test

To make sure we have backward compatible API, tests are intent to be created with verbose json result
in assertions.

There are 2 types test:

* src/tests/unit_tests.rs: these tests use a mockdb in test, it can cover simple logics and http related
  test cases. It runs fast, so you should consider create unit test instead of integration test if possible.
* tests/integration_test.rs: these are integration tests, it launches a validator node with JSON-RPC service
  running for testing. These tests are good for complex business logics when it is hard to setup mockdb data.
  For example:
      * create all kinds of roles' account and validate the data serialized is correct.
      * to confirm the CurrencyInfo exchange_rate_update_events_key is generated correctly with event data
        serialized, you need create a transaction to update exchange rate, and then pull the events by the
        CurrencyInfo#exchange_rate_update_events_key and validate the event data is correct.
  As launching validator node with database is slow, we only have one test with a list of sub test cases. Sub
  test cases will run in sequence for making deterministic result for tests.
