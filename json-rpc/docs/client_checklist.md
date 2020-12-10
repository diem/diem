This file is a checklist of requirement & technical details for a Diem client SDK implementation.

# Basics

- [ ] module structure:
  - diem
    - DiemClient: JSON-RPC APIs interface, should support application to do easy mock / stub development.
    - jsonrpc: jsonrpc client interface, include plain data classes / structs defined in Diem JSON-RPC SPEC document.
      - types: data transfer object types for jsonrpc client, should match server side JSON-RPC spec data types.
    - stdlib: move stdlib script utils.
    - testnet: testnet utils, should include FaucetService for handling testnet mint.
    - diem-types: Diem onchain data structure types.
    - utils:
      - signing
      - sha3 hashing, address parsing and converting, hex encoding / decoding
      - [DIP-4] transaction metadata
      - [DIP-5] intent identifier, account identifier
- [ ] JSON-RPC 2.0 Spec:
  - spec version validation.
  - batch requests and responses handling.
- [ ] JSON-RPC client error handling should distinguish the following 3 type errors:
  - Transport layer error, e.g. HTTP call failure.
  - JSON-RPC protocol error: e.g. server responds to non json data, or can't be parsed into [Diem JSON-RPC SPEC][1] defined data structure, or missing result & error field.
  - JSON-RPC error: error returned from server.
- [ ] Supports https.
- [ ] Client connection pool.
- [ ] Handle stale responses:
  - [ ] Parse diem_chain_id, diem_ledger_version and diem_ledger_tiemstamp in the JSONRPC response.
  - [ ] client tracks latest server response block version and timestamp, raise error when received server response contains stale version / timestamp.
    - [ ] last known blockchain version <= response version: when connecting to a cluster of fullnodes, it is possible some fullnodes are behind the head couple versions.
    - [ ] last known blockchain timestamp <= response timestamp.
    - [ ] Query / get actions/methods should retry on stable response error. Otherwise client may get the following confusing result:
      - After submit and wait for a transaction executed successfully.
      - Call get_account method to get back latest account balances.
      - It is possible we get back an account balance from a stale server that is before the transaction executed. Then the account balance is still before transaction executed balance.
    - [ ] Do not retry for submit transaction call, because the submitted transaction can be synced correctly even you submitted it to a stale server. You may receive a JSON-RPC error if same transaction was submitted twice.
    - [ ] The retry logic should be configured when initializing the Client, so that client application can control the behavior and even remove the try for them to handle retry properly for async environment.
- [ ] language specific standard release publish: e.g. java maven central repo, python pip
- [ ] Initialize a Client with chain id, JSON-RPC server URL.
  - Chain id is used for validating get methods response, making sure we are connecting to expected network.
  - Client can choose to initialize chain id by first server call.
- [ ] Handle unsigned int64 data type properly. Can be simply mark the type as unsigned int64 and use int64.
- [ ] Validate input parameters, e.g. invalid account address: "kkk". Should return / raise InvalidArgumentError.
- [ ] Send request with "client sdk name / version" as HTTP User-Agent: this is for server to recognize client sdk version.
- [ ] Decode transaction script bytes: it is possible we may add new transaction script without upgrading server, hence client decoding logic is important for client to recognize all transaction scripts.
    By upgrading client side move stdlib scripts (binary and generated type information code), we can decode latest move stdlib scripts executed on-chain.
- [ ] Create transaction hash from signed transaction: hex-encode(sha3-256([]byte("DIEM::Transaction")) + []byte(0) + signed transaction bytes) ([implementation example](https://github.com/diem/client-sdk-go/blob/master/diemtypes/hash.go#L27))
- [ ] Client interface should prefer to use AccountAddress type instead of string address.


# [DIP-4][7] Transaction Metadata support

- [ ] Non-custodial to custodial transaction metadata
- [ ] Custodial to non-custodial transaction metadata
- [ ] Custodial to Custodial transaction metadata and signature
- [ ] Refund metadata

# [DIP-5][2] Address Formating support

- [ ] bech32 encoding/decoding
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
- [ ] wait for transaction(submitted signed transaction, timeout):
  - decode submitted signed transaction
  - create transaction hash from submitted signed transaction.
  - call waitForTransaction(accountAddress, sequence, transcationHash, expirationTimeSec, timeout):

# Testing & Testnet support

- [ ] Generate ed25519 private key, derive ed25519 public keys from private key.
- [ ] Generate Single auth-keys
- [ ] Generate MultiSig auth-keys
- [ ] Mint coins through [Faucet service][6]

See [doc][5] for above concepts.

# Examples

- [ ] [p2p transfer examples](https://github.com/diem/dip/blob/master/dips/dip-4.md#transaction-examples)
- [ ] refund p2p transfer example
- [ ] create childVASP example
- [ ] Intent identifier encoding, decoding example

# Nice to have

- [ ] Async client
- [ ] CLI connects to testnet for trying out features.

[1]: https://github.com/diem/diem/blob/master/json-rpc/json-rpc-spec.md "Diem JSON-RPC SPEC"
[2]: https://github.com/diem/dip/blob/master/dips/dip-5.md "DIP-5"
[3]: https://github.com/diem/diem/blob/master/language/stdlib/transaction_scripts/doc/peer_to_peer_with_metadata.md "P2P Transafer"
[4]: https://github.com/diem/diem/tree/master/language/stdlib/transaction_scripts/doc "Move Stdlib scripts"
[5]: https://github.com/diem/diem/blob/master/client/diem-dev/README.md "Diem Client Dev Doc"
[6]: https://github.com/diem/diem/blob/master/json-rpc/docs/service_testnet_faucet.md "Faucet service"
[7]: https://github.com/diem/dip/blob/master/dips/dip-4.md "Transaction Metadata Specification"
