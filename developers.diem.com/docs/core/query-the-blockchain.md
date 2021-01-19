---
id: query-the-blockchain
title: Query the Blockchain
---

import CLICommands from 'cli-commands';
const { GetAccount, GetAccountTransactions, GetCurrencies, GetMetadata } = CLICommands;

The JSON-RPC service provides APIs for clients to query the Diem Blockchain. This tutorial will guide you through the different methods you can use. You can test these on testnet.

#### Assumptions

All commands in this document assume that:

* You are running on a Linux (Red Hat or Debian-based) or macOS system.
* You have a stable connection to the internet.

#### Prerequisites
* Complete the [My First Transaction](my-first-transaction.md) tutorial to understand how to create accounts and interact with testnet.

If you have already completed the My First Transaction tutorial, you can use the following account information for Alice and Bob to get started:
* [Account address: Hex-coded account address](my-first-transaction.md#step-4-optional-list-accounts)

## Get account information
To get information about Alice’s account, use [get_account](https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_account.md). To run this method, you will need the hex-coded account address (see step 4 of My First Transaction).

In the example below, we have a request sent using [get_account](https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_account.md) to learn information about Alice’s account and the response received.

<CLI command={GetAccount} />

## Get transactions for an account
To see all the transactions sent by Alice’s account, you can use [get_account_transactions](https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_account_transactions.md). You will need:
* Alice’s hex-code account address
* the starting sequence number
* the maximum number of transactions you want returned for this method

In the example below, we have a request sent using [get_account_transactions](https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_account_transactions.md) to learn transaction information about Alice’s account and the response received.

<CLI command={GetAccountTransactions} />

## Get the latest ledger state
You can check the current state of the testnet using [get_metadata](https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_metadata.md).

<CLI command={GetMetadata} />


## Get currencies available
Use [get_currencies](https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_currencies.md) to query the types of currencies supported by the Diem Blockchain.

<CLI command={GetCurrencies} />

## All methods
You can use different methods to query the blockchain and get the information you’re looking for. We’ve included the complete list and their descriptions below.

| Method                                                       | Description                                                  |
| ------------------------------------------------------------ | ------------------------------------------------------------ |
| [get_transactions](https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_transactions.md) | Get transactions on the Diem Blockchain.                    |
| [get_account](https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_account.md) | Get the latest account information for a given account address. |
| [get_account_transactions](https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_account_transactions.md) | Get transactions sent by a particular account.               |
| [get_metadata](https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_metadata.md) | Get the blockchain metadata (for example, state as known to the current full node). |
| [get_events](https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_events.md) | Get the events for a given event stream key.                 |
| [get_currencies](https://github.com/diem/diem/blob/master/json-rpc/docs/method_get_currencies.md) | Get information about all currencies supported by the Diem Blockchain. |


###### tags: `core`
