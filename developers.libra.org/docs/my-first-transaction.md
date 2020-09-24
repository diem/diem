---
id: my-first-transaction
title: My First Transaction
---

This document will guide you through executing your first transaction on the Libra Blockchain. Before you follow the steps to execute your first transaction, we recommend that you read the following documents to familiarize yourself with the key aspects of the Libra ecosystem and the Libra protocol:

* [Welcome](welcome-to-libra.md)
* [The Libra protocol: Key Concepts](libra-protocol.md)

We provide a command line interface (CLI) client to interact with the blockchain.

## Assumptions

All commands in this document assume that:

* You are running on a Linux (Red Hat or Debian-based) or macOS system.
* You have a stable connection to the internet.
* `git` is installed on your system.
* Homebrew is installed on a macOS system.
* `yum`or `apt-get` is installed on a Linux system.

## Steps to Submit a Transaction

In this example, we'll download the necessary Libra components and execute a transaction between two users: Alice and Bob.

Perform the following steps to submit a transaction to a validator node on the Libra testnet:

1. [Clone and build Libra Core](#clone-and-build-libra-core).
2. [Build the Libra CLI client and connect to the testnet](#build-libra-cli-client-and-connect-to-the-testnet).
3. [Create Alice’s and Bob’s accounts](#create-alice-s-and-bob-s-account).
4. [Mint coins and add to Alice’s and Bob’s accounts](#add-libra-coins-to-alice-s-and-bob-s-accounts).
5. [Submit a transaction](#submit-a-transaction).

## Clone and Build Libra Core

### Clone the Libra Core Repository

```bash
git clone https://github.com/libra/libra.git && cd libra
```
### Checkout the `testnet` Branch

```bash
git checkout testnet
```

### Install Dependencies

To setup Libra Core, change to the `libra` directory and run the setup script to install the dependencies, as shown below:

```
./scripts/dev_setup.sh
```
The setup script performs these actions:

* Installs rustup &mdash; rustup is an installer for the Rust programming language, which Libra Core is implemented in.
* Installs the required versions of the rust-toolchain.
* Installs CMake &mdash; to manage the build process.
* Installs protoc &mdash; a compiler for protocol buffers.
* Installs Go &mdash; for building protocol buffers.

If your setup fails, see [Troubleshooting](#setup)

## Build Libra CLI Client and Connect to the Testnet

To connect to a validator node running on the Libra testnet, run the client as shown below.

```bash
./scripts/cli/start_cli_testnet.sh
```

This command builds and runs the client utilizing cargo (Rust’s package manager) and connects the client to a validator node on the testnet.

Once the client connects to a node on the testnet, you will see the following output.  To quit the client at any time, use the `quit` command.

```
usage: <command> <args>

Use the following commands:

account | a
  Account operations
query | q
  Query operations
transfer | transferb | t | tb
  <sender_account_address>|<sender_account_ref_id> <receiver_account_address>|<receiver_account_ref_id> <number_of_coins> <currency_code> [gas_unit_price_in_micro_libras (default=0)] [max_gas_amount_in_micro_libras (default 400_000)] Suffix 'b' is for blocking.
  Transfer coins from one account to another.
info | i
  Print cli config and client internal information
dev
  Local Move development
help | h
  Prints this help
quit | q!
  Exit this client


Please, input commands:

libra%
```

If you have problems building the client and connecting to the testnet, refer to [Troubleshooting](#client-build-and-run).

<blockquote class="block_note">

**Note**: If you would like to run a validator node locally on your system, follow the instructions in [Run a Local Validator Node](#run-a-local-validator-node). The instructions for creating accounts, minting coins, and performing a transaction are the same as that for a node on testnet.

</blockquote>

## Create Alice’s and Bob’s Account

Once your client is connected to the testnet, you can run CLI commands to create new accounts.  We will walk you through creating accounts for two users (let's call them Alice and Bob).

### Step 1: Check If the CLI Client Is Running on Your System

A **libra%** command line prompt indicates that your Libra CLI client is running. To see the help information for the `account` command enter “account” as shown below:

```plaintext
libra% account
usage: account <arg>

Use the following args for this command:

create | c
  Create a local account--no on-chain effect. Returns reference ID to use in other operations
list | la
  Print all accounts that were created or loaded
recover | r <file path>
  Recover Libra wallet from the file path
write | w <file name>
  Save Libra wallet mnemonic recovery seed to disk
mint | mintb | m | mb <receiver_account_ref_id>|<receiver_account_address> <number_of_coins> <currency_code> [use_base_units (default=false)]
  Send currency of the given type from the faucet address to the given recipient address. Creates an account at the recipient address if one does not already exist. Suffix 'b' is for blocking
addc | addcb | ac | acb <account_address> <currency_code>
  Add specified currency to the account. Suffix 'b' is for blocking

```

### Step 2: Create Alice’s Account

Note that creating an account using the CLI does not update the blockchain, it just creates a local key-pair.

To create Alice’s account, enter this command:

`libra% account create`

Sample output on success:

```plaintext
>> Creating/retrieving next local account from wallet
Created/retrieved local account #0 address cc2219df031a68115fad9aee98e051e9
```

0 is the index of Alice’s account, and the hex string is the address of Alice’s account. The index is just a way to refer to Alice’s account. The account index is a local CLI index that can be used in other CLI commands for users to conveniently refer to the accounts they have created. The index is meaningless to the blockchain. Alice’s account will be created on the blockchain only when either money is added to Alice’s account via minting, or money is transferred to Alice’s account via a transfer from another user. Note that you may also use the hex address in CLI commands. The account index is just a convenience wrapper around the account address.

### Step 3: Create Bob’s Account

To create Bob’s account, repeat the account creation command:

`libra% account create`

Sample output on success:

```plaintext
>> Creating/retrieving next local account from wallet
Created/retrieved local account #1 address 33138303ce638c8fa469435250f5f1c3
```

1 is the index for Bob’s account, and the hex string is the address of Bob’s account.
For more details on index refer to [Create Alice’s Account.](#step-2-create-alice-s-account)

### Step 4 (Optional): List Accounts

To list the accounts you have created, enter this command:

`libra% account list`

Sample output on success:
```plaintext
User account index: 0, address: cc2219df031a68115fad9aee98e051e9, sequence number: 0, status: Local
User account index: 1, address: 33138303ce638c8fa469435250f5f1c3, sequence number: 0, status: Local
```
The sequence number for an account indicates the number of transactions that have been sent from that account. It is incremented every time a transaction sent from that account is executed and stored in the blockchain. To know more, refer to [sequence number](reference/glossary.md#sequence-number).

## Add Libra Coins to Alice’s and Bob’s Accounts

Minting and adding coins to accounts on testnet is done via Faucet. Faucet is a service that runs along with the testnet. This service only exists to facilitate minting coins for testnet and will not exist for [mainnet](reference/glossary.md#mainnet). It creates Libra with no real-world value. Assuming you have [created Alice’s and Bob’s account](#create-alice-s-and-bob-s-account), with index 0 and index 1 respectively, you can follow the steps below to add Libra to both accounts.

### Step 1: Add 110 LBR to Alice’s Account

To mint Libra and add to Alice’s account, enter this command:

`libra% account mint 0 110 LBR`

* 0 is the index of Alice’s account.
* 110  is the amount of Libra to be added to Alice’s account.
* LBR is the currency code for Libra

A successful account mint command will also create Alice’s account on the blockchain.

Sample output on success:

```plaintext
>> Creating recipient account before minting from faucet
waiting ....
transaction executed!
no events emitted
>> Sending coins from faucet
Request submitted to faucet
```
Note that when the request is submitted, it means that it has been added to the mempool (of a validator node on testnet) successfully. It does not necessarily imply that it will be successfully completed. Later, we will query the account balance to confirm if minting was successful.

If your account mint command did not submit your request successfully, refer to
[Troubleshooting](#minting-and-adding-money-to-account)

### Step 2: Add 52 LBR to Bob’s Account

To mint Libra and add to Bob’s account, enter this command:

`libra% account mint 1 52 LBR`

* 1 is the index of Bob’s account.
* 52 is the amount of Libra to be added to Bob’s account.
* LBR is the currency code for Libra
* A successful account mint command will also create Bob’s account on the blockchain. Another way to create Bob’s account on the blockchain is to transfer money from Alice’s account to Bob’s account.

Sample output on success:

```plaintext
>> Creating recipient account before minting from faucet
waiting ....
transaction executed!
no events emitted
>> Sending coins from faucet
Request submitted to faucet
```
If your account mint command did not submit your request successfully, refer to
[Troubleshooting](#minting-and-adding-money-to-account)

### Step 3: Check the Balance

To check the balance in Alice’s account, enter this command:

`libra% query balance 0`

Sample output on success:

`Balance is: 110.000000LBR`

To check the balance in Bob’s account, enter this command:

`libra% query balance 1`

Sample output on success:

`Balance is: 52.000000LBR`

## Submit a Transaction

Before we submit a transaction to transfer Libra from Alice’s account to Bob’s account, we will query the sequence number of each account. This will help us understand how executing a transaction changes the sequence number of each account.

### Query the Accounts’ Sequence Numbers

```plaintext
libra% query sequence 0
>> Getting current sequence number
Sequence number is: 0
libra% query sequence 1
>> Getting current sequence number
Sequence number is: 0
```

In `query sequence 0`, 0 is the index of Alice’s account. A sequence number of 0 for both Alice’s and Bob’s accounts indicates that no transactions from either Alice’s or Bob’s account has been executed so far.

### Transfer Money

To submit a transaction to transfer 10 LBR from Alice’s account to Bob’s account, enter this command:

`libra% transfer 0 1 10 LBR`

* 0 is the index of Alice’s account.
* 1 is the index of Bob’s account.
* 10 is the number of Libra to transfer from Alice’s account to Bob’s account.
* LBR is the currency code for Libra

Sample output on success:

```plaintext
>> Transferring
Transaction submitted to validator
To query for transaction status, run: query txn_acc_seq 0 0 <fetch_events=true|false>
```

You can use the command `query txn_acc_seq 0 0 true` (transaction by account and sequence number) to retrieve the information about the transaction you just submitted. The first parameter is the local index of the sender account, and the second parameter is the sequence number of the account. To see a sample output of this command refer to [Sample Outputs](#query-transaction-by-account-and-sequence-number).

You just submitted your transaction to a validator node on testnet, and it was included in the [mempool](reference/glossary.md#mempool) of the validator. This doesn't necessarily mean your transaction has been executed. In theory, if the system were slow or overloaded, it would take some time to see the results, and you may have to check multiple times by querying the accounts. To query an account with index 0, you can use the command  `query account_state 0.` The expected output is shown in the [Sample Outputs](#query-events) section

To troubleshoot the transfer command, refer to [Troubleshooting](#the-transfer-command).

**The Blocking Transfer Command**: You can use the `transferb` command (as shown below), instead of the `transfer` command. `transferb` will submit the transaction and return to the client prompt only after the transaction has been committed to the blockchain. An example is shown below:

`libra% transferb 0 1 10 LBR`

Refer to [Life of a Transaction](life-of-a-transaction.md) for an understanding of the lifecycle of a transaction from submission to execution and storage.

### Query Sequence Number After Transfer

```plaintext
libra% query sequence 0
>> Getting current sequence number
Sequence number is: 1
libra% query sequence 1
>> Getting current sequence number
Sequence number is: 0
```

The sequence number of 1 for Alice’s account (index 0) indicates that one transaction has been sent from Alice’s account so far. The sequence number of 0 for Bob’s account (index 1) indicates that no transaction has been sent from Bob’s account so far. Every time a transaction is sent from an account, the sequence number is incremented by 1.

### Check the Balance in Both Accounts After Transfer

To check the final balance in both accounts, query the balance again for each account as you did in [this step](#step-3-check-the-balance). If your transaction (transfer) executed successfully, you should see 100 LBR in Alice’s account and 62 LBR in Bob’s account.

```plaintext
libra% query balance 0
Balance is: 100.000000LBR
libra% query balance 1
Balance is: 62.000000LBR
```

### Congratulations!

You have successfully executed your transaction on the Libra testnet and transferred 10 LBR from Alice’s account to Bob’s account!

## Troubleshooting

### Setup

* Update Rust:
    * Run `rustup update` from your libra directory.
* Update protoc:
    * Update `protoc` to version 3.6.0 or above.
* Re-run setup script from your libra directory:
    * `./scripts/dev_setup.sh`

### Client Build and Run

If you are experiencing build failures, try to remove the cargo lock file from the libra directory:

* `rm Cargo.lock`

If your client did not connect to the testnet:

* Check your internet connection.
* Ensure that you are using the latest version of the client. Pull the latest Libra Core and rerun the client:
    * `./scripts/cli/start_cli_testnet.sh`


### Minting and Adding Money to Account

* If the validator node you connected to on testnet is unavailable, you will get a “Server unavailable” message as shown below:

  ```plaintext
  libra% account mint 0 110 LBR
  ....
  [ERROR] Error transferring coins from faucet: Server unavailable, please retry and/or check **if** host passed to the client is running
  ```
* If your balance was not updated after submitting a transaction, wait a moment and query the balance again. There may be a delay if the blockchain is experiencing a very high volume of transactions.  If your balance still is not updated, please try minting again.

* To check if an account exists, query the account state. For an account with index 0 enter this:

  `libra% query account_state 0`

### The Transfer Command

If the testnet validator node (your client was connected to) is unavailable or your connection to the testnet has timed-out, you will see this error:

```plaintext
libra% transfer 0 1 10 LBR
>> Transferring
[ERROR] Failed to perform transaction: Server unavailable, please retry and/or check if host passed to the client is running
```
To troubleshoot transfer errors:

* Check the connection to testnet.
* Query the sender account to make sure it exists. Use the following command for an account with index 0:
    * `query account_state 0`
* You can try quitting the client using `quit` or `q!`, and rerun the following command to connect to the testnet:
    * `./scripts/cli/start_cli_testnet.sh` from the libra directory

## Sample Outputs of Additional Query Commands

### Query Transaction by Account and Sequence Number

This example will query for a single transaction's details using the account and sequence number.

```plaintext
libra% query txn_acc_seq 0 0 true
>> Getting committed transaction by account and sequence number
Committed transaction: TransactionView {
    version: 2788,
    transaction: UserTransaction {
        sender: "cc2219df031a68115fad9aee98e051e9",
        signature_scheme: "Scheme::Ed25519",
        signature: "6bbdb1410d2ff27de675a40c7d4d3a6cab149302ca3e4f789100ede739009147d066da60680d34dc1aa32dcf22d21ea679da22963e0027292c8b76f287aad70d",
        public_key: "babe538296960c2822f569c9ecf086e77d42c86240f6d813e7a17e0d193199ce",
        sequence_number: 0,
        chain_id: 4,
        max_gas_amount: 1000000,
        gas_unit_price: 0,
        gas_currency: "LBR",
        expiration_timestamp_secs: 1595614184,
        script_hash: "c8bc3dda60e9662965b3223c22e3d3e3e7b6f698cf1a6930a449eb99daa35e7c",
        script: PeerToPeer {
            receiver: "33138303ce638c8fa469435250f5f1c3",
            amount: 10000000,
            currency: "LBR",
            metadata: BytesView(
                "",
            ),
            metadata_signature: BytesView(
                "",
            ),
        },
    },
    hash: "b33f9dfb4f0cb161b3f6ce84398381dbd132978f2810d66e04003ff64e1f2dd6",
    events: [
        EventView {
            key: BytesView(
                "0100000000000000cc2219df031a68115fad9aee98e051e9",
            ),
            sequence_number: 0,
            transaction_version: 2788,
            data: SentPayment {
                amount: AmountView {
                    amount: 10000000,
                    currency: "LBR",
                },
                receiver: BytesView(
                    "33138303ce638c8fa469435250f5f1c3",
                ),
                sender: BytesView(
                    "cc2219df031a68115fad9aee98e051e9",
                ),
                metadata: BytesView(
                    "",
                ),
            },
        },
        EventView {
            key: BytesView(
                "000000000000000033138303ce638c8fa469435250f5f1c3",
            ),
            sequence_number: 1,
            transaction_version: 2788,
            data: ReceivedPayment {
                amount: AmountView {
                    amount: 10000000,
                    currency: "LBR",
                },
                sender: BytesView(
                    "cc2219df031a68115fad9aee98e051e9",
                ),
                receiver: BytesView(
                    "33138303ce638c8fa469435250f5f1c3",
                ),
                metadata: BytesView(
                    "",
                ),
            },
        },
    ],
    vm_status: Executed,
    gas_used: 175,
}
```

Note that the transaction amount is shown in microlibra.

### Query Events

In the following example, we will query for “sent” events from the account at reference index 0.  You will notice there is a single event since we sent one transaction from this account.  The proof of the current state is also returned so that verification can be performed that no events are missing - this is done when the query does not return “limit” events.

```plaintext
libra% query event 0 sent 0 10
>> Getting events by account and event type.
EventView { key: BytesView("0100000000000000cc2219df031a68115fad9aee98e051e9"), sequence_number: 0, transaction_version: 2788, data: SentPayment { amount: AmountView { amount: 10000000, currency: "LBR" }, receiver: BytesView("33138303ce638c8fa469435250f5f1c3"), sender: BytesView("cc2219df031a68115fad9aee98e051e9"), metadata: BytesView("") } }
Last event state: AccountView {
    balances: [
        AmountView {
            amount: 100000000,
            currency: "LBR",
        },
    ],
    sequence_number: 1,
    authentication_key: BytesView(
        "1b72d1171fc0c2c41b06408c06e9d675cc2219df031a68115fad9aee98e051e9",
    ),
    sent_events_key: BytesView(
        "0100000000000000cc2219df031a68115fad9aee98e051e9",
    ),
    received_events_key: BytesView(
        "0000000000000000cc2219df031a68115fad9aee98e051e9",
    ),
    delegated_key_rotation_capability: false,
    delegated_withdrawal_capability: false,
    is_frozen: false,
    role: ParentVASP {
        human_name: "testnet",
        base_url: "https://libra.org",
        expiration_time: 18446744073709551615,
        compliance_key: BytesView(
            "b7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde",
        ),
        num_children: 0,
    },
}
```

### Query Account State

In this example, we will query for the state of a single account.

```plaintext
libra% query account_state 0
>> Getting latest account state
Latest account state is:
 Account: (
    cc2219df031a68115fad9aee98e051e9,
    Some(
        AuthenticationKey(
            [
                27,
                114,
                209,
                23,
                31,
                192,
                194,
                196,
                27,
                6,
                64,
                140,
                6,
                233,
                214,
                117,
                204,
                34,
                25,
                223,
                3,
                26,
                104,
                17,
                95,
                173,
                154,
                238,
                152,
                224,
                81,
                233,
            ],
        ),
    ),
)
 State: Some(
    AccountView {
        balances: [
            AmountView {
                amount: 100000000,
                currency: "LBR",
            },
        ],
        sequence_number: 1,
        authentication_key: BytesView(
            "1b72d1171fc0c2c41b06408c06e9d675cc2219df031a68115fad9aee98e051e9",
        ),
        sent_events_key: BytesView(
            "0100000000000000cc2219df031a68115fad9aee98e051e9",
        ),
        received_events_key: BytesView(
            "0000000000000000cc2219df031a68115fad9aee98e051e9",
        ),
        delegated_key_rotation_capability: false,
        delegated_withdrawal_capability: false,
        is_frozen: false,
        role: ParentVASP {
            human_name: "testnet",
            base_url: "https://libra.org",
            expiration_time: 18446744073709551615,
            compliance_key: BytesView(
                "b7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde",
            ),
            num_children: 0,
        },
    },
)
 Blockchain Version: 8012
```

## Run a Local Validator Node

Begin by [running a local validator network](run-local-network.md).

## Life of a Transaction

Once you have executed your first transaction, you may refer to the document [Life of a Transaction](life-of-a-transaction.md) for:

* A look "under the hood" at the lifecycle of a transaction from submission to execution.
* An understanding of the interactions between each logical component of a Libra validator as transactions get submitted and executed in the Libra ecosystem.

## Reference

* [Welcome page](welcome-to-libra.md).
* [Libra Protocol: Key Concepts](libra-protocol.md) &mdash; Introduces you to the fundamental concepts of the Libra protocol.
* [Getting Started With Move](move-overview.md) &mdash; Introduces you to a new blockchain programming language called Move.
* [Life of a Transaction](life-of-a-transaction.md) &mdash; Provides a look at what happens "under the hood" when a transaction is submitted and executed.
* [Libra Core Overview](libra-core-overview.md) &mdash; Provides the concept and implementation details of the Libra Core components through READMEs.
* [CLI Guide](reference/libra-cli.md) &mdash; Lists the commands (and their usage) of the Libra CLI client.
* [Libra Glossary](reference/glossary.md) &mdash; Provides a quick reference to Libra terminology.
