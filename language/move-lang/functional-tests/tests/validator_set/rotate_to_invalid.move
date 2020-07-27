//! account: bob, 1000000, 0, validator
//! account: alice

//! new-transaction
//! sender: bob
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // bob rotates his public key to invalid key
        ValidatorConfig::set_operator(account, {{alice}});
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // bob rotates his public key to invalid key
        ValidatorConfig::set_config(account, {{bob}},
                                    x"0000000000000000000000000000000000000000000000000000000000000000",
                                    x"", x"", x"", x"");
    }
}

// check: ABORTED
// check: 3

//! new-transaction
//! sender: alice
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // bob rotates his public key to a valid key
        ValidatorConfig::set_config(account, {{bob}},
                                    x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
                                    x"", x"", x"", x"");
    }
}

// check: EXECUTED
