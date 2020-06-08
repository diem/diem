//! account: ricky, 0

// --------------------------------------------------------------------
// BLESSED treasury compliant account initiate first tier

//! new-transaction
//! sender: blessed
script {
    use 0x0::LibraAccount;
    use 0x0::Coin1;
    use 0x0::Transaction;
    fun main(tc_account: &signer) {
        let dummy_auth_key_prefix = x"00000000000000000000000000000001";
        LibraAccount::create_designated_dealer<Coin1::T>(tc_account, 0xDEADBEEF, dummy_auth_key_prefix);
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
    use 0x0::Coin1;
    fun main(tc_account: &signer) {
        let approval_timestamp = 0;
        LibraAccount::mint_to_designated_dealer<Coin1::T>(tc_account, 0xDEADBEEF, 90, 0, approval_timestamp);
        LibraAccount::add_tier(tc_account, 0xDEADBEEF, 1000); // second Tier
        LibraAccount::add_tier(tc_account, 0xDEADBEEF, 10000); // third Tier
    }
}

// check: MintEvent
// check: EXECUTED

// --------------------------------------------------------------------
// Another tiered mint 100 secs later (within same 24 hour window)
// spills over 0'th tier upperbound and is therefore aborted

//! new-transaction
//! sender: blessed
script {
    use 0x0::LibraAccount;
    use 0x0::Coin1;
    fun main(tc_account: &signer) {
        let approval_timestamp = 100;
        LibraAccount::mint_to_designated_dealer<Coin1::T>(tc_account, 0xDEADBEEF, 20, 0, approval_timestamp);
    }
}

// check: ABORTED

// --------------------------------------------------------------------
// Another tiered mint (within same 24 hour window) with tier-1 index successfully mints
// even though cumulative amount spills over

//! new-transaction
//! sender: blessed
script {
    use 0x0::LibraAccount;
    use 0x0::Coin1;
    fun main(tc_account: &signer) {
        let approval_timestamp = 200;
        LibraAccount::mint_to_designated_dealer<Coin1::T>(tc_account, 0xDEADBEEF, 120, 1, approval_timestamp);
    }
}

// check: EXECUTED

// --------------------------------------------------------------------
// Another tiered mint (outside same 24 hour window) with tier-0 index
// successfully mints as the cumulative mint sum reset.

//! new-transaction
//! sender: blessed
script {
    use 0x0::LibraAccount;
    use 0x0::Coin1;
    fun main(tc_account: &signer) {
        let approval_timestamp = 86400000001; // more than 24 hours since 1st tiered mint
        LibraAccount::mint_to_designated_dealer<Coin1::T>(tc_account, 0xDEADBEEF, 90, 0, approval_timestamp);
    }
}

// check: EXECUTED


// --------------------------------------------------------------------
// Mint initiated but amount exceeds 1st tier upperbound

//! new-transaction
//! sender: blessed
script {
    use 0x0::LibraAccount;
    use 0x0::Coin1;
    fun main(tc_account: &signer) {
        LibraAccount::mint_to_designated_dealer<Coin1::T>(tc_account, 0xDEADBEEF, 1001, 1, 0);
    }
}

// check: ABORTED
// check: 5

// --------------------------------------------------------------------

//! new-transaction
//! sender: blessed
script {
    use 0x0::LibraAccount;
    fun main(tc_account: &signer) {
        LibraAccount::update_tier(tc_account, 0xDEADBEEF, 4, 1000000); // invalid tier index (max index 3)
    }
}


// check: ABORTED
// check: 3

// --------------------------------------------------------------------
// Validate regular account can not initiate mint, only Blessed treasury account

//! new-transaction
//! sender: ricky
script {
    use 0x0::LibraAccount;
    use 0x0::Coin1;
    fun main(tc_account: &signer) {
        LibraAccount::mint_to_designated_dealer<Coin1::T>(tc_account, 0xDEADBEEF, 1, 0, 0);
    }
}

// check: ABORTED
// check: 0
