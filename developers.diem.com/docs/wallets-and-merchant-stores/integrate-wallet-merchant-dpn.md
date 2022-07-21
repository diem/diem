---
title: "Integrate with the Diem Payment Network"
slug: "integrate-wallet-merchant-dpn"
hidden: false
---
If you are a qualified [Regulated VASP](../reference/glossary#regulated-vasp), use this guide to integrate your wallet or merchant store with the Diem Payment Network (DPN).

To become a qualified Regulated VASP, Diem Networks needs to authorize you to be a participant on the DPN.

## Get started

To integrate your wallet or merchant store with the DPN, you need to:
* [Create your VASP accounts](#create-your-vasp-accounts)
* [Submit a transaction](#submit-a-transaction)
* [Accept direct payments](#accept-direct-payments) if you are a Regulated VASP building a merchant store

## Create your VASP accounts

Regulated VASP accounts are reserved for a [Regulated VASP](../reference/glossary#regulated-vasp) that operates on the DPN usually on behalf of end users. There are two types of Regulated VASP accounts, ParentVASP accounts and ChildVASP accounts.
  * **ParentVASP accounts**: The ParentVASP account is your unique root account. You can have only one parent account per Regulated VASP. Diem Networks will create a ParentVASP account on your behalf with your authentication key. This parent account contains three key pieces of data:
    * `human_name`: Your unique account name.
    * `base_url`: The URL containing an endpoint to hit that off-chain APIs use to exchange information to comply with the Travel Rule. This will contain a dummy value when the ParentVASP account is created.
    * `compliance_public_key`: An [ed5519](../reference/glossary#ed5519) public key for authenticating signatures on Travel Rule payloads. This will contain a dummy value when the account is created.

  * **ChildVASP accounts**: The **ChildVASP account** is the child of your ParentVASP account. A ChildVASP account stores the address of its ParentVASP. You can have any number of ChildVASP accounts. They can help in maintaining a structured on-chain presence (e.g., separate cold/warm/hot accounts). You do not need to have ChildVASP accounts.

Read the [accounts](/docs/basics/basics-accounts) page to learn more.

To create your VASP accounts:
![Figure 1.0 Creating your VASP account](https://files.readme.io/71ef5d0-create-vasp-account.svg)
<small className="figure">Figure 1.0 Creating your VASP account</small>

1. [**Generate keys**](/docs/basics/basics-accounts#generate-an-auth-key-and-account-address)
      Generate an [Ed25519](../reference/glossary#ed5519) keypair and associated authentication key for your on-chain Regulated VASP account.
2. **Share account info**
      Share the following with the Diem Networks Treasury offline: your public key, the initial [currency of your choice](#choose-currencies), `XUS` or `ALL`, and a human-readable VASP name to use on-chain. Diem Networks will need to check if the human name is a unique value.
3. **The DPN creates your ParentVASP account**
      Once you share your account information, Diem Networks will send a transaction that creates a ParentVASP account with your authentication key.
4. **Set up the base URL and compliance public key for off-chain APIs**:
      In order to use off-chain APIs, you must send a transaction to set the base URL and compliance public key values on your ParentVASP account using [this transaction script](https://github.com/diem/diem/blob/main/language/diem-framework/transaction_scripts/doc/transaction_script_documentation.md#script-rotate_dual_attestation_info).
5. **Create your ChildVASP account**
      If you want to, you can create a new ChildVASP account from your ParentVASP account using [this transaction script](https://github.com/diem/diem/blob/master/language/diem-framework/transaction_scripts/doc/transaction_script_documentation.md#script-create_child_vasp_account). You need a ParentVASP account to create a ChildVASP account.
6. **Start transacting**
      Once the previous steps have been completed, you can send transactions from your account on-chain using the keypair from step 1.


#### Choose currencies

When you are creating your ParentVASP account, you will need to choose at least one Diem Coin currency. The Diem Coin currency that is currently available on the DPN is `XUS`(a USD stablecoin).

When there is more than one [Diem Coin currency](/docs/basics/basics-accounts#currencies) available, you can share with the DPN which one you would like to associate with your account. You can also request the DPN to choose all the Diem Coin currencies available.

You can add new Diem Coin currencies to an existing account via the [`add_currency_to_account` transaction script](https://github.com/diem/diem/blob/main/language/diem-framework/transaction_scripts/doc/transaction_script_documentation.md#script-add_currency_to_account). You can add all currencies offered on the DPN to your ChildVASP accounts by using the `add_all_currencies` flag in the account creation scripts.

When available, at the Move level, each Diem Coin currency will be a different generic instantiation of the same Diem type (i.e. `Diem<XUS>`).


## Submit a Transaction

When you submit a transaction to the DPN, you are cryptographically signing a transaction script and then waiting (by listening to the event stream) for consensus from validators. Learn more about the [lifecycle of a transaction](/docs/transactions/basics-life-of-txn).

Once you have your ParentVASP account, you can start interacting with the DPN. Before you submit your first transaction, you will need to:

* [**Learn How to Interact with DPN**](#how-to-interact-with-the-dpn): Learn how you can interact with the DPN using private full nodes.
* [**Set-up Off-chain APIs**](#set-up-off-chain-apis): Off-chain APIs to exchange data, where applicable..
* [**Choose Gas Values**](#choose-gas-values): The gas values that determine the computational resources used and the fees you will incur for each transaction.

### How to interact with the DPN
The first step to submitting transactions to the DPN is determining how to connect and interact with it. The guidance for this differs slightly based on if you are a [validator node](/docs/basics/basics-validator-nodes) operator or not.

If you are not a validator node operator, you can do one of the following:

* Communicate with a validator operator or owner of a validator node to obtain dedicated access to the validator network, using either a [FullNode](/docs/basics/basics-fullnodes) or the JSON-RPC endpoint of a FullNode.
* Leverage the public FullNode network deploying your own FullNode with a JSON-RPC endpoint.
* Access a public JSON-RPC endpoint.
* Use your own private FullNode for faster access to the blockchain state. This allows you to monitor your submitted transactions more closely. Private FullNodes also provide additional redundancy, which means that you can fall back on the well-maintained public network whenever your private FullNode is unavailable.

Your wallet integration with the DPN may be faster when done using the JSON-RPC service.

### Set up off-chain APIs

Off-chain protocols are APIs and payload specifications that support compliance and scalability on the DPN. It is executed between pairs of Regulated VASPs, such as wallets, exchanges, or designated dealers, and allows Regulated VASPs to privately exchange payment information before, while, or after settling it on the Diem Blockchain.

These off-chain APIs also provide a means to  to comply with the Travel Rule and negotiate one-time identifiers for on-chain transactions, reducing transaction linkability. To establish a connection, VASPs will look up their counterparty’s on-chain account containing a base URL for their off-chain service.  Note that each entity using an Off-Chain API must make its own determination as to whether it satisfies Travel Rule compliance.

To use off-chain APIs, you will need to set  values for the `base_url` and `compliance_public_key` associated with your on-chain ParentVASP account.

You can read more about off-chain protocols [here](https://dip.diem.com/lip-1/).


### Choose Gas Values

When an account submits a transaction for execution, **gas** is used to:

*   Track and account the computational resources used.
*   Limit the number of resources used during execution.
*   Charge a transaction fee based on the amount of resources used for execution.

Learn more about using gas to specify a transaction fee [here](/docs/basics/basics-gas-txn-fee#introduction)

To compute a transaction fee, you can choose the following gas values:


| Definition | Set value |
| ---------- | ---------- |
| `max_gas_units` | The maximum amount of gas units that can be used to execute a transaction. This bounds the computational resources that can be consumed by a transaction. To help you choose a value for `max_gas_amount`, we will be publishing a list of suggested `max_gas_amount` for each transaction before launch. We will be keeping the current lockdown restrictions the world-over in consideration for this list.<br/><br/>On testnet: `600 < max_gas_amount ≤ 2,000,000` |
| `gas_price` | The number of gas units used in the specified gas currency. Gas price is a way to move from gas units, the abstract units of resource consumption the virtual machine (VM) uses, into a transaction fee in the specified gas currency. For launch, you can set the `gas_price` to be zero or almost zero, allowing you to submit transactions without high charges. This is because the network shouldn’t have high contention.<br/><br/>On testnet: `0 ≤ gas_price ≤ 10,000` |
| `gas_currency_code` | This is the currency of the transaction fee charged to the client.  The `gas_currency` must be a registered currency on-chain ("XUS" on testnet), and must be one of the held currencies of the sending account. E.g. setting the `gas_currency` to "Coin3" would cause the transaction to be rejected since Coin3 is not a registered currency on-chain, and the sending account does not hold that currency. |
## Accept Direct Payments

If you're a Regulated VASP who wants to build and integrate your own merchant store, you will need to set up direct payments to receive payments and handle refunds.


![direct-payments.svg](https://files.readme.io/8a6c1c5-direct-payments.svg)
### Share payment information at checkout

When the DPN launches, we expect that wallets will support a basic set of payment flows that resemble peer-to-peer transactions. Accordingly, the initial range of merchant checkout options will be limited to direct payments, which are single point-in-time transactions with a known amount and currency type.

To provide an interoperable checkout experience, you will need to share payment information or context with the end user. This can happen through a generated QR code, a URI that you can copy and paste, or a mobile deep link.

This information includes:
* A fully qualified account address. This is a stable on-chain address and unique, and the generated subaddress.
* The payment amount in microunits.
* The currency code .

Once the end user's wallet service provider (Regulated VASP) receives this payment context, it can now construct a pre-populated payment for the user to authorize.

When the end user authorizes their wallet to pay the merchant per the shared payment context, their wallet will go through a similar sender flow to a peer-to-peer payment. A transaction is committed directly on-chain as long as the amount doesn't exceed the Travel Rule threshold. If the Travel Rule goes into effect, the payment will first be preflighted through the [off-chain protocol](https://dip.diem.com/dip-1/), before being committed on-chain.

More information about payment request URI serialization can be found [here](https://dip.diem.com/dip-5/).

### Detect payments

Once the end user's wallet sends you, a merchant service provider, the payment, you can check the transaction to connect it to a checkout session, confirm the payment, and signal the confirmation to the end user.

You can do this by interfacing with a FullNode to track emitted [events](/docs/basics/basics-events) for your on-chain accounts. If you see that you have received a payment, you can examine the accompanying subaddress to route the payment internally. Since the subaddress will be unique for each transaction, you can determine who the buyer (end user who sent you the payment) was. In other words, you can connect the payment received to a checkout session and signal that the payment is confirmed to the end user.


### Handle refunds

You may want to issue a refund to a customer in some cases such as when you receive duplicate payments, a payment to an invalid address, when a customer returns a purchase, or because of some customer support inquiry.

Refunds on the DPN resemble any other peer-to-peer transaction. A merchant service provider would need to construct a transaction script on behalf of the merchant, and submit it to the original customer. These transactions are subject to the Travel Rule controls and will require off-chain preflighting if the amount exceeds the Travel Rule threshold.

Unlike other transactions, refunds will contain a pointer to the original transaction that they correspond to. If the transaction exceeds the Travel Rule threshold, the `original event reference id` is included in the off-chain payload. Otherwise (ie. in the non-Travel Rule case), the `original event sequence number` is sent in the on-chain body of the refund transaction. Wallets that receive these refunds can use the reference to the original transaction to provide additional context to the inbound funds.

More information about refunds can be found [here](https://drive.google.com/file/d/1YUEtAqd0N5dl2vg5EgEQhrVQKNN0OhoI/view?usp=sharing). Additional information about transaction serialization can be found [here](https://dip.diem.com/dip-4/).

## Send Payments

You can send payments using [this transaction script](https://github.com/diem/diem/blob/main/language/diem-framework/transaction_scripts/doc/transaction_script_documentation.md#script-peer_to_peer_with_metadata).

This script requires the following:

* The receiving account must have an address. If it doesn’t, the script will be aborted.
* The accounts must both have a balance (or a zero balance) in the currency specified by `CoinType`. The zero balance is added when the `CoinType` currency is authorized for an account. This means if the sending or receiving account doesn’t have a balance (or a zero balance) in the currency specified by `CoinType`, the transaction will abort.
* A generic type parameter `CoinType` specifying the currency to be transferred.
* A metadata parameter that accepts arbitrary binary data. For most transactions, the metadata should be the subaddress of the Regulated VASP customer receiving the payment. The contents of metadata are emitted in payment events, but are not otherwise inspected on-chain (so using empty or dummy metadata is just fine for testing). As noted above, use of the metadata parameter is subject to certain limitations specified in the Participation Agreement and DPN Rules
* A metadata_signature parameter used for dual attestation in the travel rule protocol.

## Query the blockchain

Each transaction is designed to emit any number of [events](/docs/basics/basics-events) as a list. An aborting transaction never emits events, so you can use events to confirm if a transaction has been successfully executed.

For example, a peer-to-peer payment transaction emits a `SentPayment` event for the sender’s account and a `ReceivedPayment` event for the recipient account. The `SentPayment` event allows the sender to confirm that the payment was sent from their account, while a `ReceivedPayment `event allows the recipient to confirm that a payment was received in their account. Events are persisted on the Diem Blockchain and you (as a Regulated VASP) can use these events to answer your queries.

More information on event structures, and how they can be queried and the surrounding data in the JSON responses can be found [here](https://github.com/diem/diem/blob/main/json-rpc/json-rpc-spec.md).

To check for new transactions posted to your account, you need to [query the blockchain](/docs/tutorials/tutorial-query-the-blockchain) using the JSON-RPC endpoints.

* If you are sending the transaction, you query for the transaction’s sequence number.
* If you are receiving the transaction, you query for the ReceivedPaymentEvent event.
