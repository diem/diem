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
        let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
        let dummy_auth_key_prefix = x"00000000000000000000000000000001";
        LibraAccount::create_designated_dealer<Coin1>(
            account, 0xDEADBEEF, dummy_auth_key_prefix, x"", x"", pubkey, false
        );
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
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(tc_account: &signer) {
        let designated_dealer_address = 0xDEADBEEF;
        LibraAccount::tiered_mint<Coin1>(
            tc_account, designated_dealer_address, 99, 0
        );
    }
}

// check: ReceivedMintEvent
// check: MintEvent
// check: EXECUTED

// --------------------------------------------------------------------
// Mint initiated but amount exceeds 1st tier upperbound

//! new-transaction
//! sender: blessed
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(tc_account: &signer) {
        LibraAccount::tiered_mint<Coin1>(
            tc_account, 0xDEADBEEF, 5000001, 1
        );
    }
}

// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 6

// --------------------------------------------------------------------

//! new-transaction
//! sender: blessed
script {
    use 0x1::DesignatedDealer;
    use 0x1::Coin1::Coin1;
    fun main(tc_account: &signer) {
        // DesignatedDealer::update_tier(&tc_capability, 0xDEADBEEF, 4, 1000000); // invalid tier index (max index 3)
        DesignatedDealer::update_tier<Coin1>(tc_account, 0xDEADBEEF, 4, 1000000); // invalid tier index (max index 3)
    }
}

// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 3

// --------------------------------------------------------------------
// Validate regular account can not initiate mint, only Blessed treasury account

//! new-transaction
//! sender: ricky
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    fun main(tc_account: &signer) {
        LibraAccount::tiered_mint<Coin1>(
            tc_account, 0xDEADBEEF, 1, 0
        );
    }
}

// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 0
