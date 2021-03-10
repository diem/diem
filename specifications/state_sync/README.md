# State Synchronizer specification

## Abstract

This document describes the spec of a Diem node's state synchronizer component,
which is responsible for synchronizing a node's local ledger to the latest state in
the Diem Payment Network.

This document is organized as follows:

1. Overview: high-level outline of the the state synchronizer design and
functionality.
2. Sync Protocols: specify technical requirements of various sync protocols
recognized in the Diem Network.
3. Network Protocol: the State Synchronizer DiemNet protocol used to participate
as a node in the Diem Network
4. Abstracted Modules: external components that state synchronizer interfaces with


## Overview

State Synchronizer is a system component that is responsible for syncing ledger
state among peers in the Diem Payment Network. Using state sync, a Diem node
can discover and update to a more recent ledger state if available with the following
general flow:
1. A node sends a [`GetChunkRequest`](#getchunkrequest) to a remote peer
2. If there is a more recent ledger state available on the remote
peer, it sends back a corresponding [`GetChunkResponse`](#getchunkresponse) with
transactions and corresponding proofs that the transactions have been processed
by consensus.
3. Using the response data, the requester advance its local ledger state
by re-executing the transactions.


There are three types of supported sync protocols:
1. [Validator sync](#validator-sync): In general, validators learn about recent
ledger state through their participation in the [consensus protocol](../specifications/consensus),
but it is possible they still fall behind (e.g. node goes offline for a while,
network partition). In these cases, a lagging validator can rely on state
synchronization to catch up to the more recent state.

2. [Full Node sync](#full-node-sync): As full nodes do not participate in consensus,
they rely only on the state synchronizer to discover and sync to a more recent
ledger state.

3. [Waypoint sync](#waypoint-sync): When a node starts up, it can specify a waypoint
to sync its local state to - see section 3.4 in [this paper](https://github.com/diem/diem/blob/main/documentation/tech-papers/lbft-verification/lbft-verification.pdf) for more context and
motivation for waypoint sync, as this specification primarily focuses on
the technical requirements for the waypoint sync protocol. Waypoint sync
happens at node initialization and must be completed before the node participates
in other state sync protocols.

A properly implemented State Synchronizer component supports each
sync protocol as both the requester and the responder. See [Sync Protocols](#sync-protocols)
for the technical requirements for each of these sync protocols.


## Sync Protocols

This section describes the technical requirements for the sync protocols
supported in the Diem Network.

For each sync protocol, we specify:
* requester: specs for the node requesting sync
* responder: specs for the node responding to a sync request
* per-protocol specifications: expected behavior in each step of the
sync protocol (sending/receiving request/response)

### Validator sync
* Requester: validator
* Responder: validator

#### Per-protocol Specifications

##### Sending request
* requester sends [`GetChunkRequest`](#getchunkrequest) with `target` set
to `TargetType::TargetLedgerInfo` to responder
* `LedgerInfoWithSignatures` in the target is provided by [consensus](#consensus)'s `sync_to` request

##### Process request/Sending response
* Using params in the request, responder fetches from [storage](#storage) a `TransactionListWithProof`
for a sequence of transactions whose:
    * first txn has version `known_version` + 1
    * max number of txns is `limit`
    * txns belong to the same epoch
* responder sends back [`GetChunkResponse`](#getchunkresponse) with `txn_list_with_proof`
set to above list of txns and `response_li` set to `ResponseLedgerInfo::VerifiableLedgerInfo`
containing the ledger info in the corresponding request's `target', or the ledger info
with signatures corresponding to the end of the epoch, if the txns belong to an earlier epoch

##### Receiving/verifying response
* Requester performs the following verifications:
    * start version of `txn_list_with_proof` == local state's version + 1
    * verifies the ledger info in `ResponseLedgerInfo::VerifiableLedgerInfo` against local state's trusted epoch
    * executes and stores the transactions via [execution](#execution)'s `ChunkExecutor::execute_and_commit_chunk` call
* If sync to target is finished, notify [consensus](#consensus) that its
`sync_to` request has been completed.
* If the sync is not complete yet, send another chunk request based on
the new local state per this sync protocol.

### Full node sync
* Requester: full node
* Responder: validator or full node

#### Per-protocol Specifications

##### Sending request
* requester sends [`GetChunkRequest`](#getchunkrequest) with `target` set
to `TargetType::HighestAvailable` to responder

##### Process request / send response
* Using params in the request, responder fetches from [storage](#storage) a `TransactionListWithProof`
for a sequence of transactions whose:
    * first txn has version `known_version` + 1
    * max number of txns is `limit`
    * txns belong to the same epoch
* responder sends back [`GetChunkResponse`](#getchunkresponse) with `response_li`
set to `ResponseLedgerInfo::ProgressiveLedgerInfo`
* Full-node long-poll protocol:
    * If the requested state is not yet available on the responder, the responder
    can cache the response for `timeout_ms`. If in that time inverval, the
    requested ledger state becomes available (due to the responder itself advancing
    its ledger state), send the corresponding chunk response for that request.

##### Receiving response
* Requester performs the following verifications:
    * start version of `txn_list_with_proof` == local state's version + 1
    * verifies all ledger infos in `ResponseLedgerInfo::ProgressiveLedgerInfo` against local state's trusted epoch
    * executes and stores the transactions via [execution's](#execution) `ChunkExecutor::execute_and_commit_chunk` call
* Send another chunk request based on the new local state per this sync protocol

### Waypoint sync
* Requester: validator or full node
* Responder: validator or full node

#### Per-protocol Specifications

##### Sending request
* requester sends [`GetChunkRequest`](#getchunkrequest) with target `TargetType::Waypoint`
    * version in `TargetType::Waypoint` matches the version of the waypoint
    provided at node initialization

##### Processing request / Sending response
* Using params in the request, responder fetches from [storage](#storage) a `TransactionListWithProof`
for a sequence of transactions whose:
    * first txn has version `known_version` + 1
    * max number of txns is `limit`
    * txns belong to the same epoch
* responder sends back [`GetChunkResponse`](#getchunkresponse) with `response_li`
set to `ResponseLedgerInfo::LedgerInfoForWaypoint`


##### Receiving response
* Requester performs the following verifications:
    * verifies the `waypoint_li` against the requester's waypoint (provided
    at node initialization)
    * start version of `txn_list_with_proof` == local state's version + 1
    * executes and stores the transactions via [execution's](#execution) `ChunkExecutor::execute_and_commit_chunk` call
* If sync to waypoint version is not complete after processing this response,
send another chunk request based on the updated ledger state following this protocol

## Network Protocol

This section outlines the technical details of state synchronizer
messages exchanged over the wire.

### GetChunkRequest

```rust
struct GetChunkRequest {
    known_version: u64,
    current_epoch: u64,
    limit: u64,
    target: TargetType
}
```

Fields:
* `known_version` - version of the requester's latest ledger state.
* `current_epoch` - epoch of the requester's latest ledger state. If the requester's state
has reached the end of an epoch `x`, the following request's `current_epoch` would be set to `x + 1`.
* `limit` - number of transactions requested
* `target` - synchronization target of this request. See [`TargetType`](#targettype)


### TargetType

Represents the synchronization target of a [`GetChunkRequest`](#getchunkrequest).
All `LedgerInfoWithSignatures` included here represents the ledger info with signatures
that proves the ledger state that the requester is requesting sync to.

```rust
enum TargetType {
    TargetLedgerInfo(LedgerInfoWithSignatures),
    HighestAvailable {
        target_li: Option<LedgerInfoWithSignatures>,
        timeout_ms: u64,
    },
    Waypoint(Version),
}
```

Subtypes:
* `TargetLedgerInfo` - `LedgerInfoWithSignatures` represents the ledger info with signatures
that proves the ledger state that the requester is requesting sync to
* `HighestAvailable`
    * `target_li` - set to `None` if there is no known ledger info with signatures to
set as a target, else false. (Usually `None` is set if it is the full node's first time sending a chunk request).
    * `timeout_ms` - the time interval the requester will wait after sending a chunk request
    with this target type before sending another chunk request.
* Waypoint - sync request for specified version. This version matches
the version of the waypoint provided at node initialization

### GetChunkResponse

Network message used as the response for a corresponding GetChunkRequest
The chunk of transactions included in a response all belong to the epoch specified
by the `known_epoch` of the corresponding request and never crosses epoch boundaries.

```rust
struct GetChunkResponse {
    response_li: ResponseLedgerInfo,
    txn_list_with_proof: TransactionListWithProof,
}
```

Fields:
* `response_li` - all proofs in the response are built relative to this ledger info. The specifics of ledger info verification depend on its type. See [ResponseLedgerInfo](#responseledgerinfo)
* `txn_list_with_proof` - chunk of transactions with proof corresponding to the ledger info carried by the response


### ResponseLedgerInfo

```rust
enum ResponseLedgerInfo {
    VerifiableLedgerInfo(LedgerInfoWithSignatures),
    ProgressiveLedgerInfo {
        target_li: LedgerInfoWithSignatures,
        highest_li: Option<LedgerInfoWithSignatures>,
    },
    LedgerInfoForWaypoint {
        waypoint_li: LedgerInfoWithSignatures,
        end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    }
}
```
 Subtypes:
 * VerifiableLedgerInfo - contains the `LedgerInfoWithSignatures` that the
 response's `txn_list_with_proof` can be verified against
 * ProgressiveLedgerInfo
    * `target_li` - `LedgerInfoWithSignatures` that the
 response's `txn_list_with_proof` can be verified against
    * `highest_li` - set to highest `LedgerInfoWithSignatures` in responder's local state if different from
    `target_li`, else `None`
 * LedgerInfoForWaypoint
    * `waypoint_li` - `LedgerInfoWithSignatures` for the ledger state at version
    specified in corresponding request's `TargetType::Waypoint`
    * `end_of_epoch_li` - optional `LedgerInfoWithSignatures` - if the chunk of
    transactions in `txn_list_with_proof` end at an epoch, the corresponding
    `LedgerInfoWithSignatures` for that end-of-epoch, else `None`


## Abstracted Modules

In this section, we describe external modules that interact with the state
synchronizer and the expected behavior.

### Network

State synchronizer interacts with the networking module to send messages
defined in the [network protocol](#network-protocol) to peers in the network.

The following is a _sample_ interface that is not strictly required by the
specification.

```rust
/// Sends `message` to `recipient`
fn send_to(recipient: PeerId, message: StateSyncMessage)
```

### Consensus

State synchronizer processes consensus's  [`StateComputer::sync_to` calls](https://github.com/diem/diem/tree/main/specifications/consensus#statecomputer)
by going through the [validator sync protocol](#validator-sync), and notifies
consensus once the ledger state has reached the version in the ledger info
in consensus' request.
Implementing and coordinating this interaction is an implementation detail.

### Mempool

State synchronizer notifies mempool of transactions that were successfully
executed and committed when processing chunk responses so that mempool
can clean out such transactions.
Implementing and coordinating this interaction is an implementation detail.

### Execution

State synchronizer uses execution's [`ChunkExecutor` interface](https://github.com/diem/diem/tree/main/specifications/trusted_computing_base/execution_correctness#executor-interface)
to execute and store transactions to storage

### Storage

State synchronizer interacts with storage to fetch a chunk of transactions
and the corresponding proofs as `TransactionListWithProof`.

The following is a _sample_ interface that is not strictly required by the
specification.

```rust
/// Returns a batch of transactions starting at `known_version` with
/// max size `limit` with proofs built against `target_version`
fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Result<TransactionListWithProof>
```
