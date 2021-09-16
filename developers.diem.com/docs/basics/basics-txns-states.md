---
title: "Transactions and states"
slug: "basics-txns-states"
hidden: false
---
The two fundamental concepts at the heart of the Diem Blockchain are

* [Transactions](#transactions): Transactions represent the exchange of data and Diem Coins between any two accounts on the Diem Blockchain.
* [States](#ledger-state): The ledger state (state) represents the current snapshot of data on the blockchain. At any point in time, the blockchain has a ledger state.

When a submitted transaction is executed, the state of the Diem Blockchain changes.


## Transactions

When a Diem Blockchain participant submits a transaction, they are requesting the ledger state to be updated with their transaction information.

A [signed transaction](../reference/glossary#transaction) on the blockchain contains the following information:

- **Signature**: The sender uses a digital signature to verify that they signed the transacßtion.
- **Sender address**: The sender's [account address](../reference/glossary#account-address).
- **Sender public key**: The public authentication key that corresponds to the private authentication key used to sign the transaction.
- **Program**: The program comprises:
  - A Move bytecode transaction script: The Move transaction script is an arbitrary program that encodes transaction logic and interacts with resources published in the blockchain's distributed database. Move is a next generation language for secure, sandboxed, and formally verified programming.
  - An optional list of inputs to the script. For a peer-to-peer transaction, these inputs contain the recipient's information and the amount transferred to them.
  - An optional list of Move bytecode modules to publish.
- **Gas price** (in specified currency/gas units): This is the amount the sender is willing to pay per unit of [gas](../reference/glossary#gas) to execute the transaction. [Gas](basics-gas-txn-fee.md) is a way to pay for computation and storage. A gas unit is an abstract measurement of computation with no inherent real-world value.
- **Maximum gas amount**: The [maximum gas amount](../reference/glossary#maximum-gas-amount) is the maximum gas units the transaction is allowed to consume.
- **Gas currency code**: The currency code used to pay for gas.
- **Sequence number**: This is an unsigned integer that must be equal to the sender's account [sequence number](../reference/glossary#sequence-number) at the time of execution.
- **Expiration time**: The transaction ceases to be valid after this time.



## Ledger state

The Diem Blockchain's ledger state or global [state](../reference/glossary#state) comprises the state of all accounts in the blockchain. Each validator node in the blockchain must know the global state of the latest version of the blockchain's distributed database (versioned database) to execute any transaction.

### Versioned database

All of the data in the Diem Blockchain is persisted in a single-versioned distributed database. A version number is an unsigned 64-bit integer that corresponds to the number of transactions the system has executed.

This versioned database allows validator nodes to:

- Execute a transaction against the ledger state at the latest version.
- Respond to client queries about ledger history at both current and previous versions.


## Transactions change state


![FIGURE 1.0 TRANSACTIONS CHANGE STATE](/img/docs/transactions.svg)
<small className="figure">FIGURE 1.0 TRANSACTIONS CHANGE STATE</small>

Figure 1.0 represents how executing transaction T<sub>N</sub> changes the state of the Diem Blockchain from S<sub>N-1</sub> to S<sub>N</sub>.

In the figure:

| Name | Description |
| ---- | ----------- |
| Accounts **A** and **B** | Represent Alice's and Bob's accounts on the Diem Blockchain |
| S<sub>N-1</sub> | Represents the (N-1)th state of the blockchain. In this state, Alice's account A has a balance of 110 Diem Coins, and Bob's account B has a balance of 52 Diem Coins. |
| T<sub>N</sub> | This is the n-th transaction executed on the blockchain. In this example, it represents Alice sending 10 Diem Coins to Bob. |
| **F** | It is a deterministic function. F always returns the same final state for a specific initial state and a specific transaction. If the current state of the blockchain is S<sub>N-1</sub>, and transaction T<sub>N</sub> is executed on state S<sub>N-1</sub>, the new state of the blockchain is always S<sub>N</sub>. The Diem Blockchain uses the [Move language](https://diem.github.io/move) to implement the deterministic execution function F. |
| **S<sub>N</sub>** | This is the n-th state of the blockchain. When the transaction T<sub>N</sub> is applied to the blockchain, it generates the new state S<sub>N</sub> (an outcome of applying F to S<sub>N-1</sub> and T<sub>N</sub>). This causes Alice’s account balance to be reduced by 10 to 100 Diem Coins and Bob’s account balance to be increased by 10 to 62 Diem Coins. The new state S<sub>N</sub> shows these updated balances. |
