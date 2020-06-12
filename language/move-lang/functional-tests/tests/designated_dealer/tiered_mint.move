//! account: ricky, 0

// --------------------------------------------------------------------
// BLESSED treasury compliant account initiate first tier

//! new-transaction
//! sender: blessed
script {
    use 0x1::DesignatedDealer;
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
        let dummy_auth_key_prefix = x"00000000000000000000000000000001";
        LibraAccount::create_designated_dealer<Coin1>(account, &tc_capability, 0xDEADBEEF, dummy_auth_key_prefix);
        assert(DesignatedDealer::exists_at(0xDEADBEEF), 0);
        Roles::restore_capability_to_privilege(account, tc_capability);
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
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(tc_account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(tc_account);
        let designated_dealer_address = 0xDEADBEEF;
        DesignatedDealer::add_tier(&tc_capability, 0xDEADBEEF, 100); // first Tier, 0th index
        let coins = DesignatedDealer::tiered_mint<Coin1>(
            tc_account, &tc_capability, 99, designated_dealer_address, 0
        );
        LibraAccount::deposit(tc_account, designated_dealer_address, coins);
        DesignatedDealer::add_tier(&tc_capability, 0xDEADBEEF, 1000); // second Tier
        DesignatedDealer::add_tier(&tc_capability, 0xDEADBEEF, 10000); // third Tier
        Roles::restore_capability_to_privilege(tc_account, tc_capability);
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
    use 0x1::DesignatedDealer;
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(tc_account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(tc_account);
        let coins = DesignatedDealer::tiered_mint<Coin1>(
            tc_account, &tc_capability, 1001, 0xDEADBEEF, 1
        );
        LibraAccount::deposit(tc_account,  0xDEADBEEF, coins);
        Roles::restore_capability_to_privilege(tc_account, tc_capability);
    }
}

// check: ABORTED
// check: 5

// --------------------------------------------------------------------

//! new-transaction
//! sender: blessed
script {
    use 0x1::DesignatedDealer;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(tc_account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(tc_account);
        DesignatedDealer::update_tier(&tc_capability, 0xDEADBEEF, 4, 1000000); // invalid tier index (max index 3)
        Roles::restore_capability_to_privilege(tc_account, tc_capability);
    }
}


// check: ABORTED
// check: 3

// --------------------------------------------------------------------
// Validate regular account can not initiate mint, only Blessed treasury account

//! new-transaction
//! sender: ricky
script {
    use 0x1::DesignatedDealer;
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(tc_account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(tc_account);
        let coins = DesignatedDealer::tiered_mint<Coin1>(
            tc_account, &tc_capability, 1, 0xDEADBEEF, 0
        );
        LibraAccount::deposit(tc_account, 0xDEADBEEF, coins);
        Roles::restore_capability_to_privilege(tc_account, tc_capability);
    }
}

// check: ABORTED
// check: 0

// --------------------------------------------------------------------
// Tier index is one more than number of tiers, indicating unlimited minting allowed

//! new-transaction
//! sender: blessed
script {
    use 0x1::DesignatedDealer;
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(tc_account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(tc_account);
        let coins = DesignatedDealer::tiered_mint<Coin1>(
            tc_account, &tc_capability, 99999999999, 0xDEADBEEF, 3
        );
        LibraAccount::deposit(tc_account, 0xDEADBEEF, coins);
        Roles::restore_capability_to_privilege(tc_account, tc_capability);
    }
}
// check: ReceivedMintEvent
// check: MintEvent
// check: EXECUTED
