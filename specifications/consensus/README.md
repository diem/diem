# Consensus specification

## Overview

The consensus specification describes the mechanism used by the Diem Payment Network (LPN) validators to agree on both ordering and transaction execution output under byzantine fault-tolerant (BFT) conditions - at most _f_ validators (where _f_ < (_all validators_)/3) are faulty or malicious. Currently, the consensus specification is an implementation of _DiemBFT_, a BFT consensus protocol that ensures liveness and safety under partial synchrony. The [DiemBFT whitepaper](https://developers.diem.com/docs/state-machine-replication-paper) describes a high level overview of the protocol, the liveness and safety proofs, and a rationale on why DiemBFT was adopted for the LPN. This document specifies how to implement the DiemBFT protocol in order to participate as a validating node in the LPN.

This document is organized as follows:

1. [Architecture](#Architecture) - the components of the specification and how they interact.
2. [Data structures](#Data-structures) - common data structures that are part of this specification.
3. [Network messages](#Network-messages) - consensus messages that are sent across the wire to other validators.
4. [Abstracted modules](#Abstracted-modules) - The components this specification depends on.
5. [Consensus modules](#Consensus-modules) - The components built upon common data structures that are described as a part of this specification.

All network communication occurs over DiemNet and any serialization, deserialization and hashing is determined by [BCS](https://docs.rs/bcs/).

## Architecture

TODO: Add picture

## Data structures

In this section, we specify data structures specific to consensus and how to verify the validity of these data structures - both for instantiation and acceptance from other validators. Any dependent data structures not described here are specified in the [common data structures document](../common/data_structures.md). Additionally, there is context specific verification of these data structures that is described in the [consensus modules](#Consensus-modules). For example, a proposed block is invalid and not accepted if its round is less than the current round of the receiving validator.

### Payload

The payload inside a block.

```rust
type Payload = Vec<SignedTransaction>;
```

### Block

Block extends `block_data` with a unique identifier and a signature for proposed blocks.

```rust
struct Block {
    id: HashValue,
    block_data: BlockData,
    signature: Option<Ed25519Signature>,
}
```

Fields:

* `id` is the unique identifier of this block. It is calculated with the BCS serialized hash of `block_data`. This field is not serialized but calculated whenever deserialized.
* `block_data` is the container of the block with zero or more transactions
* `signature` is populated if this block was proposed by a validator. If this block is a genesis or NIL block, it is `None`.

Verification

* if `signature` is set, ensure `block_data.block_type == BlockType::Proposal {author, ..}`
* `ed25519_verify(signature, block_data.hash(), OnChainConfig.validators().public_key(author))`

### BlockData

Block data is all the information necessary to vote on a block of transactions. `Block` wraps this structure as a convenient way storage of its BCS serialized hash and a signature (if this is proposed by a validator).

```rust
struct BlockData {
    epoch: u64,
    round: u64,
    timestamp_usecs: u64,
    quorum_cert: QuorumCert,
    block_type: BlockType,
}
```

Fields

* `epoch` is an u64 that identifies the number of reconfiguration operations that have occurred in the past before this block
* `round` is a round this block data is proposed in
* `timestamp_usecs` the approximate physical time a block is proposed by a proposer. This timestamp is used for

  * Time-dependent logic in smart contracts (the current time of execution)
  * Clients determining if they are relatively up-to-date with respect to the block chain.

  It makes the following guarantees:

  1. Time Monotonicity: Time is monotonically increasing in the block chain. (i.e. If H1 < H2, H1.Time < H2.Time).
  2. If a block of transactions B is agreed on with timestamp T, then at least f+1 honest validators think that T is in the past. An honest validator will only vote on a block when its own clock >= timestamp T.
  3. If a block of transactions B has a QC with timestamp T, an honest validator will not serve such a block to other validators until its own clock >= timestamp T.
  4. Current: an honest validator is not issuing blocks with a timestamp in the future. Currently we consider a block is malicious if it was issued more that 5 minutes in the future.

* `quorum_cert` is the quorum certified ancestor and whether the quorum certified ancestor was voted on successfully
* `block_type` is the type of the block which is specified below.

Verification

* ensure that if `BlockData` is received from the network that `block_type`, is not genesis. Genesis is only constructed from end-epoch LedgerInfo locally and is never sent to other validators.
* Assume that `parent_block` is the `BlockInfo` of `quorum_cert.vote_data.proposed`.

  * ensure `round` is greater than `parent_block.round`
  * ensure `epoch` is equal to `parent_block.epoch`
  * if `parent_block.next_epoch_state` is set, ensure `block_type == BlockType::NIL` or the payload in the `BlockType::Proposal` is empty
  * if a block is a nil block, or if `parent_block.has_reconfiguration`, ensure the block and the parent block have the same timestamp, otherwise ensure `timestamp_usecs` > `parent_block.timestamp_usecs`. In any case ensure that `timestamp_usecs` is not too far (5 minutes or more) in the future compared to the current time.

* ensure that if `quorum_cert.commit_info().next_epoch_state()` is set, a proposal should not be generated if the epoch ends, it should transition to next epoch.
* verify `quorum_cert`

### BlockType

There are 3 different types of consensus blocks and `BlockType` enumerates each one.

```rust
pub enum BlockType {
    Proposal {
        payload: Vec<SignedTransaction>,
        author: AccountAddress,
    },
    NilBlock,
    Genesis,
}
```

Fields

* `Proposal` blocks are proposed by a single validator according to the round and committed history.

  * `payload` is zero or more transactions proposed.
  * `author` of the block that can be validated by the author's public key and the signature

* `NilBlock` blocks don't have authors or signatures. They are generated when a validator reaches a timeout for a round without having already voted for a block. If >2f validators vote for a `NilBlock`, it can be added to a `QuorumCert` chain and reduce round gaps and improve commit latency.
* `Genesis` blocks are the first committed block in any epoch that is identically constructed on all validators by any (potentially different) LedgerInfo that justifies the epoch change from the previous epoch. The genesis block is used as the the first root block of the BlockTree for all epochs.

Verification

* See `BlockData`

### VoteData

This structure maintains information about a block and its parent block as a helper for `QuorumCert`. Its BCS-generated hash is used for the `consensus_data_hash` field in `LedgerInfo`.

```rust
pub struct VoteData {
    proposed: BlockInfo,
    parent: BlockInfo,
}
```

Fields:

* `proposed` contains the block info being voted on.
* `parent` contains the parent block info of the `proposed`.

Verification

* ensure parent.epoch == proposed.epoch
* ensure parent.round < proposed.round
* ensure parent.timestamp_usecs <= proposed.timestamp_usecs
* ensure parent.version <= proposed.version

### Vote

A vote is sent across the network to be aggregated by another validator such that a `QuorumCert` or a `TimeoutCertificate` can be created from >2f votes.

```rust
struct Vote {
    vote_data: VoteData,
    author: AccountAddress,
    ledger_info: LedgerInfo,
    signature: Ed25519Signature,
    timeout_signature: Option<Ed25519Signature>,
}
```

Fields:

* `vote_data` is the proposed and previous block and also contains the round information being voted on. The BCS-serialized hash of `vote_data` must be the same as `ledger_info.consensus_data_hash`
* `author` is the identify of the voter with respect to its account address
* `ledger_info` contains the ledger state of the block that will be committed if >2f unique `signatures` are gathered
* `signature` is the signature from the `author` on `ledger_info` that attests to the vote of the `vote_data.proposed` block, its parent and the `ledger_info`
* `timeout_signature` is optional and can contain a signature on a `Timeout` formed by `vote_data.proposed.epoch` and `vote_data.proposed.round`

Verification

* verify `vote_data`
* ensure `vota_data.hash() == ledger_info.consensus_data_hash`
* `ed25519_verify(signature, ledger_info.hash(), OnChainConfig.validators().public_key(author))`
* if `timeout_signature` is set, `ed25519_verify(timeout_signature, (epoch, round).hash(), OnChainConfig.validators().public_key(author))`

### QuorumCert

A quorum certificate is the aggregation of >2f votes on the same block info. It can be used as a proof to advance validators to the next round.

```rust
struct QuorumCert {
    vote_data: VoteData,
    signed_ledger_info: LedgerInfoWithSignatures,
}
```

Fields:

* `vote_data` is a helper structure that includes block information about a proposed block and its parent block. The BCS hash of `vote_data` must be equal to the `signed_ledger_info`.ledger_info.consensus_data_hash as a part of verification
* `signed_ledger_info` contains the [ledger state](../common/data_structures.md#LedgerInfoWithSignatures) of the committed block. Since a signature agrees on the hash of `vote_data`, it certifies the proposed block and its parent from `vote_data` as well as the highest committed state on this blockchain branch.

Derived Fields

* `certified_block` equals `vote_data.proposed`
* `parent_block` equals `vote_data.parent`
* `ends_epoch` is a bool refers to if `signed_leger_info.ledger_info.commit_info.next_epoch_state` is set

Verification

* verify `vote_data`
* ensure `vote_data.hash() == signed_ledger_info.ledger_info.consensus_data_hash`
* if `certified_block.round()` == 0 (it certifies genesis block and is generated via end-epoch LedgerInfo)

  * ensure certified_block == parent_block
  * ensure certified_block == signed_ledger_info.ledger_info.commit_info
  * ensure signed_leger_info.signatures is empty

* else `ed25519_verify(signature, signed_ledger_info.ledger_info.hash(), OnChainConfig.validators().public_key(author)) for (author, signature) in signed_ledger_info.signatures`
* ensure `len(signed_ledger_info.signatures >= len(OnChainConfig.validators()) * 2 / 3 + 1`

### Timeout

This structure represents an epoch and a round that a validator did not receive a proposal for and indicates that a validator would like to move to the next round. The timeout interval for a round depends on the timeout formula determined by the `RoundState`. If >2f validators signatures on this structure are collected into a `TimeoutCertificate`, it can be used as a proof to advance all validators to the next round.

```rust
struct Timeout {
    epoch: u64,
    round: Round,
}
```

Fields

* `epoch` is the epoch associated with the `round` that the validator thinks has passed without a proposal for the round
* `round` is the consensus round that has timed out from a validator's point of view

Verification

* See timeout_signature in `Vote`

### TimeoutCertificate

A TimeoutCertificate is the aggregation of >2f signatures on the same `Timeout`. It can be used as a proof to advance validators to the next round too. When there's both QuorumCert and TimeoutCert present for the same round, QuorumCert is preferred.

```rust
pub struct TimeoutCertificate {
    timeout: Timeout,
    signatures: BTreeMap<Author, Ed25519Signature>,
}
```

Fields

* `timeout` is the `Timeout` being signed.
* `signatures` contains >2f signatures signed on the BCS-serialized `timeout`.

Verification

* `ed25519_verify(signature, signed_ledger_info.ledger_info.hash(), OnChainConfig.validators().public_key(author)) for (author, signature) in signatures`
* ensure `len(signed_ledger_info.signatures >= len(OnChainConfig.validators()) * 2 / 3 + 1`

### SyncInfo

This structure provides verifiable certificates that can allow a validator to catch up to another validator. The max round _r_ of the `highest_quorum_cert` and `highest_timeout_cert` have already completed according to > 2f validators and convince any validator join round _r+1_ immediately if they are currently on a round < _r+1_. A correct implementation ensures that `highest_commit_cert` and `highest_quorum_cert` are on the same chain id. If a validator learns about more recent state upon receiving `SyncInfo`, then it should catch up through [BlockRetrievalRequest](#BlockRetrievalRequest), [EpochRetrievalRequest](#EpochRetrievalRequest) or [state synchronization](../state_synchronizer/state_sync_specification.md).

```rust
pub struct SyncInfo {
    highest_quorum_cert: QuorumCert,
    highest_commit_cert: Option<QuorumCert>,
    highest_timeout_cert: Option<TimeoutCertificate>,
}
```

Fields:

* `highest_quorum_cert` is the highest round `QuorumCert` known to the creator meaning that `highest_quorum_cert.vote_data.proposed` has the highest round of any `QuorumCert` known
* `highest_commit_cert` has the highest `LedgerInfo` committed to the blockchain, it's set to None if it's the same as `highest_quorum_cert`.
* `highest_timeout_cert` is the highest timeout certificate known which `highest_timeout_cert.timeout.round` is higher than `highest_quorum_cert.vote_data.proposed.round` (if available).

Derived Fields

* `highest_round` is the `max(highest_quorum_cert.certified_block().round(), highest_timeout_cert.map_or(0, |tc| tc.round))`
* `highest_commit_round` is the `highest_commit_cert.signed_ledger_info.ledger_info.commit_info.round`

Verification

* verify all fields independently
* ensure all certificates are in the same epoch, highest_quorum_cert.certified_block().epoch == highest_commit_cert.certified_block().epoch == highest_timeout_cert.epoch if set
* ensure highest_quorum_cert has higher certified round than highest_commit_cert if set
* ensure highest_commit_cert has commit, highest_commit_cert.signed_ledger_info.ledger_info.commit_info != BlockInfo::empty

## Network messages

In this section we specify all the consensus message types that validators send across network and the verification requirements upon deserialization of the data.

### ConsensusMsg

This is the wrapper around all types that are specified below and is the only message type sent by validators with respect to consensus.

```rust
enum ConsensusMsg {
    BlockRetrievalRequest(BlockRetrievalRequest),
    BlockRetrievalResponse(BlockRetrievalResponse),
    EpochRetrievalRequest(EpochRetrievalRequest),
    ProposalMsg(ProposalMsg),
    SyncInfo(SyncInfo),
    EpochChangeProof(EpochChangeProof),
    VoteMsg(VoteMsg),
}
```

### ProposalMsg

This is the networking message that is sent by a proposer to all validators for a particular round and epoch. It contains a `sync_info` in order to justify that the that proposed block should be voted on.

```rust
struct ProposalMsg {
    proposal: Block,
    sync_info: SyncInfo,
}
```

Fields

* `proposal` is the proposed block for execution and can be voted on by a receiving validator
* `sync_info` is the consensus state that justifies the `proposal`

Verification

* verify all fields independently
* ensure `proposal.block_data.quorum_cert == sync_info.highest_quorum_cert`
* ensure `proposal.block_data.round == sync_info.`[highest_round()](#highest_round)`+ 1`
* ensure `proposal.block_data.block_type == BlockType::Proposal`

### VoteMsg

This is the highest-level vote network message that is sent after either of the following events occur.

1. A proposal was received, executed, and then is ultimately voted on by this validator.
2. A local timeout occurred on this validator for a given round `r`. To represent this timeout, `vote.timeout_signature` is set. If the validator already voted on a proposal block for `r`, the `VoteMsg` will contain the same `vote_data` and `ledger_info` as the previous vote. Otherwise, it will include a NIL block in `VoteData`.

```rust
struct VoteMsg {
    vote: Vote,
    sync_info: SyncInfo,
}
```

Fields

* `vote` contains the proposed block voting state of this validator and if it has locally timed out for a round
* `sync_info` justifies the vote is on the correct round, specifically `vote.vote_data.proposed.round == sync_info.highest_round + 1`

Verification

* verify the `vote` according to the [Vote section](#vote)
* verify the `sync_info` according to the [SyncInfo section](#syncinfo)
* ensure all `epoch` fields in `vote` and `sync_info` are equal

### BlockRetrievalRequest

A validator will send this request for missing blocks from peers since it needs all block ancestors in order to execute and vote on a proposal block. It starts with by returning a particular block, followed by its ancestors up to a preferred chunk of blocks (i.e `num_blocks`).

```rust
pub struct BlockRetrievalRequest {
    block_id: HashValue,
    num_blocks: u64,
}
```

Fields

* `block_id` is the first block id requested in a tree of blocks
* `num_blocks` specifies how many blocks are to be returned starting from `block_id` and moving backward through the tree. It is an upper bound of the number of blocks retrieved as a server may return fewer or no blocks.

Verification

* None

### BlockRetrievalResponse

Response for `BlockRetrievalRequest`.

```rust
pub struct BlockRetrievalResponse {
    status: BlockRetrievalStatus,
    blocks: Vec<Block>,
}
```

Fields

* `status` indicates whether the request `Succeeded` or `IdNotFound` when the `block_id` not found locally or `NotEnoughBlocks` when the `block_id` is found but fewer than (`num_blocks`) are returned
* `blocks` contains the requested `num_blocks` if the `status` is `Succeeded`. Blocks will be returned starting the with `block_id` followed by their respective parent block.

Verification

* ensure `status` == Succeeded

  * TODO: Consider whether `status` == NotEnoughBlocks is also correct behavior. If the number of blocks to catch up exceeds a limit, a validator should rely on [state synchronization](../state_synchronizer/state_sync_specification.md) to catch up.

* verify each blocks independently
* verify the blocks form a chain where the first one's id == the `block_id` in BlockRetrievalRequest. `[blocks[i].block_data.quorum_cert.certified_block.id == blocks[i+1].id for i in 0..blocks.len()-1]`

### EpochRetrievalRequest

Request for proof that a validator should advance its epoch. A validator requests this after receiving a claim that a newer epoch is available.

```rust
pub struct EpochRetrievalRequest {
    pub start_epoch: u64,
    pub end_epoch: u64,
}
```

Fields

* `start_epoch` is epoch where the proof should start
* `end_epoch` is epoch where the proof should end

Verification

* None

### EpochChangeProof

A vector of `LedgerInfoWithSignatures`' with contiguous increasing epoch numbers to prove a sequence of epoch changes from the first `LedgerInfoWithSignatures`'s epoch.

```rust
pub struct EpochChangeProof {
    pub ledger_info_with_sigs: Vec<LedgerInfoWithSignatures>,
    pub more: bool,
}
```

Fields

* `ledger_info_with_sigs` is a vector of `LedgerInfoWithSignatures` with contiguous increasing epoch numbers to prove a sequence of epoch changes from the epoch of the first `LedgerInfoWithSignatures`.
* `more` indicates the sender is on a higher epoch than is returned from `ledger_info_with_sigs`.

  * TODO: Consider returning the actual higher epoch so EpochRetrievalRequest can be invoked again

Derived Fields

* `start_epoch` is the start epoch of the proof, `ledger_info_with_sigs.first().ledger_info.commit_info.epoch`
* `target` is the last one of the proof, `ledger_info_with_sigs.last()`

Verification

* ensure `ledger_info_with_sigs` is not empty
* ensure `start_epoch` matches the current epoch
* ensure every ledger info has a commit info with `next_epoch_state` set
* for li in ledger_info_with_sigs, follow a chain verification starting with epoch = current epoch, validators = OnChainConfig.validators(),

  * ensure `epoch == li.ledger_info.commit_info.epoch`
  * `ed25519_verify(signature, li.ledger_info.hash(), validators.public_key(author)) for (author, signature) in li.signatures`
  * ensure `len(li.signatures >= len(OnChainConfig.validators()) * 2 / 3 + 1`
  * assign `(epoch, validators) = li.ledger_info.next_epoch_state`

### SyncInfo Message

`SyncInfo` is piggybacked alongside `ProposalMsg` and `VoteMsg`. A standalone `SyncInfo` message is sent from a validator that has newer information than its recipient.

Verification

* Same as specified in [SyncInfo](#sync_info_data)

## Abstracted modules

In this section, we specify the abstracted external components consensus relies on. We only specify the interface and the expected behavior without implementation details.

Consensus has the following abstracted modules:

1. Network - sends network messages to peer validators
2. StateComputer - persist storage of ledger history and state as well as execute transactions
3. PersistentStorage - persist consensus internal data
4. OnChainConfig - configuration data from the ledger state e.g. validator set, vm versions.
5. BlockStore - in-memory storage for blocks

### Network

```rust
fn direct_send(peer: AccountAddress, msg: ConsensusMsg);
```

* send `msg` to the `peer` validator

```rust
fn rpc(peer: AccountAddress, msg: BlockRetrievalRequest) -> Result<BlockRetrievalResponse>;
```

* remote procedure call(rpc) to peer and expect a result or timeout error
* the only request/response type is for block retrieval now but may be extended in the future

```rust
fn broadcast(msg: ConsensusMsg);
```

* `direct_send` the msg to all peer validators in the current epoch i.e. `OnChainConfig.validators()`

### StateComputer

```rust
fn compute(block: Block) -> BlockInfo;
```

* if the parent block has reconfiguration i.e. `block.quorum_cert.certified_block().next_epoch_state` is set, return the same executed result as the parent block.

  ```rust
  BlockInfo {
        id: block.id,
        round: block.block_data.round,
        ..block.quorum_cert.certified_block(),
    }
  ```

* convert the block into a block metadata txn (TODO reference) and append at the front of all user transactions for execution
* execute all transactions on top of the parent's block state (may be pending to commit)
* return the executed result i.e. BlockInfo which captures the root hash of the new merkle tree and the `next_epoch_state`

```rust
fn commit(blocks: Vec<Block>, finality_proof: LedgerInfoWithSignatures);
```

* ensure `finality_proof.commit_info()` match the last block in `blocks` and its executed result
* ensure the `blocks` form a chain and the parent of the first is the current committed block
* commit and persist `blocks` and the proof `finality_proof`

```rust
fn sync_to(target: LedgerInfoWithSignatures);
```

* fetch all the missing transactions from peers until `target.commit_info().version`
* execute all the fetched transactions
* ensure the root hash of the merkle tree matches the `target.commit_info().executed_state_id`
* commit and persist all the transactions and the proof `target`
* if `target` is across multiple `epoch`s, fetch and persist all the end-epoch LedgerInfoWithSignatures

```rust
fn get_epoch_change_ledger_info(start_epoch: u64, end_epoch: u64) -> EpochChangeProof;
```

* construct the proof with end-epoch LedgerInfoWithSignatures from `start_epoch` to `min(end_epoch, start_epoch + MAX_NUM_EPOCH_CHANGE_LEDGER_INFO) - 1`
* set `more` to true if the proof's target epoch is lower than the current epoch

### PersistentStorage

Persistent layer for block_store and safety_rules.

### OnChainConfig

```rust
fn validators() -> ValidatorSet;
```

* return the current validator set which includes the account addresses and consensus public keys, it also includes network endpoint and keys but is irrelevant to consensus.

### Cryptography

```rust
fn hash<T>(value: T) -> HashValue;
```

* hash function (TODO: reference)

```rust
fn ed25519_sign(hash: HashValue) -> Result<Signature>;
```

* sign the hash with the consensus private key corresponding to the `OnChainConfig.validators().public_key(self.account_address)`
* if the expected key not found (e.g. key manager unreachable, account address not in the set, etc.), return an error.

```rust
fn ed25519_verify(signature: Signature, hash: HashValue, public_key: PublicKey) -> bool;
```

* verify if the `signature` is signed on the `hash` corresponding to the `public_key`

### BlockStore

In this section, we specify `BlockStore` which stores the pending blocks and quorum/timeout certificates. Blocks are organized as a tree structure where the root is the latest committed block, and follows the parent relationship in the `Block` data structure.

BlockStore should implement the following read functions:

```rust
fn root(&self) -> Block;
```

* the current root block of the block tree.

```rust
fn sync_info(&self) -> SyncInfo;
```

* the current highest certificates in the block tree.

```rust
fn get_block(&self, id: HashValue) -> Option<Block>;
```

* if a block with `id` is present, return it otherwise None.

```rust
fn path_from_root(&self, id: HashValue) -> Option<Vec<Block>>;
```

* if a block with `id` is present, returns all the blocks in (root, block] otherwise None.

BlockStore should implement the following write functions:

```rust
fn build (root: (ExecutedBlock, QuorumCert, QuorumCert), blocks: Vec<Block>, quorum_cert: Vec<QuorumCert>, timeout_cert: Option<TimeoutCertificate>) -> BlockStore;
```

* build block tree with the root block and the quorum cert/commit cert on the root block and timeout cert if available.
* `execute_and_insert` all the `blocks`
* `insert_single_quorum_cert` all the `quorum_cert`

```rust
fn execute_and_insert(&self, block: Block);
```

* ensure the `block.round` is ahead of root block's round
* ensure a block with id `block.parent_id` is present
* compute the block via state_computer
* persist the `block` via PersistentStorage
* ensure local timestamp is higher than `block.timestamp`
* insert the `block` and its compute result in block_tree

```rust
fn insert_single_quorum_cert(&self, quorum_cert: QuorumCert);
```

* ensure a block with `quorum.certified_block().id()` is present
* ensure the executed result matches `quorum.certified_block()`
* persist the `quorum_cert` via PersistentStorage
* insert the `quorum_cert` in block_tree

```rust
fn insert_timeout_certificate(&self, timeout_cert: TimeoutCertificate);
```

* ensure `timeout_cert.round` is higher than the current `timeout_cert.round` in block tree
* persist the `timeout_cert` via PersistentStorage
* replace the `timeout_cert` in block_tree with the new one

```rust
fn commit(&self, finality_proof: LedgerInfoWithSignatures);
```

* ensure a block with `finality_proof.commit_info().id()` is present (refer as committed block below)
* ensure the committed block has higher round than the current root block
* commit all the blocks from `path_from_root(committed_block.id())` via StateComputer
* prune the block tree to committed block

```rust
fn prune(&self, block_id: HashValue);
```

* ensure a block with `block_id` is present (refer as new root block)
* calculate the blocks that's not in the subtree of the new root block (refer as to-be-pruned blocks)
* delete to-be-pruned blocks and associated quorum certs via PersistentStorage
* delete to-be-pruned blocks from block_tree

```rust
fn insert_quorum_cert(&self, quorum_cert: QuorumCert);
```

* ensure a block with `quorum_cert.certified_block().id()` is present in block_tree otherwise initiate rpc calls with `BlockRetrievalRequest` to peers until one of the block's parent id is present in block_tree (fetch the chain of missing blocks)
* insert the fetched blocks in block_tree via `insert_single_quorum_cert` and `execute_and_insert_block` repeatively.
* if quorum_cert is a valid commit cert (commit info is not empty), `commit(quorum_cert.ledger_info())`

```rust
fn sync_to_highest_commit_cert(&self, quorum_cert: QuorumCert);
```

* ensure the quorum_cert is a valid commit cert (commit info is not empty)
* ensure the `quorum_cert.commit_info().id()` is not present in block_tree (otherwise it can go through `insert_quorum_cert`)
* ensure the `quorum_cert.commit_info().round()` is higher than the root
* initiate rpc calls with `BlockRetrievalRequest {block_id, num_blocks: 3}` (refer the rpc result as blocks[0..3], quorum_cert[0..3] where quorum_cert[i] certifies block[i])
* persist the rpc result via PersistentStorage
* `sync_to(quorum_cert.ledger_info)` via StateComputer.
* replace self with a new block_store constructed by `build(root: (block[0], quorum_cert[0], quorum_cert[2]), blocks[1..3], quorum_cert[1..3], timeout_cert: None, ..)`
* prune all blocks in the old block store via PersistentStorage

```rust
fn add_certs(&self, sync_info: SyncInfo);
```

* `sync_to_highest_commit_cert(sync_info.highest_commit_cert)`
* `insert_quorum_cert(sync_info.highest_commit_cert)`
* `insert_quorum_cert(sync_info.highest_quorum_cert)`
* `insert_timeout_certificate(sync_info.highest_timeout_cert)`

### Mempool

The mempool is a component that stores pending transactions.
Consensus pulls transactions from there in order to create blocks, when generating proposals.
It also notifies this component when transactions have been committed.

## Consensus modules

In this section, we define a number of modules relevant to the consensus protocol:

* safety rules, used to maintain safety in the protocol
* proposer election, used to figure out who the leader of an {epoch, round} tuple is
* proposal generator, used to generate proposals when a validator enters a round when they are the leader

### Safety rules

In this section, we specify the voting rules that are essential to the safety of the consensus. The reference implementation involves Trust Computing Base and is specified independently but is optional.

#### State

SafetyState is a per-epoch state which describes two values relates to the voting rules and should be persisted.

```rust
struct SafetyState {
    preferred_round: Round,
    last_vote_round: Round,
}
```

Fields

* `preferred_round`: for any quorum cert the validator received, the highest value of `quorum_cert.parent_block().round()`
* `last_voted_round`: for any vote the validator send out, the highest value of `vote.vote_data().proposed_block().round()`

It's initialized with 0.

```rust
InitialSafetyRules = SafetyState {
    preferred_round: 0,
    last_vote_round: 0,
};
```

#### Voting Rule

There's two voting rules in the consensus protocol:

1. A validator can only vote if the round of the proposal block > `last_vote_round`.
2. A validator can only vote if the round of the quorum cert that the proposal block extends from >= `preferred_round`.

#### 3-chain Commit Rule

* Vote for a proposal `block` is also a vote for commiting the grand parent (`block.quorum_cert().parent_block()`) if we have 3 contiguous rounds certified i.e. `block.round()` == `block.quorum_cert().proposed_block().round()+1` == `block.quorum_cert().parent_block().round()+2`

#### Vote construction

Generate a `VoteData` if we decide to vote for a proposal `block` (pass voting rule).

```rust
let vote_data = VoteData {
    proposed: block.block_info(), // executed result,
    parent: block.quorum_cert().certified_block(),
}
```

We check if this vote also triggers a commit (pass commit rule).

```rust
let commit_info = if commit_rule() {
    block.quorum_cert().parent_block()
} else {
    BlockInfo::empty()
}
```

Generate a LedgerInfo with bcs seiralize hash of vote_data and sign the bcs seriealize hash to generate a vote signature which captures both vote and commit.

```rust
let ledger_info = LedgerInfo {
    commit_info,
    consensus_data_hash: vote_data.hash(),
};
let signature = ed25519_sign(ledger_info.hash());
```

Finally we construct a `Vote` as following

```rust
Vote {
    vote_data,
    author: self.account_addres,
    ledger_info,
    signature,
    timeout_signature: None,
}
```

### ProposerElection

In this section, we specify `ProposerElection` interface and the `LeaderReputation` algorithm we choose.

#### Interface

```rust
fn get_valid_proposer(round: Round) -> AccountAddress;
```

* gieven a round, return the valid proposer for this round.

#### LeaderReputation

We use a committed history based `ProposerElection` implementation which provides bias towards successful proposers to improve transaction throughput and reduce commit latency.

Proposer candidates are all validators in the current epoch, represented as vector and ordered by account address.

For a given round `r`, we define `target_round = min(max(r - 4, 0), latest_committed_round)` and fetch up to `window_size` blocks since `target_round` (included).

For the fetched blocks, if a proposer candidate is a proposer of any block or a voter of the quorum cert in any block, it's given `active_weight` otherwise `inactive_weight`.

Use the first 8 bytes of sha3-256(round) as a random u64 (little endian encoding), the chosen weight is the random number % the sum of all proposers' weights.

The first proposer in the proposer candidates such that sum of the weights from 0 to its index is greater or equal to the chosen weight is the proposer in this round.

### Proposal Generator

In this section, we specify the `ProposalGenerator` module which is responsible to generate a proposal block if the validator is the proposer and a nil block if the round is timed out.

#### Proposal generation

```rust
fn generate_proposal(round: Round) -> Block;
```

* ensure the round is higher than the round of `block_store.highest_quorum_cert` (refer as hqc)
* ensure hqc is not the end of an epoch
* if `hqc.certified_block().has_reconfiguration()`
    - use an empty `payload`
    - use `hqc.certified_block().timestamp()` as `timestamp`
* otherwise
    - pull transactions from mempool to form a `payload`, excluding pending transactions already used (by calling `block_store.path_from_root(hqc.certified_block().id())`)
    - use the current UNIX time in microseconds as a `timestamp`
* construct a BlockData as following, and sign the bcs serialized hash with consensus key to construct a Block

  ```rust
  BlockData {
    epoch: hqc.certified_block().epoch(),
    round,
    timestamp,
    quorum_cert: hqc,
    block_type: BlockType {
        payload: txns,
        author: self.account_address,
    }
  }
  ```

#### NIL block generation

```rust
fn generate_nil(round: Round) -> Block;
```

* ensure the round is higher than the round of `block_store.highest_quorum_cert` (refer as hqc)
* ensure hqc is not the end of an epoch
* construct a BlockData as following and construct a Block without signature.

  ```rust
  BlockData {
    epoch: hqc.certified_block().epoch(),
    round,
    timestamp: hqc.certified_block().timestamp(),
    quoruom_cert: hqc,
    block_type: BlockType::NilBLock,
  }
  ```

## Consensus Protocol

### Consensus State

Consensus operates in units of `epoch`s. Each epoch has fixed, agreed on validator configuration (validator sets, vm version etc.), and progresses with increasing `round`s within an epoch.

#### RoundState

RoundState describes all information about a specific round. A validator enters round `r` by receiving valid a QuorumCert or TimeoutCertificate at round `r-1`.

```rust
struct RoundState {
  round: Round,
  committed_round: Round,
  pending_votes: HashMap<Author, Vote>,
  vote_sent: Option<Vote>,
  timeout: u64,
}
```

Fields

* `round`: the current round number.
* `committed_round`: the latest committed round number.
* `pending_votes`: all the votes received for this round.
* `vote_sent`: the vote we sent on this round.
* `timeout`: the round interval - how long this round lasts before a validator unilaterally decides to join a movement to the next round.

Transition to a new round.

```rust
fn new(sync_info: SyncInfo) -> RoundState {
    let round = sync_info.highest_round() + 1;
    let committed_round = sync_info.highest_committed_round();
    let timeout = BASE_INTERVAL * EXPONENT_BASE ^ min(MAX_EXPONENT, max(0, round - committed_round - 3));
    timer.expire(timeout).send(round);
    RoundState {
        round,
        committed_round,
        pending_votes: HashMap::new(),
        vote_sent: None,
        timeout,
    }
}
```

* enter new round with quorum/timeout certificates
* calculate timeout value as exponential of unhappy round gap
* start timer to signal a local timeout of `round` after `timeout`
* `timeout` should be reasonably large considering network latency and execution time, suggested value
    * BASE_INTERVAL = 1 sec
    * EXPONENT_BASE = 1.2
    * MAX_EXPONENT = 6

#### EpochState

EpochState describes all information about a specific epoch. Each validator enters epoch `e` by receiving a valid LedgerInfoWithSignatures at epoch `e-1` carrying non-empty `next_epoch_state`.

```rust
struct EpochState {
    epoch: u64,
    round_state: RoundState,
    block_store: BlockStore,
    proposer_election: ProposerElection,
    proposal_generator: ProposalGenerator,
    safety_rules: SafetyRules,
}
```

Fields

* `epoch`: the current epoch number.
* `round_state`: the current round state in this epoch.
* `block_store`: the current block store we have, specified in [block store](#blockstore).
* `proposer_election`: decides the proposer of each round, specified in [proposer election](#proposer-election).
* `proposal_generator`: decide how to propose a block of transactions, specified in [proposal generator](#proposal-generator).
* `safety_rules`: the voting and commit rule of DiemBFT, specified in [safety rules](#safety-rules).

The initial state is constructed with an initial `LedgerInfo` on a genesis transaction following the reconfiguration state transition.

### Starting Consensus - Initialization

TODO: Add detail

### Running Consensus - State transition

In this section we specify how the state transitions with events. There are 3 types of events in consensus:

1. Reconfiguration from StateComputer
2. ConsensusMsg from network
3. Timeout of a round from timer

#### Reconfiguration

Upon receiving a reconfiguration notification (end-epoch `LedgerInfo`) from `StateComputer`, a validator transitions to a new `´epoch` with a generated genesis block at round 0 and round state at round 1\. This is the only way to enter a new epoch.

```rust
fn reconfig(ledger_info: LedgerInfoWithSignatures, config: OnChainConfig) -> EpochState {
    let epoch = ledger_info.next_epoch_state().unwrap().epoch;
    EpochState {
        epoch: epoch,
        round_state: InitialRoundState,
        block_store: BlockStore::build(
            vec![make_genesis_block(ledger_info)],
            vec![certificate_for_genesis(ledger_info)],
            None, // timeout cert
        ), // generated genesis and the quorum cert (which is also commit cert) of it
        proposer_election: ProposerElection::new(OnChainConfig.validators()), // proposer candidates's account address
        safety_rules: InitialSafetyRules,
    }
}

fn make_genesis_block(ledger_info: LedgerInfoWithSignatures) -> Block {
    // a placeholder parent of the genesis
    let ancestor = BlockInfo::empty();
    let parent_qc = QuorumCert {
        vote_data: VoteData {
            proposed: ancestor,
            parent: ancestor,
        },
        signed_ledger_info: LedgerInfoWithSignatures {
            commit_info: ancestor,
            signatures: BTreeMap::empty(),
        },
    };
    let block_data = BlockData {
        epoch: ledger_info.commit_info().epoch + 1,
        round: 0,
        timestamp_usecs: ledger_info.commit_info().timestamp,
        parent_qc,
        block_type: BlockType::Genesis,
    };
    Block {
        id: block_data.hash(),
        block_data,
        signature: None,
    }
}

fn certificate_for_genesis(ledger_info: LedgerInfoWithSignatures) -> QuorumCert {
    let genesis_block_info = BlockInfo {
        epoch: ledger_info.commit_info().epoch + 1,
        round: 0,
        next_epoch_state: None,
        ..ledger_info.commit_info(),
    };
    let vote_data = VoteData {
        proposed: genesis_block_info,
        parent: genesis_block_info,
    };
    QuorumCert {
        vote_data,
        signed_ledger_info: LedgerInfoWithSignatures {
            LedgerInfo {
                commit_info: genesis_block_info,
                consensus_data_hash: vote_data.hash(),
            }
            signatures: BTreeMap::empty(),
        },
    }
}
```

* a validator is free to prune any data from the previous epoch in PersistentStorage after the transition to a new epoch.
* genesis block and its quorum cert/commit cert are locally generated from previous end-epoch LedgerInfoWithSignatures's **deterministic** fields
* the Diem network is bootstrapped with a genesis transaction (note the different from genesis block) and a ledger_info commits it with next_epoch_state set.

#### ConsensusMsg

State transitions upon receiving a message from network (`sender` of the message is also referred below), assuming all messages are verified. State transitions are event driven from either network messages or local timeouts.

##### Helper functions

```rust
fn update(&mut self) -> Result<()> {
    let local_sync_info = self.block_store.sync_info();
    self.safety_rules.update(local_sync_info.highest_quorum_cert());
    if local_sync_info.highest_round() + 1 > self.round_state.round {
        self.round_state = RoundState::new(
            local_sync_info.highest_round() + 1,
            local_sync_info.highest_commit_round()
        );
        if self.account_address == self.proposer_election.get_valid_proposer(self.round_state.round) {
            network.broadcast(ProposalMsg {
                proposal: self.proposal_generator.generate_proposal(self.round_state.round),
                sync_info: self.block_store.sync_info(),
            )
        }
    }
}
```

* update safety rules with new quorum cert.
* transition to a new RoundState if there's a QuorumCert or TimeoutCert with higher round.
* broadcast a new proposal with local sync info if the validator is the proposer for the new round.

```rust
fn ensure_epoch(&self, msg_epoch: u64) -> Result<()>{
    if self.epoch == msg_epoch {
        return Ok(());
    }
    if msg_epoch > self.epoch {
        network.direct_send(`sender`, EpochRetrievalRequest {
            start_epoch: self.epoch,
            end_epoch: msg_epoch,
            });
    }
    if msg_epoch < self.epoch {
        // sent back EpochChangeProof from msg_epoch to self.epoch
        network.direct_send(
            `sender`,
            StateComputer.get_epoch_change_ledger_info(msg_epoch, self.epoch));
    }
    bail!("Epoch doesn't match");
}
```

* this doesn't change the state but only sends network messages
* no-op if the msg epoch is the same as the current epoch
* try to retrieve the proof if the current epoch is behind to join the message epoch
* try to send the proof if the current epoch is ahead to convince the sender to join the current epoch
* if epoch is different, return an error and ignore the message.

```rust
fn ensure_round(&mut self, msg_round: u64, sync_info: SyncInfo) -> Result<()> {
    if self.round_state.round == msg_round {
        return Ok(());
    }
    let local_sync_info = self.block_store.sync_info();
    if local_sync_info.has_newer_certificates(sync_info) {
        network.direct_send(`sender`, local_sync_info);
    }
    if sync_info.has_newer_certificates(self.block_store.sync_info()) {
        self.block_store.add_certs(sync_info)?;
        self.update()?;
    }
    ensure!(msg_round == self.round_state.round, "Round doesn't match");
    Ok()
}
```

* this function ensures the round state is at the `msg_round` if `self.round < msg_round`
* no-op if it's already the same
* if the sync_info has newer certificates, try to sync the block_store to the state, and update safety rules and transition to new RoundState
* if the validator has newer certificates, try to send to `sender` to help it move forward.
* if the round doesn't match e.g. msg_round is behind, sync failed, return errors and ignore the message.

##### Processing a ProposalMsg

```rust
fn process_proposal_msg(&mut self, proposal_msg: ProposalMsg) -> Result<()> {
    let ProposalMsg {proposal, sync_info} = proposal_msg;
    self.ensure_epoch(proposal.epoch())?;
    self.ensure_round(proposal.round(), sync_info)?;
    let executed_block = self.block_store.execute_and_insert(proposal)?;
    if let Some(vote) = self.safety_rules.vote(executed_block) {
        self.round_state.save(vote);
        network.direct_send(
            self.proposer_election.get_valid_proposer(self.round_state.round + 1),
            VoteMsg {
                vote,
                sync_info: self.block_store.sync_info(),
        });
    }
}
```

* ensure and sync the epoch and round
* execute the block
* save and persist the vote in round state
* send the vote to next proposer if decide to vote

##### Processing a VoteMsg

This section documents the steps to follow after receiving a [VoteMsg](#votemsg).

- verify the validity of the VoteMsg according to the [VoteMsg section](#votemsg)
- ensure that the message is at the right epoch by calling `ensure_epoch(vote.vote_data.proposed.epoch)`
- ensure that the message is at the right round by calling `ensure_round(vote.vote_data.proposed.round, sync_info)`
- ensure that either:
  + this validator is the proposer of the next round  (TODO: not specified)
  + or the vote is a timeout vote (a timeout signature is set)
- if the author has voted for this round already (`pending_votes[sender]` is not empty):
  + ensure that the votes are equivalent (both `vote.ledger_info.hash()` are equal)
  + only process the new vote if it is a new timeout vote (if it contains a timeout signature and the previously seen one doesn't)
- store the vote under `pending_votes[sender]`
- if there's enough votes (_2f+1_) for the same `VoteData` and `LedgerInfo`, store the following [quorum certificate](#quorumcert) in the [block store](block_store.md) via `block_store.insert_quorum_cert()`:

  ```rust
  QuorumCert {
    vote_data,
    signed_ledger_info: LedgerInfoWithSignatures::V0(LedgerInfoWithV0{
      ledger_info,
      signatures,
    }),
  }
  ```

  where `vote_data` and `ledger_info` are the common values that the votes signed, and where `signatures` is a map of validator account addresses to their vote signatures.
- if there was no QC created and the new vote is a timeout vote (contains a timeout signature), check if there are now enough (_2f+1_) timeout signatures and store the following [timeout certificate](#timeoutcertificate) in the [block store](block_store.md) via `block_store.insert_timeout_certificate()`:

  ```rust
  TimeoutCertificate {
    timeout: Timeout{
      epoch,
      round,
    },
    signatures,
  }
  ```

  where `epoch` and `round` are the current epoch and round, and where `signatures` is a map of validator account addresses to their timeout vote signatures.
- if a QC or TC was created, transition to a new round if possible by calling `update()`

<details><summary>see pseudo-code equivalent</summary>

```rust
fn process_vote_msg(&mut self, vote_msg: VoteMsg) -> Result<()> {
    let VoteMsg{ vote, sync_info } = vote_msg;
    self.ensure_epoch(vote.vote_data().proposed().epoch())?;
    self.ensure_round(vote.vote_data().proposed().round(), sync_info)?;
    if vote.timeout_signature.is_none() {
        let next_proposer = get_proposer_for(vote.vote_data.proposed.epoch, vote.vote_data.proposed.round + 1);
        ensure!(next_proposer, self.address);
    }
    match self.round_state.pending_votes.insert(`sender`, vote) {
        QuorumCert(qc) => {
            self.block_store.insert_quorum_cert(qc),
            self.update();
        }
        TimeoutCert(tc) => {
            self.block_store.insert_timeout_certificate(tc),
            self.update();
        }
    }
}
```

</details>

##### Processing a SyncInfo

```rust
fn process_sync_info(&mut self, sync_info: SyncInfo) -> Result<()> {
    self.ensure_epoch(sync_info.highest_quorum_cert().certified_block().epoch())?;
    self.ensure_round(sync_info.highest_round() + 1, sync_info)?;
}
```

* ensure and sync the epoch and round
* this message is received because a peer thinks it has newer certificates than this validator.

##### Processing a EpochChangeProof

```rust
fn process_epoch_change_proof(&mut self, proof: EpochChangeProof) -> Result<()> {
    ensure!(self.epoch == proof.start_epoch, "Proof doesn't start at the current epoch");
    proof.verify()?;
    StateComputer.sync_to(proof.target());
}
```

* ensure the proof starts at the current epoch so that we're able to verify the chain of signatures.
* sync to the target ledger info (the last one in the proof)
* a reconfiguration notification will be sent via StateComputer to transition to a new epoch

##### Processing a BlockRetrievalRequest

```rust
fn process_block_retrieval(&self, request: BlockRetreivalRequest);
```

Read only request.

* find if request.block_id is present and `num_blocks` before it (included) exist in self.block_store
* construct BlockRetrievalResposne correspondingly and send back the rpc response to the `sender`

##### Processing a EpochRetrievalRequest

```rust
fn process_epoch_retrieval(&self, request: EpochRetrievalRequest);
```

Read only request.

* ensure `request.end_epoch <= self.epoch`
* construct EpochChangeProof via `StateComputer.get_epoch_change_ledger_info(request.start_epoch, request.end_epoch)` and send it back to the `sender`

##### Processing a BlockRetrievalResponse

Not expected, this message type is only expected as a rpc response corresponding to `BlockRetrievalRequest`.

#### Timeout

A `round` number is received when the timer setup in the constructor of `RoundState` expired.

* **`process_local_timeout(round: Round)`**

  * ensure that `round_state.round` matches `round`
  * reset a timer (a validator broadcasts its vote message every `timeout` interval until it moves to the next round

    ```rust
    timer.expire(round_state.timeout).send(round)
    ```

  * if no vote has been cached (`round_state.vote_sent` is empty), create one:

    * create a NIL block by calling `nil_block = proposal_generator.generate_nil_block(round)`
    * execute and store the block by calling `executed_block = block_store.execute_and_insert_block(nil_block)`
    * obtain a signed vote by calling `safety_rules.vote(executed_block)`
    * cache the vote in `round_state.vote_sent`

  * if the cached vote `round_state.vote` has an empty `timeout_signature`, set one:

    * create a `timeout` payload with the current epoch and round:

      ```rust
      Timeout {
        epoch: round_state.epoch,
        round: round_state.round,
      }
      ```

    * obtain a `digest` of the `timeout` by calling `SHA-3-256(bcs.serialize(timeout))`
    * obtain a `signature` over the digest by calling `ed25519_sign(private_key, digest)`
    * set the `round_state.vote.timeout_signature` to the `signature`.
    * cache the vote in `round_state.vote_sent`

  * construct a `vote_message` with the cached vote and the latest `sync_info`:

    ```rust
    VoteMsg {
      vote: round_state.vote_sent,
      sync_info: block_store.sync_info(),
    }
    ```

  * broadcast the vote message by calling `network.broadcast(vote_message)`

<details><summary>see pseudo-code equivalent</summary>

```rust
fn process_local_timeout(&mut self, round: Round) -> Result<()> {
    if self.round_state.round != round {
        return Ok(())
    }
    timer.expire(self.round_state.timeout).send(round);
    let vote = match self.round_state.vote_sent {
        Some(vote) => vote,
        None => {
            let nil_block = self.proposal_generator.generate_nil_block(round)?;
            let executed_block = self.block_store.execute_and_insert_block(nil_block)?;
            self.safety_rules.vote(executed_block)?
        }
    };
    if vote.timeout_signature.is_none() {
        let signature = ed25519_sign(Timeout {
            epoch: vote.vote_data().proposed().epoch(),
            round: vote.vote_data().proposed().round(),
        }.hash());
        vote.timeout_signature = Some(signature);
    }
    self.round_state.save(vote);
    network.broadcast(VoteMsg {
        vote,
        sync_info: self.block_store.sync_info(),
    })
}
```

</details>
