# Mempool specification

## Abstract

This document describes the spec of a Diem node's mempool component,
which is responsible for accepting transaction submitted from other nodes
or directly to the node and relaying them to consensus.

This document is organized as follows:

1. Overview: high-level outline of the mempool architecture
2. Module Specifications: spec of mempool components
3. Network Protocol: the Mempool DiemNet protocol used to participate in the Diem
Payment Network
4. Abstracted Modules: external components that mempool interfaces with

## Overview

Mempool is a system component of the Diem node that is responsible for accepting,
validating, and prioritizing, and broadcasting pending transactions.

Mempool consists of a local memory-buffer that holds the transactions that
are waiting to be executed (core mempool) and a network stack (shared mempool)
that manages communication with other peers, as well as other system components.

The [JSON RPC](../../json-rpc/README.md) module proxies transactions to mempool.
Mempool validates transactions and stores it locally.

On the validator node, [consensus](../consensus/README.md) pulls ordered transactions
from mempool to form a block.

Mempool is notified by consensus or state sync once a transaction is fully executed and committed. Mempool then
drops this transaction from its local state.

A transaction remains in mempool at least until the earliest of the following:
* committed
* client-specified expiration time of the transaction is reached
* mempool-defined TTL is reached, so that no transaction stays in mempool for longer
than this TTL

## Module Specifications

This section (1) outlines the modules of mempool and their core
functionalities, and (2) defines some mempool terminology.
Note: the function signatures defined for each component's functionality
is part of a _sample_ interface not strictly required by the specification.

### Terminology

#### Transaction states in Mempool

A transaction in mempool can be (but not limited to):

* [validated](../move_adapter#Validation)

* consensus-ready: transaction has some probability of being included in the next block

  * transaction is validated
  * the sequence number of the transaction is the next sequence number of the sender account, or it is sequential to it
    * e.g. if for Alice's account, the committed sequence number if 3, and
    mempool contains transactions with sequence number 5, 6, 8, none of these transactions are consensus-ready.
    If later on mempool receives a transaction with sequence number 4,
    the only transactions with sequence numbers 4, 5, 6 are consensus-ready (but
    8 is not)

* broadcast-ready: transaction is eligible to be broadcasted to a peer node

  * transaction is consensus-ready; however, if transactions received from
  another validator peer (or peer on the same network) will *never*  be
  broadcast-ready, even if they are consensus-ready

### Core Mempool

#### Functionality: adding transaction to mempool
```rust
fn add_txn(&self, txn: SignedTransaction) -> MempoolStatus
```

Expected input:

* txn: Transaction to add to core mempool

Required behavior:

* adds the transaction to core mempool only if txn validation is successful and passes mempool checks
* returns status of adding transaction to mempool. We leave the definition
of various possible statuses open to implementation details.

#### Functionality: pulling a consensus block of transactions
```rust
fn get_block(&mut self, ...) -> Vec<SignedTransaction>
```

Required behavior:

* returns consensus-ready transactions
* if there are multiple consensus-ready transactions, transactions with higher gas are returned first

#### Functionality: fetching transactions ready for broadcast
```rust
fn read_timeline(&mut self, ...) -> Vec<SignedTransaction>
```

Required behavior:

* returns broadcast-ready transactions


#### Functionality: remove transaction from core mempool
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

#### Functionality: processing consensus requests
```rust
fn process_consensus_request(msg: ConsensusRequest)
```
Expected input:
* msg: a message from consensus. There can be two types:
    1. request for pulling a consensus block of transactions
    2. notification of rejected committed transactions

Required behavior:

* if `msg` is q get-block request, return a response with consensus-ready transactions
* if `msg` is notification of rejected txns, remove the transactions rejected from consensus from core mempool

```rust
fn process_state_sync_request(msg: CommitNotification)
```

Expected input:
* msg: a message containing:
    - transactions that were successfully committed
    - timestamp of committed block

Required behavior:

* remove successfully committed transactions
* clean out transactions with expiration time older than block timestamp
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

Mempool performs transaction signature verification and validations via the VM module.

```rust
fn validate_transaction(&self, txn: SignedTransaction)
```

* [validates](../move_adapter#Validation) a transaction

### Consensus

Consensus interacts with mempool by (1) requesting a block of transactions
ready for consensus, and (2) notifying mempool of rejected committed transactions
- see [here](#functionality:-processing-consensus-requests) for the required
behavior for this interaction. Otherwise, implementing and coordinating this
interaction is an implementation detail.

### State Sync

Mempool receives notification from state sync about committed transactions -
see [here](#functionality:-processing-state-sync-requests) for the required
behavior for this interaction. Otherwise, implementing and coordinating
this interaction is an implementation detail.

### Network

Mempool interacts with the networking module to send messages
defined in the [network protocol](#network-protocol) to peers in the network.

The following is a _sample_ interface that is not strictly required by the
specification.

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
        request_id: Vec<u8>,
        transactions: Vec<SignedTransaction>,
    },
    BroadcastTransactionsResponse {
        request_id: Vec<u8>,
        retry: bool,
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
  * `retry` - retry signal from recipient if there are txns in corresponding broadcast
  that were rejected from mempool but may succeed on resend
  * `backoff` - backpressure signal from ACK sender to broadcast request sender
