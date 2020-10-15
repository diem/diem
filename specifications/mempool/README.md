# Mempool Specification

## Abstract

This document describes the spec of a Libra Node's mempool component (i.e. the minimum requirements for a component to function as a Libra node's mempool)

This document is organized as follows:

1. Overview: high-level outline of the mempool architecture
2. Module Specifications: spec of mempool components
3. Network Protocol: the Mempool LibraNet protocol used to participate as a node in the Libra Network
4. Abstracted Modules: external components that mempool interfaces with

## Overview

TODO diagram

Mempool is a system component of Libra Node that is responsible for accepting, validating, prioritizing and broadcasting pending transactions.

Mempool consists of local memory-buffer that holds the transactions that are waiting to be executed (core mempool) and network stack(shared mempool) that manages communication with other peers, as well as other system components.

[Client interface](https://github.com/libra/libra/blob/master/json-rpc/json-rpc-spec.md) module proxies transactions to mempool. Mempool validates tranasaction and stores it locally.

Mempool periodically shares new available transactions with upstream peers in its network.
This helps maintain a pseudo-global ordering. Communication with other peers is done via [LibraNet DirectSend protocol](../network/messaging-v1.md#protocol-directsend). Sender [broadcasts](#MempoolSyncMsg) batch of new transactions. Once receiver validates new batch, it will store them locally and issue back ACK request.

On validator node [consensus module](../consensus/spec.md) pulls ordered transactions from mempool to form a block.

Mempool is notified once transaction is fully executed and committed. Mempool then drops this transaction from its local state.



## Module Specifications

This section outlines the required modules of mempool and their core functionalities, and defines some mempool terminology.
Note: the function signatures defined for each component are not restrictive - more input/output parameters can be included depending on the implementation logic.
Also, the representation of the inputs/outputs is flexible, as long as the main information necessary is available in some equivalent format

### Terminology

#### Transactions states in Mempool

A transaction in mempool can be (but not limited to):

* validated: checked that the following conditions are met

  * the sequence number of the transaction is not older than the latest transaction in the blockchain for the same account
  * transaction has enough gas
  * transaction signature is valid
  * transaction has not expired at the time of validation
  * transaction payload is a valid Move script

* consensus-ready: transaction has some probability of being included in the next block

  * transaction is validated
  * the sequence number of the transaction is the next sequence number of the sender account, or it is sequential to it

* broadcast-ready: transaction is eligible to be broadcasted to a peer node

  * transaction is consensus-ready
  * transactions that a validator receives from another validator peer are not broadcast-ready, even if they are consensus-ready

### Core Mempool

```rust
fn add_txn(&self, txn: SignedTransaction, ...) -> MempoolStatus
```

Expected input:

* txn: Transaction to add to core mempool

Required behavior:

* adds the transaction to core mempool only if txn validation is successful and passes mempool checks
* returns status of adding transaction to mempool TODO describe MempoolStatus

```rust
fn get_block(&mut self, batch_size: u64, ...) -> Vec<SignedTransaction>
```

Expected inputs:

* `batch_size` - maximum number of transactions to return

Required behavior:

* returns maximum `batch_size` consensus-ready transactions
* if there are multiple consensus-ready transactions, transactions with higher gas are returned first

```rust
fn read_timeline(&mut self, count: usize, ...) -> Vec<SignedTransaction>
```

Expected inputs:

* `count` - maximum number of transactions to return

Required behavior:

* returns maximum `count` broadcast-ready transactions

```rust
fn remove_transaction(txn: SignedTransaction, is_rejected: bool)
```

Expected inputs:

* `txn` - transaction to remove from core mempool
* `is_rejected` - whether this transaction was rejected from consensus or not

Required behavior:

* removes the transaction from core mempool if it exists
* if `is_rejected`, all subsequent transactions for that account must be removed

### Shared Mempool

```rust
fn process_consensus_request(msg: ConsensusRequest)
```

Required behavior:

* if `msg` is `ConsensusRequest::GetBlockRequest`, return `ConsensusResponse::GetBlockResponse` with consensus-ready transactions
* if `msg` is `ConsensusRequest::RejectNotification`, remove the transactions rejected from consensus from core mempool and return `ConsensusResponse::CommitResponse`

```rust
fn process_state_sync_request(msg: CommitNotification)
```

Required behavior:

* remove successfully committed transactions
* clean out transactions with expiration time older than block tiemstamp
* send `CommitResponse` to callback

```rust
fn process_client_transactions(txn: SignedTransaction, callback)
```

Expected input:

* `txn` - transaction submitted by JSON RPC client proxied to mempool by JSON RPC service
* `callback` - callback to JSON RPC service

Required behavior:

* validate and submit transaction to mempool
* send back transaction submission result to callback

```rust
fn broadcast_single_peer(peer, peer_sync_data)
```

Expected input:

* `peer` - peer to broadcast to
* `peer_sync_data` - peer-specific information to determine which transactions to broadcast to this peer. This can contain information like the transaction broadcast history for this peer

required:

* crafts `MempoolSyncMsg::BroadcastTransactionsRequest` with broadcast-ready transactions for this peer
* network-sends MempoolSyncMsg to peer

```rust
fn process_transaction_broadcast(msg: MempoolSyncMsg)
```

Expected input:

* `MempoolSyncMsg::BroadcastTransactionsRequest`

Required behavior:

* validate and submit transactions to mempool
* craft corresponding `MempoolSyncMsg::BroadcastTransactionsResponse` and send to peer (via `network::send_to(peer, msg)`)

## Abstracted Modules

### VM

```rust
fn validate_transaction(&self, txn: SignedTransaction)
```

* [validates](####transactions-states-in-mempool) a transaction

### Consensus

Consensus interacts with mempool by sending the following requests:

```rust
pub enum ConsensusRequest {
    /// request to pull block to submit to consensus
    GetBlockRequest(
        // max block size
        u64,
        // transactions to exclude from requested block
        Vec<TransactionExclusion>,
        // callback to send response back to sender
        oneshot::Sender<Result<ConsensusResponse>>,
    ),
    /// notifications about *rejected* committed txns
    RejectNotification(
        // committed transactions
        Vec<CommittedTransaction>,
        // callback to send response back to sender
        oneshot::Sender<Result<ConsensusResponse>>,
    ),
}
```

Mempool sends consensus back these responses:

```rust
/// Response sent from mempool to consensus
pub enum ConsensusResponse {
    /// block to submit to consensus
    GetBlockResponse(
        // transactions in block
        Vec<SignedTransaction>,
    ),
    /// ACK for commit notification
    CommitResponse(),
}
```

### State Sync

State sync sends mempool the following request:

```rust
/// notification from state sync to mempool of commit event
/// This notifies mempool to remove committed txns
pub struct CommitNotification {
    /// committed transactions
    pub transactions: Vec<CommittedTransaction>,
    /// timestamp of committed block
    pub block_timestamp_usecs: u64,
    /// callback to send back response from mempool to State Sync
    pub callback: oneshot::Sender<Result<CommitResponse>>,
}
```

Mempool sends back to state sync the following ACK response:

```rust
/// ACK response to commit notification
pub struct CommitResponse {
    /// error msg if applicable - empty string if commit was processed successfully by mempool
    pub msg: String,
}
```

### Network

```rust
fn send_to(recipient: PeerId, message: MempoolSyncMsg)
```

* sends `message` to `recipient`

### JSON RPC

JSON RPC service proxies the transaction submission it receives to mempool with

1. transaction to submit
2. callback to send a transaction submission result to JSON RPC service

## Network Protocol

### MempoolSyncMsg

Network message used for broadcast of pending transactions to remote peers

```rust
pub enum MempoolSyncMsg {
    BroadcastTransactionsRequest {
        request_id: String,
        transactions: Vec<SignedTransaction>,
    },
    BroadcastTransactionsResponse {
        request_id: String,
        retry_txns: Vec<u64>,
        backoff: bool,
    },
}
```

Fields:

* `BroadcastTransactionsRequest` - represents broadcast request issued by the sender

  * `request_id` - unique id of sync request. Can be used by sender for rebroadcast analysis
  * `transactions` - shared transactions in this batch

* `BroadcastTransactionsResponse` - used by receiver to notify sender that `BroadcastTransactionsRequest` with the matching request_id was received, but does not provide any indication of whether the transactions passed remote validation and/or will be forwarded further

  * `request_id` - id of the corresponding `BroadcastTransactionsRequest` of this response
  * `retry_txns` - list of indices in the `transactions` broadcasted in the corresponding `BroadcastTransactionsRequest` that did not enter mempool but may succeed if retried later
  * `backoff` - backpressure signal from ACK sender to sync requester

The only requirement for `request_id` is that it is unique across all broadcasts for a given peer recipient.
