//! account: bob, 1000000, 0, validator
//! account: alice, 0, 0, address

//! new-transaction
//! sender: diemroot
//! args: 0, {{alice}}, {{alice::auth_key}}, b"alice"
stdlib_script::AccountCreationScripts::create_validator_operator_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
    use 0x1::ValidatorConfig;
    fun main(account: signer) {
        let account = &account;
        // bob rotates his public key to invalid key
        ValidatorConfig::set_operator(account, {{alice}});
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    use 0x1::ValidatorConfig;
    fun main(account: signer) {
        let account = &account;
        // bob rotates his public key to invalid key
        ValidatorConfig::set_config(account, {{bob}},
                                    x"0000000000000000000000000000000000000000000000000000000000000000",
                                    x"", x"");
    }
}
// check: "Keep(ABORTED { code: 519,"

//! new-transaction
//! sender: alice
script {
    use 0x1::ValidatorConfig;
    fun main(account: signer) {
        let account = &account;
        // bob rotates his public key to a valid key
        ValidatorConfig::set_config(account, {{bob}},
                                    x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
                                    x"", x"");
    }
}
// check: "Keep(EXECUTED)"
