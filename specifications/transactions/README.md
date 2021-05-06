# Diem Transactions and Payments

## Overview

The Diem Payment network supports transactions between VASPs (Virtual Asset Service Providers) and between VASPs and DDs (Designated Dealers). This document covers the end-to-end flow for these transactions derived from the DIPs.

Diem supports two methods for transacting on-chain:
* Subaddresses that can be exchanged directly between two parties off-chain
* ReferenceIds that can be generated via an off-chain API between two parties

In the case of subaddresses, a participant can directly provide a subaddress to one or more other participants, who can then submit transactions directly to that party on-chain. However, re-use of subaddresses creates on-chain linkage across transactions that may constitute publicly identifiable material. To address this, participants can also leverage the Off-Chain Protocol to produce a one-time use reference id for the on-chain payment. The Off-Chain Protocol also provides functionality for dual attestation of KYC (Know Your Customer) exchange per Diem's Travel Rule requirements.

## Introduction

### On-Chain

A Diem transaction consists of a sender, a script or script function, and a set of arguments. Currency transfer transaction arguments include a recipient, an amount, a currency, and metadata. Diem leverages the metadata to provide context behind a transaction. The metadata itself is not validated by the Diem blockchain or Move standard library.

When a transaction is submitted to the blockchain it is evaluated many times both at the general transaction layer as well as at the peer-to-peer settlement layer. Successfully executed transaction are committed to the blockchain and debit the sender's account by the specified amount in addition to gas (network usage) costs and credit the recipient's account. Transactions that fail any general transaction layer [validations](https://github.com/diem/diem/tree/main/specifications/move_adapter#Validation) are discarded as early as possible. Transactions that pass these validations but fail during execution, for example, the sending account has insufficient funds for the transfer, will be committed to the blockchain as an aborted transaction. Aborted transactions still charge the sending account gas fees.

### Off-Chain

Users do not have direct ability to send and receive payments on the Diem network and must do so through VASPs. One method to do so, is to store potentially personally identifiable information in a transaction metadata, Diem also supports an off-chain framework that can negotiate one-time use transaction identifiers called `reference_id`s. The off-chain framework is also used by support travel rule compliance.

### Refunds

Diem defines refunds as part of the transaction framework. The refund includes a pointer to the transaction that is being refunded and the reason for the refund, for example, an errant transaction to a non-existent destination as well as user-driven refunds. However, use of the refund type is optional.

## Common Flows

### Transfer with Subaddresses

Bob would like to send Alice 10 XUS:
* Alice loads her VASP's mobile APP and generates an intent identifier
* The intent identifier contains a subaddress that uniquely identifies Alice along with additional parameters specifying sending 10 XUS
* Alice shares this with Bob via a QR code
* The intent identifier opens Bob's VASP's mobile app and confirms Bob's intent to send 10 XUS
* When Bob confirms, his VASP debits his account 10 XUS and generates a Diem on-chain transaction that credits Alice's VASP
* Within the transaction, Bob's VASP embeds a subaddress referencing Bob and the subaddress referencing Alice that was contained within the intent identifier
* Upon processing the transaction, Alice's VASP credits Alice's account 10 XUS

### Transfer with Subaddress Exceeding Travel Rule Limits

In the case that Bob would send Alice an amount that exceeds the travel rule limit, the flow changes:

* Bob's VASP will initiate an off-chain KYC exchange with Alice's VASP
* Upon successful completion of the exchange, Alice and Bob's VASP would have agreed on a `reference_id` and Alice's VASP would have provided a signature for the transaction metadata that Bob's VASP will submit to the chain
* Bob's VASP will submit a transaction metadata that contains the `reference_id` instead of the subaddresses

### Refunds

There are two cases for refunds, those initiated by the recipient and those that have no valid recipient. In practice, those initiated by a recipient can be sent back as a transfer to the originating VASP and subaddress. Diem also provides for native refunds that contain a reason for the refund, so that VASPs need not speculate or confer off-chain on the reason why funds were returned, e.g., a purchase refund or an invalid subaddress.

## Transactions Overview

The rest of this specification contains the following components:
* [On-Chain Data and Transactions](onchain) including metadata and blockchain data
* [Off-Chain Identifiers](offchain-identity) currently only subaddresses
* [Off-Chain API](offchain-api) specifically the protocol and not any applications
* [Off-Chain PingCommand](ping-command) for validation of the API and health checks
* [Off-Chain Travel Rule Exchange](travel-rule) for performing off-chain KYC exchanges
