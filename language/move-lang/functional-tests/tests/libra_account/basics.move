//! account: bob, 10000LBR
//! account: alice, 10000LBR

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
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(sender: &signer) {
        let cap = Roles::extract_privilege_to_capability<LibraRootRole>(sender);
        LibraAccount::initialize(sender, &cap);
        Roles::restore_capability_to_privilege(sender, cap);
    }
}
// check: ABORTED
// check: 3

//! new-transaction
//! sender: bob
script {
    use 0x1::LBR::LBR;
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        let coins = LibraAccount::withdraw_from<LBR>(&with_cap, 10);
        LibraAccount::restore_withdraw_capability(with_cap);
        LibraAccount::deposit_to(account, coins);
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
// check: ABORTED
// check: 12

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
// check: ABORTED
// check: 11

//! new-transaction
script {
    use 0x1::LibraAccount;
    use 0x1::LBR::LBR;
    fun main(account: &signer) {
        LibraAccount::create_unhosted_account<LBR>(account, 0xDEADBEEF, x"", false);
    }
}
// check: ABORTED
// check: 12

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
//! sender: association
script {
    use 0x1::LibraAccount;
    use 0x1::LBR::LBR;
    use 0x1::Testnet;
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        Testnet::remove_testnet(account);
        let r = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraAccount::create_testnet_account<LBR>(account, &r, 0xDEADBEEF, x"");
        Testnet::initialize(account);
        Roles::restore_capability_to_privilege(account, r);
    }
}
// check: ABORTED
// check: 10042

//! new-transaction
//! sender: association
script {
    use 0x1::Testnet;
    fun main(account: &signer) {
        Testnet::remove_testnet(account);
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
        LibraAccount::pay_from<LBR>(&with_cap, {{alice}}, 10000);
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: what is this testing?
// chec: ABORTED
// chec: 9001

//! new-transaction
//! sender: association
script {
    use 0x1::Testnet;
    fun main(account: &signer) {
        Testnet::initialize(account);
    }
}
// check: EXECUTED
