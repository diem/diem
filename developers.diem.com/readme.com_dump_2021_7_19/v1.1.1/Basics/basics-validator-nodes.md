---
title: "Validator nodes"
slug: "basics-validator-nodes"
hidden: false
metadata: 
  title: "Validator nodes"
  description: "Discover how validator nodes run the DiemBFT consensus protocol to execute and store transactions on the Diem Blockchain."
createdAt: "2021-02-23T00:17:19.743Z"
updatedAt: "2021-05-21T21:29:45.242Z"
---
A Diem node is a peer entity of the Diem ecosystem that tracks the <<glossary:state>> of the Diem Blockchain. Clients interact with the blockchain via Diem nodes. There are two types of nodes:
* Validator nodes
* <a href="doc:basics-fullnodes">FullNodes</a>

Each Diem node comprises several logical components:
* <<glossary:JSON-RPC service>> (disabled in validator nodes)
* [Mempool](doc:basics-validator-nodes#mempool) 
* [Consensus (disabled in FullNodes)](doc:basics-validator-nodes#consensus)
* [Execution](doc:basics-validator-nodes#execution)
* [Virtual Machine](doc:basics-validator-nodes#virtual-machine)
* [Storage](doc:basics-validator-nodes#storage) 
* [State synchronizer](doc:basics-validator-nodes#state-synchronizer) 

The <<glossary:Diem Core>> software can be configured to run as a validator node or as a FullNode.

## Introduction

When a transaction is submitted to the Diem Blockchain, validator nodes run a distributed <<glossary:consensus protocol>>, execute the transaction, and store the transaction and the execution results on the blockchain. Validator nodes decide which transactions will be added to the blockchain and in which order.

The Diem Payment Network uses a Byzantine Fault Tolerance (BFT) consensus protocol for validator nodes to agree on the ledger of finalized transactions and their execution. Validator nodes process these transactions to include them in the blockchain’s database, which they maintain. This means that validator nodes always have the current <<glossary:state>> of the blockchain.

A validator node communicates directly with other validator nodes over a hidden network. It may be configured to store either all or part of the historical data from the Diem Blockchain. <a href="doc:basics-fullnodes" target="_blank">FullNodes</a> are an external validation resource for finalized transaction history. They receive transactions from upstream nodes and then re-execute them locally (the same way a validator node executes transactions). FullNodes store the results of the re-execution to local storage. In doing so, they will notice and can provide evidence if there is any attempt to rewrite history. This helps to ensure that the validator nodes are not colluding on arbitrary transaction execution.

[block:callout]
{
  "type": "info",
  "body": "The DiemBFT consensus protocol provides fault tolerance of up to one-third of malicious validator nodes.",
  "title": "<<glossary:DiemBFT>>"
}
[/block]
## Validator node components

Each Diem node comprises several logical components:
* <<glossary:JSON-RPC service>> (disabled in validator nodes)
* [Mempool](doc:basics-validator-nodes#mempool) 
* [Consensus (disabled in FullNodes)](doc:basics-validator-nodes#consensus)
* [Execution](doc:basics-validator-nodes#execution)
* [Virtual Machine](doc:basics-validator-nodes#virtual-machine)
* [Storage](doc:basics-validator-nodes#storage) 
* [State synchronizer](doc:basics-validator-nodes#state-synchronizer) 


[block:image]
{
  "images": [
    {
      "image": [
        "https://files.readme.io/d5f1bfa-validator.svg",
        "validator.svg",
        267,
        150,
        "#f3f4fb"
      ]
    }
  ]
}
[/block]
### Mempool

Mempool is a validator node component that holds an in-memory buffer of transactions that have been submitted but not yet agreed upon and executed. This buffer is replicated between validator nodes. 

The JSON-RPC service of a FullNode sends transactions to a validator node's mempool. Mempool performs initial checks on the requests to protect the other parts of the validator node from corrupt or high volume input. When a new transaction passes the initial checks and is added to the mempool, it is then shared to the mempools of other validator nodes in the Diem Payment Network. 

When a validator node is the leader, its consensus component pulls the transactions from its mempool and proposes the order of the transactions that form a block. The validator quorum then votes on the proposal.

### Consensus

Consensus is the validator node component that is responsible for ordering blocks of transactions and agreeing on the results of execution by participating in the consensus protocol with other validator nodes in the network.

### Execution

Execution is a validator node component that coordinates the execution of a block of transactions and maintains a transient state. The consensus component votes on this transient state. The execution component maintains an in-memory representation of the execution results until the consensus component commits the block to the distributed database. 

The execution component uses the virtual machine to execute transactions. 

### Virtual machine

The virtual machine component is used to run the Move program included in a submitted transaction and determine the results. 

A validator node's mempool uses the virtual machine component to perform validation checks on transactions, while its execution component uses it to execute transactions. 


### Storage

The storage component is used to persist agreed upon blocks of transactions and their execution results.


### State synchronizer

Validator nodes use their state synchronizer component to “catch up” to the latest state of the blockchain.