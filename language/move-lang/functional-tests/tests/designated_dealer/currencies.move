// Check that the add_all_currencies flag does the expected thing


//! new-transaction
//! sender: blessed
script {
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::LBR::LBR;
    fun main(account: &signer) {
        let dummy_auth_key_prefix = x"00000000000000000000000000000001";
        LibraAccount::create_designated_dealer<Coin1>(account, 0x2, copy dummy_auth_key_prefix, false);
        LibraAccount::create_designated_dealer<Coin1>(account, 0x3, dummy_auth_key_prefix, true);

        assert(LibraAccount::accepts_currency<Coin1>(0x2), 0);
        assert(!LibraAccount::accepts_currency<Coin2>(0x2), 1);
        assert(!LibraAccount::accepts_currency<LBR>(0x2), 2);

        assert(LibraAccount::accepts_currency<Coin1>(0x3), 3);
        assert(LibraAccount::accepts_currency<Coin2>(0x3), 4);
        assert(LibraAccount::accepts_currency<LBR>(0x3), 5);
    }
}

// check: EXECUTED
