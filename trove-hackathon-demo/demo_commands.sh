# local setup
export ROOT_KEY="local_root.key"
export TC_KEY="local_root.key" # yes, the same one for root and tc in local node
export JSON_RPC_ENDPOINT="http://0.0.0.0:8080"
export REST_API_ENDPOINT="localhost:8081"

# trove testnet setup
export ROOT_KEY="root.key"
export TC_KEY="tc.key"
export JSON_RPC_ENDPOINT="http://trove.aws.hlw3truzy4ls.com"
export REST_API_ENDPOINT="https://api.trove.aws.hlw3truzy4ls.com"

# create the "trove foundation" account
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path $TC_KEY --account-address 0xB1E55ED create-basic-account 0xF351399F57CA26FA57C967A5448C3700 41ff1c357d5ef705e9682c5bd374fc24
# get the resource types of the newly created account
curl ${REST_API_ENDPOINT}/accounts/0xF351399F57CA26FA57C967A5448C3700/resources | jq '.[] | .type.module'
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path trove_account.key --account-address 0xF351399F57CA26FA57C967A5448C3700 create-basic-account 0x34CD440B72D03907100007D7BC0080A1 44f0d42a415212a4ab59749efb6ac9a9
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path bars_account.key --account-address 0x34CD440B72D03907100007D7BC0080A1 create-basic-account 0x3132E2B5216A46DFCF8154079954C129 45ecaf56addd68aa8be08e2763c09857
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path bars_account.key --account-address 0x34CD440B72D03907100007D7BC0080A1 create-basic-account 0x1A08E8165BB9225702495E8CB6E57E61 5839ada2d75b6c43f194123337a11c5e
# initialize NFT
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path $ROOT_KEY --account-address 0xA550C18 init-multi-token
# verify resources
curl ${REST_API_ENDPOINT}/accounts/0xA550C18/resources | jq '.[] | select(.type.module | contains("NFT"))'

# register user (give BARS capability to mint on behalf of the user)
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path user_account1.key --account-address 0x3132E2B5216A46DFCF8154079954C129 register-bars-user
# Look at all the resources added
curl ${REST_API_ENDPOINT}/accounts/0x3132E2B5216A46DFCF8154079954C129/resources | jq '.[] | .type.name'
# Creation delegation resource
curl ${REST_API_ENDPOINT}/accounts/0x3132E2B5216A46DFCF8154079954C129/resources | jq '.[] | select(.type.name | contains("CreationDelegation"))'

# register another user
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path user_account2.key --account-address 0x1A08E8165BB9225702495E8CB6E57E61 register-bars-user
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path bars_account.key --account-address 0x34CD440B72D03907100007D7BC0080A1 mint-bars-nft --creator-addr 0x3132E2B5216A46DFCF8154079954C129 --creator-name "David Marcus" --content-uri "www.diem.com" --amount 100
# Token data collection
curl ${REST_API_ENDPOINT}/accounts/0x3132E2B5216A46DFCF8154079954C129/resources | jq '.[] | select(.type.name | contains("TokenDataCollection"))'
# Token gallery
curl ${REST_API_ENDPOINT}/accounts/0x3132E2B5216A46DFCF8154079954C129/resources | jq '.[] | select(.type.name | contains("NFTGallery"))'
# Transfer NFT
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path user_account1.key --account-address 0x3132E2B5216A46DFCF8154079954C129 transfer-bars-nft --to 0x1A08E8165BB9225702495E8CB6E57E61 --amount 10 --creator 0x3132E2B5216A46DFCF8154079954C129 --creation-num 2
# Verify the new balance of the second user
curl ${REST_API_ENDPOINT}/accounts/0x1A08E8165BB9225702495E8CB6E57E61/resources | jq '.[] | select(.type.name | contains("NFTGallery"))'

# Setup trove framework account to require on-chain voting for module publishing
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path trove_account.key --account-address 0xF351399F57CA26FA57C967A5448C3700 module-publish-set-approval --enable

export MODULE_PATH="/Users/dmitrip/BARSToken.mv"
# Propose the given module for pre-approval
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path trove_account.key --account-address 0xF351399F57CA26FA57C967A5448C3700 propose-approve-module --path $MODULE_PATH
# Vote for the given module approval
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path trove_account.key --account-address 0xF351399F57CA26FA57C967A5448C3700 vote-approve-module --path $MODULE_PATH --counter 0
# Now that the module is approved, one can publish it
cargo run -- --jsonrpc-endpoint $JSON_RPC_ENDPOINT --account-key-path trove_account.key --account-address 0xF351399F57CA26FA57C967A5448C3700 publish-module --path $MODULE_PATH
# Verify the module is published
curl ${REST_API_ENDPOINT}/accounts/0xF351399F57CA26FA57C967A5448C3700/modules | jq