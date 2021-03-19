//! account: ricky, 0

// --------------------------------------------------------------------
// BLESSED treasury compliant account initiate first tier

//! new-transaction
//! sender: blessed
script {
    use 0x1::DesignatedDealer;
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    fun main(account: signer) {
    let account = &account;
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
    fun main(tc_account: signer) {
    let tc_account = &tc_account;
        let designated_dealer_address = 0xDEADBEEF;
        DiemAccount::tiered_mint<XUS>(
            tc_account, designated_dealer_address, 99, 0
        );
    }
}

// check: ReceivedMintEvent
// check: MintEvent
// check: "Keep(EXECUTED)"


//TODO(moezinia) add burn txn once specific address directive sender complete
// and with new burn flow
