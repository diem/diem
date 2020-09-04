---
id: welcome-to-libra
title: Welcome
---

Welcome to the Libra developer site! The Libra Association’s mission is to enable a simple global payment system and financial infrastructure that empowers billions of people.

<blockquote>
The world needs a reliable and interoperable payment system that can deliver on the promise of “the internet of money.” Securing your financial assets on your mobile device should be simple and intuitive. Moving money around globally, and in a compliant way, should be as easy and cost-effective as — and even safer and more secure than — sending a message or sharing a photo, no matter where you are, what you do, or how much you earn. New product innovation and additional entrants lower barriers to access and facilitate frictionless payments for more people.
- <a href="https://libra.org/en-us/whitepaper">Libra White Paper</a>
</blockquote>

The Libra payment system is built on a secure, scalable, and reliable blockchain. It is backed by a reserve of high-quality liquid assets comprising cash or ca​sh eq​uivalents and very short-term government secu​rities. This will help ensure that people and businesses have confidence that their Libra Coins can be converted into their local currency. It is governed by the Libra Association and its subsidiary Libra Networks, tasked with developing and operating the Libra network and the Libra project.

<blockquote>
The goal of the Libra Blockchain is to serve as a foundation for financial services, including a new global payment system that meets the daily financial needs of billions of people. Through the process of evaluating existing options, we decided to build a new blockchain based on the following three requirements:
<ul>
  <li>Able to scale to billions of accounts, which requires high transaction throughput, low latency, and an efficient, high-capacity storage system.</li>
  <li>Highly secure to ensure the safety of funds and financial data.</li>
  <li>Flexible, so that it can power future innovation in financial services.</li>
</ul>
— <a href="https://libra.org/en-us/whitepaper">Libra White Paper</a>
</blockquote>

The Association has open-sourced an early preview of the Libra [testnet](https://developers.libra.org/docs/reference/glossary#testnet), with accompanying documentation. The testnet is still under development, but you can read, build, and provide feedback right away. In contrast to the forthcoming Libra mainnet, the testnet merely simulates a digital payment system and the coins on the testnet have _no real-world value_.

The documentation discusses:

- Where to learn about new technology, such as the Libra protocol, the Move language, and the **Libra Byzantine Fault Tolerance (LibraBFT) consensus** protocol.
- How to experiment with the prototype firsthand by [sending transactions](https://developers.libra.org/docs/my-first-transaction) to the testnet.
- How to be part of the community built around this new payment system.

<blockquote class="block_note">
Note: The Libra protocol and APIs are not final. One of the key tasks in evolving the prototype is formalizing the protocol and APIs. We welcome experimentation with the software on the testnet, but developers should expect that protocols and APIs may change. As part of our <a href="https://libra.org/en-US/blog/">regular communication</a>, we will publish our progress towards stable APIs. You can also stay up to date on the latest developments by signing up for our developer newsletter <a href="https://developers.libra.org/newsletter_form">here</a>
</blockquote>

### The Libra Protocol

The Libra protocol implements a cryptographically authenticated database to record accounts and their balances. The database stores a ledger of programmable resources, such as Libra Coins.

The Libra Blockchain uses a new smart contract language called Move, which was developed specifically for the Libra network. To allow for the flexibility to meet new requirements over time, we chose to implement as much of the Libra protocol as possible in Move, leading to fast, easy, and secure development.

The database is maintained by a distributed network of validator nodes that follow the LibraBFT consensus protocol. The protocol can tolerate up to one-third of the validator nodes being compromised and still guarantee consistency in processing transfers of Libra Coins. As part of the LibraBFT protocol, the validator nodes generate cryptographic signatures, attesting to the state of the Libra Blockchain. The Libra Blockchain uses a Merkle tree data structure to allow any user, anywhere in the world, to combine the cryptographic signatures of the validator nodes with a small piece of data — known as a “proof” — to get an authenticated record of any transaction on the Libra Blockchain, knowing that the transaction can never be changed or reversed.

### Move: A new blockchain programming language

Move is a new programming language for implementing transaction logic and “smart contracts” on the Libra Blockchain. Because the goal of the Libra project is to one day serve billions of people, Move is designed with safety and security as the highest priorities.

Move takes insights from past security incidents with smart contracts and creates a language that makes it inherently easier to write code that fulfills the author’s intent. This lessens the risk of unintended bugs or security incidents. Specifically, Move is designed to prevent assets from being cloned. It enables “resource types” that constrain digital assets to the same properties as physical assets: a resource has a single owner, it can only be spent once, and the creation of new resources is restricted.

The Move language makes the development of critical transaction code easier. It enables the secure implementation of the Libra project’s governance policies, such as the management of Libra Coins and the network of validator nodes. We anticipate that the ability for developers to create contracts will be available over time. This will support the evolution and validation of Move.

You can refer to [Getting Started With Move](https://developers.libra.org/docs/move-overview) for further information.

### Byzantine Fault Tolerance (BFT) consensus approach

The Libra payment system uses a BFT [consensus protocol](https://developers.libra.org/docs/reference/glossary#consensus-protocol) to form agreement among [validator nodes](https://developers.libra.org/docs/reference/glossary#validator-node) on the ledger of finalized transactions and their execution. The LibraBFT [consensus protocol](https://developers.libra.org/docs/reference/glossary#consensus-protocol) provides fault tolerance of up to one-third of malicious validators.

Each validator node maintains the history of all the transactions on the blockchain. Internally, a validator node needs to keep the current state to execute transactions and to calculate the next state. You can learn more about the logical components of a validator node in [Life of a Transaction](https://developers.libra.org/docs/life-of-a-transaction).

In addition to validator nodes, the Libra network will have full nodes that verify the history of the chain. The full nodes can serve queries about the blockchain state. They additionally constitute an external validation resource of the history of finalized transactions and their execution. They receive transactions from upstream nodes and then re-execute them locally (the same way a validator executes transactions). Full nodes store results of re-execution to local storage. In doing so, full nodes will notice and can provide evidence if there is any attempt to rewrite history. This helps ensure that the validators are not colluding on arbitrary transaction execution.

### Developers

The Libra project welcomes a wide variety of developers, ranging from people who contribute to Libra protocol to those who build applications that use the blockchain. The term “developer” encompasses all of these groups. Developers might:

- Build a local instance of the Libra network.
- Build applications to interact with the Libra network.
- Write smart contracts to execute on the blockchain.
- Contribute to the Libra Blockchain software.

### Getting Started

The Libra repository contains a command-line interface (CLI) for submitting transactions to the testnet. My First [Transaction](https://developers.libra.org/docs/my-first-transaction) guides you through executing your first transaction on the Libra Blockchain using the Libra CLI client. The CLI allows a participant to construct, sign, and submit transactions to a [validator node](https://developers.libra.org/docs/reference/glossary#validator-node). Similarly, it allows a participant to issue queries to the Libra Blockchain (through the validator node or a full node), request the status of a transaction or account, and verify the response.

<blockquote class="block_note">
Note: While all developers are free to use the tesnet, mainnet will follow a phased rollout plan. Initially, the network will only be accessible to <a href="https://libra.org/en-US/white-paper/#lexicon">Designated Dealers</a> and <a href="https://libra.org/en-US/white-paper/#lexicon">Regulated Virtual Asset Service Providers (VASPs)</a> while the Association continues to develop its certification process for other VASPs and its compliance framework for <a href="https://libra.org/en-US/white-paper/#lexicon">Unhosted Wallets</a> based on the feedback received from regulators. The Association intends to make the network accessible to <a href="https://libra.org/en-US/white-paper/#lexicon">Certified VASPs</a> and Unhosted Wallets once the relevant compliance frameworks have been finalized. We know that due to our phased rollout plans, not all aspects of the Libra network will be available immediately to some developers, but we are excited to work with the community to drive the evolution of these features. For more details, click <a href="https://libra.org/en-US/white-paper/#compliance-and-the-prevention-of-illicit-activity">here</a>.
</blockquote>

### Stay Updated

Check out the Libra network’s [documentation](/docs/welcome-to-libra) and [community](http://community.libra.org) sites, and stay up to date by signing up for our newsletter [here](/newsletter_form).

Tell us your plan to build a [product or service](/developer_form/). We know that not all aspects of the Libra network will be available immediately to some developers. We're excited to work with the community to evolve these features, and look forward to your participation!
