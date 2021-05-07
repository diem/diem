//! account: freddymac
//! account: alice, 0, 0, address

//! sender: blessed
script {
    use 0x1::XUS::XUS;
    use 0x1::DiemAccount;
    use 0x1::BCS;
    fun main(account: signer) {
        let account = &account;
        let addr: address = @0x111101;
        assert(!DiemAccount::exists_at(addr), 83);
        DiemAccount::create_parent_vasp_account<XUS>(account, addr, BCS::to_bytes(&addr), x"aa", false);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script{
    use 0x1::DiemAccount;
    use 0x1::DiemId;

    fun main(tc_account: signer) {
        let addr: address = @0x111101;
        assert(!DiemAccount::exists_at(addr), 83);
        let tc_account = &tc_account;

        /// add a diem id domain
        DiemId::update_diem_id_domain(tc_account, addr, x"aa", false);

        /// check if diem id domain is added to DiemIdDomains
        let account_domains = borrow_global_mut<DiemId::DiemIdDomains>(address);
        diem_id_domain = DiemId::DiemIdDomain {
            domain: domain
        };
        assert(Vector::contains(&account_domains.domains, diem_id_domain));

        /// check if diem id domain event is added to DiemIdDomainManager
        let domain_events = borrow_global_mut<DiemId::DiemIdDomainManager>(address).diem_id_domain_events;
        assert(Vector::contains(&account_domains.domains, diem_id_domain));
    }
}
// check: "Keep(EXECUTED)"
// check: DiemIdDomainEvent


//! new-transaction
//! sender: blessed
script{
    use 0x1::DiemAccount;
    use 0x1::DiemId;

    fun main(tc_account: signer) {
        let addr: address = @0x111101;
        assert(!DiemAccount::exists_at(addr), 83);
        let tc_account = &tc_account;

        /// remove a diem id domain
        DiemId::update_diem_id_domain(tc_account, addr, x"aa", false);

        /// check if diem id domain is removed from DiemIdDomains
        let account_domains = borrow_global_mut<DiemId::DiemIdDomains>(address);
        diem_id_domain = DiemId::DiemIdDomain {
            domain: domain
        };
        assert(Vector::contains(&account_domains.domains, diem_id_domain));

        /// check if diem id domain event is added to DiemIdDomainManager
        let domain_events = borrow_global_mut<DiemId::DiemIdDomainManager>(address).diem_id_domain_events;
        assert(Vector::contains(&account_domains.domains, diem_id_domain));
    }
}
// check: "Keep(EXECUTED)"
// check: DiemIdDomainEvent

//! new-transaction
//! sender: freddymac
script{
    use 0x1::DiemAccount;
    use 0x1::DiemId;

    fun main(account: signer) {
        /// check if vasp account tries to add domain id, it fails
        let account = &account;
        let addr: address = @0x111101;
        DiemId::update_diem_id_domain(account, addr, x"aa", false);
    }
}
// check: "Keep(ABORTED { code: 258,"


/// check if domain is removed properly