//! account: bob, 10000LBR
//! account: alice, 0LBR
//! account: abby, 0, 0, address

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
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 0

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
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let rot_cap = LibraAccount::extract_key_rotation_capability(account);
        LibraAccount::rotate_authentication_key(&rot_cap, x"123abc");
        LibraAccount::restore_key_rotation_capability(rot_cap);
    }
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 8

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
