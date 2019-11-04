# Libra Client Specification

**warning:**
**This specification is a work-in-progress and is largely incomplete.**
**You are discouraged from using it to implement a client, but are encouraged to contribute to it.**

The Libra network is currently composed of Validators that come to consensus on the state of the blockchain. 
The current way of accessing data from the blockchain is to query these Validators directly.
As Libra is a decentralized cryptocurrency, a client should not blindly trust responses to their queries.
Instead, clients can use a combination of cryptographic signatures and merkle trees to ensure that every response is legitimate.
This is ensured as long as more than two thirds of the Validators are trusted.
This document specifies how a client can query validators and how a client must verify responses.

## Table of Contents

* [Libra Client Specification](#libra-client-specification)
    * [Client Requirements](#client-requirements)
    * [Server Interface](#server-interface)
    * [Merkle Tree Proof Verification](#merkle-tree-proof-verification)
        * [Bitmaps](#bitmaps)
        * [Constants](#constants)
        * [Hash Function](#hash-function)
        * [Accumulator Proof Verification](#accumulator-proof-verification)
        * [Sparse Merkle Tree Proof Verification](#sparse-merkle-tree-proof-verification)
    * [SubmitTransaction Request](#submittransaction-request)
    * [SubmitTransaction Response](#submittransaction-response)
    * [UpdateToLatestLedger Requests](#updatetolatestledger-requests)
        * [GetAccountStateRequest](#getaccountstaterequest)
        * [GetAccountTransactionBySequenceNumberRequest](#getaccounttransactionbysequencenumberrequest)
        * [GetEventsByEventAccessPathRequest](#geteventsbyeventaccesspathrequest)
        * [GetTransactionsRequest](#gettransactionsrequest)
    * [UpdateToLatestLedger Response](#updatetolatestledger-response)
        * [GetAccountStateResponse](#getaccountstateresponse)
        * [GetAccountTransactionBySequenceNumberResponse](#getaccounttransactionbysequencenumberresponse)
        * [GetEventsByEventAccessPathResponse](#geteventsbyeventaccesspathresponse)
        * [GetTransactionsResponse](#gettransactionsresponse)
    * [Security Considerations](#security-considerations)
    * [Appendix](#appendix)
        * [Appendix A - VM Status Code](#appendix-a---vm-status-code)


## Client Requirements

Clients that want to query the blockchain must be prepared to validate responses. To do this, they must be aware of two different values:

* **Epoch number**. It indicates changes of validators.
* **Validator set**. It lists the public keys allowed to sign states of the blockchain for a specific *epoch*.

For this reason, a client needs to be initialized with an *epoch number* and its matching *validator set*.
In addition, it must be able to mutate this state, as new epoch numbers might imply different validator set.

A client lagging behind (for example at epoch 3, when the current epoch is 6) will need to query and validate several messages from a validator in order to update its state to the last epoch. This might add some overhead to clients that do not often query the blockchain (TODO: time estimation).

In addition, a client keeps a `client_state` with the following fields:

* `version`: the version of the latest ledger info it knows. By default 0.
* `transaction_root`: the root hash of the latest transaction merkle tree it knows. By default empty. (TODO: or genesis?)
* `epoch_num`: the epoch number it is currently synced to.
* `validator_set`: the validator set corresponding to this epoch number.

While `version` and `transaction_root` can have default values, meaning that the client only knows the genesis state, a client must initialize its `epoch_num` and `validator_set` field in order to validate responses from validators.

(TODO: where to find latest epoch_num + validator_set?)

## Server Interface

Validators expose their public interface via a component called **Admission Control**.
Clients and validators communicate using the GRPC protocol. 
For this reason, data structures throughout this document are specified as [Protocol Buffers version 3](https://developers.google.com/protocol-buffers/docs/reference/proto3-spec) messages.

Admission Control exposes the following GRPC interface:

```proto
service AdmissionControl {
  rpc SubmitTransaction(SubmitTransactionRequest)
      returns (SubmitTransactionResponse) {}

  rpc UpdateToLatestLedger(
      types.UpdateToLatestLedgerRequest)
      returns (types.UpdateToLatestLedgerResponse) {}
}
```

Where **SubmitTransaction** is the interface to submit arbitrary transactions to the network and **UpdateToLatestLedger** is the interface for querying blockchain information in different ways.

The rest of this document goes through each possible requests and responses, specifying the details of validating server responses. 

## Merkle Tree Proof Verification

Validators can provide snapshot of the blockchain at different point in time.
This information is structured in several merkle tree structures and a "ledger root" is signed by a quorum of validators.

There are two types of merkle trees: **append-only accumulators** (TODO: more explanation) and **sparse merkle trees**. (TODO: jellyfish?)

The ledger root authenticates an append-only tree of "transaction infos". 
Each *transaction info* contains:

* A signed transaction hash: this does not contain the transaction itself, but a hash of the signed transaction. (TODO: malleability warning?)
* A state root: the root of a sparse merkle tree, where leafs are account states
* A event root: the root of an append-only tree, where leafs are events (TODO: is this correct?)
* gas used: gas used by the execution of the transaction
* major status: this indicates if the transaction was excecuted successfuly
    
(TODO: diagram)

To recapitulate, there are three trees:

* A ledger info tree: append-only.
* An event tree: append-only.
* A state tree: sparse.

Proofs for each type of tree can be inclusion or non-inclusion proofs.
(TODO: what's a proof?)
(TODO: explanation for each type of tree, what is the path? right/left nodes?)

### Bitmaps

A proof for a Merkle Tree includes information on the path a leaf or internal node takes to the root.
A way to encode this path is to use a string of bits, where every bit indicates if the next node is on the left or on the right.
Using a bitstring as a datastructure to compress our information into bits is called a **bitmap**.
Libra defines bitmaps this way:

* The most-left bits hold information about nodes near the root.
* The most-right bits hold information about nodes near the leaf.

In addition, to verify a proof, a client needs to know the siblings of each step of the path up to the root.
These siblings are sometimes legitimate nodes, or placeholders. 
As such, proofs come with a bitmap that indicates if the next sibling is a placeholder or not.

<!-- diagram of sibling + bitmap in a merkle tree -->

First, we define `node_type` which indicates if a sibling node in the tree is a placeholder (default) or not. It also indicates if the end of the bitmap has been reached.

```c
Enum node_type {
    default     // if bit is 0
    non_default // if bit is 1
    end         // end of the bitmap
}
```

Second, we define `node_position` which indicates if a node is on the left or on the right.

```c
Enum node_position {
    left    // if bit is 0
    right   // if bit is 1
    end     // end of the bitmap
}
```

There are two types of bitmaps used by Libra:

* Bitmaps of 8 bytes.
* Bitmaps of 32 bytes.

We abstract each type of bitmap in this specification via `Bitmap8` and `Bitmap32`:

**`Bitmap8(bitmap: uint64)`** takes a bitmap of 8 bytes. A function `Next()` acts on this abstraction to provide a `node_type`, reading bits from the right. Leading zeros are ignored.

For example:

```php
bitmap = Bitmap8(bitmap=0b0000_0000)
bitmap.Next() // returns `end`

bitmap = Bitmap8(bitmap=0b0000_0001)
bitmap.Next() // returns `non_default`
bitmap.Next() // returns `end`

bitmap = Bitmap8(bitmap=0b0000_0010)
bitmap.Next() // returns `default`
bitmap.Next() // returns `non_default`
bitmap.Next() // returns `end`

bitmap = Bitmap8(bitmap=0b0001_1010)
bitmap.Next() // returns `default`
bitmap.Next() // returns `non_default`
bitmap.Next() // returns `default`
bitmap.Next() // returns `non_default`
bitmap.Next() // returns `non_default`
bitmap.Next() // returns `end`
```

**`Bitmap32(bitmap: [32]byte, path: [32]byte)`** takes two bitmaps of 32-byte each. A function `Next()` acts on this abstraction to provide a tuple `(node_type, node_position)` which extracts the `node_type` from the first bitmap and the position of the current node from the second bitmap. If the bitmap has `n` trailing zeros, the first `n` bits are of both arguments are ignored.

For example (TODO: don't both arguments have to be 32-byte?):

```php
bitmap = Bitmap32(bitmap=[0b1101_0000, 0b1000_0000], path=[0b1011_1001, 0b1011_1001])
bitmap.Next() // returns `(non_default, right)`
bitmap.Next() // returns `(default, right)`
bitmap.Next() // returns `(default, left)`
bitmap.Next() // returns `(default, left)`
bitmap.Next() // returns `(default, right)`
bitmap.Next() // returns `(non_default, right)`
bitmap.Next() // returns `(default, right)`
bitmap.Next() // returns `(non_default, left)`
bitmap.Next() // returns `(non_default, right)`
bitmap.Next() // returns `(end, end)`
```

### Constants

A client must be aware of a few constants in order to verify Merkle tree proofs:

**domain separation for hashing**.

* "SparseMerkleLeaf"
* "SparseMerkleInternal"
* "TransactionAccumulator"

**hash suffix**:

* HASH_SUFFIX = []byte("@@$$LIBRA$$@@")

**placeholders for merkle trees**:

* ACCUMULATOR_PLACEHOLDER_HASH = []byte("ACCUMULATOR_PLACEHOLDER_HASH") + []byte{0, 0, 0, 0}
* SARSE_MERKLE_PLACEHOLDER_HASH = []byte("SPARSE_MERKLE_PLACEHOLDER_HASH") + []byte{0, 0}

### Hash Function

An implementation of the cryptographic hash function SHA-3-256 of output length 256-bit must be available to the client in order to verify proofs.

### Accumulator Proof Verification

Response validations rely on the following function to validate an accumulator proof:

`verify_accumulator_element(DOMAIN, expected_root_hash, element_hash, element_index, proof)`

The function returns `true` if the proof is valid, `false` otherwise.

the function has the following inputs:

* `DOMAIN`. A bytestring used as domain-separator for the hash function.
* `expected_root_hash`. An array of 32-byte.
* `element_hash`. An array of 32-byte.
* `element_index`. A uint64 value.
* `proof`. A protobuf message explained below.

```proto
message AccumulatorProof {
  uint64 bitmap = 1;
  repeated bytes non_default_siblings = 2;
}
```

* **`bitmap`**. The bitmap indicating which siblings are default. 1 means non-default and 0 means default. The LSB corresponds to the sibling at the bottom of the accumulator. The leftmost 1-bit corresponds to the sibling at the level just below root level in the accumulator, since this one is always non-default.
* **`non_default_siblings`**. The non-default siblings. The ones near the root are at the beginning of the list.

In order to validate the proof, a client must first do some validation checks:

* Ensure that the number of bit set to 1 in `proof.bitmap` is equal to the length of `proof.non_default_siblings`.
* Ensure that `leadingZeros >= 1` where `leadingZeroes` is the number of leading zeros in `proof.bitmap`.

Proof Verification in pseudo-code:

```py
bitmap = Bitmap8(bitmap=proof.bitmap)
current_hash = element_hash
loop {    
    default = bitmap.Next()
    if default == end {
        break
    }
    if default {
        sibling = ACCUMULATOR_PLACEHOLDER_HASH
    } else {
        sibling = proof.non_default_siblings.pop()
    }

    if element_index & 1 == 0 {
        current_hash = SHA-3-256(DOMAIN, HASH_SUFFIX, sibling, current_hash)
    } else {
        current_hash = SHA-3-256(DOMAIN, HASH_SUFFIX, current_hash, sibling)
    }

    element_index = element_index >> 1
}

if current_hash == expected_root_hash {
    return true
} else {
    return false
}
```

### Sparse Merkle Tree Proof Verification

Response validations rely on the following function to validate a sparse merkle tree proof:

`verify_sparse_merkle_element(expected_root_hash, element_key, element_blob, proof)`
<!-- TODO: create two functions, one where element_blob is set, one where it is not set -->

The function returns `true` if the proof is valid, `false` otherwise.

The function has the following inputs:

* `expected_root_hash`. An array of 32-byte.
* `element_key`. An array of 32-byte.
* `element_blob`. Option<AccountStateBlob>
* `proof`. A protobuf message as explained below 

```proto
message SparseMerkleProof {
  bytes leaf = 1;
  bytes bitmap = 2;
  repeated bytes non_default_siblings = 3;
}
```

* `leaf`. TKTK
* `bitmap`. The bitmap indicating which siblings are default. 1 means non-default and 0 means default. The MSB of the first byte corresponds to the sibling at the top of the Sparse Merkle Tree. The rightmost 1-bit of the last byte corresponds to the sibling at the bottom, since this one is always non-default.
* `non_default_siblings`. The non-default siblings. The ones near the root are at the beginning of the list.

Validation:

* Ensure that `proof.leaf` is empty or 32-bytes.
* Split `proof.leaf` in two 32-byte arrays:
    - the first part is a key
    - the second part is a value_hash
    - `leaf = None | Some((key, value_hash))`

* Ensure that the byte-length of `proof.bitmap` is smaller or equal to 32 bytes.
* Ensure that the last byte of `proof.bitmap` is not zero
* Ensure that the number of bit set to 1 in `proof.bitmap` is equal to the length of `proof.non_default_siblings`.

* Ensure that each item of proof.non_default_siblings` is of size 32-byte.

* If `element_index` is set
    - if `proof.leaf` is `Some((key, value_hash)`
        - ensure that `element_key == proof_key`
        - ensure that `hash == proof_value_hash`
    - if `proof.leaf` is `None`, abort.
* If `element_index` is not set
    - if `proof.leaf` is `Some((key, value_hash)`
        - ensure that `element_key != proof_key`
        - ensure that `element_key.common_prefix_bits_len(proof_key) >= siblings.len()`
    - if `proof.leaf` is `None`, continue.

Proof Verification:

* If `proof.leaf` is `None`, set `current_hash` to `SPARSE_MERKLE_PLACEHOLDER_HASH`.
* If `proof.leaf` is `Some((key, value_hash)` set `current_hash` to the output of `SHA-3-256(SparseMerkleLeafHasher, HASH_SUFFIX, key, value_hash)`.

```py
bitmap = Bitmap32(bitmap=proof.bitmap, path=element_key)
loop {    
    default, left = bitmap.Next()
    if default == end {
        break
    }
    if default {
        sibling = SPARSE_MERKLE_PLACEHOLDER_HASH
    } else {
        sibling = proof.non_default_siblings.pop()
    }

    if left {
        current_hash = SHA-3-256(SparseMerkleInternal, HASH_SUFFIX, sibling, current_hash)
    } else {
        current_hash = SHA-3-256(SparseMerkleInternal, HASH_SUFFIX, current_hash, sibling)
    }
}

if current_hash == expected_root_hash {
    return true
} else {
    return false
}
```

## SubmitTransaction Request

A client can submit a transaction to the network by encapsulating its LCS (TODO: definition of LCS) encoding into a `SignedTransaction` message wrapped into a `SubmitTransactionRequest`.

```proto
message SubmitTransactionRequest {
  types.SignedTransaction signed_txn = 1;
}

message SignedTransaction {
    bytes signed_txn = 5;
}
```

## SubmitTransaction Response

A response to a `SubmitTransaction` request will either contain some error codes, indicating that the transaction was not successfully processed by the validator, or nothing indicating success.

**This by no mean implies that the transaction was included on the chain**. 
At this point, a client must poll the network in order to verify that its transaction was successfuly processed.
TODO: how? GetAccountTransactionBySequenceNumberRequest? Or better with events?

This section specifies how to validate a response to a `SubmitTransactionRequest` message.

The response is a `SubmitTransactionResponse` message as specified below:

```proto
message SubmitTransactionResponse {
  // The status of a transaction submission can either be a VM status, or
  // some other admission control/mempool specific status e.g. Blacklisted.
  oneof status {
    types.VMStatus vm_status = 1;
    AdmissionControlStatus ac_status = 2;
    mempool_status.MempoolAddTransactionStatus mempool_status = 3;
  }
  bytes validator_id = 4;
}
```

* **`status`**. It indicates either a success or error message.
* **`validator_id`**. (TODO: what is that???)

(TODO: how to parse validator_id ?)

Depending on the type of `status`, the message must be processed differently.

**AdmissionControlStatus**. An admission control status.

```proto
message AdmissionControlStatus {
  AdmissionControlStatusCode code = 1;
  string message = 2;
}

enum AdmissionControlStatusCode {
  Accepted = 0;
  Blacklisted = 1;
  Rejected = 2;
}
```

If `status` is of type `AdmissionControlStatus`, the code indicates:

* `Acccepted`. The validator accepted the transaction. It is now pending to be processed by the network.
* `Blacklisted`. The sender is blacklisted
* `Rejected`. The transaction was rejected, e.g. due to incorrect signature.

If the `code` is not `Accepted`, the `message` field should provide more information on the reason.

If the `code` is `Accepted`, the client still needs to poll the blockchain in order to ensure that the transaction was processed. (TODO:how?)
This means that at this point in time, the client should not increment its sequence number.

**VMStatus**. A VM status indicates that the transaction did not pass validation checks of the MOVE Virtual Machine.

```proto
message VMStatus {
    uint64 major_status = 1;
    bool has_sub_status = 2;
    uint64 sub_status = 3;
    bool has_message = 4;
    string message = 5;
}
```

First, match the `major_status` according to [Appendix A](#Appendix-A) to learn more about the error.
If `has_sub_status` is set, match `sub_status` with [Appendix A](#Appendix-A) to learn more about the error. (TODO: how?)
If `has_message` is set, the field `message` should contain more information about the error.

**MempoolAddTransactionStatus**. A mempool status indicates.

```proto
message MempoolAddTransactionStatus {
  MempoolAddTransactionStatusCode code = 1;
  string message = 2;
}

enum MempoolAddTransactionStatusCode {
  Valid = 0;
  InsufficientBalance = 1;
  InvalidSeqNumber = 2;
  MempoolIsFull = 3;
  TooManyTransactions = 4;
  InvalidUpdate = 5;
}
```

The `code` field indicates a potential error (TODO: why is there a valid response since we already have that with AC?).

* **`Valid`**. Transaction was sent to Mempool.
* **`InsufficientBalance`**. The sender does not have enough balance for the transaction..
* **`InvalidSeqNumber`**. Sequence number is old, etc..
* **`MempoolIsFull`**. Mempool is full (reached max global capacity).
* **`TooManyTransactions`**. Account reached max capacity per account.
* **`InvalidUpdate`**. Invalid update. Only gas price increase is allowed.

The `message` field provides more information.

## UpdateToLatestLedger Requests

`UpdateToLatestLedger` requests allow a client to obtain specific data from the blockchain.

Below is the specification of the `UpdateToLatestLedgerRequest` message:

```proto
message UpdateToLatestLedgerRequest {
    uint64 client_known_version = 1;
    repeated RequestItem requested_items = 2;
}

message RequestItem {
    oneof requested_items {
        GetAccountStateRequest get_account_state_request = 1;
        GetAccountTransactionBySequenceNumberRequest get_account_transaction_by_sequence_number_request = 2;
        GetEventsByEventAccessPathRequest get_events_by_event_access_path_request = 3;
        GetTransactionsRequest get_transactions_request = 4;
    }
}
```

A client who only wants to update its state can simply omit the `requested_items` field.
To batch queries, a client can also include several `requested_items` (TODO: up to how many?)

There are four types of queries, listed below:

* **GetAccountStateRequest**. Get account state (balance, sequence number, etc.)
* **GetAccountTransactionBySequenceNumberRequest**. Get single transaction by account + sequence number.
* **GetEventsByEventAccessPathRequest**. Get events by event access path.
* **GetTransactionsRequest**. Get transactions.

In the following subsections, we go over constructing each type of requests.

### GetAccountStateRequest

```proto
message GetAccountStateRequest {
    bytes address = 1;
}
```

* **`address`**. Account for which we are fetching the state.

### GetAccountTransactionBySequenceNumberRequest

```proto
message GetAccountTransactionBySequenceNumberRequest {
    bytes account = 1;
    uint64 sequence_number = 2;
    bool fetch_events = 3;
}
```

* **`account`**. Account for which to query transactions.
* **`sequence_number`**. Sequence number for the transaction requested.
* **`fetch_events`**. Set to true to fetch events for the transaction at this version.


### GetEventsByEventAccessPathRequest

```proto
message GetEventsByEventAccessPathRequest {
    AccessPath access_path = 1;
    uint64 start_event_seq_num = 2;
    bool ascending = 3;
    uint64 limit = 4;
}
```

* **`access_path`**. TKTK
* **`start_event_seq_num`**. The sequence number of the event to start with for this query. Use a sequence number of MAX_INT to represent the latest.
* **`ascending`**. If ascending is true this query will return up to `limit` events that were emitted after `start_event_seq_num`. Otherwise it will return up to `limit` events before the offset. Both cases are inclusive.
* **`limit`**. Limit number of results.

### GetTransactionsRequest

```proto
// Get up to limit transactions starting from start_version.
message GetTransactionsRequest {
    uint64 start_version = 1;
    uint64 limit = 2;
    bool fetch_events = 3;
}
```

* **`start_version`**. The version of the transaction to start with for this query. Use a version of MAX_INT to represent the latest.
* **`limit`**. Limit number of results.
* **`fetch_events`**. Set to true to fetch events for the transaction at each version.

## UpdateToLatestLedger Response

Responses to `UpdateToLatestLedger` requests must be accompanied with cryptographic proofs that the data is correct.
To do this, a recent `LedgerInfoWithSignatures` is provided, which contains the root of a merkle tree.
This merkle tree can authenticate the whole state of the blockchain at any point in time.
Specifically, a `LedgerInfoWithSignatures` authenticates a specific version of the blockchain, which is the state of the blockchain after the execution of one transaction.
A `LedgerInfoWithSignatures` can thus be used to verify any data contained in the response, with the only caveat that this data might be valid only at this version of the blockchain.

<!-- types/get_with_proofs::UpdateToLatestLedgerReponse::verify() -->

* verify ledgerInfo
    epoch_num
    signatures
    timestamp
    version
* verify response items based on each request item 
    len(response) == len(request)


```proto
message UpdateToLatestLedgerResponse {
    repeated ResponseItem response_items = 1;
    LedgerInfoWithSignatures ledger_info_with_sigs = 2;
    // these two are used for validator set changes?
    repeated ValidatorChangeEventWithProof validator_change_events = 3;
    AccumulatorConsistencyProof ledger_consistency_proof = 4;
}

message ResponseItem {
    oneof response_items {
        GetAccountStateResponse get_account_state_response = 3;
        GetAccountTransactionBySequenceNumberResponse
            get_account_transaction_by_sequence_number_response = 4;
        GetEventsByEventAccessPathResponse get_events_by_event_access_path_response = 5;
        GetTransactionsResponse get_transactions_response = 6;
    }
}
```

Here is a description of each fields contained in `UpdateToLatestLedgerResponse`:

* `response_items`. The response along with cryptographic proofs to each request items we've made.
* `ledger_info_with_sigs`. The ledger info containing the root of truth used to validate proofs.
* `validator_change_events`. ?
* `ledger_consistency_proof`. ?

For completion, here are the data structures related to the `LedgerInfoWithSignatures`:

```proto
message LedgerInfoWithSignatures {
  repeated ValidatorSignature signatures = 1;
  LedgerInfo ledger_info = 2;
}

message ValidatorSignature {
  bytes validator_id = 1;
  bytes signature = 2;
}

message LedgerInfo {
  uint64 version = 1;
  bytes transaction_accumulator_hash = 2;
  bytes consensus_data_hash = 3;
  bytes consensus_block_id = 4;
  uint64 epoch_num = 5;
  uint64 timestamp_usecs = 6;
  ValidatorSet next_validator_set = 7;
}

message ValidatorSet {
  repeated ValidatorPublicKeys validator_public_keys = 1;
}

message ValidatorPublicKeys {
  bytes account_address = 1;
  bytes consensus_public_key = 2;
  uint64 consensus_voting_power = 3;
  bytes network_signing_public_key = 4;
  bytes network_identity_public_key = 5;
}
```

Well-formedness validation:
<!-- try_from protobuf -->

* Ensure that `ledger_info_with_sigs.ledger_info.transaction_accumulator_hash` is 32-byte.
* Ensure that `ledger_info_with_sigs.ledger_info.consensus_data_hash` is 32-byte.
* Ensure that `ledger_info_with_sigs.ledger_info.consensus_block_id` is 32-byte.
* TODO: we should only verify what we care about after.

<!-- LedgerInfoWithSignatures -->
When receiving an `UpdateToLatestLedgerResponse`, a client must first validate the `LedgerInfoWithSignatures` received. 
This step is important as the received `LedgerInfoWithSignatures` contains the root of all data structures.

* If `client_state.version` exists, ensure that `ledger_info_with_sigs.ledger_info.version >= client_state.version`.
* If `client_state.epoch_num` != `ledger_info_with_sigs.ledger_info.epoch_num` you are doomed. (TODO: what to do?)
* Ensure that the signatures in `LedgerInfoWithSignatures` forms a quorum with `client_state.validator_set`. (TODO: should we explain how?)
* Ensure that the number of `response_items` is the same as the number of items we requested.
* Ensure that `ledger_info_with_sigs.ledger_info.timestamp_usecs` is close to your time
    - this is missing from our implementation
    - how should we really verify timestamp? 30min window? 3s window?

If one of this step cannot be correctly validated, abort the process.
If all the steps are valid, and `ledger_info_with_sigs.ledger_info.version > client_state.version`:

* Set `client_state.version` to `ledger_info_with_sigs.ledger_info.version`.
* Set `client_state.transaction_root` to `ledger_info_with_sigs.ledger_info.transaction_accumulator_hash`.

Next, verify each `response_items` one by one. 
We specify in the next subsections how to validate each type of response.

Note that each of the following response handlers also has access to a relevant `matching_request` that contains the matching request.

### GetAccountStateResponse

```proto
message GetAccountStateResponse {
    AccountStateWithProof account_state_with_proof = 1;
}

message AccountStateWithProof {
  uint64 version = 1;
  AccountStateBlob blob = 2;
  AccountStateProof proof = 3;
}

message AccountStateBlob { bytes blob = 1; }

message AccountStateProof {
  AccumulatorProof ledger_info_to_transaction_info_proof = 1;
  TransactionInfo transaction_info = 2;
  SparseMerkleProof transaction_info_to_account_proof = 3;
}
```

<!-- try_from protobuf -->

Well-formedness check:

* (TODO: Ensure that all important fields are set)

<!-- account_state_with_proof.verify(ledger_info, ledger_info.version(), *address) -->

Check:

* Ensure `client_state.version == account_state_with_proof.version`
* Ensure `account_state_with_proof.proof.transaction_info.version <= client_state.version` (TODO: shouldn't it be `==`?)

Proofs:

* Compute `element_hash = HashTransactionInfo(account_state_with_proof.proof.transaction_info)` (TODO: how to do this?)
* Call `verify_accumulator_element(client_state.transaction_root, element_hash, client_state.version, account_state_with_proof.proof.ledger_info_to_transaction_info_proof)`. Abort if it returns `false`.

* Compute `accountAddressHash = HashAccountAddress(matching_request.GetAccountStateRequest.address)` (TODO: how to do this?)
* If `account_state_with_proof.blob.blob` exists,
    - compute `blobHash = HashAccountState(account_state_with_proof.blob.blob)`
    - Call `verify_sparse_merkle_element(account_state_with_proof.proof.transaction_info.state_root_hash, accountAddressHash, blobHash,  account_state_with_proof.proof.transaction_info_to_account_proof)`. Abort if it returns `false`.
* Otherwise, call `verify_sparse_merkle_element(account_state_with_proof.proof.transaction_info.state_root_hash, accountAddressHash, NULL,  account_state_with_proof.proof.transaction_info_to_account_proof)`. Abort if it returns `false`. 

Result:

* Parse the `account_state_with_proof.blob` (TODO: how?).

### GetAccountTransactionBySequenceNumberResponse

<!-- verify_get_txn_by_seq_num_resp(
            ledger_info,
            *account,
            *sequence_number,
            *fetch_events,
            signed_transaction_with_proof.as_ref(),
            proof_of_current_sequence_number.as_ref(),
        )
-->

```proto
message GetAccountTransactionBySequenceNumberResponse {
  // When the transaction requested is committed, return the committed
  // transaction with proof.
  SignedTransactionWithProof signed_transaction_with_proof = 2;
  // When the transaction requested is not committed, we give a proof that
  // shows the current sequence number is smaller than what would have been if
  // the transaction was committed.
  AccountStateWithProof proof_of_current_sequence_number = 3;
}
```

### GetEventsByEventAccessPathResponse

<!-- verify_get_events_by_access_path_resp(
            ledger_info,
            access_path,
            *start_event_seq_num,
            *ascending,
            *limit,
            events_with_proof,
            proof_of_latest_event,
        )
-->

```proto
message GetEventsByEventAccessPathResponse {
    // Returns an event and proof of each of the events in the request. The first
    // element of proofs will be the closest to `start_event_seq_num`.
    repeated EventWithProof events_with_proof = 1;

    // If the number of events returned is less than `limit` for an ascending
    // query or if start_event_seq_num > the latest seq_num for a descending
    // query,  returns the state of the account containing the given access path
    // in the latest state. This allows the client to verify that there are in
    // fact no extra events.
    //
    // The LedgerInfoWithSignatures which is on the main
    // UpdateToLatestLedgerResponse can be used to validate this.
    AccountStateWithProof proof_of_latest_event = 2;
}
```

### GetTransactionsResponse

<!-- verify_get_txns_resp(
            ledger_info,
            *start_version,
            *limit,
            *fetch_events,
            txn_list_with_proof,
        )
-->

```proto
message GetTransactionsResponse {
    TransactionListWithProof txn_list_with_proof = 1;
}
```

## Security Considerations

* different protobuffer implementations do different things
    - you might have to check if each field and submessage of each message is set before acting on it
* when I say values of 32 bytes, they are represented as bytearrays in protobuf, so you must verify that they are indeed 32 bytes

## Appendix

### Appendix A - VM Status Code

<!--
// The statuses and errors produced by the VM can be categorized into a
// couple different types:
// 1. Validation Statuses: all the errors that can (/should) be
//    the result of executing the prologue -- these are primarily used by
//    the vm validator and AC.
// 2. Verification Errors: errors that are the result of performing
//    bytecode verification (happens at the time of publishing).
// 3. VM Invariant Errors: errors that arise from an internal invariant of
//    the VM being violated. These signify a problem with either the VM or
//    bytecode verifier.
// 4. Binary Errors: errors that can occur during the process of
//    deserialization of a transaction.
// 5. Runtime Statuses: errors that can arise from the execution of a
//    transaction (assuming the prologue executes without error). These are
//    errors that can occur during execution due to things such as division
//    by zero, running out of gas, etc. These do not signify an issue with
//    the VM.
-->

|  ID  | STATUS                    |
|------|---------------------------|
| 0    | UNKNOWN_VALIDATION_STATUS |
| 1    | INVALID_SIGNATURE         |
| 2    | INVALID_AUTH_KEY          |
| 3    | SEQUENCE_NUMBER_TOO_OLD |
| 4    | SEQUENCE_NUMBER_TOO_NEW |
| 5    | INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE |
| 6    | TRANSACTION_EXPIRED |
| 7    | SENDING_ACCOUNT_DOES_NOT_EXIST |
| 8    | REJECTED_WRITE_SET |
| 9    | INVALID_WRITE_SET |
| 10   | EXCEEDED_MAX_TRANSACTION_SIZE |
| 11   | UNKNOWN_SCRIPT |
| 12   | UNKNOWN_MODULE |
| 13   | MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND |
| 14   | MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS |
| 15   | GAS_UNIT_PRICE_BELOW_MIN_BOUND |
| 16   | GAS_UNIT_PRICE_ABOVE_MAX_BOUND |
| 1000 | UNKNOWN_VERIFICATION_ERROR |
| 1001 | INDEX_OUT_OF_BOUNDS |
| 1002 | RANGE_OUT_OF_BOUNDS |
| 1003 | INVALID_SIGNATURE_TOKEN |
| 1004 | INVALID_FIELD_DEF |
| 1005 | RECURSIVE_STRUCT_DEFINITION |
| 1006 | INVALID_RESOURCE_FIELD |
| 1007 | INVALID_FALL_THROUGH |
| 1008 | JOIN_FAILURE |
| 1009 | NEGATIVE_STACK_SIZE_WITHIN_BLOCK |
| 1010 | UNBALANCED_STACK |
| 1011 | INVALID_MAIN_FUNCTION_SIGNATURE |
| 1012 | DUPLICATE_ELEMENT |
| 1013 | INVALID_MODULE_HANDLE |
| 1014 | UNIMPLEMENTED_HANDLE |
| 1015 | INCONSISTENT_FIELDS |
| 1016 | UNUSED_FIELD |
| 1017 | LOOKUP_FAILED |
| 1018 | VISIBILITY_MISMATCH |
| 1019 | TYPE_RESOLUTION_FAILURE |
| 1020 | TYPE_MISMATCH |
| 1021 | MISSING_DEPENDENCY |
| 1022 | POP_REFERENCE_ERROR |
| 1023 | POP_RESOURCE_ERROR |
| 1024 | RELEASEREF_TYPE_MISMATCH_ERROR |
| 1025 | BR_TYPE_MISMATCH_ERROR |
| 1026 | ABORT_TYPE_MISMATCH_ERROR |
| 1027 | STLOC_TYPE_MISMATCH_ERROR |
| 1028 | STLOC_UNSAFE_TO_DESTROY_ERROR |
| 1029 | RET_UNSAFE_TO_DESTROY_ERROR |
| 1030 | RET_TYPE_MISMATCH_ERROR |
| 1031 | FREEZEREF_TYPE_MISMATCH_ERROR |
| 1032 | FREEZEREF_EXISTS_MUTABLE_BORROW_ERROR |
| 1033 | BORROWFIELD_TYPE_MISMATCH_ERROR |
| 1034 | BORROWFIELD_BAD_FIELD_ERROR |
| 1035 | BORROWFIELD_EXISTS_MUTABLE_BORROW_ERROR |
| 1036 | COPYLOC_UNAVAILABLE_ERROR |
| 1037 | COPYLOC_RESOURCE_ERROR |
| 1038 | COPYLOC_EXISTS_BORROW_ERROR |
| 1039 | MOVELOC_UNAVAILABLE_ERROR |
| 1040 | MOVELOC_EXISTS_BORROW_ERROR |
| 1041 | BORROWLOC_REFERENCE_ERROR |
| 1042 | BORROWLOC_UNAVAILABLE_ERROR |
| 1043 | BORROWLOC_EXISTS_BORROW_ERROR |
| 1044 | CALL_TYPE_MISMATCH_ERROR |
| 1045 | CALL_BORROWED_MUTABLE_REFERENCE_ERROR |
| 1046 | PACK_TYPE_MISMATCH_ERROR |
| 1047 | UNPACK_TYPE_MISMATCH_ERROR |
| 1048 | READREF_TYPE_MISMATCH_ERROR |
| 1049 | READREF_RESOURCE_ERROR |
| 1050 | READREF_EXISTS_MUTABLE_BORROW_ERROR |
| 1051 | WRITEREF_TYPE_MISMATCH_ERROR |
| 1052 | WRITEREF_RESOURCE_ERROR |
| 1053 | WRITEREF_EXISTS_BORROW_ERROR |
| 1054 | WRITEREF_NO_MUTABLE_REFERENCE_ERROR |
| 1055 | INTEGER_OP_TYPE_MISMATCH_ERROR |
| 1056 | BOOLEAN_OP_TYPE_MISMATCH_ERROR |
| 1057 | EQUALITY_OP_TYPE_MISMATCH_ERROR |
| 1058 | EXISTS_RESOURCE_TYPE_MISMATCH_ERROR |
| 1059 | BORROWGLOBAL_TYPE_MISMATCH_ERROR |
| 1060 | BORROWGLOBAL_NO_RESOURCE_ERROR |
| 1061 | MOVEFROM_TYPE_MISMATCH_ERROR |
| 1062 | MOVEFROM_NO_RESOURCE_ERROR |
| 1063 | MOVETOSENDER_TYPE_MISMATCH_ERROR |
| 1064 | MOVETOSENDER_NO_RESOURCE_ERROR |
| 1065 | CREATEACCOUNT_TYPE_MISMATCH_ERROR |
| 1066 | MODULE_ADDRESS_DOES_NOT_MATCH_SENDER |
| 1067 | NO_MODULE_HANDLES |
| 1068 | POSITIVE_STACK_SIZE_AT_BLOCK_END |
| 1069 | MISSING_ACQUIRES_RESOURCE_ANNOTATION_ERROR |
| 1070 | EXTRANEOUS_ACQUIRES_RESOURCE_ANNOTATION_ERROR |
| 1071 | DUPLICATE_ACQUIRES_RESOURCE_ANNOTATION_ERROR |
| 1072 | INVALID_ACQUIRES_RESOURCE_ANNOTATION_ERROR |
| 1073 | GLOBAL_REFERENCE_ERROR |
| 1074 | CONTRAINT_KIND_MISMATCH |
| 1075 | NUMBER_OF_TYPE_ACTUALS_MISMATCH |
| 1076 | LOOP_IN_INSTANTIATION_GRAPH |
| 1077 | UNUSED_LOCALS_SIGNATURE |
| 1078 | UNUSED_TYPE_SIGNATURE |
| 2000 | UNKNOWN_INVARIANT_VIOLATION_ERROR |
| 2001 | OUT_OF_BOUNDS_INDEX |
| 2002 | OUT_OF_BOUNDS_RANGE |
| 2003 | EMPTY_VALUE_STACK |
| 2004 | EMPTY_CALL_STACK |
| 2005 | PC_OVERFLOW |
| 2006 | LINKER_ERROR |
| 2007 | LOCAL_REFERENCE_ERROR |
| 2008 | STORAGE_ERROR |
| 2009 | INTERNAL_TYPE_ERROR |
| 2010 | EVENT_KEY_MISMATCH |
| 3000 | UNKNOWN_BINARY_ERROR |
| 3001 | MALFORMED |
| 3002 | BAD_MAGIC |
| 3003 | UNKNOWN_VERSION |
| 3004 | UNKNOWN_TABLE_TYPE |
| 3005 | UNKNOWN_SIGNATURE_TYPE |
| 3006 | UNKNOWN_SERIALIZED_TYPE |
| 3007 | UNKNOWN_OPCODE |
| 3008 | BAD_HEADER_TABLE |
| 3009 | UNEXPECTED_SIGNATURE_TYPE |
| 3010 | DUPLICATE_TABLE |
| 3011 | VERIFIER_INVARIANT_VIOLATION |
| 4000 | UNKNOWN_RUNTIME_STATUS |
| 4001 | EXECUTED |
| 4002 | OUT_OF_GAS |
| 4003 | RESOURCE_DOES_NOT_EXIST |
| 4004 | RESOURCE_ALREADY_EXISTS |
| 4005 | EVICTED_ACCOUNT_ACCESS |
| 4006 | ACCOUNT_ADDRESS_ALREADY_EXISTS |
| 4007 | TYPE_ERROR |
| 4008 | MISSING_DATA |
| 4009 | DATA_FORMAT_ERROR |
| 4010 | INVALID_DATA |
| 4011 | REMOTE_DATA_ERROR |
| 4012 | CANNOT_WRITE_EXISTING_RESOURCE |
| 4013 | VALUE_SERIALIZATION_ERROR |
| 4014 | VALUE_DESERIALIZATION_ERROR |
| 4015 | DUPLICATE_MODULE_NAME |
| 4016 | ABORTED |
| 4017 | ARITHMETIC_ERROR |
| 4018 | DYNAMIC_REFERENCE_ERROR |
| 4019 | CODE_DESERIALIZATION_ERROR |
| 4020 | EXECUTION_STACK_OVERFLOW |
| 4021 | CALL_STACK_OVERFLOW |
| 4022 | NATIVE_FUNCTION_ERROR |
| -1   | UNKNOWN_STATUS      |

Where `-1` represents the maximum value possible for a uint64 type.


```
pub mod sub_status {

    pub const AEU_UNKNOWN_ARITHMETIC_ERROR: u64 = 0;
    pub const AEU_UNDERFLOW: u64 = 1;
    pub const AEO_OVERFLOW: u64 = 2;
    pub const AED_DIVISION_BY_ZERO: u64 = 3;

    // Dynamic Reference status sub-codes
    pub const DRE_UNKNOWN_DYNAMIC_REFERENCE_ERROR: u64 = 0;
    pub const DRE_MOVE_OF_BORROWED_RESOURCE: u64 = 1;
    pub const DRE_GLOBAL_REF_ALREADY_RELEASED: u64 = 2;
    pub const DRE_MISSING_RELEASEREF: u64 = 3;
    pub const DRE_GLOBAL_ALREADY_BORROWED: u64 = 4;

    // Native Function Error sub-codes
    pub const NFE_VECTOR_ERROR_BASE: u64 = 0;
}
```