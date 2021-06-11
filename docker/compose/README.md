---
id: docker_compose
title: Diem Docker-Compose Configuration
custom_edit_url: https://github.com/diem/diem/edit/main/docker/compose/README.md
---

This directory contains the following compose configurations:
* **validator-testnet**: creates a single validator test network, and a faucet that directly connects to it
* **client-cli**: creates a CLI client which connects to the above validator/faucet
* **public_full_node**: creates a public fullnode, and it can be configured to connect to any existing network (e.g. testnet, Mainnet).
* **monitoring**: creates a monitoring stack which can be used to collect metrics and virtulize it on a dashboard. This can be installed together with other compose configurations and provides simple monitoring for the deployment.
* **data-restore**: creates a diem db-restore job to restore a data volume from provided S3 bucket. This can be used to quickly restore fullnode for an exsiting blockchain to avoid spending long time on state-sync.

To use these compositions:
1. [Download](https://docs.docker.com/install/) and install Docker and Docker Compose (comes with Docker for Mac and Windows).
2. Open your favorite terminal and enter the appropriate composition directory
3. Run `docker-compose up`

To build your own complete testnet:
1. Start the **validator-testnet** and **faucet**:
    1. Enter the **validator-testnet** directory `cd validator-testnet`
    2. Start the composition `docker-compose up -d`
    3. Return to the compose directory: `cd ..`
 2. Enjoy your testnet:
    1. Faucet will be available at http://127.0.0.1:8000
    2. JSON-RPC will be available at http://127.0.0.1:8080


If you would like to run the CLI client and interact with your testnet:
   1. Ensure the **validator-testnet** and the **faucet** are running
   2. Enter the **client-cli** directory `cd client-cli`
   3. Start the composition with **`run`, not `up`**: `docker-compose run client-cli`
   4. You should be in an interactive session in the CLI. Type `h` and press `enter` to see commands
   5. To create a new account, type `account create`; then you can reference it and add funds to it with `account mint 0 10 XUS` (mint 10 XUS for account #0)

If you would like to clear the validator/blockchain data and start from scratch, either remove the docker volume `diem-shared`,
or run `docker-compose run validator rm -rf '/opt/diem/var/*'` from the **validator-testnet** directory.

To clear just the validator logs, run  `docker-compose run validator rm -rf '/opt/diem/var/validator.log'`
