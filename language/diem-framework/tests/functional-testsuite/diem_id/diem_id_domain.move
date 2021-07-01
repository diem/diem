//! account: bob, 0, 0, address

//! sender: blessed
script {
    use 0x1::XUS::XUS;
    use 0x1::DiemAccount;
    use 0x1::DiemId;

    fun main(account: signer) {
        let tc_account = &account;
        let addr: address = @{{bob}};
        assert(!DiemAccount::exists_at(addr), 83);
        DiemAccount::create_parent_vasp_account<XUS>(tc_account, addr, {{bob::auth_key}}, x"aa", false);
        assert(DiemId::tc_domain_manager_exists(), 77);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script{
    use 0x1::DiemAccount;
    use 0x1::DiemId;

    fun main(tc_account: signer) {
        let addr: address = @{{bob}};
        assert(DiemAccount::exists_at(addr), 455);
        let tc_account = &tc_account;
        let domain_name = b"diem";

        /// add a diem id domain
        DiemId::add_diem_id_domain(tc_account, addr, copy domain_name);

        /// check if diem id domain is added to DiemIdDomains
        assert(DiemId::has_diem_id_domain(addr, domain_name), 5);
    }
}
// check: DiemIdDomainEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script{
    use 0x1::DiemAccount;
    use 0x1::DiemId;

    fun main(tc_account: signer) {
        let addr: address = @{{bob}};
        assert(DiemAccount::exists_at(addr), 455);
        let tc_account = &tc_account;
        let domain_name = b"diem";

        /// add the same diem ID domain to the bob account, expect it to fail
        DiemId::add_diem_id_domain(tc_account, addr, copy domain_name);

        /// check if the previously added domain is still there
        assert(DiemId::has_diem_id_domain(addr, domain_name), 389);
    }
}
// check: "Keep(ABORTED { code: 775,"

//! new-transaction
//! sender: blessed
script{
    use 0x1::DiemAccount;
    use 0x1::DiemId;

    fun main(tc_account: signer) {
        let addr: address = @{{bob}};
        assert(DiemAccount::exists_at(addr), 2);
        let tc_account = &tc_account;
        let domain_name = b"diem";

        /// remove a diem id domain
        DiemId::remove_diem_id_domain(tc_account, addr, copy domain_name);

        /// check if diem id domain is removed from DiemIdDomains
        assert(!DiemId::has_diem_id_domain(addr, domain_name), 205);
    }
}
// check: DiemIdDomainEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script{
    use 0x1::DiemAccount;
    use 0x1::DiemId;

    fun main(tc_account: signer) {
        let addr: address = @{{bob}};
        assert(DiemAccount::exists_at(addr), 455);
        let tc_account = &tc_account;
        let domain_name = b"aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffffgggg";

        /// Try adding a domain ID longer than 63 characters, expect to fail
        DiemId::add_diem_id_domain(tc_account, addr, copy domain_name);

        /// check that diem id domain is not added to DiemIdDomains
        assert(!DiemId::has_diem_id_domain(addr, domain_name), 888);
    }
}
// check: "Keep(ABORTED { code: 1287,"

//! new-transaction
//! sender: bob
script{
    use 0x1::DiemId;

    fun main(account: signer) {
        /// check if vasp account tries to add domain id, it fails
        let account = &account;
        let addr: address = @{{bob}};
        DiemId::add_diem_id_domain(account, addr, b"diem");
    }
}
// check: "Keep(ABORTED { code: 258,"
