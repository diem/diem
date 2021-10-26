cargo run -- --account-key-path local_root.key --account-address 0xB1E55ED create-basic-account 0xF351399F57CA26FA57C967A5448C3700 41ff1c357d5ef705e9682c5bd374fc24
cargo run -- --account-key-path trove_account.key --account-address 0xF351399F57CA26FA57C967A5448C3700 create-basic-account 0x34CD440B72D03907100007D7BC0080A1 44f0d42a415212a4ab59749efb6ac9a9
cargo run -- --account-key-path bars_account.key --account-address 0x34CD440B72D03907100007D7BC0080A1 create-basic-account 0x3132E2B5216A46DFCF8154079954C129 45ecaf56addd68aa8be08e2763c09857
cargo run -- --account-key-path bars_account.key --account-address 0x34CD440B72D03907100007D7BC0080A1 create-basic-account 0x1A08E8165BB9225702495E8CB6E57E61 5839ada2d75b6c43f194123337a11c5e
cargo run -- --account-key-path local_root.key --account-address 0xA550C18 init-multi-token
cargo run -- --account-key-path user_account1.key --account-address 0x3132E2B5216A46DFCF8154079954C129 register-bars-user
cargo run -- --account-key-path user_account2.key --account-address 0x1A08E8165BB9225702495E8CB6E57E61 register-bars-user
cargo run -- --account-key-path bars_account.key --account-address 0x34CD440B72D03907100007D7BC0080A1 mint-bars-nft --creator-addr 0x3132E2B5216A46DFCF8154079954C129 --creator-name "David Marcus" --content-uri "www.diem.com" --amount 100
cargo run -- --account-key-path user_account1.key --account-address 0x3132E2B5216A46DFCF8154079954C129 transfer-bars-nft --to 0x1A08E8165BB9225702495E8CB6E57E61 --amount 10 --creator 0x3132E2B5216A46DFCF8154079954C129 --creation-num 2

# Setup trove framework account to require on-chain voting for module publishing
export MODULE_PATH="/Users/dmitrip/diem/language/diem-framework/DPN/releases/artifacts/current/modules/BARSToken.mv"
cargo run -- --account-key-path trove_account.key --account-address 0xF351399F57CA26FA57C967A5448C3700 module-publish-set-approval --enable
# Propose the given module for pre-approval
cargo run -- --account-key-path trove_account.key --account-address 0xF351399F57CA26FA57C967A5448C3700 propose-approve-module --path $MODULE_PATH
# Vote for the given module approval
cargo run -- --account-key-path trove_account.key --account-address 0xF351399F57CA26FA57C967A5448C3700 vote-approve-module --path $MODULE_PATH --counter 0
# Now that the module is approved, one can publish it
cargo run -- --account-key-path trove_account.key --account-address 0xF351399F57CA26FA57C967A5448C3700 publish-module --path $MODULE_PATH
