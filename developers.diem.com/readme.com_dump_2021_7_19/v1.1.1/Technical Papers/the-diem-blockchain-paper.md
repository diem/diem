---
title: "The Diem Blockchain"
slug: "the-diem-blockchain-paper"
hidden: false
metadata: 
  title: "The Diem Blockchain"
  description: "Read the technical paper on the Diem Blockchain and how it works."
createdAt: "2021-02-04T01:06:43.773Z"
updatedAt: "2021-03-30T03:44:55.778Z"
---
***Note to readers: On December 1, 2020, the Libra Association was renamed to Diem Association. This report was published before the Association released White Paper v2.0 in April 2020, which included a number of key updates to the Libra payment system. Outdated links have been removed, but otherwise, this report has not been modified to incorporate the updates and should be read in that context. Features of the project as implemented may differ based on regulatory approvals or other considerations, and may evolve over time.***

## Abstract

The Diem Blockchain is a decentralized, programmable database designed to support a low-volatility cryptocurrency that will have the ability to serve as an efficient medium of exchange for billions of people around the world. We present a proposal for the Diem protocol, which implements the Diem Blockchain and aims to create a financial infrastructure that can foster innovation, lower barriers to entry, and improve access to financial services. To validate the design of the Diem protocol, we have built an open-source prototype implementation — _Diem Core_ — in anticipation of a global collaborative effort to advance this new ecosystem.

The Diem protocol allows a set of replicas — referred to as validators — from different authorities to jointly maintain a database of programmable resources. These resources are owned by different user accounts authenticated by public key cryptography and adhere to custom rules specified by the developers of these resources. Validators process transactions and interact with each other to reach consensus on the state of the database. Transactions are based on predefined and, in future versions, user-defined smart contracts in a new programming language called _Move_.

We use Move to define the core mechanisms of the blockchain, such as the currency and validator membership. These core mechanisms enable the creation of a unique governance mechanism that builds on the stability and reputation of existing institutions in the early days but transitions to a fully open system over time.

### Downloads
[block:html]
{
  "html": "<d-publication-link \n  image=\"https://diem-developers-components.netlify.app/images/diem-blockchain-pdf.png\"\n  doc-link=\"https://diem-developers-components.netlify.app/papers/the-diem-blockchain/2020-05-26.pdf\"\n  title=\"The Diem Blockchain\"\n></d-publication-link>"
}
[/block]