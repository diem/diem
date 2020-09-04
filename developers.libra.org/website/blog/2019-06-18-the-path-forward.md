---
author: Libra Engineering Team
title: Libra: The Path Forward
---

<script>
    var slug = location.pathname.slice(location.pathname.lastIndexOf('/')+1);
    var redirect = 'https://libra.org/en-US/blog/' + slug;
    window.location = redirect;
</script>

Today we are announcing the **Libra testnet**, a live demonstration of an early prototype of the technology behind [Libra](https://libra.org/) — a simple global currency and financial infrastructure that can empower billions of people.

**Libra Core** is the open-source implementation of the [Libra protocol](https://developers.libra.org/docs/the-libra-blockchain-paper) — the specification of how transactions are submitted, ordered, executed, and recorded within the Libra ecosystem. This is the first implementation of the Libra protocol and the [Move language](https://developers.libra.org/docs/move-paper). This post, the Libra developer website, and Libra Core are published by the [Libra Association](https://libra.org/en-us/non-profit-association), an independent, not-for-profit membership organization tasked with evolving the Libra ecosystem. 

### Design Philosophy

We designed the Libra Blockchain for security, scalability, and reliability, so consumers and businesses can depend on it from day one. In order to reach these goals in a reasonable time frame, we have made conservative choices initially, using well-understood technologies ranging from Merkle trees to pseudonymous transactions. However, we know that the needs of the Libra ecosystem will evolve over time, which motivates the creation of the Move language to provide flexibility for the blockchain. Together these approaches form the philosophy that has led to the design of a currency that is reserve-backed in order to minimize volatility, and a permissioned network that will transition to permissionless.

We know that, as of today, this technology can't meet every use case for every user. **But, this is just a starting point**. Because extensibility is baked into the heart of the Libra protocol, through hard work and a collaborative community effort, we are confident that the Libra ecosystem can fulfill the vision of becoming a financial infrastructure that empowers billions of people.

### A note About Versioning and protocol Evolution

**The Libra protocol and Libra Core APIs are not final**. As discussed above, one of the key tasks in evolving the prototype is formalizing the protocol and APIs. We welcome experimentation with the software in the testnet, but developers should expect that work may be required to publish applications using these APIs. As part of our regular communication, we will publish our progress toward stable APIs.

## Building the Libra Community

Over the past year, engineers from Facebook's Calibra team have designed an early prototype of Libra Core. The project is now being open sourced, and **governance of the project has transitioned to the Libra Association**.

The Libra Core project aims to follow in the footsteps of many successful open-source projects that start with and are driven by a core set of developers. In this early phase, we’ve adopted a simple process focused on rapid development; however, we are dedicated to evolving this process as the technology matures and the association expands. We have asked Calibra to manage the day-to-day development of Libra Core over the coming months, readying it for launch. That said, we are looking to build a collaborative community around Libra and will transition to an open and more formal process for making development decisions prior to the launch of the network. **No single company — including Facebook or Calibra — will have the ability to determine the future evolution of the Libra Blockchain.**

In order to meet the objectives of both maintaining the development velocity needed for launch and transitioning to open development, we are first beginning with a focus on **transparency**:

* Our code is open source, as will be the progress we make going forward.
* Discussions about the Libra ecosystem are moving to open locations, such as our [Discourse forum](https://libra.trydiscourse.com/) and GitHub issues. We look forward to collaborating on our plans, designs, and APIs.
* We will publish monthly updates on our progress and our upcoming plans.
* We will push a daily release of the testnet at 11 a.m. Pacific Time. Initially, this will reset the state of the testnet on each push. As the project stabilizes, we anticipate moving to a weekly cadence and preserving the content of the testnet at each release.

Over the coming months, we will continue to push forward a **framework for open development**:

* We will migrate to a fully open development model where all pull requests, issues, and design documents are developed publicly and collaboratively
* We are working on continuous integration, which will allow us to scale the management of pull requests.
* Association members will use the governance process to determine a framework for managing the development of the Libra Core software and the Libra protocol.

## **What's Next?**

Because this is an early prototype of our work, our focus so far has been building the core infrastructure behind the project. There is a long road ahead before the system is ready to launch. The project is under active development. In the coming months, you'll be able to see our progress and increasingly participate in creating the Libra ecosystem.

Below is a list of some major features we are hoping to implement this year, many of which are described in greater detail in our technical papers: 

### Libra Core

* Addressing:
    * We will formalize a specification for sharing payment addresses on the Libra Blockchain.
* Clients:
    * APIs — These APIs should provide ergonomic methods to meet real-world use cases, such as submitting transactions, accessing blockchain data, and monitoring for incoming payments. Possible approaches could include encouraging the use of a library, which acts as a light client or encouraging the use of RPCs to communicate with a process that runs the client.
* Consensus:
    * Increasing resiliency against liveness attacks — One advantage of the LibraBFT framework is that the correctness of the protocol is concentrated to a single software component — this is work which we have already completed. We plan to improve our resiliency against attacks on the liveness of the protocol by applying techniques, such as using more robust leader election mechanisms and enhancing inter-validator communication to increase the spread of information within the network. We have done an initial exploration of these and other mechanisms in the [LibraBFT paper](https://developers.libra.org/docs/state-machine-replication-paper) and are working to finalize our approach.
    * We will investigate the use of efficient signature aggregation to reduce the size of quorum certificates.
    * Mechanized proofs — We plan to start using mechanized proofs to verify our tech report and protocol claims.
* Move modules:
    * We will build modules that:
        * Manage the validator set (including staking, key rotation, and adding/removing validators) and integrate it into other system components, such as networking and consensus.
        * Track the supply of Libra coins in the system and allow the association to mint and burn Libra coins to keep the supply synchronized with the real-world reserve assets.
    * We will implement the Libra Investment Tokens as a Move resource.
    * We will implement cold wallets and multisig wallets that allow Libra users and association members to add extra security for their Libra coin and Libra Investment Token holdings. 
* Networking:
    * Full Nodes — Validator APIs to support full nodes (nodes that have a full replica of the blockchain but do *not* participate in consensus). This feature allows for the creation of replicas that can support scaling access to the blockchain and the auditing of the correct execution of transactions.
    * Gossip — Gossip-based approach to inter-validator communication may be necessary as the number of validator nodes grows.
    * Bootstrapping / Discovery — In the initial prototype, we have included placeholder implementations for finding the current validator set and bootstrapping the network. These components will need to be more fully formed before launch.
* Robust testing of real-world scenarios:
    * We will perform thorough testing of real-world scenarios that are likely to come up in a production environment, as well as ones that we hope never happen but need to be prepared for. These tests will include situations such as denial-of-service attacks, protocol upgrades, and the compromise of over one-third of the validator network.
    * We will prioritize projects to increase the resilience of the appropriate part of our infrastructure — for example, increasing our ability to withstand denial-of-service attacks by allowing for multiple admission control instances, ensuring that we have appropriate incentives to prevent excessive usage of storage on the blockchain, or creating run books for protocol upgrades.
* Security:
    * We will apply the “trusted computing base” (TCB) philosophy to security. This approach means taking important parts of the Libra Core software and ensuring that they have a minimal set of dependencies. We've already started to go down this route by designing the validator software into discrete components. We will continue down this route, ensuring that the essential components of Libra Core are isolated. For example, this means ensuring that the module that signs votes in the consensus protocol should be isolated from less critical components.
* Serialization:
    * We currently use Protocol Buffers as a storage format for transactions. While we've thought out the security implications of this design (e.g., we designed the system with the fact that Protocol Buffer serialization is not canonical), we are considering if using the canonical serialization framework we've used in other parts of the system might better suit our needs.
* Storage:
    * Pruning — We will allow a node to configure the pruning of historical storage — validators may prune past data aggressively while other nodes might keep a full replica.
* Research:
    * We know that to fulfill the mission of enabling a financial infrastructure that empowers billions of people we will need to address currently unresolved research challenges. Key research challenges include defining the path toward permissionless, security, and the usability of the blockchain.

### Libra Protocol

* Right now, Libra Core's implementation is a de facto specification for the protocol. As we finalize the design, we plan to turn this into a formal specification for the protocol. This will allow for easier auditing, design discussion, and other implementations.

### Move Language

* Events:
    * We will improve Move’s existing functionality for notifying clients about on-chain activity via events. We will expose a dedicated type for event streams, finalize the format of event access path, and add events for important system module events, such as validator set changes or minting/burning Libra.
* Generics and collections:
    * We will add parametric polymorphism with *kind* (resource or nonresource) constraints. This will allow us to write general-purpose business logic for managing resources (e.g., we can write a multisig wallet that can work for any kind of resource instead of only for Libra coin).
    * We will add container types, such as vectors and maps. Containers will greatly increase the expressivity of Move and are required to implement some core system modules (e.g., the validator-set module manages a collection of validators).
* Move developer experience:
    * We will improve the local development and testing experience for users of the Move intermediate representation (IR).
    * We will continue working on a format for specifying functional correctness properties of Move programs and an automated verification tool for Move bytecode. We will apply this pipeline to the core Move modules first, but design it with the intention of supporting verification of arbitrary user-defined specifications.
* Versioning:
    * We will design a versioning scheme for Move modules, resources, and transaction scripts. Currently, Move modules can never be upgraded or deleted. Sustainable software infrastructure must support updates and deletion.
