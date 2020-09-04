---
id: libra-protocol
title: Libra Protocol: Key Concepts
---

The Libra Blockchain is a cryptographically authenticated distributed database based on the Libra protocol. This document briefly describes the key concepts of the Libra protocol. For a detailed description of all the elements of the Libra protocol, refer to the [Libra Blockchain technical paper](the-libra-blockchain-paper.md).

The Libra Blockchain is operated by a distributed network of [validator nodes](reference/glossary.md#validator-node), also known as validators. The validators collectively follow a [consensus protocol](reference/glossary.md#consensus-protocol) to agree on an ordering of finalized transactions in the blockchain.

The Libra testnet is a demonstration of an early prototype of the Libra Blockchain software — Libra Core.

## Transactions and States

At the heart of the Libra protocol are two fundamental concepts — transactions and states. At any point in time, the blockchain has a “state.” The state (or ledger state) represents the current snapshot of data on the chain. Executing a transaction changes the state of the blockchain.

![Figure 1.1 A Transaction changes state.](assets/illustrations/transactions.svg)
<small class="figure">FIGURE 1.1 TRANSACTIONS CHANGE STATE.</small>

Figure 1.1 represents the change of state of the Libra Blockchain that occurs when a transaction is executed. For example, at state S~N-1~, Alice has a balance of 110 LBR, and Bob has a balance of 52 LBR. When a transaction is applied to the blockchain, it generates a new state. To transition from S~N-1~ to S~N~, transaction T~N~ is applied against the state S~N-1~. This causes Alice’s balance to be reduced by 10 LBR and Bob’s balance to be increased by 10 LBR. The new state S~N~ now shows these updated balances. In figure 1.1:

- **A** and **B** represent Alice’s and Bob’s accounts in the blockchain.
- **S~N-1~** represents the (N-1)^th^ state of the blockchain.
- **T~N~** is the n-th transaction executed on the blockchain.
  - In this example, T~N~ is - “send 10 LBR from person A’s account to person B’s account.”
- **F** is a deterministic function. F always returns the same final state for a specific initial state and a specific transaction. If the current state of the blockchain is S~N-1~, and transaction T~N~ is executed on state S~N-1~, the new state of the blockchain is always S~N~.
- **S~N~** is the n-th state of the blockchain. S~N~ is an outcome of applying F to S~N-1~ and T~N~.

The Libra protocol uses the [Move language](move-overview.md) to implement the deterministic execution function F.

### Transactions

Clients of the Libra Blockchain submit transactions to request updates to the ledger state. A signed transaction on the blockchain contains:

- **Sender address** — Account address of the sender of the transaction.
- **Sender public key** — The public key that corresponds to the private key used to sign the transaction.
- **Program** — The program is comprised of the following:
  - A Move bytecode transaction script.
  - An optional list of inputs to the script. For a peer-to-peer transaction, the inputs contain the information about the recipient and the amount transferred to the recipient.
  - An optional list of Move bytecode modules to publish.
- **Gas price** (in specified currency/gas units) — The amount the sender is willing to pay per unit of [gas](reference/glossary.md#gas) to execute the transaction. Gas is a way to pay for computation and storage. A gas unit is an abstract measurement of computation with no inherent real-world value.
- **Maximum gas amount** — The maximum units of gas the transaction is allowed to consume.
- **Sequence number** — An unsigned integer that must be equal to the sequence number stored under the sender’s account.
- **Expiration time** — The time after which the transaction ceases to be valid.
- **Signature** — The digital signature of the sender.

The transaction script is an arbitrary program that encodes the logic of a transaction and interacts with resources published in the distributed database of the Libra Blockchain.

### Ledger State

The ledger state, or global state of the Libra Blockchain, is comprised of the state of all accounts in the blockchain. To execute transactions, each validator must know the global state of the latest version of the blockchain's distributed database. See [versioned database](#versioned-database).

## Versioned Database

All of the data in the Libra Blockchain is persisted in a single-versioned distributed database. A version number is an unsigned 64-bit integer that corresponds to the number of transactions the system has executed.

The versioned database allows validators to:

- Execute a transaction against the ledger state at the latest version.
- Respond to client queries about ledger history at both current and previous versions.

## Account

A Libra account is a container for Move modules and Move resources. It is identified by an [account address](reference/glossary.md#account-address). This essentially means that the state of each account is comprised of both code and data:

- **[Move modules](move-overview.md#move-modules-allow-composable-smart-contracts)** contain code (type and procedure declarations), but they do not contain data. The procedures of a module encode the rules for updating the global state of the blockchain.
- **[Move resources](move-overview.md#move-has-first-class-resources)** contain data but no code. Every resource value has a type that is declared in a module published in the distributed database of the blockchain.

An account may contain an arbitrary number of Move resources and Move modules.

#### Account Address

The address of a Libra account is a 16 byte value. Users can claim addresses using digital signatures. The account address is derived from a cryptographic hash of a user’s public verification key concatenated with a signature scheme identifier byte. Libra supports two signature schemes: Ed25519 and MultiEd25519 (for multi signature transactions). To sign a transaction sent from their account address, the user (or the custodial client representing the user) must use the private key corresponding to that account.

## Proof

All of the data in the Libra Blockchain is stored in a single-versioned distributed database. The storage is used to persist agreed upon blocks of transactions and their execution results. The blockchain is represented as an ever-growing [Merkle tree of transactions](reference/glossary.md#merkle-trees). A “leaf” is appended to the tree for each transaction executed on the blockchain.

- A proof is a way to verify the truth of data in the Libra Blockchain.
- Every operation stored on the blockchain can be verified cryptographically, and the resultant proof also proves that no data has been omitted. For example, if the client queried the latest _n_ transactions from an account, the proof verifies that no transactions are omitted from the query response.

In a blockchain, the client does not need to trust the entity from which it is receiving data. A client could query for the state of an account, ask whether a specific transaction was processed, and so on. As with other Merkle trees, the ledger history can provide an $O(\log n)$-sized proof of a specific transaction object, where _n_ is the total number of transactions processed.

## Validator Node (Validator)

Clients of the Libra Blockchain create transactions and submit them to a validator node. A validator node runs a consensus protocol (together with other validator nodes), executes the transactions, and stores the transactions and the execution results in the blockchain. Validator nodes decide which transactions will be added to the blockchain and in which order.
![Figure 1.1 Logical components of a validator.](assets/illustrations/validator.svg)
<small class="figure">FIGURE 1.2 LOGICAL COMPONENTS OF A VALIDATOR.</small>

A validator node contains the following logical components:

**Admission Control (AC)**

- Admission Control is the sole external interface of the validator node. Any request made by a client to the validator node goes to AC first.
- AC performs initial checks on the requests to protect the other parts of the validator node from corrupt or high volume input.

**Mempool**

- Mempool is a buffer that holds the transactions that are “waiting” to be executed.
- When a new transaction is added to a validator node’s mempool, this validator node’s mempool shares this transaction with the mempools of other validators in the system.

**Consensus**

- The consensus component is responsible for ordering blocks of transactions and agreeing on the results of execution by participating in the [consensus protocol](reference/glossary.md#consensus) with other validator nodes in the network.

**Execution**

- The execution component utilizes the virtual machine (VM) to execute transactions.
- Execution’s job is to coordinate the execution of a block of transactions and maintain a transient state that can be voted upon by consensus.
- Execution maintains an in-memory representation of the results of execution until consensus commits the block to the distributed database.

**Virtual Machine (VM)**

- AC and Mempool use the VM component to perform validation checks on transactions.
- VM is used to run the program included in a transaction and determine the results.

**Storage**

The storage component is used to persist agreed upon blocks of transactions and their execution results.

For information on interactions of each validator component with other components, refer to [Life of a Transaction](life-of-a-transaction.md).

## Reference

- [Welcome Page](welcome-to-libra.md).
- [My First Transaction](my-first-transaction.md) — Guides you through executing your very first transaction on the Libra Blockchain using the Libra CLI client.
- [Getting Started with Move](move-overview.md) — Introduces you to a new blockchain programming language called Move.
- [Life of a Transaction](life-of-a-transaction.md) — Provides a look at what happens “under the hood” when a transaction is submitted and executed.
- [Libra Core Overview](libra-core-overview.md) — Provides the concept and implementation details of the Libra Core components through READMEs.
- [CLI Guide](reference/libra-cli.md) — Lists the commands (and their usage) of the Libra CLI client.
- [Libra Glossary](reference/glossary.md) — Provides a quick reference to Libra terminology.
