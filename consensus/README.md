---
id: consensus
title: Consensus
custom_edit_url: https://github.com/libra/libra/edit/master/consensus/README.md
---

# Consensus

The consensus component supports state machine replication using the LibraBFT consensus protocol.

## Overview

A consensus protocol allows a set of validators to create the logical appearance of a single database. The consensus protocol replicates submitted transactions among the validators, executes potential transactions against the current database, and then agrees on a binding commitment to the ordering of transactions and resulting execution. As a result, all validators can maintain an identical database for a given version number following the [state machine replication paradigm](https://dl.acm.org/citation.cfm?id=98167). The Libra protocol uses a variant of the [HotStuff consensus protocol](https://arxiv.org/pdf/1803.05069.pdf), a recent Byzantine fault-tolerant ([BFT](https://en.wikipedia.org/wiki/Byzantine_fault)) consensus protocol, called LibraBFT. It provides safety (all honest validators agree on commits and execution) and liveness (commits are continually produced) in the partial synchrony model defined in the paper "Consensus in the Presence of Partial Synchrony" by Dwork, Lynch, and Stockmeyer ([DLS](https://groups.csail.mit.edu/tds/papers/Lynch/jacm88.pdf)) and mentioned in the paper ["Practical Byzantine Fault Tolerance" (PBFT)](http://pmg.csail.mit.edu/papers/osdi99.pdf) by Castro and Liskov, as well as newer protocols such as [Tendermint](https://arxiv.org/abs/1807.04938). In this document, we present a high-level description of the LibraBFT protocol and discuss how the code is organized. Refer to the [Libra Blockchain Paper](https://developers.libra.org/docs/the-libra-blockchain-paper) to learn more about how LibraBFT fits into the Libra protocol. For details on the specifications and proofs of LibraBFT, read the full [technical report](https://developers.libra.org/docs/state-machine-replication-paper).

Agreement on the database state must be reached between validators, even if
there are Byzantine faults. The Byzantine failures model allows some validators
to arbitrarily deviate from the protocol without constraint, with the exception
of being computationally bound (and thus not able to break cryptographic assumptions). Byzantine faults are worst-case errors where validators collude and behave maliciously to try to sabotage system behavior. A consensus protocol that tolerates Byzantine faults caused by malicious or hacked validators can also mitigate arbitrary hardware and software failures.

LibraBFT assumes that a set of 3f + 1 votes is distributed among a set of validators that may be honest or Byzantine. LibraBFT remains safe, preventing attacks such as double spends and forks when at most f votes are controlled by Byzantine validators &mdash; also implying that at least 2f+1 votes are honest.  LibraBFT remains live, committing transactions from clients, as long as there exists a global stabilization time (GST), after which all messages between honest validators are delivered to other honest validators within a maximal network delay $\Delta$ (this is the partial synchrony model introduced in [DLS](https://groups.csail.mit.edu/tds/papers/Lynch/jacm88.pdf)). In addition to traditional guarantees, LibraBFT maintains safety when validators crash and restart — even if all validators restart at the same time.

### LibraBFT Overview

In LibraBFT, validators receive transactions from clients and share them with each other through a shared mempool protocol. The LibraBFT protocol then proceeds in a sequence of rounds. In each round, a validator takes the role of leader and proposes a block of transactions to extend a certified sequence of blocks (see quorum certificates below) that contain the full previous transaction history. A validator receives the proposed block and checks their voting rules to determine if it should vote for certifying this block. These simple rules ensure the safety of LibraBFT — and their implementation can be cleanly separated and audited. If the validator intends to vote for this block, it executes the block’s transactions speculatively and without external effect. This results in the computation of an authenticator for the database that results from the execution of the block. The validator then sends a signed vote for the block and the database authenticator to the leader. The leader gathers these votes to form a quorum certificate that provides evidence of $\ge$ 2f + 1 votes for this block and broadcasts the quorum certificate to all validators.

A block is committed when a contiguous 3-chain commit rule is met. A block at round k is committed if it has a quorum certificate and is confirmed by two more blocks and quorum certificates at rounds k + 1 and k + 2. The commit rule eventually allows honest validators to commit a block. LibraBFT guarantees that all honest validators will eventually commit the block (and proceeding sequence of blocks linked from it). Once a sequence of blocks has committed, the state resulting from executing their transactions can be persisted and forms a replicated database.

### Advantages of the HotStuff Paradigm

We evaluated several BFT-based protocols against the dimensions of performance, reliability, security, ease of robust implementation, and operational overhead for validators. Our goal was to choose a protocol that would initially support at least 100 validators and would be able to evolve over time to support 500–1,000 validators. We had three reasons for selecting the HotStuff protocol as the basis for LibraBFT: (i) simplicity and modularity; (ii) ability to easily integrate consensus with execution; and (iii) promising performance in early experiments.

The HotStuff protocol decomposes into modules for safety (voting and commit rules) and liveness (pacemaker). This decoupling provides the ability to develop and experiment independently and on different modules in parallel. Due to the simple voting and commit rules, protocol safety is easy to implement and verify. It is straightforward to integrate execution as a part of consensus to avoid forking issues that arise from non-deterministic execution in a leader-based protocol. Finally, our early prototypes confirmed high throughput and low transaction latency as independently measured in [HotStuff]((https://arxiv.org/pdf/1803.05069.pdf)). We did not consider proof-of-work based protocols, such as [Bitcoin](https://bitcoin.org/bitcoin.pdf), due to their poor performance
and high energy (and environmental) costs.

### HotStuff Extensions and Modifications

In LibraBFT, to better support the goals of the Libra ecosystem, we extend and adapt the core HotStuff protocol and implementation in several ways. Importantly, we reformulate the safety conditions and provide extended proofs of safety, liveness, and optimistic responsiveness. We also implement a number of additional features. First, we make the protocol more resistant to non-determinism bugs, by having validators collectively sign the resulting state of a block rather than just the sequence of transactions. This also allows clients to use quorum certificates to authenticate reads from the database. Second, we design a pacemaker that emits explicit timeouts, and validators rely on a quorum of those to move to the next round — without requiring synchronized clocks. Third, we intend to design an unpredictable leader election mechanism in which the leader of a round is determined by the proposer of the latest committed block using a verifiable random function [VRF](https://people.csail.mit.edu/silvio/Selected%20Scientific%20Papers/Pseudo%20Randomness/Verifiable_Random_Functions.pdf). This mechanism limits the window of time in which an adversary can launch an effective denial-of-service attack against a leader. Fourth, we use aggregate signatures that preserve the identity of validators who sign quorum certificates. This allows us to provide incentives to validators that contribute to quorum certificates. Aggregate signatures also do not require a complex [threshold key setup](https://www.cypherpunks.ca/~iang/pubs/DKG.pdf).

## Implementation Details

The consensus component is mostly implemented in the [Actor](https://en.wikipedia.org/wiki/Actor_model) programming model &mdash; i.e., it uses message-passing to communicate between different subcomponents with the [tokio](https://tokio.rs/) framework used as the task runtime. The primary exception to the actor model (as it is accessed in parallel by several subcomponents) is the consensus data structure *BlockStore* which manages the blocks, execution, quorum certificates, and other shared data structures. The major subcomponents in the consensus component are:

* **TxnManager** is the interface to the mempool component and supports the pulling of transactions as well as removing committed transactions. A proposer uses on-demand pull transactions from mempool to form a proposal block.
* **StateComputer** is the interface for accessing the execution component. It can execute blocks, commit blocks, and can synchronize state.
* **BlockStore** maintains the tree of proposal blocks, block execution, votes, quorum certificates, and persistent storage. It is responsible for maintaining the consistency of the combination of these data structures and can be concurrently accessed by other subcomponents.
* **RoundManager** is responsible for processing the individual events (e.g., process_new_round, process_proposal, process_vote). It exposes the async processing functions for each event type and drives the protocol.
* **Pacemaker** is responsible for the liveness of the consensus protocol. It changes rounds due to timeout certificates or quorum certificates and proposes blocks when it is the proposer for the current round.
* **SafetyRules** is responsible for the safety of the consensus protocol. It processes quorum certificates and LedgerInfo to learn about new commits and guarantees that the two voting rules are followed &mdash; even in the case of restart (since all safety data is persisted to local storage).

All consensus messages are signed by their creators and verified by their receivers. Message verification occurs closest to the network layer to avoid invalid or unnecessary data from entering the consensus protocol.

## How is this module organized?

    consensus
    ├── src
    │   ├── block_storage          # In-memory storage of blocks and related data structures
    │   ├── consensusdb            # Database interaction to persist consensus data for safety and liveness
    │   ├── liveness               # Pacemaker, proposer, and other liveness related code
    │   └── test_utils             # Mock implementations that are used for testing only
    └── consensus-types            # Consensus data types (i.e. quorum certificates)
    └── safety-rules               # Safety (voting) rules
