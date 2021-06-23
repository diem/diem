/// Test to check that we can write identity into the NetworkIdentity and update it
//! account: bob, 0, 0, address

//! sender: blessed
script {
    use DiemFramework::XUS::XUS;
    use DiemFramework::DiemAccount;

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
    use DiemFramework::DiemAccount;
    use DiemFramework::NetworkIdentity;
    use Std::Signer;
    use Std::Vector;

    fun main(tc_account: signer) {
        let addr: address = @{{bob}};
        assert(DiemAccount::exists_at(addr), 455);
        let tc_account = &tc_account;
        let input = b"id_1";

        /// Add identities
        NetworkIdentity::add_identities(tc_account, Vector::singleton(copy input));

        let identities: vector<vector<u8>> = NetworkIdentity::get(Signer::address_of(tc_account));
        assert(Vector::contains(&identities, &input), 0);
    }
}
// check: "NetworkIdentityChangeNotification"
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script{
    use DiemFramework::NetworkIdentity;
    use Std::Signer;
    use Std::Vector;

    fun main(tc_account: signer) {
        let tc_account = &tc_account;
        let addr = Signer::address_of(tc_account);
        let original = b"id_1";
        let input = b"id_2";

        /// Ensure that original value still exists before changing
        let identities: vector<vector<u8>> = NetworkIdentity::get(copy addr);
        assert(Vector::contains(&identities, &original), 0);

        NetworkIdentity::add_identities(tc_account, Vector::singleton(copy input));
        let identities: vector<vector<u8>> = NetworkIdentity::get(addr);
        assert(Vector::contains(&identities, &original), 0);
        assert(Vector::contains(&identities, &input), 0);
    }
}
// check: "NetworkIdentityChangeNotification"
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script{
    use DiemFramework::NetworkIdentity;
    use Std::Signer;
    use Std::Vector;

    fun main(tc_account: signer) {
        let tc_account = &tc_account;
        let addr = Signer::address_of(tc_account);
        let original = b"id_1";
        let input = b"id_2";

        NetworkIdentity::remove_identities(tc_account, Vector::singleton(copy input));
        let identities: vector<vector<u8>> = NetworkIdentity::get(addr);
        assert(Vector::contains(&identities, &original), 0);
        assert(!Vector::contains(&identities, &input), 0);
    }
}
// check: "NetworkIdentityChangeNotification"
// check: "Keep(EXECUTED)"
