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


//TODO(moezinia) add burn txn once specific address directive sender complete
// and with new burn flow
