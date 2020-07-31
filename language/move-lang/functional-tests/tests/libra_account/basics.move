//! account: bob, 10000LBR
//! account: alice, 0LBR
//! account: abby, 0, 0, address
//! account: doris, 0Coin1, 0

module Holder {
    use 0x1::Signer;

    resource struct Hold<T> {
        x: T
    }

    public fun hold<T>(account: &signer, x: T) {
        move_to(account, Hold<T>{x})
    }

    public fun get<T>(account: &signer): T
    acquires Hold {
        let Hold {x} = move_from<Hold<T>>(Signer::address_of(account));
        x
    }
}

//! new-transaction
script {
    use 0x1::LibraAccount;
    fun main(sender: &signer) {
        LibraAccount::initialize(sender);
    }
}
// check: "Keep(ABORTED { code: 0,"

//! new-transaction
//! sender: bob
script {
    use 0x1::LBR::LBR;
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<LBR>(&with_cap, {{bob}}, 10, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::LBR::LBR;
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<LBR>(&with_cap, {{abby}}, 10, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(ABORTED { code: 17,"

//! new-transaction
//! sender: bob
script {
    use 0x1::Coin1::Coin1;
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<Coin1>(&with_cap, {{abby}}, 10, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(ABORTED { code: 19,"

//! new-transaction
//! sender: bob
script {
    use 0x1::LBR::LBR;
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<LBR>(&with_cap, {{doris}}, 10, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(ABORTED { code: 18,"

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let rot_cap = LibraAccount::extract_key_rotation_capability(account);
        LibraAccount::rotate_authentication_key(&rot_cap, x"123abc");
        LibraAccount::restore_key_rotation_capability(rot_cap);
    }
}
// check: "Keep(ABORTED { code: 8,"

//! new-transaction
script {
    use 0x1::LibraAccount;
    use {{default}}::Holder;
    fun main(account: &signer) {
        Holder::hold(
            account,
            LibraAccount::extract_key_rotation_capability(account)
        );
        Holder::hold(
            account,
            LibraAccount::extract_key_rotation_capability(account)
        );
    }
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 9

//! new-transaction
script {
    use 0x1::LibraAccount;
    use 0x1::Signer;
    fun main(sender: &signer) {
        let cap = LibraAccount::extract_key_rotation_capability(sender);
        assert(
            *LibraAccount::key_rotation_capability_address(&cap) == Signer::address_of(sender), 0
        );
        LibraAccount::restore_key_rotation_capability(cap);
        let with_cap = LibraAccount::extract_withdraw_capability(sender);

        assert(
            *LibraAccount::withdraw_capability_address(&with_cap) == Signer::address_of(sender),
            0
        );
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::LBR::LBR;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<LBR>(&with_cap, {{alice}}, 10000, x"", x"");
        LibraAccount::restore_withdraw_capability(with_cap);
        assert(LibraAccount::balance<LBR>({{alice}}) == 10000, 60)
    }
}
// check: EXECUTED

//! new-transaction
//! sender: libraroot
//! type-args: 0x1::Coin1::Coin1
//! args: 0, 0x0, {{bob::auth_key}}, b"bob", b"boburl", x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d", true
stdlib_script::create_parent_vasp_account
// check: "Keep(ABORTED { code: 10,"

//! new-transaction
//! sender: libraroot
//! type-args: 0x1::Coin1::Coin1
//! args: 0, {{abby}}, x"", b"bob", b"boburl", x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d", true
stdlib_script::create_parent_vasp_account
// check: "Keep(ABORTED { code: 8,"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::LibraAccount;
    fun main() {
        LibraAccount::create_libra_root_account({{libraroot}}, {{libraroot::auth_key}});
    }
}
// check: "Keep(ABORTED { code: 0,"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        LibraAccount::create_treasury_compliance_account(account, {{blessed}}, {{blessed::auth_key}});
    }
}
// check: "Keep(ABORTED { code: 0,"
