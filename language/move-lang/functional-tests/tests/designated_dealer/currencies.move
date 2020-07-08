// Check that the add_all_currencies flag does the expected thing


//! new-transaction
//! sender: blessed
script {
    use 0x1::DualAttestation;
    use 0x1::LibraAccount;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::LBR::LBR;
    fun main(account: &signer) {
        let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
        let dummy_auth_key_prefix = x"00000000000000000000000000000001";
        LibraAccount::create_designated_dealer<Coin1>(
            account, 0x2, copy dummy_auth_key_prefix, b"name", b"url", copy pubkey, false);
        LibraAccount::create_designated_dealer<Coin1>(
            account, 0x3, dummy_auth_key_prefix, b"A", b"B", copy pubkey, true
        );

        assert(LibraAccount::accepts_currency<Coin1>(0x2), 0);
        assert(!LibraAccount::accepts_currency<Coin2>(0x2), 1);
        assert(!LibraAccount::accepts_currency<LBR>(0x2), 2);
        assert(DualAttestation::human_name(0x2) == b"name", 77);
        assert(DualAttestation::base_url(0x2) == b"url", 78);
        assert(DualAttestation::compliance_public_key(0x2) == pubkey, 79);

        assert(LibraAccount::accepts_currency<Coin1>(0x3), 3);
        assert(LibraAccount::accepts_currency<Coin2>(0x3), 4);
        assert(LibraAccount::accepts_currency<LBR>(0x3), 5);
    }
}

// check: EXECUTED
