---
title: "Gas and transaction fees"
slug: "basics-gas-txn-fee"
hidden: false
---
When a transaction is executed in the Diem Payment Network (DPN), the network resources used are tracked and measured using gas.

## Introduction

Gas ensures that all Move programs running on the Diem Blockchain terminate. This bounds the computational resources used. Gas also provides the ability to charge a transaction fee, partly based on consumed resources during execution.

When a client submits a transaction for execution to the Diem Blockchain, it contains a specified:

* `max_gas_amount`: The maximum amount of gas units that can be used to execute a transaction. This bounds the computational resources that can be consumed by a transaction.
* `gas_price`: The number of gas units used in the specified gas currency. Gas price is a way to move from gas units, the abstract units of resource consumption the virtual machine (VM) uses, into a transaction fee in the specified gas currency.
* `gas_currency`: This is the currency of the transaction fee.

The transaction fee charged to the client will be at most `gas_price * max_gas_amount`.
<BlockQuote type="info">
The gas price, and hence the transaction fee, should rise-and-fall with contention in the DPN. At launch, we expect gas prices to be at or near zero. But in periods of high contention, you can prioritize transactions using the gas price, which will encourage sending only needed transactions during such times.
</BlockQuote>

## Types of resource usage consumed

For the VM to execute a transaction, the gas system needs to track the primary resources the DPN and VM use. These fall into three resource "dimensions":

1. The computational cost of executing the transaction.
2. The network cost of sending the transaction over the DPN.
3. The storage cost of data created and read during the transaction on the Diem Blockchain.

The first two of these resources (computational and network) are ephemeral. On the other hand, storage is long lived. Once data is allocated, that data persists until it is deleted. In the case of accounts, the data lives indefinitely.

Each of these resource dimensions can fluctuate independently of the other. However, we also have only one gas price. This means that each resource dimension's gas usage needs to be correct, because the gas price only acts as a multiplier to the total gas usage, and does not specify not usage by dimension. The essential property that we design for is that gas usage of a transaction needs to be as highly correlated as possible with the real-world cost associated with executing a transaction.

## Using gas to compute transaction fee

When you send a transaction, the transaction fee (in the specifed gas currency) for execution is the gas price multiplied by the VM's computed resource usage for that transaction.

At different times in the transaction flow, different aspects of resource usage are charged. The basics of the transaction flow and the gas-related logic are detailed in the following diagram:
![FIGURE 1.0 Gas and Transaction Flow](/img/docs/using-gas.svg)
<small className="figure">FIGURE 1.0 Gas and Transaction Flow</small>




In the diagram, both the prologue and epilogue sections are marked in the same color. This is because these sections of the transaction flow need to be **unmetered**:
* In the prologue, it's not known if the submitting account has sufficient funds to cover its gas liability, or if the user submitting the transaction even has authority over the submitting account. Due to this lack of knowledge, when the prologue is executed, it needs to be unmetered. Deducting gas for transactions that fail the prologue could allow unauthorized deductions from accounts.
* The epilogue is in part responsible for debiting the execution fee from the submitting account and distributing it. Because of this, the epilogue must run even if the transaction execution has run out of gas. Likewise, we don't want it to run out of gas while debiting the submitting account as this would cause additional computation to be performed without any transaction fee being charged.

This means that the minimum transaction fee, `MIN_TXN_FEE`, needs to be enough to cover the average cost of running the prologue and epilogue.

After the prologue has run, and we've checked in part that the account can cover its gas liability, the rest of the transaction flow starts with the "gas tank" full at `max_gas_amount`. The `MIN_TXN_FEE` is charged, after which the gas tank is then deducted (or "charged") for each instruction the VM executes. This per-instruction deduction continues until either:
* The transaction execution is complete, after which the cost of storing the transaction data is charged, and the epilogue is run and the execution fee deducted, or
* The "gas tank" becomes empty, in which case an `OutOfGas` error is raised.

In the former, the fee is collected and the result of the transaction is persisted on the Diem Blockchain. The latter causes the execution of the transaction to stop when the error is raised, following which the total gas liability of the transaction is collected. No other remnants of the execution are committed other than the deduction in this case.

## Using gas to prioritize a transaction

When you send a transaction, it is prioritized based on different criteria. One of these is the normalized gas price for the transaction.

For transactions that are subject to ordering by gas price (i.e., non-governance transactions) these prices are first normalized to Diem Coins. This is done by using the current gas currency to Diem Coin conversion rate that is stored on-chain. Transactions are then ranked (in part) based upon this normalized gas price.

For example:

* Bob sends a transaction with `gas_price` 10 and `gas_currency` of “BobCoins”.
* Alice sends a transaction at the same time with `gas_price` 20 and `gas_currency` of “AliceCoins”.

If the on-chain “BobCoins” to Diem Coins exchange rate is 2.1 and the on-chain “AliceCoins” to Diem Coins exchange rate is 1,
* Bob’s transaction has a normalized gas price of `10 * 2.1 = 21`.
* Alice’s transaction has a normalized gas price of `20 * 1 = 20`.

Then, Bob’s transaction would be ranked higher than Alice’s.

## Core design principles
Three central principles have motivated the design of gas in Move:

| Design Principle | Description |
| ---------- | ---------- |
| Move is Turing complete | Because of this, determining if a given Move program terminates cannot be decided statically. However, by ensuring that <br/>  - every bytecode instruction has a non-zero gas consumption, and <br/>  - the amount of gas that any program can be started with is bounded, <br/>  we get this termination property for programs almost free of cost. |
| Discourage DDoS attacks and encourage judicious use of the DPN | The gas usage for a transaction is correlated with resource consumption such as the time taken to execute a transaction. The gas price, and hence the transaction fee, should rise-and-fall with contention in the DPN. At launch, we expect gas prices to be at or near zero. But in periods of high contention, you can prioritize transactions using the gas price, which will encourage sending only needed transactions during such times. |
| The resource usage of a program needs to be agreed upon in consensus | This means that the method of accounting for resource consumption needs to be deterministic. This rules out other means of tracking resource usage, such as cycle counters or any type of timing-based methods as they are not guaranteed to be deterministic across nodes. The method for tracking resource usage needs to be abstract. |
