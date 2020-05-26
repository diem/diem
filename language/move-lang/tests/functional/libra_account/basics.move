//! account: bob, 10000LBR
//! account: alice, 10000LBR

module Holder {
    use 0x0::Transaction;

    resource struct Hold<T> {
        x: T
    }

    public fun hold<T>(account: &signer, x: T) {
        move_to(account, Hold<T>{x})
    }

    public fun get<T>(): T
    acquires Hold {
        let Hold {x} = move_from<Hold<T>>(Transaction::sender());
        x
    }
}

//! new-transaction
script {
    use 0x0::LibraAccount;
    fun main(sender: &signer) {
        LibraAccount::initialize(sender);
    }
}
// check: ABORTED
// check: 0

//! new-transaction
//! sender: bob
script {
    use 0x0::LBR;
    use 0x0::LibraAccount;
    fun main() {
        let coins = LibraAccount::withdraw_from_sender<LBR::T>(10);
        LibraAccount::deposit_to_sender(coins);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x0::LibraAccount;
    fun main() {
        LibraAccount::rotate_authentication_key(x"123abc");
    }
}
// check: ABORTED
// check: 12

//! new-transaction
script {
    use 0x0::LibraAccount;
    use {{default}}::Holder;
    fun main(account: &signer) {
        Holder::hold(
            account,
            LibraAccount::extract_sender_key_rotation_capability()
        );
        Holder::hold(
            account,
            LibraAccount::extract_sender_key_rotation_capability()
        );
    }
}
// check: ABORTED
// check: 11

//! new-transaction
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::create_unhosted_account<LBR::T>(0xDEADBEEF, x"", false);
    }
}
// check: ABORTED
// check: 12

//! new-transaction
script {
    use 0x0::LibraAccount;
    use 0x0::Transaction;
    fun main() {
        let cap = LibraAccount::extract_sender_key_rotation_capability();
        Transaction::assert(*LibraAccount::key_rotation_capability_address(&cap) == Transaction::sender(), 0);
        LibraAccount::restore_key_rotation_capability(cap);
        let with_cap = LibraAccount::extract_sender_withdrawal_capability();

        Transaction::assert(*LibraAccount::withdrawal_capability_address(&with_cap) == Transaction::sender(), 0);
        LibraAccount::restore_withdrawal_capability(with_cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    use 0x0::Testnet;
    fun main() {
        Testnet::remove_testnet();
        LibraAccount::create_testnet_account<LBR::T>(0xDEADBEEF, x"");
        Testnet::initialize();
    }
}
// check: ABORTED
// check: 10042

//! new-transaction
//! sender: association
script {
    use 0x0::Testnet;
    fun main() {
        Testnet::remove_testnet();
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::pay_from_sender<LBR::T>({{alice}}, 10000);
    }
}
// TODO: what is this testing?
// chec: ABORTED
// chec: 9001

//! new-transaction
//! sender: association
script {
    use 0x0::Testnet;
    fun main() {
        Testnet::initialize();
    }
}
// check: EXECUTED
