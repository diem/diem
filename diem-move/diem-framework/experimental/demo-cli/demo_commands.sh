# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

# local setup
export ROOT_KEY="local_root.key"
export TC_KEY="local_root.key"
export JSON_RPC_ENDPOINT="http://0.0.0.0:8080"
export REST_API_ENDPOINT="localhost:8080"
# run a local node
cargo run --bin diem-node -- --test --lazy --open-publishing --genesis-modules diem-move/diem-framework/experimental/releases/artifacts/current --seed 0000000000000000000000000000000000000000000000000000000000000000

cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path $TC_KEY --account-address 0xB1E55ED create-basic-account 0xF351399F57CA26FA57C967A5448C3700 41ff1c357d5ef705e9682c5bd374fc24
# get all the resource types of the newly created account
curl ${REST_API_ENDPOINT}/accounts/0xF351399F57CA26FA57C967A5448C3700/resources | jq '.[] | .type'
# look at the event
curl "${REST_API_ENDPOINT}/accounts/0xB1E55ED/transactions?start=0&limit=1" | jq
