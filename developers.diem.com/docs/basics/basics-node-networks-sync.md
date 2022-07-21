---
title: "Node networks and synchronization"
slug: "basics-node-networks-sync"
hidden: false
---
In this page, you will learn about:
* Types of node networks
* How and when a Diem node synchronizes to the latest state of the Diem Blockchain

## Node networks
Depending on their configuration, Diem nodes can form different peer-to-peer state synchronization networks. Each Diem node can participate in multiple networks as shown in the figure below.
* Validator node network
* Public FullNode network

Validator nodes and public FullNodes form a two-tiered architecture. A public FullNode network is not peer-to-peer, and it receives updates on new blocks only from the validator node it is connected to.

![v-fn-network.svg](/img/docs/v-fn-network.svg)
### Separate network stacks
For each type of Diem node, the Diem Payment Network (DPN) has a separate network stack. The advantages of having separate network stacks include:
* Clean separation between the different networks.
* Better support for security preferences (bidirectional vs server authentication).
* Allowance for isolated discovery protocols (on-chain discovery for validator node's public endpoint vs manual configuration for private organizations).

## Node synchronization
A Diem node synchronizes to the latest state of the Diem Blockchain when:
* It comes online for the first time (bootstrap).
* It restarts.
* It comes online after being offline for some time.
* When there is a network partition.
* FullNodes synchronize with their upstream nodes continuously during a normal workload.

### State synchronizer
Each Diem node contains a State Synchronizer (SS) component which is used to synchronize the state of a node to its upstream peers. This component has the same function for all types of Diem nodes. It utilizes the dedicated peer-to-peer network stack to perform synchronization and it uses a long-polling API.

The upstream peers that are used for synchronizing to the latest state of the blockchain are different for each type of node:
* Validator nodes use the validator node network.
* Public FullNodes can either use the initial set of peers or the validator nodes that are open for public access.

### Synchronization API
The Diem node's state synchronizer communicates with the upstream nodes' state synchronizers to get chunks of transactions to synchronize with the current state of the Diem Blockchain. Learn more about how this works in the specifications [here](https://github.com/diem/diem/tree/main/specifications/state_sync).
