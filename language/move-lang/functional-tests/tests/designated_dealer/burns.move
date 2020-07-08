//! account: ricky, 0

// --------------------------------------------------------------------
// BLESSED treasury compliant account initiate first tier

//! new-transaction
//! sender: blessed
script {
    use 0x1::DesignatedDealer;
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(account: &signer) {
        let dummy_auth_key_prefix = x"00000000000000000000000000000001";
        LibraAccount::create_designated_dealer<Coin1>(account, 0xDEADBEEF, dummy_auth_key_prefix, false);
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
    use 0x1::DesignatedDealer;
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(tc_account: &signer) {
        let designated_dealer_address = 0xDEADBEEF;
        DesignatedDealer::add_tier(tc_account, 0xDEADBEEF, 100); // first Tier, 0th index
        LibraAccount::tiered_mint<Coin1>(
            tc_account, designated_dealer_address, 99, 0
        );
        DesignatedDealer::add_tier(tc_account, 0xDEADBEEF, 1000); // second Tier
        DesignatedDealer::add_tier(tc_account, 0xDEADBEEF, 10000); // third Tier
    }
}

// check: ReceivedMintEvent
// check: MintEvent
// check: EXECUTED


//TODO(moezinia) add burn txn once specific address directive sender complete
// and with new burn flow
