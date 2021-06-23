/// Test to check that we can write identity into the NetworkIdentity and update it
//! account: bob, 0, 0, address

//! sender: blessed
script {
    use 0x1::XUS::XUS;
    use 0x1::DiemAccount;

    fun main(account: signer) {
        let tc_account = &account;
        let addr: address = @{{bob}};
        assert(!DiemAccount::exists_at(addr), 83);
        DiemAccount::create_parent_vasp_account<XUS>(tc_account, addr, {{bob::auth_key}}, x"aa", false);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script{
    use 0x1::DiemAccount;
    use 0x1::NetworkIdentity;
    use 0x1::Signer;

    fun main(tc_account: signer) {
        let addr: address = @{{bob}};
        assert(DiemAccount::exists_at(addr), 455);
        let tc_account = &tc_account;
        let input = b"id_1,id_2";

        /// Add identities
        NetworkIdentity::update_identities(tc_account, copy input);

        let identities: vector<u8> = NetworkIdentity::get(Signer::address_of(tc_account));
        assert(input == identities, 0);
    }
}
// check: "NetworkIdentityChangeNotification"
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script{
    use 0x1::NetworkIdentity;
    use 0x1::Signer;

    fun main(tc_account: signer) {
        let tc_account = &tc_account;
        let addr = Signer::address_of(tc_account);
        let original = b"id_1,id_2";
        let input = b"id_1,id_2,id_3";

        /// Ensure that original value still exists before changing
        let identities: vector<u8> = NetworkIdentity::get(copy addr);
        assert(original == identities, 0);

        /// Replace identities
        NetworkIdentity::update_identities(tc_account, copy input);
        let identities: vector<u8> = NetworkIdentity::get(addr);
        assert(input == identities, 0);
    }
}
// check: "NetworkIdentityChangeNotification"
// check: "Keep(EXECUTED)"
