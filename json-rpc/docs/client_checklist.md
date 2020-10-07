This file is a checklist of requirement & technical details for a Libra client SDK implementation.

# Basics

- [ ] module structure:
  - libra
    - LibraClient: JSON-RPC APIs interface, should support application to do easy mock / stub development.
    - jsonrpc: jsonrpc client interface, include plain data classes / structs defined in Libra JSON-RPC SPEC document.
      - types: data transfer object types for jsonrpc client, should match server side JSON-RPC spec data types.
    - stdlib: move stdlib script utils.
    - testnet: testnet utils, should include FaucetService for handling testnet mint.
    - libra-types: Libra onchain data structure types.
    - utils:
      - signing
      - sha3 hashing, address parsing and converting, hex encoding / decoding
      - [LIP-4] transaction metadata
      - [LIP-5] intent identifier, account identifier
- [ ] JSON-RPC 2.0 Spec:
  - spec version validation.
  - batch requests and responses handling.
- [ ] JSON-RPC client error handling should distinguish the following 3 type errors:
  - Transport layer error, e.g. HTTP call failure.
  - JSON-RPC protocol error: e.g. server responds to non json data, or can't be parsed into [Libra JSON-RPC SPEC][1] defined data structure, or missing result & error field.
  - JSON-RPC error: error returned from server.
- [ ] https.
- [ ] Client connection pool.
- [ ] Handle stale responses:
  - [ ] client tracks latest server response block version and timestamp, raise error when received server response contains stale version / timestamp.
    - [ ] last known blockchain version >= response version + 30: when connecting to a cluster of fullnodes, it is possible some fullnodes are behind the head couple versions.
    - [ ] last known blockchain timestamp >= response timestamp + 30 seconds.
  - [ ] parse and use libra_chain_id, libra_ledger_version and libra_ledger_tiemstamp in the JSONRPC response.
- [ ] Parsing and gen Libra Account Identifier (see [LIP-5][2])
  - bech32 addresses/subaddresses support
- [ ] language specific standard release publish: e.g. java maven central repo, python pip
- [ ] Multi-network: initialize Client with chain id, JSON-RPC server URL
- [ ] Handle unsigned int64 data type properly
- [ ] Validate server chain id: client should be initialized with chain id and validate server response chain id is the same.
- [ ] Validate input parameters, e.g. invalid account address: "kkk". Should return / raise InvalidArgumentError.
- [ ] Send request with "client sdk name / version" as HTTP User-Agent: this is for server to recognize client sdk version, so that server can block a specific client version if we found unacceptable bugs.
- [ ] Decode transaction script bytes

# [LIP-4][7] support

- [ ] Non-custodial to custodial transaction
- [ ] Custodial to non-custodial transaction
- [ ] Custodial to Custodial transaction
- [ ] Refund

# [LIP-5][2] support

- [ ] Encode and decode account identifier
- [ ] Encode and decode intent identifier

# Read from Blockchain

- [ ] Get metadata
- [ ] Get currencies
- [ ] Get events
- [ ] Get transactions
- [ ] Get account
- [ ] Get account transaction
- [ ] Get account transactions
- [ ] Get account events
- [ ] Handle error response
- [ ] Serialize result JSON to typed data structure
- [ ] Forward compatible: ignore unknown fields for
- [ ] Backward compatible: new fields are optional

# Submit Transaction

- [ ] Submit [p2p transfer][3] transaction
- [ ] Submit other [Move Stdlib scripts][4]
- [ ] waitForTransaction(accountAddress, sequence, transcationHash, expirationTimeSec, timeout):
  - for given signed transaction sender address, sequence number, expiration time (or 5 sec timeout) to wait and validate execution result is executed, otherwise return/raise an error / flag to tell it is not executed.
  - when signedTransactionHash validation failed, it should return / raise TransactionSequenceNumberConflictError
  - when transaction execution vm_status is not "executed", it should return / raise TransactionExecutionFailure
  - when transaction expired, it should return / raise TransactionExpiredError: compare the transaction expirationTimeSec with response latest ledger timestamp. If response latest ledger timestamp >= transaction expirationTimeSec, then we are sure the transaction will never be executed successfully.
    - Note: response latest ledger timestamp unit is microsecond, expirationTimeSec's unit is second.

# Testnet support

- [ ] Generate ed25519 private key, derive ed25519 public keys from private key.
- [ ] Generate Single auth-keys
- [ ] Generate MultiSig auth-keys
- [ ] Mint coins through [Faucet service][6]

See [doc][5] for above concepts.

# Examples

- [ ] [p2p transfer examples](https://github.com/libra/lip/blob/master/lips/lip-4.md#transaction-examples)
- [ ] refund p2p transfer example
- [ ] create childVASP example
- [ ] Intent identifier encoding, decoding example

# Nice to have

- [ ] Async client
- [ ] CLI connects to testnet for trying out features.

[1]: https://github.com/libra/libra/blob/master/json-rpc/json-rpc-spec.md "Libra JSON-RPC SPEC"
[2]: https://github.com/libra/lip/blob/master/lips/lip-5.md "LIP-5"
[3]: https://github.com/libra/libra/blob/master/language/stdlib/transaction_scripts/doc/peer_to_peer_with_metadata.md "P2P Transafer"
[4]: https://github.com/libra/libra/tree/master/language/stdlib/transaction_scripts/doc "Move Stdlib scripts"
[5]: https://github.com/libra/libra/blob/master/client/libra-dev/README.md "Libra Client Dev Doc"
[6]: https://github.com/libra/libra/blob/master/json-rpc/docs/service_testnet_faucet.md "Faucet service"
[7]: https://github.com/libra/lip/blob/master/lips/lip-4.md "Transaction Metadata Specification"
