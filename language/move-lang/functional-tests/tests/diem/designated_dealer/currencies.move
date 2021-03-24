// Check that the add_all_currencies flag does the expected thing


//! new-transaction
//! sender: blessed
script {
    use 0x1::DualAttestation;
    use 0x1::DiemAccount;
    use 0x1::XUS::XUS;
    use 0x1::XDX::XDX;
    fun main(account: signer) {
    let account = &account;
        let dummy_auth_key_prefix = x"00000000000000000000000000000001";
        DiemAccount::create_designated_dealer<XUS>(
            account, 0x2, copy dummy_auth_key_prefix, b"name", false
        );
        DiemAccount::create_designated_dealer<XUS>(
            account, 0x3, dummy_auth_key_prefix, b"other_name", true
        );

        assert(DiemAccount::accepts_currency<XUS>(0x2), 0);
        assert(!DiemAccount::accepts_currency<XDX>(0x2), 2);
        assert(DualAttestation::human_name(0x2) == b"name", 77);
        assert(DualAttestation::base_url(0x2) == b"", 78);
        assert(DualAttestation::compliance_public_key(0x2) == b"", 79);

        assert(DiemAccount::accepts_currency<XUS>(0x3), 3);
        assert(DiemAccount::accepts_currency<XDX>(0x3), 5);
    }
}

// check: "Keep(EXECUTED)"
