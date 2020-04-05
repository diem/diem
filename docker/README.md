# Docker

This directory contains [Docker](https://www.docker.com/) configuration and scripts for building
images of the validator node, faucet server and client.

## Building Docker images

1. [Download](https://docs.docker.com/install/) and install Docker.
2. Build the docker containers:
   A. Dynamic validator: From the top level directory, run `docker/validator-dynamic/build.sh`
   B. Mint (Faucet): From the top level directory, run `docker/mint/build.sh`
3. To test locally, run the docker containers:
   A. Dynamic validator: run `docker/validator-dynamic/run.sh`
        Note: the Base validator can be run locally but requires substantial maual configuration.
   B. Mint (Faucet): run `docker/mint/run.sh`
4. Run the client as follows:
   `cargo run -p cli --bin cli -- -u http://localhost:8080 -f localhost:9080
