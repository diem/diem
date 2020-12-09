---
id: docker_compose
title: Diem Docker-Compose Configuration
custom_edit_url: https://github.com/diem/diem/edit/master/docker/compose/README.md
---

This directory contains the following compose configurations:
* **validator-testnet**: creates a single validator test network
* **mint**: creates a mint (faucet) that directly connects to validator-testnet

To use these compositions:
1. [Download](https://docs.docker.com/install/) and install Docker and Docker Compose (comes with Docker for Mac and Windows).
2. Open your favorite terminal and enter the appropriate composition directory
3. Run `docker-compose up`

To build your own complete testnet:
0. Review both **validator-testnet** and **mint** docker-compose.yaml and ensure the image points to the same tagged image.
1. Start the **validator-testnet**:
    1. Enter the **validator-testnet** diretory `cd validator-testnet`
    2. Start the composition `docker-compose up -d`
    3. Confirm that waypoint.txt is not empty
    4. Return to the compose directory: `cd ..`
2. Start **mint**:
    1. Enter the **mint** directory: `cd mint`
    2. Copy the testnet waypoint: `cp ../validator-testnet/waypoint.txt .`
    3. Copy the testnet mint.key: `cp ../validator-testnet/diem_root_key mint.key`
    4. Start the composition `docker-compose up -d`
    5. Return to the compose directory: `cd ..`
3. Enjoy your testnet:
    1. Faucet/mint will be available at http://127.0.0.1:8000
    2. JSON-RPC will be available at http://127.0.0.1:8080
