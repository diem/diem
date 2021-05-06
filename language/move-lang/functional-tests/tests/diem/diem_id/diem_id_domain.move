//! account: freddymac
//! account: bob, 0, 0, address

//! new-transaction
//! sender: blessed
script {
    use 0x1::XUS::XUS;
    use 0x1::DiemAccount;
    use 0x1::BCS;
    use 0x1::DiemId;
    fun main(account: signer) {
        let account = &account;
        let addr: address = {{bob}}
        assert(!DiemAccount::exists_at(addr), 83);
        DiemAccount::create_parent_vasp_account<XUS>(account, addr, BCS::to_bytes(&addr), x"aa", false);
        assert(DiemId::has_diem_id_domains(addr), 1);
    }
}
// check: "Keep(EXECUTED)"


//! new-transaction
//! sender: blessed
script{
    fun main() {
        let addr: address = {{bob}}

    }
}
// check: "Keep(EXECUTED)"
