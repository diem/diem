---
title: "Proof"
slug: "basics-merkle-proof"
hidden: false
---

A proof is a way to verify the truth of data in the Diem Blockchain.

All data in the Diem Blockchain is stored in a single-version distributed database. A validator node's [storage component](basics-validator-nodes.md#storage) is used to persist agreed upon blocks of transactions and their execution results. The blockchain is represented as an ever-growing [Merkle tree](..reference/glossary#merkle-trees). A “leaf” is appended to the tree for each transaction executed on the blockchain.

Every operation stored on the blockchain can be verified cryptographically. The resultant proof also proves that the validator nodes all agreed on the state at that time. For example, if the client queried the latest _n_ transactions from an account, the proof verifies that no transactions are added, omitted, or modified from the query response.

In a blockchain, the client does not need to trust the entity from which it is receiving data. A client could query for the state of an account, ask whether a specific transaction was processed, and so on. As with other Merkle trees, the ledger history can provide an $O(\log n)$-sized proof of a specific transaction object, where _n_ is the total number of transactions processed.
