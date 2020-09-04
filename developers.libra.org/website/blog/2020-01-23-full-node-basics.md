---
author: Libra Engineering Team
title: Full node basics: an introduction to full nodes in the Libra network
---

<script>
    let items = document.getElementsByClassName("post-meta");   
    for (var i = items.length - 1; i >= 0; i--) {
        if (items[i].innerHTML = '<p class="post-meta">January 23, 2020</p>') items[i].innerHTML = '<p class="post-meta">January 23, 2020</p>';
    }
    var slug = location.pathname.slice(location.pathname.lastIndexOf('/')+1);
    var redirect = 'https://libra.org/en-US/blog/' + slug;
    window.location = redirect;    
</script>

_**Note:** Libra Core software is still under development; the information and the terminology used in this document are subject to change._

This is the first post in a series on full nodes in the Libra network. Our goal for this post is to introduce the basic concepts associated with full nodes and talk about the motivation behind full nodes. A Libra node is a peer entity that participates in the Libra network and tracks the state of the Libra Blockchain. Libra clients (e.g., wallet applications) interact with the Libra Blockchain via Libra nodes. There are two types of nodes in the Libra network: full nodes and validator nodes. The Libra protocol can be configured to run either as a validator node or as a full node.

The functionality of being a validator and being a full node are related. For example, before joining the set of validators, a new validator will often run as a full node in order to synchronize to the blockchain. An entity that was formerly a validator, but has exited the validator set, might continue operation as a full node.

## Why are full nodes important?

- Full nodes ("FNs") enable external validation of blockchain correctness. FNs revalidate and replay all the transactions committed by validators when they synchronize to the current state of the blockchain. This allows full nodes to ensure that transactions have been executed according to the rules of the protocol.
- FNs make it easier to scale read traffic. They serve read queries from clients.
- FNs enable greater insight into the blockchain for third-party blockchain applications such as explorers, wallets, exchanges, and so on. If an entity (e.g., an exchange or a wallet) runs a full node, that entity can:
  - Get a consistent view of the Libra network
  - Avoid rate limitations on read traffic
  - Run custom analytics on historical data
  - Get notifications about particular on-chain events
- FNs protect validators from Distributed Denial of Service (DDoS) attacks.

## Who runs Libra full nodes?

It's important to understand that all full nodes have the same functionality and store the same data, irrespective of who runs them. The accuracy of the data stored by full nodes is authenticated by validators. So, who might run full nodes?

- A wallet or exchange might run a full node to handle traffic from its own internal services, avoiding the need to rely on an external service.
- A validator might run a full node to scale the operations of that validator.
- An entity might also decide to run a full node as a public service to:
  - Provide a scalable public read/write infrastructure
  - Provide a public history of the blockchain and safeguard the blockchain against forks and invalid protocol changes

## When would a Libra client interact with a full node?

Libra clients (e.g., wallet applications) use full nodes to access information from the Libra Blockchain. If a Libra validator operates a set of full nodes, a Libra client interacts with those full nodes instead of directly interacting with the validator itself.

- A full node can answer all read queries from clients without interacting with validators.
- A full node proxies write requests (e.g., transaction submissions) from clients to the validator associated with the full node.

## What's the difference between validator nodes and full nodes?

| **Validator nodes**                                                                                          | **Full nodes**                                                                                   |
| ------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| Are run by members of the validator set (i.e., Libra Association Members)                                    | Are run by anybody                                                                               |
| Validate transactions as part of the consensus process that is part of the operation of the Libra Blockchain | Validate transactions as a part of synchronizing to the Libra Blockchain from validators         |
| Are focused on writes — they accept transactions from clients and insert them into the Libra Blockchain      | Are focused on reads — they can handle queries from clients and provide scale for the validators |
| Provide signatures that authenticate the state of the Libra Blockchain                                       | Rely on validator signatures to prove the correctness of their results to clients                |
| Are only required to have knowledge of the current state of the blockchain                                   | Maintain historical information about the state of the blockchain                                |

_**Note**: A read query is a request to fetch the current state of the blockchain (e.g., account balance, last transactions, etc.). A write request is the submission of a new transaction._

## Does a full node stay synchronized with the blockchain?

A full node synchronizes to the latest state of the blockchain when:

- It comes online for the first time (bootstrap)
- It restarts
- It comes online after being offline for some time
- After a network partition heals

During normal FN workload, FNs synchronize with their upstream nodes continuously using the `StateSynchronizer` protocol.

## What's next?

In a future post in this series, we'll talk about running full nodes on the Libra network.
