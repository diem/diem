//! account: ricky, 0

// --------------------------------------------------------------------
// BLESSED treasury compliant account initiate first tier

//! new-transaction
//! sender: association
script {
    use 0x0::DesignatedDealer;
    use 0x0::Coin1::Coin1;
    use 0x0::LibraAccount;
    fun main(account: &signer) {
        let dummy_auth_key_prefix = x"00000000000000000000000000000001";
        LibraAccount::create_designated_dealer<Coin1>(account, 0xDEADBEEF, dummy_auth_key_prefix);
        assert(DesignatedDealer::exists_at(0xDEADBEEF), 0);
    }
}

// check: EXECUTED

// --------------------------------------------------------------------
// Blessed treasury initiate mint flow given DD creation
// Test add and update tier functions

//! new-transaction
//! sender: blessed
script {
    use 0x0::DesignatedDealer;
    use 0x0::LibraAccount;
    use 0x0::Coin1::Coin1;
    fun main(tc_account: &signer) {
        DesignatedDealer::add_tier(tc_account, 0xDEADBEEF, 100); // first Tier, 0th index
        let coins = DesignatedDealer::tiered_mint<Coin1>(
            tc_account, 99, 0xDEADBEEF, 0
        );
        LibraAccount::deposit(tc_account, 0xDEADBEEF, coins);
        DesignatedDealer::add_tier(tc_account, 0xDEADBEEF, 1000); // second Tier
        DesignatedDealer::add_tier(tc_account, 0xDEADBEEF, 10000); // third Tier
    }
}

// check: MintEvent
// check: EXECUTED

// --------------------------------------------------------------------
// Mint initiated but amount exceeds 1st tier upperbound

//! new-transaction
//! sender: blessed
script {
    use 0x0::DesignatedDealer;
    use 0x0::LibraAccount;
    use 0x0::Coin1::Coin1;
    fun main(tc_account: &signer) {
        let coins = DesignatedDealer::tiered_mint<Coin1>(
            tc_account, 1001, 0xDEADBEEF, 1
        );
        LibraAccount::deposit(tc_account,  0xDEADBEEF, coins);
    }
}

// check: ABORTED
// check: 5

// --------------------------------------------------------------------

//! new-transaction
//! sender: blessed
script {
    use 0x0::DesignatedDealer;
    fun main(tc_account: &signer) {
        DesignatedDealer::update_tier(tc_account, 0xDEADBEEF, 4, 1000000); // invalid tier index (max index 3)
    }
}


// check: ABORTED
// check: 3

// --------------------------------------------------------------------
// Validate regular account can not initiate mint, only Blessed treasury account

//! new-transaction
//! sender: ricky
script {
    use 0x0::DesignatedDealer;
    use 0x0::LibraAccount;
    use 0x0::Coin1::Coin1;
    fun main(tc_account: &signer) {
        let coins = DesignatedDealer::tiered_mint<Coin1>(
            tc_account, 1, 0xDEADBEEF, 0
        );
        LibraAccount::deposit(tc_account, 0xDEADBEEF, coins);
    }
}

// check: ABORTED
// check: 0
