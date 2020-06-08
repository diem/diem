//! account: ricky, 0

// --------------------------------------------------------------------
// BLESSED treasury compliant account initiate first tier

//! new-transaction
//! sender: blessed
script {
    use 0x0::LibraAccount;
    use 0x0::Coin1::Coin1;
    use 0x0::Transaction;
    fun main(tc_account: &signer) {
        let dummy_auth_key_prefix = x"00000000000000000000000000000001";
        LibraAccount::create_designated_dealer<Coin1>(tc_account, 0xDEADBEEF, dummy_auth_key_prefix);
        Transaction::assert(LibraAccount::is_designated_dealer(0xDEADBEEF), 0);
        LibraAccount::add_tier(tc_account, 0xDEADBEEF, 100); // first Tier, 0th index
    }
}

// check: EXECUTED

// --------------------------------------------------------------------
// Blessed treasury initiate mint flow given DD creation
// Test add and update tier functions

//! new-transaction
//! sender: blessed
script {
    use 0x0::LibraAccount;
    use 0x0::Coin1::Coin1;
    fun main(tc_account: &signer) {
        LibraAccount::mint_to_designated_dealer<Coin1>(tc_account, 0xDEADBEEF, 99, 0);
        LibraAccount::add_tier(tc_account, 0xDEADBEEF, 1000); // second Tier
        LibraAccount::add_tier(tc_account, 0xDEADBEEF, 10000); // third Tier
    }
}

// check: MintEvent
// check: EXECUTED
