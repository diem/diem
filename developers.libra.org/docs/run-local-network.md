---
id: run-local-network
title: Run a Local Network
---

`libra-node` supports running a single node test network on your computer.  Running a local network makes it easier to test and debug your code changes. You can use the CLI command `dev` to compile, publish, and execute Move Intermediate Representation (IR) programs on your local cluster of nodes. Refer to [Run Move Programs Locally](run-move-locally.md) for further details.

## Prerequisites

To install all the required dependencies, run the build script as described in [Clone and Build Libra Core](my-first-transaction.md#clone-and-build-libra-core). 

> Ensure your command line is in the root directory of the cloned Libra GitHub repo.

## Run a local network

<blockquote class="block_note">

**Note:** The local network of validator nodes will not be connected to the testnet; currently, it is not possible to connect the local validator network to the testnet.

</blockquote>

To start a validator node locally on your computer and create your own local blockchain network
(not connected to the Libra testnet), ensure that you have run the build script as described in
[Setup Libra Core](#setup-libra-core), change to the root directory of the Libra Core repository,
and run `libra-node` as shown below:

```bash
$ cd ~/libra
$ cargo run -p libra-node -- --test
```

This will output a lot of information that will be required for starting the cli tool:
* ChainId -- `$CHAIN_ID`
* Libra root key path -- `$FAUCET_KEY`
* Waypoint -- `$WAYPOINT`
* JSON-RPC Endpoint -- `http://127.0.0.1:8080`

In another terminal, start a `libra-client` using that output:
```bash
$ cd ~/libra
$ cargo run -p cli -- -c $CHAIN_ID -m $FAUCET_KEY -u http://127.0.0.1:8080 --waypoint $WAYPOINT
```

At the end you will have:
* A single validator test network
* A Libra CLI client connected to that network

The configuration management of the validator network generates:

* A genesis transaction.
* A waypoint that serves as a verifiable checkpoint into the blockchain.
* A chain id to uniquely distinguish this chain from other chains.
* Libra root key (also known as a mint or faucet key) for the account that's allowed to perform the mint operation.

## Reference

* [CLI Guide](reference/libra-cli.md) &mdash; Lists the commands of the Libra CLI client.
* [My First Transaction](my-first-transaction.md) &mdash; Provides step-by-step guidance on creating new accounts on the Libra Blockchain and running transactions.
