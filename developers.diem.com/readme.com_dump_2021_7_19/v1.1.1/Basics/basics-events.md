---
title: "Events"
slug: "basics-events"
hidden: false
metadata: 
  title: "Events"
  description: "Read about the different types of events in the Diem Blockchain and how you can query for them"
createdAt: "2021-02-04T01:03:19.432Z"
updatedAt: "2021-04-21T21:53:34.905Z"
---
Events are Move data that are emitted during the execution of a transaction on the Diem Blockchain. 

For example, whenever a payment transaction is sent or received on-chain, the transaction will emit the relevant data using the `SentPaymentEvent` and `ReceivedPaymentEvent`. This data is stored in the EventStore, which is part of the Diem Blockchain’s ledger state. You can query the EventStore to get proof of executed transactions on-chain.

## Introduction

Events are grouped into **event streams** based on event type. The Diem Framework uses different types of events for payments, minting/burning, and system operations. A detailed list of the event types accessible from JSON-RPC is available in the <a href="https://github.com/diem/diem/blob/main/json-rpc/docs/type_event.md" target="_blank">JSON-RPC documentation</a>.

### Payment transaction events

Whenever a payment transaction is committed, it emits the following events:

* A `SentPaymentEvent` to the sender’s SentPaymentEvent stream
* A `ReceivedPaymentEvent` to the recipient's ReceivedPaymentEvent stream

Both `SentPaymentEvent` and `ReceivedPaymentEvent` have the same structure:

| Field Name | Type                                                         | Description                                                  |
| ---------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| amount     | <a href="https://github.com/diem/diem/blob/main/json-rpc/docs/type_amount.md)" target="_blank">Amount</a> | Amount received from the sender of the transaction           |
| sender     | string                                                       | Hex-encoded address of the account whose balance was debited to perform this deposit. If the deposited funds came from a mint transaction, the sender address will be 0x0...0. |
| receiver   | string                                                       | Hex-encoded address of the account whose balance was credited by this deposit.<br /> |
| metadata   | string                                                       | An optional field that can contain extra metadata for the event. This information can be used by an off-chain API to implement a sub-addressing scheme for a wallet. |

### Minting and burning events

Minting and burning transactions produce events that are associated with a specific currency type. Each registered currency has a `CurrencyInfo<CoinType>` resource on chain, which contains four related event streams:

* `BurnEvent`
* `PreburnEvent`
* `CancelBurnEvent`
* `MintEvent`

### System operation events

There are also several events related to system-level operations, for example:

* `UpgradeEvent` — when a special system maintenance WriteSet transaction is applied
* `NewEpochEvent` — when an on-chain configuration is modified
* `NewBlockEvent` — when a new block of transactions is added to the Diem Blockchain

## Event concepts
[block:image]
{
  "images": [
    {
      "image": [
        "https://files.readme.io/b01f98b-events-fig1.svg",
        "events-fig1.svg",
        267,
        150,
        "#ffffff"
      ],
      "caption": "Figure 1.0 EventHandle and event streams in the Diem Framework"
    }
  ]
}
[/block]
* **Event stream**: Events are grouped into “**event streams,**” which are append-only vectors of ContractEvent payloads. For example, an account can have a SentPaymentEvent stream and a ReceivePaymentEvent stream. The entries in an event stream are assigned sequence numbers beginning from zero.
* **Event key**: Each event stream is associated with a key value, which is a globally unique 40-byte array that is defined when the stream is created. The event key for an account’s event stream will never be modified.
* **EventHandle**: An EventHandle contains the event key and the number of events in the stream. This count is always equal to the biggest sequence number for the events in the stream. An EventHandle resource is typically embedded inside other Move resources to record related events. For example, every DiemAccount resource contains a `sent_events` EventHandle for a SentPaymentEvents stream and also a `received_events` EventHandle for a ReceivedPaymentEvents stream. Event streams are referenced from the StateStore via EventHandle Move resources.
* **ContractEvent**: A ContractEvent payload contains the event key, the sequence number, and a serialized Move value, along with a tag to identify the type of that value. Different kinds of events are distinguished by the Move value types, such as `SentPaymentEvent` or `ReceivedPaymentEvent`, which are emitted to the event streams.

The Event module in the Diem Framework specifies the details of event implementation in Move. The views of event information from JSON-RPC are often presented somewhat differently than the underlying implementation. For example, EventHandles are typically displayed as only the key values, e.g., `sent_events_key` instead of a `sent_events` EventHandle structure containing the key. This rest of this page shows examples of events as they are viewed through the JSON-RPC interface.


## How to query events
There are several JSON-RPC API methods associated with events:
* The `get_account_transaction`, `get_account_transactions`, and `get_transactions` API methods each have an option to also return the events emitted by the transactions.
* `get_events` returns the events from a specific event stream.


### Get `SentPaymentEvent` for an account

This example demonstrates how to query a `SentPaymentEvent` for an account. In this example, account 0x996b67d has two event streams, with 3 sent payments and 2 received payments:
[block:image]
{
  "images": [
    {
      "image": [
        "https://files.readme.io/670f7d0-events-fig2.svg",
        "events-fig2.svg",
        267,
        150,
        "#ffffff"
      ],
      "caption": "Figure 1.1 Example event streams for a Diem Account"
    }
  ]
}
[/block]
1. The first step is to find the event key for the account’s `SentPaymentEvent` stream. We can send a <a href="https://github.com/diem/diem/blob/main/json-rpc/docs/method_get_account.md" target="_blank">`get_account`</a> query to the JSON-RPC endpoint to get the state of that <a href="https://github.com/diem/diem/blob/main/json-rpc/docs/type_account.md" target="_blank">account`</a>, including two event keys: one for the `SentPaymentEvent` stream (the `sent_events_key` field) and one for the `ReceivedPaymentEvent` stream (the `received_events_key` field). The response will look like the following:
[block:code]
{
  "codes": [
    {
      "code": "    {\n      \"diem_chain_id\" : 2,\n      \"jsonrpc\" : \"2.0\",\n      \"diem_ledger_timestampusec\" : 1597084681499780,\n      \"result\" : {\n       ...\n       \"received_events_key\" : \"00000000000000001668f6be25668c1a17cd8caf6b8d2f25\",\n       \"sent_events_key\" : \"01000000000000001668f6be25668c1a17cd8caf6b8d2f25\",\n       ...\n      },\n      \"id\" : 1,\n      \"diem_ledger_version\" : 1303433\n    }",
      "language": "json",
      "name": "get_account response"
    }
  ]
}
[/block]
2. The next step is to use the <a href="https://github.com/diem/diem/blob/main/json-rpc/docs/method_get_events.md" target="_blank">`get_events`</a> API method to fetch the event details. In the JSON-RPC query, we can specify to fetch one event beginning with sequence number 2 from the `sent_events_key` event stream. The response will look like the following:
[block:code]
{
  "codes": [
    {
      "code": "{\n     \"id\": 1,\n     \"jsonrpc\": \"2.0\",\n     \"diem_chain_id\": 2,\n     \"result\": [\n      {\n       \"data\": {\n        \"amount\": {\n         \"amount\": 200,\n         \"currency\": \"XUS\"\n        },\n        \"metadata\": \"\",\n        \"receiver\": \"280081f\",\n        \"sender\": \"996b67d\",\n        \"type\": \"sentpayment\"\n       },\n       \"key\": \"01000000000000001668f6be25668c1a17cd8caf6b8d2f25\",\n       \"sequence_number\": 2,\n       \"transaction_version\": 106495\n      },\n    }",
      "language": "json",
      "name": "get_events response"
    }
  ]
}
[/block]
Note that the JSON-RPC view of the event data is a bit different from the underlying `SentPaymentEvent`, which has a single payee field instead of both receiver and sender.