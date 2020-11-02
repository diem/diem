---
id: run-local-network
title: Run a Local Network
---

You can run a local test validator network to test and debug services you are developing for the Diem Blockchain and to build and test Move module code changes. This local test network is not part of the Diem Ecosystem and is only meant for testing and development purposes.

You can use the Diem CLI command dev to compile, publish, and execute Move Intermediate Representation (IR) programs on your local test validator network.

>
>Note: Your local test network of validator nodes will not be connected to the testnet, and will not be using the actual validator nodes on the Diem Blockchain.
>

## Getting Started


You can run a local test validator network in two ways: using the Diem Core source code or Docker. The Diem Core source code is useful when testing modifications to the Diem Core code base. Docker is particularly useful when building services on top of the Diem Blockchain as there is no build overhead and the ledger state persists across restarts of the network by default.


### Using Diem Core source code

1. Download and clone the Diem Core repository from GitHub and prepare your developer environment by running the following commands:

    ```
    git clone https://github.com/diem/diem.git
    cd diem
    ./scripts/dev_setup.sh
    source ~/.cargo/env
    ```
2. Run the process: `cargo run -p diem-node -- --test`. Note, after starting this process, the config path: `/distinct/tmp/path/0/node.yaml`.

> You can later restore the ledger state by running `diem-node` with a previously used configuration path: `cargo run -p diem-node -- --test --config /distinct/tmp/path`.


### Using Docker

1. Install Docker and Docker-Compose.
2. Create a directory for your local test validator network.
3. Download the validator testnet docker compose.
4. Create configuration files in the same directory so that the data can be exported out of the docker container:
    ```
    # Linux / Mac
    touch genesis.blob diem_root_key waypoint.txt

    # Windows
    fsutil file createnew genesis.blob 0
    fsutil file createnew diem_root_key 0
    fsutil file createnew waypoint.txt 0
    Run docker-compose: docker-compose up
    ```

## Interacting with the local test validator network
After starting your local test validator network, the following will be output on your terminal:

```

validator_1  | Entering test mode, this should never be used in production!
validator_1  | Completed generating configuration:
validator_1  | 	Log file: "/opt/diem/var/validator.log"
validator_1  | 	Config path: "/opt/diem/var/0/node.yaml"
validator_1  | 	Diem root key path: "/opt/diem/var/mint.key"
validator_1  | 	Waypoint: 0:7ff525d33f685a5cf26a71b393fa5159874c8f0c2861c382905f49dcb6991cb6
validator_1  | 	JSON-RPC endpoint: 0.0.0.0:8080
validator_1  | 	FullNode network: /ip4/0.0.0.0/tcp/7180
validator_1  | 	ChainId: TESTING

```
This will output a lot of information that will be required for starting the Diem CLI tool:
* ChainId - For testing
* Diem root key path - Available in the docker compose folder under `diem_root_key`.
* Waypoint - Printed on screen and available in the docker compose folder under waypoint.txt
* JSON-RPC Endpoint - `http://127.0.0.1:8080`.


In another terminal, start a `diem-client` using data from the output above:

```
$ cd ~/diem
$ cargo run -p cli -- -c $CHAIN_ID -m $ROOT_KEY -u http://127.0.0.1:8080 --waypoint $WAYPOINT
```

At the end of this tutorial you will have:
* A single local validator test network
* A Diem CLI client connected to that network

The configuration management of the validator network generates:
* A genesis transaction.
* A waypoint that serves as a verifiable checkpoint into the blockchain.
* A chain id to uniquely distinguish this chain from other chains.
* Diem root key (also known as a mint or faucet key) for the account that's allowed to perform the mint operation.



###### tags: `core`, `node`
