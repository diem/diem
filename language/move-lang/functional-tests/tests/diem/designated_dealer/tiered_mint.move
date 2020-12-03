//! account: ricky, 0

// --------------------------------------------------------------------
// BLESSED treasury compliance acccount creates DD with tiers of one coin type

//! new-transaction
//! sender: blessed
script {
    use 0x1::DesignatedDealer;
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    fun main(account: &signer) {
        let dummy_auth_key_prefix = x"00000000000000000000000000000001";
        DiemAccount::create_designated_dealer<XUS>(
            account, 0xDEADBEEF, dummy_auth_key_prefix, x"", false
        );
        assert(DesignatedDealer::exists_at(0xDEADBEEF), 0);
    }
}


// check: "Keep(EXECUTED)"

// --------------------------------------------------------------------
// Blessed treasury initiate mint flow given DD creation
// Test add and update tier functions

//! new-transaction
//! sender: blessed
script {
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    fun main(tc_account: &signer) {
        let designated_dealer_address = 0xDEADBEEF;
        DiemAccount::tiered_mint<XUS>(
            tc_account, designated_dealer_address, 99*1000000, 0
        );
    }
}

// check: ReceivedMintEvent
// check: MintEvent
// check: "Keep(EXECUTED)"

// --------------------------------------------------------------------
// Mint initiated but amount exceeds 1st tier upperbound

//! new-transaction
//! sender: blessed
script {
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    fun main(tc_account: &signer) {
        DiemAccount::tiered_mint<XUS>(
            tc_account, 0xDEADBEEF, 5000001*1000000, 1
        );
    }
}

// check: "Keep(ABORTED { code: 1287,"

// --------------------------------------------------------------------
// Mint initiated and is below 2nd tier upperbound

//! new-transaction
//! sender: blessed
script {
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    fun main(tc_account: &signer) {
        DiemAccount::tiered_mint<XUS>(
            tc_account, 0xDEADBEEF, 5000001*1000000, 2
        );
    }
}

// check: "Keep(EXECUTED)"

// --------------------------------------------------------------------
// Mint initiated and is below 3rd tier upperbound

//! new-transaction
//! sender: blessed
script {
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    fun main(tc_account: &signer) {
        DiemAccount::tiered_mint<XUS>(
            tc_account, 0xDEADBEEF, 50000001*1000000, 3
        );
    }
}

// check: "Keep(EXECUTED)"

// --------------------------------------------------------------------
// Too large

//! new-transaction
//! sender: blessed
script {
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    fun main(tc_account: &signer) {
        DiemAccount::tiered_mint<XUS>(
            tc_account, 0xDEADBEEF, 500000001*1000000, 3
        );
    }
}

// check: "Keep(ABORTED { code: 1287,"

// --------------------------------------------------------------------

//! new-transaction
//! sender: blessed
script {
    use 0x1::DesignatedDealer;
    use 0x1::XUS::XUS;
    fun main(tc_account: &signer) {
        // DesignatedDealer::update_tier(&tc_capability, 0xDEADBEEF, 4, 1000000); // invalid tier index (max index 3)
        DesignatedDealer::update_tier<XUS>(tc_account, 0xDEADBEEF, 4, 1000000); // invalid tier index (max index 3)
    }
}

// check: "Keep(ABORTED { code: 775,"

// --------------------------------------------------------------------
// Validate regular account can not initiate mint, only Blessed treasury account

//! new-transaction
//! sender: ricky
script {
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    fun main(tc_account: &signer) {
        DiemAccount::tiered_mint<XUS>(
            tc_account, 0xDEADBEEF, 1, 0
        );
    }
}

// check: "Keep(ABORTED { code: 258,"
